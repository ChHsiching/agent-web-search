"""Integration test for the MCP stdio server shell (ticket #11).

Verifies the ADR-0004 stability disciplines end-to-end against the real
(unbundled) server process:
- the server responds to ``initialize`` and ``tools/list``.
- stdout contains ONLY valid JSON-RPC frames (no stray log pollution).
- ``tools/call`` returns a response without crashing.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from collections.abc import Iterator
from pathlib import Path

import pytest

from agent_web_search import __version__

# Locate the project root so we can run the server module from source.
ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def server_env() -> dict[str, str]:
    env = os.environ.copy()
    # Ensure the src layout is importable when running python -m.
    env["PYTHONPATH"] = str(ROOT / "src") + os.pathsep + env.get("PYTHONPATH", "")
    return env


def _spawn(env: dict[str, str]) -> subprocess.Popen[bytes]:
    return subprocess.Popen(
        [sys.executable, "-m", "agent_web_search"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )


def _send(proc: subprocess.Popen[bytes], obj: dict) -> None:
    assert proc.stdin is not None
    proc.stdin.write((json.dumps(obj) + "\n").encode())
    proc.stdin.flush()


def _read(proc: subprocess.Popen[bytes]) -> dict:
    assert proc.stdout is not None
    line = proc.stdout.readline()
    assert line, "expected a JSON-RPC line on stdout"
    return json.loads(line.decode())


INIT = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test", "version": "0.1"},
    },
}
INITIALIZED = {"jsonrpc": "2.0", "method": "notifications/initialized"}
TOOLS_LIST = {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}


def test_initialize_handshake_responds(server_env: dict[str, str]) -> None:
    proc = _spawn(server_env)
    try:
        _send(proc, INIT)
        resp = _read(proc)
        assert resp["jsonrpc"] == "2.0"
        assert resp["id"] == 1
        name = resp["result"]["serverInfo"]["name"]
        assert "agent-web-search" in name, f"server name: {name}"
        # The version on the wire must match the package version — pyproject
        # is the single source, so a mismatch means the link broke somewhere.
        version = resp["result"]["serverInfo"]["version"]
        assert version == __version__, f"server version: {version!r} != {__version__!r}"
        assert resp["result"]["capabilities"]["tools"] is not None
    finally:
        proc.kill()
        proc.wait(timeout=5)


def test_tools_list_advertises_web_search(
    server_env: dict[str, str]
) -> None:
    proc = _spawn(server_env)
    try:
        _send(proc, INIT)
        _read(proc)
        _send(proc, INITIALIZED)
        _send(proc, TOOLS_LIST)
        resp = _read(proc)
        tools = resp["result"]["tools"]
        assert len(tools) == 1, f"exactly one tool, got {len(tools)}"
        tool = tools[0]
        assert tool["name"] == "web_search"

        props = tool["inputSchema"]["properties"]
        assert props["search_query"]["type"] == "string"
        for opt in (
            "search_domain_filter",
            "search_recency_filter",
            "content_size",
            "location",
        ):
            assert props[opt]["type"] == "string", f"optional {opt} present"
        required = tool["inputSchema"]["required"]
        assert required == ["search_query"]
    finally:
        proc.kill()
        proc.wait(timeout=5)


def test_stdout_is_clean_json_rpc_only(server_env: dict[str, str]) -> None:
    """The ADR-0004 discipline: nothing but JSON-RPC on stdout.

    Runs initialize + tools/list (no tools/call network dependency) and
    asserts every stdout line parses as JSON.
    """
    proc = _spawn(server_env)
    try:
        for line in (INIT, INITIALIZED, TOOLS_LIST):
            _send(proc, line)
        # Give the server a moment to emit its responses.
        import time

        time.sleep(0.3)
        # communicate() closes stdin itself; closing it manually first makes
        # communicate()'s internal stdin.flush() raise "flush of closed file"
        # on Linux (Windows tolerates it). Don't pre-close.
        out, _ = proc.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        out, _ = proc.communicate()

    parsed = 0
    for line in out.decode().splitlines():
        if not line.strip():
            continue
        json.loads(line)  # raises if stdout pollution
        parsed += 1
    assert parsed >= 2, f"expected >=2 JSON-RPC responses, got {parsed}"


def test_unknown_tool_call_gets_a_response_and_server_survives(
    server_env: dict[str, str],
) -> None:
    """ADR-0004 discipline #3: a bad request degrades to a readable
    response, never a crash.

    Calls an unregistered tool name, asserts a well-formed JSON-RPC
    response comes back (either a result or an error object), then asserts
    the server is still alive by issuing another request afterwards.
    """
    proc = _spawn(server_env)
    try:
        _send(proc, INIT)
        _read(proc)
        _send(proc, INITIALIZED)
        _send(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {"name": "no_such_tool", "arguments": {}},
            },
        )
        resp = _read(proc)
        assert resp["id"] == 3
        assert "result" in resp or "error" in resp, f"well-formed response: {resp}"

        # Still alive: tools/list must answer afterwards.
        _send(proc, TOOLS_LIST)
        resp2 = _read(proc)
        assert resp2["id"] == 2
        assert resp2["result"]["tools"][0]["name"] == "web_search"
    finally:
        proc.kill()
        proc.wait(timeout=5)


@pytest.mark.integration
def test_tools_call_returns_json_shaped_response(
    server_env: dict[str, str]
) -> None:
    """tools/call now runs the real search pipeline (network).

    We only assert the server returns a well-formed JSON-RPC response whose
    content is valid JSON (a results list or an error object), tolerant of
    live rate-limiting. Live results aren't guaranteed.

    Marked ``integration`` because it hits the live ddgs/DuckDuckGo pipeline
    and is therefore weather-dependent. CI deselects it (``-m "not
    integration"``); run it locally with a bare ``python -m pytest`` to
    exercise the real network path.
    """
    proc = _spawn(server_env)
    try:
        _send(proc, INIT)
        _read(proc)
        _send(proc, INITIALIZED)
        _send(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "web_search",
                    "arguments": {
                        "search_query": "rust programming language",
                        "location": "us",
                    },
                },
            },
        )
        resp = _read(proc)
        assert resp["id"] == 3
        assert "error" not in resp, f"no JSON-RPC error: {resp}"
        text = resp["result"]["content"][0]["text"]
        data = json.loads(text)
        # Either a results array or an error object.
        assert isinstance(data, list) or "error" in data, f"valid shape: {data}"
        if isinstance(data, list) and data:
            r = data[0]
            for field in ("title", "url", "summary", "site_name", "favicon"):
                assert field in r, f"result has {field}"
    finally:
        proc.kill()
        proc.wait(timeout=10)
