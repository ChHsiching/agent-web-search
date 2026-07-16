//! MCP protocol layer — the sole owner of stdout.
//!
//! Advertises the `web_search_prime` tool with a schema matching the target
//! tool, so the result is a drop-in replacement. For now the handler returns a
//! stub; the full search pipeline is wired up in a later ticket.

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::{schemars, tool, tool_handler, tool_router, ServiceExt};
use serde::{Deserialize, Serialize};

/// Parameters for the `web_search_prime` tool, matching the target tool 1:1.
///
/// Only `search_query` is required; the rest are optional filters mapped to
/// SearXNG query params downstream.
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
///
/// `summary` holds the page-body Extract (raw text) for the top results, or
/// the source Snippet for the rest — never a generated summary.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub summary: String,
    pub site_name: String,
    pub favicon: String,
}

/// The MCP server handler. Owns the single `web_search_prime` tool.
#[derive(Debug, Clone)]
pub struct WebSearchServer {
    tool_router: ToolRouter<WebSearchServer>,
}

impl WebSearchServer {
    /// Construct a new server with its tool router initialized.
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
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
    fn web_search_prime(
        &self,
        Parameters(_params): Parameters<WebSearchParams>,
    ) -> String {
        // Stub: returns a placeholder until the search pipeline is wired up.
        // The real implementation will call the search orchestration layer.
        String::from("[]")
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
                "A free, unlimited web-search tool. Call web_search_prime with a query."
                    .into(),
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
/// before the handshake completes.
pub async fn serve_stdio() -> anyhow::Result<()> {
    let service = WebSearchServer::new()
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
