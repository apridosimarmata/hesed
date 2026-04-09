mod audit;
mod authz;
mod breaker;
mod config;
mod discovery;
mod dlp;
mod heartbeat;
mod hitl;
mod interceptor;
mod proxy;
mod stdio;

use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    let cfg = config::Config::load(&config_path)?;

    // Stdout is the JSON-RPC channel — logs MUST go to stderr.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("poimen=info".parse()?))
        .with_writer(std::io::stderr)
        .json()
        .init();

    tracing::info!(mode = ?cfg.mode, "starting mcp-sidecar (stdio)");

    // Spawn the child MCP server process
    let child = Arc::new(stdio::StdioChild::spawn(&cfg.upstream)?);
    let state = Arc::new(proxy::SidecarState::new(cfg.clone(), child)?);

    if state.agent_api_key.is_some() {
        tracing::info!("AGENT_API_KEY set — authz will resolve via backend");
    } else {
        tracing::warn!("AGENT_API_KEY not set — all tool calls will be rejected");
    }

    // Discover upstream tools on startup (best-effort)
    discovery::refresh(&state).await;

    // Start heartbeat only in dynamic mode
    if cfg.mode == config::ConfigMode::Dynamic {
        if let Some(hb_config) = cfg.heartbeat {
            heartbeat::spawn(
                state.clone(),
                hb_config.central_url,
                hb_config.interval_secs,
                hb_config.api_key,
            );
        } else {
            tracing::warn!("dynamic mode requires [heartbeat] config — falling back to static rules");
        }
    } else {
        tracing::info!("static mode — using rules from config file only");
    }

    // Run the stdio loop: read from stdin, intercept, forward to child, write to stdout
    stdio::run_stdio_loop(state).await?;

    tracing::info!("shutdown complete");
    Ok(())
}
