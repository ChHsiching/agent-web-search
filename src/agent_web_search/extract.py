"""Page-body extraction, content_size mapping, and URL derivation.

Pure functions — no network, no I/O — so they are fully testable standalone.

- ``extract``: HTML -> main-content plain text, truncated to a word limit, via
  the ``readability-lxml`` port of Mozilla's Readability (ADR-0003). This is
  the Extract that populates the ``summary`` field of the top results. It is
  deliberately NOT a summary — raw page body, truncated (ADR-0001).
- ``word_limit_for``: maps the ``content_size`` tool param to a word count.
- ``derive_site_name`` / ``derive_favicon``: zero-dependency URL derivation
  (ADR-0005).
"""

from __future__ import annotations

import logging
import re
from typing import Final
from urllib.parse import urlsplit

from lxml import html as lxml_html
from readability import Document

log = logging.getLogger(__name__)

# Word limits for content_size (ADR-0001 scope decision).
MEDIUM_WORDS: Final[int] = 500
HIGH_WORDS: Final[int] = 2500


def word_limit_for(content_size: str | None) -> int:
    """Map the web_search content_size string to a word limit.

    "high" -> HIGH_WORDS, anything else (including None) -> MEDIUM_WORDS,
    matching the target tool's default.
    """
    if content_size == "high":
        return HIGH_WORDS
    return MEDIUM_WORDS


def extract(html_text: str, word_limit: int) -> str:
    """Extract main-content plain text from HTML and truncate to word_limit.

    Uses readability to identify the article body (stripping nav/sidebar/
    scripts/ads), then converts the cleaned HTML to plain text and truncates
    at a word boundary. Returns an empty string on empty/malformed input or
    when no main content is detectable. Never raises.
    """
    if not html_text or not html_text.strip() or word_limit <= 0:
        return ""

    try:
        doc = Document(html_text)
        summary_html = doc.summary(html_partial=True)
    except Exception as exc:  # noqa: BLE001 — readability can be finicky
        log.debug("readability extraction failed: %s", exc)
        return ""

    plain = _html_to_text(summary_html)
    if not plain.strip():
        return ""
    return _truncate_words(plain, word_limit)


def _html_to_text(html_fragment: str) -> str:
    """Convert an HTML fragment to plain text, preserving whitespace."""
    try:
        tree = lxml_html.fromstring(html_fragment)
        text = tree.text_content()
    except Exception:  # noqa: BLE001 — fall back to regex stripping
        text = re.sub(r"<[^>]+>", " ", html_fragment)
    # Collapse whitespace.
    return re.sub(r"\s+", " ", text).strip()


def _truncate_words(text: str, word_limit: int) -> str:
    """Truncate to at most word_limit words, appending an ellipsis if cut."""
    words = text.split()
    if len(words) <= word_limit:
        return " ".join(words)
    return " ".join(words[:word_limit]) + "\u2026"


def derive_site_name(url: str) -> str:
    """Derive a site name from a URL: strip leading www., return the host.

    Zero external dependency (ADR-0005). Returns "" on non-URL input.
    """
    if not url:
        return ""
    try:
        host = urlsplit(url).hostname or ""
    except ValueError:
        return ""
    return host.removeprefix("www.")


def derive_favicon(url: str) -> str:
    """Derive a favicon URL: {scheme}://{host}/favicon.ico.

    Constructed from the URL, never fetched (ADR-0005). Returns "" on non-URL.
    """
    if not url:
        return ""
    try:
        parts = urlsplit(url)
    except ValueError:
        return ""
    if not parts.scheme or not parts.hostname:
        return ""
    return f"{parts.scheme}://{parts.hostname}/favicon.ico"
