//! Entry point for agent-web-search, a stdio MCP web-search server.
//!
//! Startup discipline (ADR-0004):
//! - stdout is JSON-RPC only; all logs go to stderr via tracing.
//! - the MCP `initialize` handshake waits on no network call.
//! - startup never panics; errors degrade gracefully.

mod extract;
mod mcp;
mod search;
mod sources;
mod fanout;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All diagnostics to stderr — stdout belongs to JSON-RPC exclusively.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "starting agent-web-search MCP server"
    );

    // The searx.space instance fetch runs post-handshake in the background
    // (sources ticket). Nothing here blocks the initialize handshake.
    mcp::serve_stdio().await
}
