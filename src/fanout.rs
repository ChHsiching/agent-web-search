//! Concurrent fan-out + batched retry + in-memory health scoring.
//!
//! This is the reliability core (ADR-0001). A query is sent concurrently to
//! the top few ranked instances; the first successful JSON response wins and
//! the rest are cancelled. If a whole batch fails or times out, the next batch
//! is tried down the ranked list. An in-memory health score demotes instances
//! that fail repeatedly so they stop being chosen as primaries.
//!
//! All network access goes through the `Fetch` trait (defined in `sources`),
//! so fan-out scheduling, batched retry, and health promotion/demotion are
//! deterministically testable with a fake fetcher.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::timeout;

use crate::search::SearxngQuery;
use crate::sources::{Fetch, Instance};

/// How many instances to query concurrently per batch.
const BATCH_SIZE: usize = 3;

/// Per-instance request timeout. A dead instance is abandoned quickly, not
/// waited on.
const PER_INSTANCE_TIMEOUT: Duration = Duration::from_secs(5);

/// Owns the ranked instance list and the in-memory health scores. Cloning
/// produces an independent view sharing the same data via `Arc`, so multiple
/// queries can fan out against the same pool.
#[derive(Clone)]
pub struct Fanout {
    inner: Arc<FanoutInner>,
}

struct FanoutInner {
    /// Instances ranked by latency median (fastest first). The source of
    /// truth from searx.space, set at refresh time. Updated via
    /// `update_instances` when a background refresh completes.
    base_ranking: std::sync::RwLock<Vec<Instance>>,
    /// Consecutive failure count per instance base_url. Higher = more demoted.
    health: std::sync::Mutex<HashMap<String, u32>>,
}

/// The outcome of a fan-out search.
pub enum FanoutResult {
    /// A response body (raw SearXNG JSON) was obtained from some instance.
    Success { body: String, source: String },
    /// Every eligible instance was exhausted without a usable response.
    Exhausted,
}

