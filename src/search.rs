//! Search data-transformation layer: query building, result parsing, and
//! per-result URL derivation.
//!
//! These are pure functions — no network, no I/O — so they are fully testable
//! without any HTTP seam. The orchestration ticket composes these with the
//! fanout and extract modules to form a complete search call.
//!
//! Vocabulary note (CONTEXT.md): a **Snippet** is the short description the
//! search Source returns in `content`; an **Extract** is the page-body text we
//! fetch separately. This module deals in Snippets only — Extract is applied
//! downstream by the orchestration layer.

use serde::Deserialize;

/// A raw result parsed from a SearXNG JSON response, before Extract is applied.
/// Only the fields we care about are captured; the rest are ignored.
#[derive(Debug, Clone, PartialEq)]
pub struct RawResult {
    pub title: String,
    pub url: String,
    /// The Snippet: the short description returned by the search source.
    pub snippet: String,
}

/// The five `web_search_prime` parameters, in the form the query builder and
/// parser consume. This mirrors the MCP `WebSearchParams` but lives in the
/// search layer (no MCP dependency) so it stays pure and testable.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchRequest {
    pub query: String,
    pub domain_filter: Option<String>,
    pub recency_filter: Recency,
    pub location: Locale,
}

/// Recency filter values from `web_search_prime`, mapped to SearXNG `time_range`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Recency {
    OneDay,
    OneWeek,
    OneMonth,
    OneYear,
    NoLimit,
}

impl Recency {
    /// Parse the `web_search_prime` recency string. Unknown values default to
    /// `NoLimit` (matching the target tool's default).
    pub fn from_str_lossy(s: &Option<String>) -> Self {
        match s.as_deref() {
            Some("oneDay") => Recency::OneDay,
            Some("oneWeek") => Recency::OneWeek,
            Some("oneMonth") => Recency::OneMonth,
            Some("oneYear") => Recency::OneYear,
            _ => Recency::NoLimit,
        }
    }

    /// The SearXNG `time_range` parameter value, or `None` when unbounded.
    pub fn to_time_range(self) -> Option<&'static str> {
        match self {
            Recency::OneDay => Some("day"),
            Recency::OneWeek => Some("week"),
            Recency::OneMonth => Some("month"),
            Recency::OneYear => Some("year"),
            Recency::NoLimit => None,
        }
    }
}

/// Locale from `web_search_prime`, mapped to SearXNG `locale`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locale {
    Cn,
    Us,
}

impl Locale {
    /// Parse the `web_search_prime` location string. Unknown values default to
    /// `cn` (matching the target tool's default).
    pub fn from_str_lossy(s: &Option<String>) -> Self {
        match s.as_deref() {
            Some("us") => Locale::Us,
            _ => Locale::Cn,
        }
    }

    /// The SearXNG `locale` parameter value.
    pub fn to_locale(self) -> &'static str {
        match self {
            Locale::Cn => "zh-CN",
            Locale::Us => "en-US",
        }
    }
}

/// A query ready to be appended to a SearXNG instance's `/search` path, either
/// as query-string params or as a full URL against a given base.
#[derive(Debug, Clone, PartialEq)]
pub struct SearxngQuery {
    /// The `q` value, possibly augmented with `site:` for domain filtering.
    pub q: String,
    pub time_range: Option<&'static str>,
    pub locale: &'static str,
}

impl SearxngQuery {
    /// Build the full search URL against a given SearXNG instance base URL.
    pub fn to_url(&self, base: &str) -> String {
        let base = base.trim_end_matches('/');
        let mut url = format!(
            "{base}/search?q={}&format=json&categories=general&locale={}",
            urlencoding(&self.q),
            self.locale,
        );
        if let Some(tr) = self.time_range {
            url.push_str("&time_range=");
            url.push_str(tr);
        }
        url
    }
}

