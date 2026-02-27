//! Data structure Redis tools (hgetall, lrange, smembers, zrange, xinfo_stream, xrange, xlen,
//! pubsub_channels, pubsub_numsub)

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
- redis_xinfo_stream: Get stream metadata (length, groups, first/last entry)\n\
- redis_xrange: Get stream entries in a range\n\
- redis_xlen: Get stream length\n\
- redis_pubsub_channels: List active pub/sub channels\n\
- redis_pubsub_numsub: Get subscriber counts for channels\n\
";

/// Build a sub-router containing all data structure Redis tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(hgetall(state.clone()))
        .tool(lrange(state.clone()))
        .tool(smembers(state.clone()))
        .tool(zrange(state.clone()))
        .tool(xinfo_stream(state.clone()))
        .tool(xrange(state.clone()))
        .tool(xlen(state.clone()))
        .tool(pubsub_channels(state.clone()))
        .tool(pubsub_numsub(state.clone()))
}

/// Input for HGETALL command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HgetallInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
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
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

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
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
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
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

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
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
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
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

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
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
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
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

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

/// Input for XINFO STREAM command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct XinfoStreamInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Stream key to inspect
    pub key: String,
}

/// Build the xinfo_stream tool
pub fn xinfo_stream(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_xinfo_stream")
        .description(
            "Get stream metadata using XINFO STREAM, including length, consumer groups, \
             first and last entry, and other stream details.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, XinfoStreamInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XinfoStreamInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let result: redis::Value = redis::cmd("XINFO")
                    .arg("STREAM")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("XINFO STREAM failed: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Stream '{}':\n{}",
                    input.key,
                    super::format_value(&result)
                )))
            },
        )
        .build()
}

/// Input for XRANGE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct XrangeInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Stream key
    pub key: String,
    /// Start ID (default: "-" for beginning)
    #[serde(default = "default_xrange_start")]
    pub start: String,
    /// End ID (default: "+" for end)
    #[serde(default = "default_xrange_end")]
    pub end: String,
    /// Maximum number of entries to return
    #[serde(default)]
    pub count: Option<usize>,
}

fn default_xrange_start() -> String {
    "-".to_string()
}

fn default_xrange_end() -> String {
    "+".to_string()
}

/// Build the xrange tool
pub fn xrange(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_xrange")
        .description(
            "Get stream entries in a range using XRANGE. Use start=\"-\" and end=\"+\" \
             for all entries. Optionally limit with count.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, XrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XrangeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let mut cmd = redis::cmd("XRANGE");
                cmd.arg(&input.key).arg(&input.start).arg(&input.end);

                if let Some(count) = input.count {
                    cmd.arg("COUNT").arg(count);
                }

                let result: redis::Value = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("XRANGE failed: {}", e)))?;

                // Format stream entries
                let formatted = match &result {
                    redis::Value::Array(entries) if entries.is_empty() => {
                        format!("(empty stream or key '{}' not found)", input.key)
                    }
                    redis::Value::Array(entries) => {
                        let mut output =
                            format!("Stream '{}' ({} entries):\n", input.key, entries.len());
                        for entry in entries {
                            output.push_str(&super::format_value(entry));
                            output.push('\n');
                        }
                        output
                    }
                    _ => super::format_value(&result),
                };

                Ok(CallToolResult::text(formatted))
            },
        )
        .build()
}

/// Input for XLEN command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct XlenInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Stream key
    pub key: String,
}

/// Build the xlen tool
pub fn xlen(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_xlen")
        .description("Get the number of entries in a stream using XLEN")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, XlenInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XlenInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let len: i64 = redis::cmd("XLEN")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("XLEN failed: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Stream '{}': {} entries",
                    input.key, len
                )))
            },
        )
        .build()
}

/// Input for PUBSUB CHANNELS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PubsubChannelsInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Optional glob-style pattern to filter channels
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Build the pubsub_channels tool
pub fn pubsub_channels(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_pubsub_channels")
        .description(
            "List active pub/sub channels using PUBSUB CHANNELS. \
             Optionally filter with a glob-style pattern.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, PubsubChannelsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<PubsubChannelsInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let mut cmd = redis::cmd("PUBSUB");
                cmd.arg("CHANNELS");

                if let Some(ref pattern) = input.pattern {
                    cmd.arg(pattern);
                }

                let channels: Vec<String> = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("PUBSUB CHANNELS failed: {}", e)))?;

                if channels.is_empty() {
                    return Ok(CallToolResult::text("No active pub/sub channels"));
                }

                Ok(CallToolResult::text(format!(
                    "Active channels ({}):\n{}",
                    channels.len(),
                    channels.join("\n")
                )))
            },
        )
        .build()
}

/// Input for PUBSUB NUMSUB command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PubsubNumsubInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Channel names to get subscriber counts for (omit for all)
    #[serde(default)]
    pub channels: Option<Vec<String>>,
}

/// Build the pubsub_numsub tool
pub fn pubsub_numsub(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_pubsub_numsub")
        .description(
            "Get subscriber counts for pub/sub channels using PUBSUB NUMSUB. \
             Provide channel names to query specific channels.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, PubsubNumsubInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<PubsubNumsubInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let mut cmd = redis::cmd("PUBSUB");
                cmd.arg("NUMSUB");

                if let Some(ref channels) = input.channels {
                    for channel in channels {
                        cmd.arg(channel);
                    }
                }

                // PUBSUB NUMSUB returns alternating channel name + count
                let result: Vec<redis::Value> = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("PUBSUB NUMSUB failed: {}", e)))?;

                if result.is_empty() {
                    return Ok(CallToolResult::text("No subscriber information available"));
                }

                let mut output = String::from("Channel subscriber counts:\n");
                for pair in result.chunks(2) {
                    if pair.len() == 2 {
                        let channel = super::format_value(&pair[0]);
                        let count = super::format_value(&pair[1]);
                        output.push_str(&format!("  {}: {} subscribers\n", channel, count));
                    }
                }

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}
