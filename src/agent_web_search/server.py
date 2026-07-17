"""MCP server layer — the sole owner of stdout.

Advertises the ``web_search`` tool with a parameter schema matching the paid
``web_search_prime`` tool, so the result is a drop-in replacement (same params,
independent tool name).
"""

from __future__ import annotations

import logging
from typing import Any

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import TextContent, Tool

from . import __version__

log = logging.getLogger(__name__)

# The input schema for web_search — matches the paid web_search_prime tool's
# params 1:1 (independent tool name, same parameters = drop-in).
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
    """Build the MCP server with the web_search tool registered."""
    server = Server(
        "agent-web-search",
        version=__version__,
        instructions=(
            "A free, unlimited web-search tool. Call web_search with a query."
        ),
    )

    @server.list_tools()
    async def list_tools() -> list[Tool]:
        return [
            Tool(
                name="web_search",
                description=_TOOL_DESCRIPTION,
                inputSchema=_TOOL_SCHEMA,
            )
        ]

    @server.call_tool()
    async def call_tool(
        name: str, arguments: dict[str, Any]
    ) -> list[TextContent]:
        if name != "web_search":
            return [TextContent(type="text", text=f"unknown tool: {name}")]

        import anyio
        import json

        from .orchestrate import orchestrate

        query = arguments.get("search_query", "")
        log.info("web_search call: query=%r", query)

        try:
            results = await anyio.to_thread.run_sync(
                lambda: orchestrate(
                    query=query,
                    domain_filter=arguments.get("search_domain_filter"),
                    recency=arguments.get("search_recency_filter"),
                    location=arguments.get("location"),
                    content_size=arguments.get("content_size"),
                )
            )
        except Exception as exc:  # noqa: BLE001 — surface as a clean error
            log.warning("search failed: %s", exc)
            return [
                TextContent(
                    type="text",
                    text=json.dumps(
                        {"error": f"search failed: {exc}"},
                        ensure_ascii=False,
                    ),
                )
            ]

        return [TextContent(type="text", text=json.dumps(results, ensure_ascii=False))]

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