/// Build a SearXNG query from the search request parameters.
///
/// `domain_filter` is folded into the query via SearXNG's `site:` syntax
/// (SearXNG treats it as part of `q`). The recency and locale map to their
/// dedicated params.
pub fn build_searxng_query(req: &SearchRequest) -> SearxngQuery {
    let q = match &req.domain_filter {
        Some(domain) => format!("{} site:{}", req.query, domain),
        None => req.query.clone(),
    };
    SearxngQuery {
        q,
        time_range: req.recency_filter.to_time_range(),
        locale: req.location.to_locale(),
    }
}

/// Deserialize target for a single SearXNG result object. Captures only the
/// fields we read; extra fields are ignored via `serde(default)`.
#[derive(Debug, Deserialize)]
struct SearxngResultRaw {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

/// Deserialize target for the SearXNG JSON envelope.
#[derive(Debug, Deserialize)]
struct SearxngResponse {
    #[serde(default)]
    results: Vec<SearxngResultRaw>,
}

/// Parse a SearXNG JSON response into raw results. Skips entries missing a
/// url or title (they are not useful search results). Never panics — invalid
/// JSON returns an empty list.
pub fn parse_results(json: &str) -> Vec<RawResult> {
    let resp: SearxngResponse = match serde_json::from_str(json) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    resp.results
        .into_iter()
        .filter_map(|r| {
            let url = r.url?;
            if url.trim().is_empty() {
                return None;
            }
            Some(RawResult {
                title: r.title.unwrap_or_default(),
                url,
                snippet: r.content.unwrap_or_default(),
            })
        })
        .collect()
}

/// Derive a site name from a result URL: strip leading `www.` and return the
/// host's main label (ADR-0005). Zero external dependency.
pub fn derive_site_name(raw_url: &str) -> String {
    let host = extract_host(raw_url).unwrap_or_default();
    let host = host.trim_start_matches("www.");
    host.to_string()
}

/// Derive a favicon URL from a result URL: `{scheme}://{host}/favicon.ico`
/// (ADR-0005). Never fetched by us — a constructed string only.
pub fn derive_favicon(raw_url: &str) -> String {
    match extract_scheme_and_host(raw_url) {
        Some((scheme, host)) => format!("{scheme}://{host}/favicon.ico"),
        None => String::new(),
    }
}

/// Extract the host portion of a URL, without the scheme or path.
fn extract_host(url: &str) -> Option<&str> {
    let after_scheme = url.split("://").nth(1)?;
    let host_end = after_scheme
        .find(['/', ':', '?', '#'])
        .unwrap_or(after_scheme.len());
    Some(&after_scheme[..host_end])
}

/// Extract the (scheme, host) pair from a URL.
fn extract_scheme_and_host(url: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = url.split_once("://")?;
    let host_end = rest.find(['/', ':', '?', '#']).unwrap_or(rest.len());
    Some((scheme, &rest[..host_end]))
}

/// Minimal percent-encoding for the query string. Encodes characters that are
/// not allowed unencoded in a URL query value. Uses a small allowlist rather
/// than a full URL-encoding crate to keep dependencies minimal.
fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(query: &str) -> SearchRequest {
        SearchRequest {
            query: query.to_string(),
            domain_filter: None,
            recency_filter: Recency::NoLimit,
            location: Locale::Cn,
        }
    }

    #[test]
    fn builds_plain_query() {
        let q = build_searxng_query(&req("rust async"));
        assert_eq!(q.q, "rust async");
        assert_eq!(q.time_range, None);
        assert_eq!(q.locale, "zh-CN");
    }

    #[test]
    fn domain_filter_uses_site_syntax() {
        let mut r = req("tokio runtime");
        r.domain_filter = Some("docs.rust-lang.org".into());
        let q = build_searxng_query(&r);
        assert_eq!(q.q, "tokio runtime site:docs.rust-lang.org");
    }

    #[test]
    fn query_url_has_required_params() {
        let q = build_searxng_query(&req("rust async"));
        let url = q.to_url("https://searx.example.org");
        assert!(url.contains("/search?"));
        assert!(url.contains("format=json"));
        assert!(url.contains("categories=general"));
        assert!(url.contains("locale=zh-CN"));
        assert!(url.contains("q=rust+async"));
    }

