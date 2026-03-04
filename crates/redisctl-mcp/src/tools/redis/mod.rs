//! Direct Redis database tools

mod diagnostics;
mod keys;
mod raw;
mod server;
mod structures;

#[allow(unused_imports)]
pub use diagnostics::*;
#[allow(unused_imports)]
pub use keys::*;
#[allow(unused_imports)]
pub use raw::*;
#[allow(unused_imports)]
pub use server::*;
#[allow(unused_imports)]
pub use structures::*;

use std::sync::Arc;

use tower_mcp::{McpRouter, ToolError};

use crate::state::AppState;

/// All tool names registered by the Database (Redis) toolset.
pub const TOOL_NAMES: &[&str] = &[
    // server
    "redis_ping",
    "redis_info",
    "redis_dbsize",
    "redis_client_list",
    "redis_cluster_info",
    "redis_slowlog",
    "redis_config_get",
    "redis_memory_stats",
    "redis_latency_history",
    "redis_acl_list",
    "redis_acl_whoami",
    "redis_module_list",
    "redis_config_set",
    "redis_flushdb",
    // diagnostics
    "redis_health_check",
    "redis_key_summary",
    "redis_hotkeys",
    "redis_connection_summary",
    // keys
    "redis_keys",
    "redis_get",
    "redis_type",
    "redis_ttl",
    "redis_exists",
    "redis_memory_usage",
    "redis_scan",
    "redis_object_encoding",
    "redis_object_freq",
    "redis_object_idletime",
    "redis_object_help",
    "redis_set",
    "redis_del",
    "redis_expire",
    "redis_rename",
    // structures
    "redis_hgetall",
    "redis_lrange",
    "redis_smembers",
    "redis_zrange",
    "redis_xinfo_stream",
    "redis_xrange",
    "redis_xlen",
    "redis_pubsub_channels",
    "redis_pubsub_numsub",
    "redis_hset",
    "redis_hdel",
    "redis_lpush",
    "redis_rpush",
    "redis_lpop",
    "redis_rpop",
    "redis_sadd",
    "redis_srem",
    "redis_zadd",
    "redis_zrem",
    "redis_xadd",
    "redis_xtrim",
    // raw
    "redis_command",
];

/// Get all Database tool names as owned strings.
pub fn tool_names() -> Vec<String> {
    TOOL_NAMES.iter().map(|s| (*s).to_string()).collect()
}

/// Resolve a Redis URL from the provided inputs.
///
/// Resolution order:
/// 1. If `url` is provided, use it directly (backward compatible)
/// 2. If `profile` is provided, resolve via profile system
/// 3. Fall back to `state.database_url`
pub(crate) fn resolve_redis_url(
    url: Option<String>,
    profile: Option<&str>,
    state: &AppState,
) -> Result<String, ToolError> {
    if let Some(url) = url {
        return Ok(url);
    }
    if let Some(profile_name) = profile {
        return state
            .database_url_for_profile(Some(profile_name))
            .map_err(|e| {
                ToolError::new(format!(
                    "Failed to resolve database profile '{}': {}",
                    profile_name, e
                ))
            });
    }
    // Try default profile resolution (no explicit profile name)
    if state.database_url.is_none()
        && let Ok(url) = state.database_url_for_profile(None)
    {
        return Ok(url);
    }
    state.database_url.clone().ok_or_else(|| {
        ToolError::new(
            "No Redis URL provided, no profile configured, and no default database URL set",
        )
    })
}

/// Helper to format Redis values for display
pub(crate) fn format_value(v: &redis::Value) -> String {
    match v {
        redis::Value::Nil => "(nil)".to_string(),
        redis::Value::Int(i) => i.to_string(),
        redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
        redis::Value::SimpleString(s) => s.clone(),
        redis::Value::Array(arr) => format!(
            "[{}]",
            arr.iter().map(format_value).collect::<Vec<_>>().join(", ")
        ),
        _ => format!("{:?}", v),
    }
}

/// Build an MCP sub-router containing all Redis database tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(server::router(state.clone()))
        .merge(keys::router(state.clone()))
        .merge(structures::router(state.clone()))
        .merge(diagnostics::router(state.clone()))
        .merge(raw::router(state))
}
