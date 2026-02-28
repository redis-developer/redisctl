//! Direct Redis database tools

mod diagnostics;
mod keys;
mod server;
mod structures;

#[allow(unused_imports)]
pub use diagnostics::*;
#[allow(unused_imports)]
pub use keys::*;
#[allow(unused_imports)]
pub use server::*;
#[allow(unused_imports)]
pub use structures::*;

use std::sync::Arc;

use tower_mcp::{McpRouter, ToolError};

use crate::state::AppState;

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
        .merge(diagnostics::router(state))
}
