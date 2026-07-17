"""Search orchestration: compose ddgs search + page-body extraction + output.

The end-to-end search call: take the five parameters, run the ddgs search
(returns title/url/snippet), fetch page bodies for the top results, run the
extractor on each (gated by content_size), derive site_name/favicon, and
assemble the final output list of result dicts.

Per ADR-0001, Extract is applied to the top results only (hardcoded, not
exposed); the rest carry the source Snippet. content_size controls the
extractor word limit. A page fetch/extraction failure degrades that single
result to its Snippet without failing the whole search.

Network access (search + page fetch) is dependency-injected so the orchestration
is fully testable without touching the network.
"""

from __future__ import annotations

import logging
from typing import Any, Protocol

import anyio
import httpx

from .extract import derive_favicon, derive_site_name, extract, word_limit_for
from .search import Searcher, search as ddgs_search

log = logging.getLogger(__name__)

# How many top results get a page-body Extract applied.
EXTRACT_TOP_N = 3
# Browser-like UA — many sites block non-browser UAs.
_USER_AGENT = (
    "Mozilla/5.0 (compatible; agent-web-search/0.1; "
    "+https://github.com/ChHsiching/agent-web-search)"
)


class PageFetcher(Protocol):
    """Abstract page-body fetcher — the seam for testing page extraction."""

    def __call__(self, url: str) -> str: ...


class HttpxPageFetcher:
    """Production page fetcher backed by httpx. Short timeout."""

    def __init__(self) -> None:
        self._client = httpx.Client(
            timeout=httpx.Timeout(8.0),
            headers={"User-Agent": _USER_AGENT},
            follow_redirects=True,
        )

    def __call__(self, url: str) -> str:
        resp = self._client.get(url)
        resp.raise_for_status()
        return resp.text

    def close(self) -> None:
        self._client.close()


def orchestrate(
    *,
    query: str,
    domain_filter: str | None,
    recency: str | None,
    location: str | None,
    content_size: str | None,
    searcher: Searcher | None = None,
    page_fetcher: PageFetcher | None = None,
) -> list[dict[str, Any]]:
    """Run a complete search and return assembled output results.

    Each result is ``{title, url, summary, site_name, favicon}``. The top
    EXTRACT_TOP_N results get a page-body Extract in ``summary``; the rest
    carry the source Snippet. Page-fetch failures degrade gracefully.
    """
    raw_results = ddgs_search(
        query=query,
        domain_filter=domain_filter,
        recency=recency,
        location=location,
        searcher=searcher,
    )

    word_limit = word_limit_for(content_size)
    fetcher = page_fetcher if page_fetcher is not None else HttpxPageFetcher()
    own_fetcher = page_fetcher is None

    try:
        assembled = _assemble(
            raw_results, word_limit, fetcher
        )
    finally:
        if own_fetcher and hasattr(fetcher, "close"):
            fetcher.close()

    return assembled


def _assemble(
    raw_results: list[dict[str, Any]],
    word_limit: int,
    page_fetcher: PageFetcher,
) -> list[dict[str, Any]]:
    """Assemble output results: apply Extract to the top N, derive fields."""
    # Fetch+extract the top N concurrently (they are independent). anyio moves
    # each blocking fetch to a worker thread.
    top_n = raw_results[:EXTRACT_TOP_N]
    rest = raw_results[EXTRACT_TOP_N:]

    extracted_summaries: list[str | None] = [None] * len(top_n)

    async def _fetch_all() -> None:
        async with anyio.create_task_group() as tg:

            async def _one(idx: int, url: str) -> None:
                extracted_summaries[idx] = await anyio.to_thread.run_sync(
                    _fetch_and_extract, page_fetcher, url, word_limit
                )

            for i, r in enumerate(top_n):
                url = r.get("href") or r.get("url") or ""
                tg.start_soon(_one, i, url)

    anyio.run(_fetch_all)

    results: list[dict[str, Any]] = []
    for i, raw in enumerate(raw_results):
        url = raw.get("href") or raw.get("url") or ""
        snippet = raw.get("body") or ""
        if i < EXTRACT_TOP_N:
            summary = extracted_summaries[i] or snippet
        else:
            summary = snippet
        results.append(
            {
                "title": raw.get("title") or "",
                "url": url,
                "summary": summary,
                "site_name": derive_site_name(url),
                "favicon": derive_favicon(url),
            }
        )
    return results


def _fetch_and_extract(
    fetcher: PageFetcher, url: str, word_limit: int
) -> str | None:
    """Fetch a page and run the extractor. None on any failure."""
    if not url:
        return None
    try:
        html = fetcher(url)
    except Exception as exc:  # noqa: BLE001 — any fetch error degrades to None
        log.debug("page fetch failed for %s: %s", url, exc)
        return None
    text = extract(html, word_limit)
    return text or None
