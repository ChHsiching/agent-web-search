//! Configuration values shared across modules.
//!
//! Lightweight constants and pure mappings — no I/O, no state. Kept separate
//! from the logic that consumes them so they can be tuned in one place.

/// Word limit for `content_size = "medium"` (~500 words of page body).
pub const MEDIUM_WORDS: usize = 500;

/// Word limit for `content_size = "high"` (~2500 words of page body).
pub const HIGH_WORDS: usize = 2500;

/// How many top results get a page-body Extract applied. The rest return only
/// the source Snippet. Hardcoded (not exposed to the agent) to bound per-query
/// fetch cost.
pub const EXTRACT_TOP_N: usize = 3;

/// Map the `web_search_prime` `content_size` string to a word limit.
///
/// "high" → HIGH_WORDS, anything else (including None) → MEDIUM_WORDS
/// (matching the target tool's default).
pub fn word_limit_for(content_size: &Option<String>) -> usize {
    match content_size.as_deref() {
        Some("high") => HIGH_WORDS,
        _ => MEDIUM_WORDS,
    }
}
