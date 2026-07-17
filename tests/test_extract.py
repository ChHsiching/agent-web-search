"""Unit tests for extract + content_size mapping + URL derivation (ticket #12).

Pure-function tests — no network. Covers the Extract pipeline (readability),
word truncation, content_size mapping, and URL derivation edge cases.
"""

from __future__ import annotations

from agent_web_search.extract import (
    HIGH_WORDS,
    MEDIUM_WORDS,
    derive_favicon,
    derive_site_name,
    extract,
    word_limit_for,
)


def _article_html(body: str) -> str:
    """A realistic article fixture with nav/sidebar/footer noise to strip."""
    return (
        "<html><head><title>Test Article</title></head><body>"
        "<nav><a href='/'>Home</a> <a href='/about'>About</a></nav>"
        "<div class='sidebar'><p>Advertisements and links</p></div>"
        f"<article><h1>Real Article Title</h1>{body}</article>"
        "<footer>Copyright stuff nobody needs in an extract</footer>"
        "</body></html>"
    )


def test_extracts_article_body_text() -> None:
    para = (
        "This is the main article content that should be extracted. "
        "It contains multiple sentences about an interesting technical topic. "
        "The reader wants exactly this text and nothing else from the page."
    )
    html = _article_html(f"<p>{para}</p>")
    result = extract(html, 1000)
    assert result, "non-empty article should yield text"
    assert "main article content" in result


def test_truncates_to_word_limit() -> None:
    words = [f"word{i}" for i in range(50)]
    body = f"<p>{' '.join(words)}</p>"
    html = _article_html(body)
    result = extract(html, 10)
    count = len(result.split())
    # Ellipsis counts as one token, so at most word_limit + 1.
    assert count <= 11, f"truncated to ~10 words, got {count}: {result!r}"


def test_empty_input_returns_empty_string() -> None:
    assert extract("", 500) == ""
    assert extract("   \n\t  ", 500) == ""


def test_zero_word_limit_returns_empty_string() -> None:
    html = _article_html("<p>Some content here</p>")
    assert extract(html, 0) == ""


def test_malformed_html_does_not_raise() -> None:
    # Must not raise; may be empty or partial.
    result = extract("<html><body><p>unclosed paragraph", 500)
    assert isinstance(result, str)


def test_word_limit_for_maps_sizes() -> None:
    assert word_limit_for(None) == MEDIUM_WORDS
    assert word_limit_for("medium") == MEDIUM_WORDS
    assert word_limit_for("high") == HIGH_WORDS
    assert word_limit_for("garbage") == MEDIUM_WORDS
    assert HIGH_WORDS > MEDIUM_WORDS


def test_derive_site_name_strips_www() -> None:
    assert derive_site_name("https://www.example.com/path") == "example.com"
    assert (
        derive_site_name("https://docs.rust-lang.org/x") == "docs.rust-lang.org"
    )
    assert derive_site_name("not a url") == ""
    assert derive_site_name("") == ""


def test_derive_favicon_builds_url() -> None:
    assert (
        derive_favicon("https://www.example.com/some/page")
        == "https://www.example.com/favicon.ico"
    )
    assert derive_favicon("http://docs.rs/tokio") == "http://docs.rs/favicon.ico"
    assert derive_favicon("garbage") == ""
    assert derive_favicon("") == ""


def test_derive_favicon_ignores_path_and_port() -> None:
    assert (
        derive_favicon("https://example.com:8443/deep/path?q=1")
        == "https://example.com/favicon.ico"
    )
