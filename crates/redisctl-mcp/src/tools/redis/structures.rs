//! Data structure Redis tools (hgetall, lrange, smembers, zrange)

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

pub(super) const INSTRUCTIONS: &str = "\
### Redis Database - Data Structures\n\
- redis_hgetall: Get all hash fields\n\
- redis_lrange: Get list range\n\
- redis_smembers: Get set members\n\
- redis_zrange: Get sorted set range\n\
";

/// Build a sub-router containing all data structure Redis tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(hgetall(state.clone()))
        .tool(lrange(state.clone()))
        .tool(smembers(state.clone()))
        .tool(zrange(state.clone()))
}

/// Input for HGETALL command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HgetallInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Hash key to get
    pub key: String,
}

/// Build the hgetall tool
pub fn hgetall(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hgetall")
        .description("Get all fields and values from a hash")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, HgetallInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HgetallInput>| async move {
                let url = input
                    .url
                    .or_else(|| state.database_url.clone())
                    .ok_or_else(|| ToolError::new("No Redis URL provided or configured"))?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let result: Vec<(String, String)> = redis::cmd("HGETALL")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("HGETALL failed: {}", e)))?;

                if result.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "(empty hash or key '{}' not found)",
                        input.key
                    )));
                }

                let output = result
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(CallToolResult::text(format!(
                    "Hash '{}' ({} fields):\n{}",
                    input.key,
                    result.len(),
                    output
                )))
            },
        )
        .build()
}

/// Input for LRANGE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LrangeInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// List key
    pub key: String,
    /// Start index (0-based)
    #[serde(default)]
    pub start: i64,
    /// Stop index (-1 for all)
    #[serde(default = "default_stop")]
    pub stop: i64,
}

fn default_stop() -> i64 {
    -1
}

/// Build the lrange tool
pub fn lrange(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_lrange")
        .description("Get a range of elements from a list. Use start=0, stop=-1 for all elements.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, LrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LrangeInput>| async move {
                let url = input
                    .url
                    .or_else(|| state.database_url.clone())
                    .ok_or_else(|| ToolError::new("No Redis URL provided or configured"))?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let result: Vec<String> = redis::cmd("LRANGE")
                    .arg(&input.key)
                    .arg(input.start)
                    .arg(input.stop)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("LRANGE failed: {}", e)))?;

                if result.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "(empty list or key '{}' not found)",
                        input.key
                    )));
                }

                let output = result
                    .iter()
                    .enumerate()
                    .map(|(i, v)| format!("{}: {}", i, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(CallToolResult::text(format!(
                    "List '{}' ({} elements):\n{}",
                    input.key,
                    result.len(),
                    output
                )))
            },
        )
        .build()
}

/// Input for SMEMBERS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SmembersInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Set key
    pub key: String,
}

/// Build the smembers tool
pub fn smembers(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_smembers")
        .description("Get all members of a set")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, SmembersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SmembersInput>| async move {
                let url = input
                    .url
                    .or_else(|| state.database_url.clone())
                    .ok_or_else(|| ToolError::new("No Redis URL provided or configured"))?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let result: Vec<String> = redis::cmd("SMEMBERS")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("SMEMBERS failed: {}", e)))?;

                if result.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "(empty set or key '{}' not found)",
                        input.key
                    )));
                }

                Ok(CallToolResult::text(format!(
                    "Set '{}' ({} members):\n{}",
                    input.key,
                    result.len(),
                    result.join("\n")
                )))
            },
        )
        .build()
}

/// Input for ZRANGE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZrangeInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Start index (0-based)
    #[serde(default)]
    pub start: i64,
    /// Stop index (-1 for all)
    #[serde(default = "default_stop")]
    pub stop: i64,
    /// Include scores in output
    #[serde(default)]
    pub withscores: bool,
}

/// Build the zrange tool
pub fn zrange(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zrange")
        .description("Get a range of members from a sorted set by index. Use withscores=true to include scores.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ZrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZrangeInput>| async move {
                let url = input
                    .url
                    .or_else(|| state.database_url.clone())
                    .ok_or_else(|| ToolError::new("No Redis URL provided or configured"))?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                if input.withscores {
                    let result: Vec<(String, f64)> = redis::cmd("ZRANGE")
                        .arg(&input.key)
                        .arg(input.start)
                        .arg(input.stop)
                        .arg("WITHSCORES")
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| ToolError::new(format!("ZRANGE failed: {}", e)))?;

                    if result.is_empty() {
                        return Ok(CallToolResult::text(format!(
                            "(empty sorted set or key '{}' not found)",
                            input.key
                        )));
                    }

                    let output = result
                        .iter()
                        .enumerate()
                        .map(|(i, (member, score))| format!("{}: {} (score: {})", i, member, score))
                        .collect::<Vec<_>>()
                        .join("\n");

                    Ok(CallToolResult::text(format!(
                        "Sorted set '{}' ({} members):\n{}",
                        input.key,
                        result.len(),
                        output
                    )))
                } else {
                    let result: Vec<String> = redis::cmd("ZRANGE")
                        .arg(&input.key)
                        .arg(input.start)
                        .arg(input.stop)
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| ToolError::new(format!("ZRANGE failed: {}", e)))?;

                    if result.is_empty() {
                        return Ok(CallToolResult::text(format!(
                            "(empty sorted set or key '{}' not found)",
                            input.key
                        )));
                    }

                    let output = result
                        .iter()
                        .enumerate()
                        .map(|(i, v)| format!("{}: {}", i, v))
                        .collect::<Vec<_>>()
                        .join("\n");

                    Ok(CallToolResult::text(format!(
                        "Sorted set '{}' ({} members):\n{}",
                        input.key,
                        result.len(),
                        output
                    )))
                }
            },
        )
        .build()
}
