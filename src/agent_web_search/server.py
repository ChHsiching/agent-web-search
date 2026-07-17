"""MCP server layer — the sole owner of stdout.

Advertises the ``web_search_prime`` tool with a schema matching the target
tool, so the result is a drop-in replacement. For now the handler returns a
stub; the full search pipeline is wired up in a later ticket (#15).
"""

from __future__ import annotations

import logging
from typing import Any

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import TextContent, Tool

from . import __version__

log = logging.getLogger(__name__)

# The input schema for web_search_prime — matches the target tool 1:1 so the
# tool is a drop-in replacement. All optional params default to None.
_TOOL_SCHEMA: dict[str, Any] = {
    "type": "object",
    "properties": {
        "search_query": {
            "type": "string",
            "description": (
                "Content to be searched, it is recommended that the search "
                "query not exceed 70 characters."
            ),
        },
        "search_domain_filter": {
            "type": "string",
            "description": (
                "Limit results to a whitelist domain, e.g. "
                '"www.example.com".'
            ),
        },
        "search_recency_filter": {
            "type": "string",
            "description": (
                "Time range: oneDay, oneWeek, oneMonth, oneYear, "
                "noLimit (default)."
            ),
        },
        "content_size": {
            "type": "string",
            "description": (
                'Summary length: "medium" (default, ~500 words) or '
                '"high" (~2500 words).'
            ),
        },
        "location": {
            "type": "string",
            "description": 'Region: "cn" (default) or "us".',
        },
    },
    "required": ["search_query"],
    "additionalProperties": False,
}

_TOOL_DESCRIPTION = (
    "Search web information, returns results including web page title, "
    "web page URL, web page summary, website name, website icon, etc."
)


def create_server() -> Server:
    """Build the MCP server with the web_search_prime tool registered."""
    server = Server("agent-web-search")

    @server.list_tools()
    async def list_tools() -> list[Tool]:
        return [
            Tool(
                name="web_search_prime",
                description=_TOOL_DESCRIPTION,
                inputSchema=_TOOL_SCHEMA,
            )
        ]

    @server.call_tool()
    async def call_tool(
        name: str, arguments: dict[str, Any]
    ) -> list[TextContent]:
        if name != "web_search_prime":
            return [TextContent(type="text", text=f"unknown tool: {name}")]

        # Stub: returns an empty result list until the search pipeline is
        # wired up (ticket #15). The real implementation will call the search
        # orchestration layer.
        query = arguments.get("search_query", "")
        log.info("web_search_prime stub call: query=%r", query)
        return [TextContent(type="text", text="[]")]

    return server


async def serve() -> None:
    """Run the stdio MCP server until the transport closes.

    stdout is reserved for JSON-RPC frames exclusively — logging is configured
    in ``__main__`` to go to stderr (ADR-0004). No network call is made before
    the handshake completes.
    """
    server = create_server()
    options = server.create_initialization_options()
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, options, raise_exceptions=False)


def run() -> None:
    """Synchronous entry: run the async server under anyio/asyncio."""
    import anyio

    anyio.run(serve)