impl Fanout {
    /// Build a fan-out pool from a ranked instance list.
    pub fn new(instances: Vec<Instance>) -> Self {
        Self {
            inner: Arc::new(FanoutInner {
                base_ranking: std::sync::RwLock::new(instances),
                health: std::sync::Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Replace the instance ranking (e.g. after a background searx.space
    /// refresh). Health scores are preserved so a refresh doesn't lose what
    /// we learned about flaky instances.
    pub fn update_instances(&self, instances: Vec<Instance>) {
        *self.inner.base_ranking.write().unwrap() = instances;
    }

    /// The health-adjusted ordering of instances, used to pick batches.
    ///
    /// Instances are sorted by (health_penalty, latency_median): a healthy
    /// fast instance comes first; one with consecutive failures is pushed
    /// behind slower-but-reliable ones.
    fn ranked(&self) -> Vec<Instance> {
        let health = self.inner.health.lock().unwrap();
        let mut ranked = self.inner.base_ranking.read().unwrap().clone();
        ranked.sort_by(|a, b| {
            let ha = *health.get(&a.base_url).unwrap_or(&0);
            let hb = *health.get(&b.base_url).unwrap_or(&0);
            ha.cmp(&hb)
                .then(a.latency_median.total_cmp(&b.latency_median))
        });
        ranked
    }

    /// Whether the pool currently has any instances.
    pub fn is_empty(&self) -> bool {
        self.inner.base_ranking.read().unwrap().is_empty()
    }

    /// Search across the instance pool.
    ///
    /// Fans `query` out to BATCH_SIZE instances concurrently, takes the first
    /// success, and records health outcomes. On a full-batch failure, retries
    /// with the next batch until the pool is exhausted.
    pub async fn search(&self, fetcher: &dyn Fetch, query: &SearxngQuery) -> FanoutResult {
        let ranked = self.ranked();
        let batches = ranked.chunks(BATCH_SIZE);

        for batch in batches {
            match self.try_batch(fetcher, query, batch).await {
                Some(Success { body, source }) => {
                    self.record_success(&source);
                    return FanoutResult::Success { body, source };
                }
                None => {
                    // Whole batch failed; health recorded inside try_batch.
                }
            }
        }
        FanoutResult::Exhausted
    }

    /// Try one batch concurrently. Returns the first success, or None if the
    /// entire batch failed. Health is recorded per instance outcome.
    ///
    /// Uses `select_all` so futures share the current task's lifetime (no
    /// `'static` requirement that `JoinSet::spawn` would impose on the
    /// borrowed fetcher). The first success short-circuits; the rest are
    /// dropped (cancelled) when the futures vec goes out of scope.
    async fn try_batch(
        &self,
        fetcher: &dyn Fetch,
        query: &SearxngQuery,
        batch: &[Instance],
    ) -> Option<Success> {
        use futures::future::select_all;
        use std::future::Future;

        // Build the per-instance futures: a timed fetch returning an outcome.
        let futures: Vec<Pin<Box<dyn Future<Output = (String, FetchOutcome)> + Send>>> = batch
            .iter()
            .map(|inst| {
                let url = query.to_url(&inst.base_url);
                let base_url = inst.base_url.clone();
                let f = async move {
                    let result = timeout(PER_INSTANCE_TIMEOUT, fetcher.get(&url)).await;
                    (base_url, FetchOutcome::from(result))
                };
                Box::pin(f) as Pin<Box<dyn Future<Output = _> + Send>>
            })
            .collect();

        if futures.is_empty() {
            return None;
        }

        let mut remaining = futures;
        loop {
            // select_all races the remaining futures; the first to complete wins.
            let (outcome, _idx, rest) = select_all(remaining).await;
            remaining = rest;

            match outcome {
                (_, FetchOutcome::Ok(body)) => {
                    // Success — we don't know which of the OTHER futures would
                    // have succeeded, so we just return; dropping `remaining`
                    // cancels them. Their health is left unrecorded (neither
                    // success nor failure), which is fine — they simply didn't
                    // complete in time.
                    let source = outcome.0;
                    return Some(Success { body, source });
                }
                (base_url, FetchOutcome::Err) => {
                    self.record_failure(&base_url);
                    if remaining.is_empty() {
                        return None;
                    }
                    // Continue racing the rest.
                }
            }
        }
    }

    fn record_success(&self, base_url: &str) {
        let mut health = self.inner.health.lock().unwrap();
        health.remove(base_url);
    }

    fn record_failure(&self, base_url: &str) {
        let mut health = self.inner.health.lock().unwrap();
        *health.entry(base_url.to_string()).or_insert(0) += 1;
    }

    /// The current failure count for an instance (for testing/inspection).
    pub fn failure_count(&self, base_url: &str) -> u32 {
        self.inner
            .health
            .lock()
            .unwrap()
            .get(base_url)
            .copied()
            .unwrap_or(0)
    }
}

struct Success {
    body: String,
    source: String,
}

/// The outcome of a single instance fetch within a batch.
enum FetchOutcome {
    /// Got a 2xx response body.
    Ok(String),
    /// Timed out, errored, or non-2xx.
    Err,
}

impl FetchOutcome {
    fn from(result: Result<Result<String, anyhow::Error>, tokio::time::error::Elapsed>) -> Self {
        match result {
            Ok(Ok(body)) => FetchOutcome::Ok(body),
            Ok(Err(_)) | Err(_) => FetchOutcome::Err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{build_searxng_query, SearchRequest};
    use std::sync::Mutex;

    fn inst(url: &str, latency: f64) -> Instance {
        Instance {
            base_url: url.to_string(),
            latency_median: latency,
        }
    }

    fn query() -> SearxngQuery {
        build_searxng_query(&SearchRequest {
            query: "test".into(),
            domain_filter: None,
            recency_filter: crate::search::Recency::NoLimit,
            location: crate::search::Locale::Cn,
        })
    }

    /// A fake fetcher with per-URL canned responses and a fail set.
    struct FakeFetcher {
        responses: Mutex<Vec<(String, String)>>,
    }

    impl FakeFetcher {
        fn new() -> Self {
            Self {
                responses: Mutex::new(Vec::new()),
            }
        }
        /// Set a response for URLs containing `match_substr`.
        fn on(&self, match_substr: &str, body: &str) {
            self.responses
                .lock()
                .unwrap()
                .push((match_substr.to_string(), body.to_string()));
        }
    }

    #[async_trait::async_trait]
    impl Fetch for FakeFetcher {
        async fn get(&self, url: &str) -> anyhow::Result<String> {
            let responses = self.responses.lock().unwrap();
            for (sub, body) in responses.iter() {
                if url.contains(sub) {
                    return Ok(body.clone());
                }
            }
            anyhow::bail!("no canned response for {url}")
        }
    }

    #[tokio::test]
    async fn first_success_wins() {
        let fake = FakeFetcher::new();
        fake.on("fast.example", r#"{"results":[{"title":"hi","url":"https://x"}]}"#);
        fake.on("ok.example", r#"{"results":[{"title":"hi","url":"https://y"}]}"#);

        let fanout = Fanout::new(vec![
            inst("https://fast.example/", 0.1),
            inst("https://ok.example/", 0.2),
        ]);
        let result = fanout.search(&fake, &query()).await;

        match result {
            FanoutResult::Success { body, source } => {
                assert!(body.contains("results"));
                assert!(source.contains("example"));
            }
            FanoutResult::Exhausted => panic!("expected success"),
        }
    }

    #[tokio::test]
    async fn batched_retry_on_all_fail() {
        let fake = FakeFetcher::new();
        // First batch (3 instances) all fail; only the 4th succeeds.
        fake.on("good.example", r#"{"results":[]}"#);

        let fanout = Fanout::new(vec![
            inst("https://bad1.example/", 0.1),
            inst("https://bad2.example/", 0.2),
            inst("https://bad3.example/", 0.3),
            inst("https://good.example/", 0.4),
        ]);
        let result = fanout.search(&fake, &query()).await;

        match result {
            FanoutResult::Success { source, .. } => {
                assert!(source.contains("good.example"), "succeeded via retry: {source}");
            }
            FanoutResult::Exhausted => panic!("should have retried to the good instance"),
        }
        // The three bad instances should have recorded failures.
        assert_eq!(fanout.failure_count("https://bad1.example/"), 1);
        assert_eq!(fanout.failure_count("https://bad2.example/"), 1);
        assert_eq!(fanout.failure_count("https://bad3.example/"), 1);
    }

    #[tokio::test]
    async fn exhaustion_when_all_instances_fail() {
        let fake = FakeFetcher::new(); // no responses -> all fail
        let fanout = Fanout::new(vec![
            inst("https://a.example/", 0.1),
            inst("https://b.example/", 0.2),
        ]);
        let result = fanout.search(&fake, &query()).await;
        assert!(matches!(result, FanoutResult::Exhausted));
    }

    #[tokio::test]
    async fn health_demotes_repeatedly_failing_instance() {
        // A flaky instance that always fails, in its own batch (it's the only
        // instance, so every batch is just it, and it always records a failure).
        // After several searches it accumulates health penalty.
        let fake = FakeFetcher::new(); // no responses -> everything fails
        let fanout = Fanout::new(vec![
            inst("https://flaky.example/", 0.05),
        ]);

        for _ in 0..3 {
            let result = fanout.search(&fake, &query()).await;
            assert!(matches!(result, FanoutResult::Exhausted));
        }

        // After repeated failures, flaky's health penalty should be >= 3.
        assert!(
            fanout.failure_count("https://flaky.example/") >= 3,
            "flaky should have accumulated failures, got {}",
            fanout.failure_count("https://flaky.example/")
        );
    }

    #[tokio::test]
    async fn ranked_order_places_healthier_ahead_of_failing() {
        // Two instances: one healthy (low health penalty), one with accumulated
        // failures. Verify the healthy one ranks first.
        let fake = FakeFetcher::new();
        let fanout = Fanout::new(vec![
            inst("https://a.example/", 0.1),
            inst("https://b.example/", 0.2),
        ]);
        // Artificially penalize b.
        fanout.record_failure("https://b.example/");
        fanout.record_failure("https://b.example/");

        let ranked = fanout.ranked();
        assert_eq!(ranked[0].base_url, "https://a.example/");
        assert_eq!(ranked[1].base_url, "https://b.example/");
    }

    #[tokio::test]
    async fn success_clears_health_penalty() {
        let fake = FakeFetcher::new();
        fake.on("recover.example", r#"{"results":[]}"#);

        let fanout = Fanout::new(vec![inst("https://recover.example/", 0.1)]);

        // Artificially set a failure count.
        fanout.record_failure("https://recover.example/");
        assert_eq!(fanout.failure_count("https://recover.example/"), 1);

        // A successful search clears it.
        let _ = fanout.search(&fake, &query()).await;
        assert_eq!(fanout.failure_count("https://recover.example/"), 0);
    }
}
