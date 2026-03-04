//! Proxy management tools for Redis Enterprise

use std::sync::Arc;

use redis_enterprise::proxies::ProxyHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;
use crate::tools::wrap_list;

/// Input for listing proxies
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListProxiesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_enterprise_proxies tool
pub fn list_proxies(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_proxies")
        .description("List all proxy instances in the Redis Enterprise cluster with their status and configuration.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListProxiesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListProxiesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ProxyHandler::new(client);
                let proxies = handler
                    .list()
                    .await
                    .tool_context("Failed to list proxies")?;

                wrap_list("proxies", &proxies)
            },
        )
        .build()
}

/// Input for getting a specific proxy
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetProxyInput {
    /// Proxy UID
    pub uid: u32,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_enterprise_proxy tool
pub fn get_proxy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_proxy")
        .description("Get detailed information about a specific proxy instance by UID.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetProxyInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetProxyInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ProxyHandler::new(client);
                let proxy = handler
                    .get(input.uid)
                    .await
                    .tool_context("Failed to get proxy")?;

                CallToolResult::from_serialize(&proxy)
            },
        )
        .build()
}

/// Input for getting proxy stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetProxyStatsInput {
    /// Proxy UID
    pub uid: u32,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_enterprise_proxy_stats tool
pub fn get_proxy_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_proxy_stats")
        .description("Get statistics for a specific proxy instance including connection counts and throughput.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetProxyStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetProxyStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ProxyHandler::new(client);
                let stats = handler
                    .stats(input.uid)
                    .await
                    .tool_context("Failed to get proxy stats")?;

                CallToolResult::from_serialize(&stats)
            },
        )
        .build()
}

/// Input for updating a proxy
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateProxyInput {
    /// Proxy UID to update
    pub uid: u32,
    /// Updated proxy configuration as JSON (e.g., max_connections, threads)
    pub updates: Value,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_enterprise_proxy tool
pub fn update_proxy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_proxy")
        .description(
            "Update a proxy instance's configuration. Pass the fields to update as JSON \
             (e.g., max_connections, threads). Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateProxyInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateProxyInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ProxyHandler::new(client);
                let update = serde_json::from_value(input.updates)
                    .map_err(|e| McpError::tool(format!("Invalid proxy update: {}", e)))?;
                let result = handler
                    .update(input.uid, update)
                    .await
                    .tool_context("Failed to update proxy")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
    "list_enterprise_proxies",
    "get_enterprise_proxy",
    "get_enterprise_proxy_stats",
    "update_enterprise_proxy",
];

/// Build the proxy sub-router
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(list_proxies(state.clone()))
        .tool(get_proxy(state.clone()))
        .tool(get_proxy_stats(state.clone()))
        .tool(update_proxy(state.clone()))
}
