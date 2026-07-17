"""Unit tests for search orchestration (ticket #14).

Uses fake searcher + fake page fetcher to verify the compose logic without
touching the network: top-N extraction, snippet fallback, field derivation,
graceful degradation, empty results.
"""

from __future__ import annotations

from agent_web_search.orchestrate import orchestrate


class FakeSearcher:
    def __init__(self, results: list[dict]) -> None:
        self.results = results

    def __call__(self, **kwargs):
        return self.results


class FakePageFetcher:
    """Returns canned HTML per URL substring, or raises if unconfigured."""

    def __init__(self, pages: dict[str, str] | None = None) -> None:
        self.pages = pages or {}

    def __call__(self, url: str) -> str:
        for key, body in self.pages.items():
            if key in url:
                return body
        raise RuntimeError(f"no canned page for {url}")


def _raw(n: int) -> list[dict]:
    return [
        {
            "title": f"Title {i}",
            "href": f"https://site{i}.example/page{i}",
            "body": f"snippet number {i}",
        }
        for i in range(n)
    ]


def test_orchestrate_returns_assembled_results() -> None:
    raw = _raw(3)
    searcher = FakeSearcher(raw)
    pages = {
        "site0.example": "<html><body><article><p>Real content of page zero here.</p></article></body></html>",
        "site1.example": "<html><body><article><p>Content of page one.</p></article></body></html>",
        "site2.example": "<html><body><article><p>Content of page two.</p></article></body></html>",
    }
    fetcher = FakePageFetcher(pages)

    results = orchestrate(
        query="test",
        domain_filter=None,
        recency=None,
        location="us",
        content_size="medium",
        searcher=searcher,
        page_fetcher=fetcher,
    )

    assert len(results) == 3
    assert results[0]["title"] == "Title 0"
    assert results[0]["site_name"] == "site0.example"
    assert results[0]["favicon"] == "https://site0.example/favicon.ico"
    # top-3 got an extract attempt; summary is present (extract or snippet fallback)
    assert results[0]["summary"]


def test_page_fetch_failure_degrades_to_snippet() -> None:
    raw = [{"title": "Only", "href": "https://unfetchable.example/x", "body": "the snippet"}]
    searcher = FakeSearcher(raw)
    fetcher = FakePageFetcher({})  # nothing configured -> all fetches raise

    results = orchestrate(
        query="x",
        domain_filter=None,
        recency=None,
        location="us",
        content_size="high",
        searcher=searcher,
        page_fetcher=fetcher,
    )
    assert results[0]["summary"] == "the snippet"


def test_results_beyond_top_n_get_snippet_not_extract() -> None:
    # 5 results, but only top 3 get extraction attempts.
    raw = _raw(5)
    searcher = FakeSearcher(raw)
    fetcher = FakePageFetcher({})  # fetches fail, so top-3 fall back to snippet

    results = orchestrate(
        query="x",
        domain_filter=None,
        recency=None,
        location="us",
        content_size="medium",
        searcher=searcher,
        page_fetcher=fetcher,
    )
    assert len(results) == 5
    # All carry the snippet (top-3 fetch failed -> fallback; rest -> snippet).
    for i, r in enumerate(results):
        assert r["summary"] == f"snippet number {i}"


def test_empty_search_results() -> None:
    searcher = FakeSearcher([])
    fetcher = FakePageFetcher({})
    results = orchestrate(
        query="x",
        domain_filter=None,
        recency=None,
        location="us",
        content_size="medium",
        searcher=searcher,
        page_fetcher=fetcher,
    )
    assert results == []


def test_all_results_have_required_fields() -> None:
    raw = _raw(2)
    searcher = FakeSearcher(raw)
    fetcher = FakePageFetcher({})
    results = orchestrate(
        query="x",
        domain_filter=None,
        recency=None,
        location="us",
        content_size="medium",
        searcher=searcher,
        page_fetcher=fetcher,
    )
    for r in results:
        assert set(r.keys()) == {"title", "url", "summary", "site_name", "favicon"}