    #[test]
    fn query_url_includes_time_range_when_set() {
        let mut r = req("news");
        r.recency_filter = Recency::OneWeek;
        let q = build_searxng_query(&r);
        let url = q.to_url("https://x.example");
        assert!(url.contains("time_range=week"));
    }

    #[test]
    fn query_url_omits_time_range_when_no_limit() {
        let q = build_searxng_query(&req("x"));
        let url = q.to_url("https://x.example");
        assert!(!url.contains("time_range"));
    }

    #[test]
    fn recency_enum_maps_all_values() {
        assert_eq!(Recency::OneDay.to_time_range(), Some("day"));
        assert_eq!(Recency::OneWeek.to_time_range(), Some("week"));
        assert_eq!(Recency::OneMonth.to_time_range(), Some("month"));
        assert_eq!(Recency::OneYear.to_time_range(), Some("year"));
        assert_eq!(Recency::NoLimit.to_time_range(), None);
    }

    #[test]
    fn recency_from_str_lossy_defaults_unknown_to_no_limit() {
        assert_eq!(Recency::from_str_lossy(&None), Recency::NoLimit);
        assert_eq!(
            Recency::from_str_lossy(&Some("garbage".into())),
            Recency::NoLimit
        );
        assert_eq!(
            Recency::from_str_lossy(&Some("oneDay".into())),
            Recency::OneDay
        );
    }

    #[test]
    fn locale_maps_cn_and_us() {
        assert_eq!(Locale::Cn.to_locale(), "zh-CN");
        assert_eq!(Locale::Us.to_locale(), "en-US");
        assert_eq!(Locale::from_str_lossy(&Some("us".into())), Locale::Us);
        assert_eq!(Locale::from_str_lossy(&None), Locale::Cn);
    }

    #[test]
    fn parses_realistic_searxng_response() {
        let json = r#"{
            "results": [
                {"title": "Tokio", "url": "https://tokio.rs/", "content": "A runtime for Rust", "engine": ["google"], "score": 8.5},
                {"title": "Docs", "url": "https://docs.rs/tokio", "content": "API docs", "score": 3.0}
            ],
            "number_of_results": 2
        }"#;
        let results = parse_results(json);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Tokio");
        assert_eq!(results[0].url, "https://tokio.rs/");
        assert_eq!(results[0].snippet, "A runtime for Rust");
    }

    #[test]
    fn parse_skips_results_without_url() {
        let json = r#"{"results": [
            {"title": "No URL", "content": "x"},
            {"title": "Empty URL", "url": "  ", "content": "y"},
            {"title": "Good", "url": "https://ok.example", "content": "z"}
        ]}"#;
        let results = parse_results(json);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://ok.example");
    }

    #[test]
    fn parse_invalid_json_returns_empty() {
        assert!(parse_results("not json").is_empty());
        assert!(parse_results("").is_empty());
    }

    #[test]
    fn derive_site_name_strips_www() {
        assert_eq!(
            derive_site_name("https://www.example.com/path"),
            "example.com"
        );
        assert_eq!(derive_site_name("https://docs.rust-lang.org/x"), "docs.rust-lang.org");
        assert_eq!(derive_site_name("not a url"), "");
    }

    #[test]
    fn derive_favicon_builds_url() {
        assert_eq!(
            derive_favicon("https://www.example.com/some/page"),
            "https://www.example.com/favicon.ico"
        );
        assert_eq!(
            derive_favicon("http://docs.rs/tokio"),
            "http://docs.rs/favicon.ico"
        );
        assert_eq!(derive_favicon("garbage"), "");
    }

    #[test]
    fn urlencoding_handles_special_chars() {
        assert_eq!(urlencoding("rust async"), "rust+async");
        assert_eq!(urlencoding("a&b=c"), "a%26b%3Dc");
        assert_eq!(urlencoding("plain123-_.~"), "plain123-_.~");
    }
}
