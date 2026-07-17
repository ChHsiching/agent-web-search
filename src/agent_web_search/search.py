"""DuckDuckGo search via the ``ddgs`` library + parameter mapping.

Maps the five ``web_search_prime`` parameters to ``ddgs.text()`` arguments and
runs the search. This is the backend connection (ADR-0006): ``ddgs`` handles
the anti-bot/rate-limit/retry logic; we just map params and call it.

The searcher is abstracted behind :class:`Searcher` so tests can inject a fake
without hitting the network.
"""

from __future__ import annotations

import logging
from typing import Any, Protocol

log = logging.getLogger(__name__)

# How many results to request from ddgs. We keep ~10 to match the target tool.
DEFAULT_MAX_RESULTS = 10


class Searcher(Protocol):
    """Abstract search backend — the seam for testing.

    A callable taking the mapped ddgs arguments and returning a list of result
    dicts (each with at least ``title``, ``href``/``url``, ``body``).
    """

    def __call__(self, **kwargs: Any) -> list[dict[str, Any]]: ...


# --- parameter mapping ---

_RECENCY_TO_TIMELIMIT: dict[str, str] = {
    "oneDay": "d",
    "oneWeek": "w",
    "oneMonth": "m",
    "oneYear": "y",
}

_LOCATION_TO_REGION: dict[str, str] = {
    "cn": "cn-zh",
    "us": "us-en",
}


def map_recency(recency: str | None) -> str | None:
    """Map the web_search_prime recency string to a ddgs timelimit.

    Unknown/None values map to None (no limit), matching the target default.
    """
    if recency is None:
        return None
    return _RECENCY_TO_TIMELIMIT.get(recency)


def map_location(location: str | None) -> str:
    """Map the web_search_prime location to a ddgs region. Default us-en."""
    return _LOCATION_TO_REGION.get(location or "", "us-en")


def build_keywords(query: str, domain_filter: str | None) -> str:
    """Build the ddgs query, folding in a site: filter if given."""
    if domain_filter:
        return f"{query} site:{domain_filter}"
    return query


def search(
    *,
    query: str,
    domain_filter: str | None,
    recency: str | None,
    location: str | None,
    max_results: int = DEFAULT_MAX_RESULTS,
    searcher: Searcher | None = None,
) -> list[dict[str, Any]]:
    """Run a DuckDuckGo search via ddgs, returning raw result dicts.

    Each result has ``title``, ``href`` (url), and ``body`` (snippet) keys from
    ddgs. ``content_size`` is NOT handled here — it is consumed downstream by
    the extract step.

    ``searcher`` is injectable; defaults to the real ddgs backend.
    """
    keywords = build_keywords(query, domain_filter)
    region = map_location(location)
    timelimit = map_recency(recency)

    backend = searcher if searcher is not None else _ddgs_search
    log.info(
        "ddgs search: keywords=%r region=%s timelimit=%s max=%d",
        keywords,
        region,
        timelimit,
        max_results,
    )
    return backend(
        query=keywords,
        region=region,
        timelimit=timelimit,
        max_results=max_results,
    )


def _ddgs_search(*, query: str, region: str, timelimit: str | None, max_results: int) -> list[dict[str, Any]]:
    """The real ddgs backend call. Synchronous (ddgs is sync)."""
    from ddgs import DDGS

    with DDGS() as dds:
        results = list(
            dds.text(
                query=query,
                region=region,
                timelimit=timelimit,
                max_results=max_results,
            )
        )
    return results
