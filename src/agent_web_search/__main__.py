"""Entry point for agent-web-search, a stdio MCP web-search server.

Startup discipline (ADR-0004):
- stdout is JSON-RPC only; all logs go to stderr.
- the MCP ``initialize`` handshake waits on no network call.
- startup never crashes; errors degrade gracefully.

Run with: ``python -m agent_web_search`` or the ``agent-web-search`` script.
"""

import logging
import sys


def main() -> None:
    # All diagnostics to stderr — stdout belongs to JSON-RPC exclusively.
    logging.basicConfig(
        stream=sys.stderr,
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )

    # Import here so that logging is configured before server code runs, and
    # so that import errors surface after logging is ready.
    from agent_web_search.server import run

    try:
        run()
    except KeyboardInterrupt:
        pass
    except Exception as exc:  # noqa: BLE001 — top-level guard
        logging.getLogger("agent_web_search").exception("server exited: %s", exc)
        sys.exit(1)


if __name__ == "__main__":
    main()
