"""Cross-platform smoke test for the built binary.

Used by the release CI to verify the PyInstaller-bundled binary actually
starts and responds to the MCP initialize handshake. Runs on Windows,
Linux, and macOS without shell-specific syntax (no heredoc).

Usage: python scripts/smoke_test.py
Exits non-zero on failure.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time


def main() -> int:
    exe = "dist/agent-web-search.exe" if sys.platform == "win32" else "dist/agent-web-search"
    if not os.path.exists(exe):
        print(f"FAIL: binary not found at {exe}", file=sys.stderr)
        return 1

    t0 = time.monotonic()
    proc = subprocess.Popen(
        [exe],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        init = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "ci", "version": "0.1"},
            },
        }
        assert proc.stdin is not None
        assert proc.stdout is not None
        proc.stdin.write((json.dumps(init) + "\n").encode())
        proc.stdin.flush()

        line = proc.stdout.readline()
        if not line:
            stderr = proc.stderr.read().decode() if proc.stderr else ""
            print(f"FAIL: no response from binary. stderr:\n{stderr}", file=sys.stderr)
            return 1

        resp = json.loads(line)
        name = resp.get("result", {}).get("serverInfo", {}).get("name", "")
        if "agent-web-search" not in name:
            print(f"FAIL: bad server name: {name}", file=sys.stderr)
            return 1

        print(f"smoke test OK: {time.monotonic() - t0:.2f}s startup, server={name}")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    sys.exit(main())
