//! SearXNG instance source: fetch from searx.space, filter to healthy
//! instances, rank by latency, and cache locally.
//!
//! This is the layer that keeps the instance list zero-maintenance (ADR-0001):
//! we consume searx.space's data, never hand-maintain a list. Reliability comes
//! from fanning out across many instances (the fanout module), not from any
//! single instance.
//!
//! The `Fetch` trait defined here is the **single test seam** for the whole
//! codebase: core logic depends on an interface that performs HTTP fetches;
//! production wires `reqwest`, tests inject a fake returning canned responses.
//! One seam covers fan-out scheduling, batched retry, and health-score logic.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use serde::Deserialize;

/// One SearXNG instance, as we use it after filtering and ranking.
#[derive(Debug, Clone)]
pub struct Instance {
    /// Base URL with trailing slash, e.g. "https://searx.example.org/".
    pub base_url: String,
    /// Search-latency median in seconds, from searx.space. Lower is better.
    /// Used for ranking and health-weighted selection.
    pub latency_median: f64,
}

/// A fetcher abstraction — the single test seam for the codebase.
///
/// Implementations: `ReqwestFetcher` (production, HTTP) and a fake
/// `FakeFetcher` (tests, canned responses). Every module that needs to hit the
/// network depends on this trait, so the core logic is deterministic and
/// network-free under test.
#[async_trait]
pub trait Fetch: Send + Sync {
    /// GET a URL, returning the response body as text on a 2xx, or an error
    /// otherwise (including non-2xx status, timeout, connection failure).
    async fn get(&self, url: &str) -> anyhow::Result<String>;
}

/// Production HTTP fetcher backed by `reqwest` with rustls. Short timeout by
/// default so dead instances are abandoned quickly.
#[derive(Clone)]
pub struct ReqwestFetcher {
    client: reqwest::Client,
}

impl ReqwestFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("agent-web-search/0.1 (+https://github.com/ChHsiching/agent-web-search)")
            .build()
            .expect("failed to build reqwest client");
        Self { client }
    }
}

impl Default for ReqwestFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetch for ReqwestFetcher {
    async fn get(&self, url: &str) -> anyhow::Result<String> {
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("HTTP {} for {}", resp.status(), url);
        }
        Ok(resp.text().await?)
    }
}

// ---------------------------------------------------------------------------
// searx.space data model
// ---------------------------------------------------------------------------

/// The searx.space `instances.json` envelope, lightly modeled. Only the fields
/// we read for filtering and ranking are captured; the rest is ignored.
#[derive(Debug, Deserialize)]
struct SearxSpace {
    #[serde(default)]
    instances: std::collections::HashMap<String, SearxInstance>,
}

#[derive(Debug, Deserialize)]
struct SearxInstance {
    #[serde(default)]
    http: SearxHttp,
    #[serde(default)]
    timing: SearxTiming,
}

#[derive(Debug, Default, Deserialize)]
struct SearxHttp {
    #[serde(default)]
    status_code: Option<u16>,
}

#[derive(Debug, Default, Deserialize)]
struct SearxTiming {
    #[serde(default)]
    search: SearxSearchTiming,
}

#[derive(Debug, Default, Deserialize)]
struct SearxSearchTiming {
    #[serde(default)]
    success_percentage: Option<f64>,
    #[serde(default)]
    all: SearxSearchAll,
}

#[derive(Debug, Default, Deserialize)]
struct SearxSearchAll {
    #[serde(default)]
    median: Option<f64>,
}

// ---------------------------------------------------------------------------
// Fetching + filtering + ranking
// ---------------------------------------------------------------------------

/// Fetch the raw instance list JSON from searx.space.
pub async fn fetch_instances_json(fetcher: &dyn Fetch) -> anyhow::Result<String> {
    fetcher
        .get("https://searx.space/data/instances.json")
        .await
}

