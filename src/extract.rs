//! Page-body extraction: HTML → word-limited text Extract.
//!
//! Uses `readabilityrs` (a Rust port of Mozilla's modern Readability algorithm
//! — see ADR-0003) to pull the main article content out of a page, then
//! truncates the resulting plain text to a word limit.
//!
//! This is the Extract that populates the `summary` field of the top results.
//! It is deliberately NOT a summary — it is raw page-body text, truncated. The
//! agent reads and interprets it; we do not pre-digest content (ADR-0001).
//!
//! Pure function: no network, no I/O. Fully testable without any HTTP seam.

use readabilityrs::Readability;

/// Extract main-content plain text from an HTML page and truncate it to
/// `word_limit` words.
///
/// Uses Readability to identify the article body (stripping navigation,
/// sidebars, scripts, ads), takes its plain-text form (`text_content`), and
/// truncates at a word boundary so the result never ends mid-word.
///
/// Returns an empty string when given empty/unparseable HTML, HTML with no
/// detectable main content, or when Readability itself returns `None`. Never
/// panics.
pub fn extract(html: &str, word_limit: usize) -> String {
    if html.trim().is_empty() || word_limit == 0 {
        return String::new();
    }

    let text = match Readability::new(html, None, None) {
        Ok(readability) => match readability.parse() {
            Some(article) => article.text_content.unwrap_or_default(),
            None => return String::new(),
        },
        Err(_) => return String::new(),
    };

    truncate_words(&text, word_limit)
}

/// Truncate `text` to at most `word_limit` words, cutting at a word boundary
/// and appending an ellipsis when truncation occurs.
fn truncate_words(text: &str, word_limit: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= word_limit {
        return words.join(" ");
    }
    let mut truncated = words[..word_limit].join(" ");
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A realistic article-shaped HTML fixture with nav/sidebar noise that
    /// Readability should strip, leaving the article body.
    fn article_html(body: &str) -> String {
        format!(
            r#"<html><head><title>Test Article</title></head>
            <body>
              <nav><a href="/">Home</a> <a href="/about">About</a></nav>
              <div class="sidebar"><p>Advertisements and links</p></div>
              <article>
                <h1>Real Article Title</h1>
                {body}
              </article>
              <footer>Copyright stuff nobody needs in an extract</footer>
            </body></html>"#
        )
    }

    #[test]
    fn extracts_article_body_text() {
        let long_para = "This is the main article content that should be extracted. \
            It contains multiple sentences about an interesting technical topic. \
            The reader wants exactly this text and nothing else from the page.";
        let html = article_html(&format!("<p>{long_para}</p>"));
        let result = extract(&html, 1000);
        assert!(!result.is_empty(), "non-empty article should yield text");
        assert!(
            result.contains("main article content"),
            "result should contain the body text: {result}"
        );
    }

    #[test]
    fn truncates_to_word_limit() {
        // Build a body with many distinct words.
        let words: Vec<String> = (0..50).map(|i| format!("word{i}")).collect();
        let body = format!("<p>{}</p>", words.join(" "));
        let html = article_html(&body);
        let result = extract(&html, 10);
        let word_count = result.split_whitespace().count();
        // The ellipsis counts as one token, so at most word_limit + 1.
        assert!(
            word_count <= 11,
            "truncated to ~10 words plus ellipsis, got {word_count}: {result}"
        );
    }

    #[test]
    fn empty_input_returns_empty_string() {
        assert_eq!(extract("", 500), "");
        assert_eq!(extract("   \n\t  ", 500), "");
    }

    #[test]
    fn zero_word_limit_returns_empty_string() {
        let html = article_html("<p>Some content here</p>");
        assert_eq!(extract(&html, 0), "");
    }

    #[test]
    fn malformed_html_does_not_panic() {
        let result = extract("<html><body><p>unclosed paragraph", 500);
        // Must not panic; may be empty or partial — the contract is no-panic.
        let _ = result;
    }

    #[test]
    fn html_with_no_article_content_returns_empty_or_minimal() {
        // A page that is pure navigation/boilerplate with no real article body.
        let html = r#"<html><body><nav><a href="/">x</a></nav></body></html>"#;
        let result = extract(html, 500);
        // Either empty or very short — the contract is it doesn't crash and
        // doesn't return the nav boilerplate as if it were content.
        assert!(
            !result.to_lowercase().contains("href"),
            "should not return raw nav markup"
        );
    }
}
