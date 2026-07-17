"""Unit tests for ddgs parameter mapping and the search seam (ticket #13).

Uses a fake searcher to verify parameter mapping without hitting the network.
The real ddgs backend is exercised by the integration test (ticket #15).
"""

from __future__ import annotations

from agent_web_search.search import (
    build_keywords,
    map_location,
    map_recency,
    search,
)


class FakeSearcher:
    """Records the kwargs it was called with, returns a canned result list."""

    def __init__(self, results: list[dict] | None = None) -> None:
        self.calls: list[dict] = []
        self.results = results if results is not None else [
            {"title": "T", "href": "https://example.com", "body": "snippet"}
        ]

    def __call__(self, **kwargs):
        self.calls.append(kwargs)
        return self.results


def test_map_recency_all_values() -> None:
    assert map_recency("oneDay") == "d"
    assert map_recency("oneWeek") == "w"
    assert map_recency("oneMonth") == "m"
    assert map_recency("oneYear") == "y"
    assert map_recency("noLimit") is None
    assert map_recency(None) is None
    assert map_recency("garbage") is None


def test_map_location_defaults() -> None:
    assert map_location("cn") == "cn-zh"
    assert map_location("us") == "us-en"
    assert map_location(None) == "us-en"
    assert map_location("zz") == "us-en"


def test_build_keywords_with_domain() -> None:
    assert build_keywords("tokio", "docs.rust-lang.org") == "tokio site:docs.rust-lang.org"
    assert build_keywords("tokio", None) == "tokio"


def test_search_passes_mapped_params() -> None:
    fake = FakeSearcher()
    search(
        query="rust async",
        domain_filter="example.com",
        recency="oneWeek",
        location="cn",
        searcher=fake,
    )
    assert len(fake.calls) == 1
    call = fake.calls[0]
    assert call["query"] == "rust async site:example.com"
    assert call["region"] == "cn-zh"
    assert call["timelimit"] == "w"
    assert call["max_results"] == 10


def test_search_no_filters_uses_defaults() -> None:
    fake = FakeSearcher()
    search(query="hello", domain_filter=None, recency=None, location=None, searcher=fake)
    call = fake.calls[0]
    assert call["query"] == "hello"
    assert call["region"] == "us-en"
    assert call["timelimit"] is None


def test_search_returns_results() -> None:
    fake = FakeSearcher(results=[{"title": "A", "href": "https://a", "body": "b1"},
                                  {"title": "B", "href": "https://b", "body": "b2"}])
    results = search(query="x", domain_filter=None, recency=None, location=None, searcher=fake)
    assert len(results) == 2
    assert results[0]["title"] == "A"


def test_search_empty_results() -> None:
    fake = FakeSearcher(results=[])
    results = search(query="x", domain_filter=None, recency=None, location=None, searcher=fake)
    assert results == []
