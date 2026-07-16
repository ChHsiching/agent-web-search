//! MCP protocol layer — the sole owner of stdout.
//!
//! Advertises the `web_search_prime` tool with a schema matching the target
//! tool, and wires tool calls into the full search pipeline (orchestrate).
//! The server holds the fetcher and fan-out pool as shared state; a background
//! task refreshes the instance list from searx.space after the handshake
//! (ADR-0004: the handshake itself waits on nothing).

use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::{schemars, tool, tool_handler, tool_router, ServiceExt};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::fanout::Fanout;
use crate::search::{self, Locale, Recency, SearchRequest};
use crate::sources::{self, Fetch, ReqwestFetcher};

/// Parameters for the `web_search_prime` tool, matching the target tool 1:1.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSearchParams {
    /// Content to be searched, it is recommended that the search query not
    /// exceed 70 characters.
    pub search_query: String,
    /// Limit results to a whitelist domain, e.g. "www.example.com".
    #[serde(default)]
    pub search_domain_filter: Option<String>,
    /// Time range: oneDay, oneWeek, oneMonth, oneYear, noLimit (default).
    #[serde(default)]
    pub search_recency_filter: Option<String>,
    /// Summary length: "medium" (default, ~500 words) or "high" (~2500 words).
    #[serde(default)]
    pub content_size: Option<String>,
    /// Region: "cn" (default) or "us".
    #[serde(default)]
    pub location: Option<String>,
}

/// One search result, schema-matching the target tool's output.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub summary: String,
    pub site_name: String,
    pub favicon: String,
}

/// Runtime state shared across tool calls: the HTTP fetcher and the fan-out
/// pool (which owns the instance ranking + health scores).
#[derive(Clone)]
pub struct ServerState {
    pub fetcher: Arc<ReqwestFetcher>,
    pub fanout: Fanout,
}

/// The MCP server handler. Owns the single `web_search_prime` tool plus the
/// shared search state.
#[derive(Clone)]
pub struct WebSearchServer {
    tool_router: ToolRouter<WebSearchServer>,
    state: ServerState,
}

impl WebSearchServer {
    /// Construct a new server with its tool router and search state.
    /// Instances are seeded from cache (if any) and refreshed in the
    /// background after the handshake.
    pub fn new() -> Self {
        let fetcher = Arc::new(ReqwestFetcher::new());
        let initial = sources::load_cache()
            .map(|c| c.instances)
            .unwrap_or_default();
        let fanout = Fanout::new(initial.clone());

        // Spawn the background refresh. This runs after construction (and
        // crucially does not block the handshake). It keeps the instance list
        // fresh from searx.space without any maintenance.
        let fetcher_clone = Arc::clone(&fetcher);
        let fanout_clone = fanout.clone();
        tokio::spawn(async move {
            info!("background instance refresh starting");
            let instances = sources::refresh(fetcher_clone).await;
            if instances.is_empty() {
                warn!("background refresh returned no instances; keeping previous list");
            } else {
                info!(count = instances.len(), "refreshed instance list");
                fanout_clone.update_instances(instances);
            }
        });

        Self {
            tool_router: Self::tool_router(),
            state: ServerState { fetcher, fanout },
        }
    }
}

impl Default for WebSearchServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl WebSearchServer {
    #[tool(description = "Search web information, returns results including web page title, web page URL, web page summary, website name, website icon, etc.")]
    async fn web_search_prime(
        &self,
        Parameters(params): Parameters<WebSearchParams>,
    ) -> String {
        // If the pool is empty (cold start, no cache, refresh not done yet),
        // do a synchronous one-shot refresh for this query only (ADR-0004).
        if self.state.fanout.is_empty() {
            let instances =
                sources::refresh(Arc::clone(&self.state.fetcher) as Arc<dyn Fetch>).await;
            if !instances.is_empty() {
                self.state.fanout.update_instances(instances);
            }
        }

        let request = SearchRequest {
            query: params.search_query,
            domain_filter: params.search_domain_filter,
            recency_filter: Recency::from_str_lossy(&params.search_recency_filter),
            location: Locale::from_str_lossy(&params.location),
        };

        let outcome = search::orchestrate(
            &request,
            &params.content_size,
            &self.state.fanout,
            self.state.fetcher.as_ref(),
        )
        .await;

        match outcome {
            search::SearchOutcome::Ok(results) => {
                let out: Vec<SearchResult> = results
                    .into_iter()
                    .map(|r| SearchResult {
                        title: r.title,
                        url: r.url,
                        summary: r.summary,
                        site_name: r.site_name,
                        favicon: r.favicon,
                    })
                    .collect();
                serde_json::to_string(&out).unwrap_or_else(|_| String::from("[]"))
            }
            search::SearchOutcome::NoSource => {
                r#"{"error":"No search source available. All SearXNG instances were exhausted."}"#
                    .to_string()
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for WebSearchServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "agent-web-search".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            capabilities: rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some(
                "A free, unlimited web-search tool. Call web_search_prime with a query.".into(),
            ),
            ..Default::default()
        }
    }
}

/// Start the stdio MCP server. This completes the `initialize` handshake and
/// then serves requests until the transport closes.
///
/// stdout is reserved for JSON-RPC frames exclusively — the tracing subscriber
/// in `main` routes all logs to stderr (ADR-0004). No network call is made
/// before the handshake completes; the background refresh is spawned inside
/// `WebSearchServer::new`, which runs after the handshake sets up the runtime.
pub async fn serve_stdio() -> anyhow::Result<()> {
    let service = WebSearchServer::new()
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