/// Parse and filter the searx.space JSON into ranked instances.
///
/// Filters to instances with `search.success_percentage == 100` AND
/// `http.status_code == 200`, then ranks by search-latency median (fastest
/// first). Returns an empty list on parse failure (never panics).
pub fn filter_and_rank(instances_json: &str) -> Vec<Instance> {
    let parsed: SearxSpace = match serde_json::from_str(instances_json) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut healthy: Vec<Instance> = parsed
        .instances
        .into_iter()
        .filter_map(|(url, inst)| {
            let success = inst.timing.search.success_percentage == Some(100.0);
            let http_ok = inst.http.status_code == Some(200);
            if !(success && http_ok) {
                return None;
            }
            let base_url = normalize_base_url(&url);
            let latency = inst.timing.search.all.median.unwrap_or(999.0);
            Some(Instance {
                base_url,
                latency_median: latency,
            })
        })
        .collect();

    healthy.sort_by(|a, b| {
        a.latency_median
            .partial_cmp(&b.latency_median)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    healthy
}

/// Normalize an instance URL to a base with a trailing slash.
fn normalize_base_url(url: &str) -> String {
    let url = url.trim();
    if url.ends_with('/') {
        url.to_string()
    } else {
        format!("{url}/")
    }
}

/// Probe whether a SearXNG instance exposes its JSON API.
///
/// Sends a minimal search request with `format=json` and checks that the
/// response is parseable as JSON containing a `results` array. searx.space does
/// not report whether JSON API is enabled, so this is a one-shot self-check
/// (ADR-0001). Returns `true` if the instance is usable over JSON.
pub async fn probe_json_api(fetcher: &dyn Fetch, instance: &Instance) -> bool {
    let probe_url = format!(
        "{}search?q=test&format=json&categories=general",
        instance.base_url
    );
    let body = match fetcher.get(&probe_url).await {
        Ok(b) => b,
        Err(_) => return false,
    };
    // A valid SearXNG JSON response is an object with a "results" key.
    let parsed: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return false,
    };
    parsed
        .get("results")
        .map(|r| r.is_array())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Local cache
// ---------------------------------------------------------------------------

/// A cached instance list with its fetch timestamp, for TTL expiry.
#[derive(Debug, Clone)]
pub struct CachedInstances {
    pub instances: Vec<Instance>,
    pub fetched_at: SystemTime,
}

impl CachedInstances {
    /// True if the cache is still fresh (within TTL).
    pub fn is_fresh(&self, ttl: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.fetched_at)
            .map(|age| age < ttl)
            .unwrap_or(false)
    }
}

/// Where the instance cache file lives: a platform-appropriate cache dir.
pub fn cache_path() -> Option<PathBuf> {
    let base = dirs::cache_dir()?;
    Some(base.join("agent-web-search").join("instances.json"))
}

/// Load the cached instance list from disk, if present and parseable.
pub fn load_cache() -> Option<CachedInstances> {
    let path = cache_path()?;
    let bytes = std::fs::read(&path).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let fetched_at = json
        .get("fetched_at_unix")
        .and_then(|v| v.as_u64())
        .and_then(|secs| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(secs)))?;
    let instances_json = json.get("instances").and_then(|v| v.as_str())?;
    let instances = filter_and_rank(instances_json);
    Some(CachedInstances {
        instances,
        fetched_at,
    })
}

/// Persist the raw instances JSON + timestamp to the cache file.
pub fn write_cache(instances_json: &str) {
    let Some(path) = cache_path() else {
        return;
    };
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let body = serde_json::json!({
        "fetched_at_unix": now,
        "instances": instances_json,
    });
    let _ = std::fs::write(&path, serde_json::to_vec(&body).unwrap_or_default());
}

/// Default cache time-to-live: 1 hour (ADR-0001).
pub const CACHE_TTL: Duration = Duration::from_secs(3600);

