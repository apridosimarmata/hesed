use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A tool exposed by the upstream MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

/// Refresh the discovered tools in the sidecar state via the stdio child.
pub async fn refresh(state: &Arc<crate::proxy::SidecarState>) {
    let tools = state.stdio_child.discover_tools().await;
    let mut lock = state.discovered_tools.write().await;
    *lock = tools;
}
