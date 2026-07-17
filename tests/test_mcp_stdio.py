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
        assert resp["result"]["capabilities"]["tools"] is not None
    finally:
        proc.kill()
        proc.wait(timeout=5)


def test_tools_list_advertises_web_search_prime(
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
        assert tool["name"] == "web_search_prime"

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
        proc.stdin and proc.stdin.close()
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


def test_tools_call_returns_stub_without_error(
    server_env: dict[str, str]
) -> None:
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
                    "name": "web_search_prime",
                    "arguments": {"search_query": "test"},
                },
            },
        )
        resp = _read(proc)
        assert resp["id"] == 3
        assert "error" not in resp, f"no JSON-RPC error: {resp}"
        text = resp["result"]["content"][0]["text"]
        # Stub returns "[]".
        assert json.loads(text) == []
    finally:
        proc.kill()
        proc.wait(timeout=5)