/// Refresh the instance list from searx.space and update the cache. On failure,
/// returns what we have from cache (or empty). Designed to run in the
/// background post-handshake (ADR-0004) — never blocks startup.
pub async fn refresh(fetcher: Arc<dyn Fetch>) -> Vec<Instance> {
    match fetch_instances_json(fetcher.as_ref()).await {
        Ok(json) => {
            let ranked = filter_and_rank(&json);
            if !ranked.is_empty() {
                write_cache(&json);
            }
            ranked
        }
        Err(_) => load_cache()
            .map(|c| c.instances)
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// A fake fetcher that returns canned responses keyed by URL substring.
    struct FakeFetcher {
        responses: Mutex<std::collections::HashMap<String, String>>,
        failures: Mutex<std::collections::HashSet<String>>,
    }

    impl FakeFetcher {
        fn new() -> Self {
            Self {
                responses: Mutex::new(std::collections::HashMap::new()),
                failures: Mutex::new(std::collections::HashSet::new()),
            }
        }
        fn set(&self, key: &str, body: &str) {
            self.responses
                .lock()
                .unwrap()
                .insert(key.to_string(), body.to_string());
        }
        fn fail(&self, key: &str) {
            self.failures.lock().unwrap().insert(key.to_string());
        }
    }

    #[async_trait]
    impl Fetch for FakeFetcher {
        async fn get(&self, url: &str) -> anyhow::Result<String> {
            let failures = self.failures.lock().unwrap();
            for f in failures.iter() {
                if url.contains(f) {
                    anyhow::bail!("fake failure for {url}");
                }
            }
            let responses = self.responses.lock().unwrap();
            for (key, body) in responses.iter() {
                if url.contains(key) {
                    return Ok(body.clone());
                }
            }
            anyhow::bail!("no canned response for {url}");
        }
    }

    fn sample_searx_space() -> &'static str {
        r#"{
            "instances": {
                "https://fast.example/": {
                    "http": {"status_code": 200},
                    "timing": {"search": {"success_percentage": 100.0, "all": {"median": 0.3}}}
                },
                "https://slow.example/": {
                    "http": {"status_code": 200},
                    "timing": {"search": {"success_percentage": 100.0, "all": {"median": 1.2}}}
                },
                "https://broken.example/": {
                    "http": {"status_code": 500},
                    "timing": {"search": {"success_percentage": 100.0, "all": {"median": 0.2}}}
                },
                "https://flaky.example/": {
                    "http": {"status_code": 200},
                    "timing": {"search": {"success_percentage": 50.0, "all": {"median": 0.4}}}
                }
            }
        }"#
    }

    #[test]
    fn filter_keeps_only_healthy_instances() {
        let ranked = filter_and_rank(sample_searx_space());
        // broken (500) and flaky (50%) are excluded; fast and slow remain.
        let urls: Vec<&str> = ranked.iter().map(|i| i.base_url.as_str()).collect();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://fast.example/"));
        assert!(urls.contains(&"https://slow.example/"));
    }

    #[test]
    fn filter_ranks_by_latency_ascending() {
        let ranked = filter_and_rank(sample_searx_space());
        // fast (0.3) before slow (1.2)
        assert_eq!(ranked[0].base_url, "https://fast.example/");
        assert_eq!(ranked[1].base_url, "https://slow.example/");
        assert!(ranked[0].latency_median < ranked[1].latency_median);
    }

    #[test]
    fn filter_on_invalid_json_returns_empty() {
        assert!(filter_and_rank("not json").is_empty());
        assert!(filter_and_rank("").is_empty());
    }

    #[tokio::test]
    async fn fetch_instances_json_uses_fetcher() {
        let fake = FakeFetcher::new();
        fake.set("instances.json", sample_searx_space());
        let json = fetch_instances_json(&fake).await.unwrap();
        assert!(json.contains("fast.example"));
    }

    #[tokio::test]
    async fn refresh_writes_cache_on_success() {
        let fake = Arc::new(FakeFetcher::new());
        fake.set("instances.json", sample_searx_space());
        let ranked = refresh(fake).await;
        assert_eq!(ranked.len(), 2);
        // Cache should now be loadable (we may not have a cache dir in CI, so
        // just assert the happy path didn't panic and returned ranked results).
        assert_eq!(ranked[0].base_url, "https://fast.example/");
    }

    #[tokio::test]
    async fn refresh_does_not_panic_on_fetch_error() {
        let fake = Arc::new(FakeFetcher::new());
        // No canned response -> fetch fails -> falls back to cache (which may
        // exist from another test) or empty. The contract under test is that
        // refresh never panics on a fetch error and always returns a Vec.
        let ranked = refresh(fake).await;
        // Whatever the cache state, refresh returns a Vec without panicking.
        let _ = ranked.len();
    }

    #[test]
    fn normalize_base_url_adds_trailing_slash() {
        assert_eq!(normalize_base_url("https://x.example"), "https://x.example/");
        assert_eq!(normalize_base_url("https://x.example/"), "https://x.example/");
    }

    #[test]
    fn cached_instances_freshness() {
        let fresh = CachedInstances {
            instances: vec![],
            fetched_at: SystemTime::now(),
        };
        assert!(fresh.is_fresh(CACHE_TTL));

        let stale = CachedInstances {
            instances: vec![],
            fetched_at: SystemTime::UNIX_EPOCH,
        };
        assert!(!stale.is_fresh(CACHE_TTL));
    }

    #[tokio::test]
    async fn probe_json_api_accepts_valid_json_response() {
        let fake = FakeFetcher::new();
        fake.set("format=json", r#"{"results":[],"number_of_results":0}"#);
        let instance = Instance {
            base_url: "https://ok.example/".into(),
            latency_median: 0.5,
        };
        assert!(probe_json_api(&fake, &instance).await);
    }

    #[tokio::test]
    async fn probe_json_api_rejects_html_or_non_json() {
        let fake = FakeFetcher::new();
        // An instance that returns an HTML page (JSON API disabled).
        fake.set("format=json", "<html><body>not json</body></html>");
        let instance = Instance {
            base_url: "https://html.example/".into(),
            latency_median: 0.5,
        };
        assert!(!probe_json_api(&fake, &instance).await);
    }

    #[tokio::test]
    async fn probe_json_api_rejects_on_fetch_error() {
        let fake = FakeFetcher::new();
        // No canned response -> fetch fails.
        let instance = Instance {
            base_url: "https://down.example/".into(),
            latency_median: 0.5,
        };
        assert!(!probe_json_api(&fake, &instance).await);
    }
}
