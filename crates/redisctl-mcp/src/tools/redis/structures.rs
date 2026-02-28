//! Data structure Redis tools (hgetall, lrange, smembers, zrange, xinfo_stream, xrange, xlen,
//! pubsub_channels, pubsub_numsub, hset, hdel, lpush, rpush, lpop, rpop, sadd, srem, zadd,
//! zrem, xadd, xtrim)

use std::collections::HashMap;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;

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
        .tool(hset(state.clone()))
        .tool(hdel(state.clone()))
        .tool(lpush(state.clone()))
        .tool(rpush(state.clone()))
        .tool(lpop(state.clone()))
        .tool(rpop(state.clone()))
        .tool(sadd(state.clone()))
        .tool(srem(state.clone()))
        .tool(zadd(state.clone()))
        .tool(zrem(state.clone()))
        .tool(xadd(state.clone()))
        .tool(xtrim(state))
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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HgetallInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HgetallInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: Vec<(String, String)> = redis::cmd("HGETALL")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HGETALL failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, LrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LrangeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: Vec<String> = redis::cmd("LRANGE")
                    .arg(&input.key)
                    .arg(input.start)
                    .arg(input.stop)
                    .query_async(&mut conn)
                    .await
                    .tool_context("LRANGE failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SmembersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SmembersInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: Vec<String> = redis::cmd("SMEMBERS")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("SMEMBERS failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZrangeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                if input.withscores {
                    let result: Vec<(String, f64)> = redis::cmd("ZRANGE")
                        .arg(&input.key)
                        .arg(input.start)
                        .arg(input.stop)
                        .arg("WITHSCORES")
                        .query_async(&mut conn)
                        .await
                        .tool_context("ZRANGE failed")?;

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
                        .tool_context("ZRANGE failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, XinfoStreamInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XinfoStreamInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: redis::Value = redis::cmd("XINFO")
                    .arg("STREAM")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("XINFO STREAM failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, XrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XrangeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("XRANGE");
                cmd.arg(&input.key).arg(&input.start).arg(&input.end);

                if let Some(count) = input.count {
                    cmd.arg("COUNT").arg(count);
                }

                let result: redis::Value = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("XRANGE failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, XlenInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XlenInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let len: i64 = redis::cmd("XLEN")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("XLEN failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, PubsubChannelsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<PubsubChannelsInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("PUBSUB");
                cmd.arg("CHANNELS");

                if let Some(ref pattern) = input.pattern {
                    cmd.arg(pattern);
                }

                let channels: Vec<String> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("PUBSUB CHANNELS failed")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, PubsubNumsubInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<PubsubNumsubInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

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
                    .tool_context("PUBSUB NUMSUB failed")?;

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

// --- Write tools ---

/// Input for HSET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HsetInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
    /// Field-value pairs to set
    pub fields: HashMap<String, String>,
}

/// Build the hset tool
pub fn hset(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hset")
        .description(
            "Set one or more field-value pairs in a hash. Creates the hash if it does not \
             exist. Returns the number of fields that were added (not updated).",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, HsetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HsetInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("HSET");
                cmd.arg(&input.key);
                for (field, value) in &input.fields {
                    cmd.arg(field).arg(value);
                }

                let added: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("HSET failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - {} field(s) added to hash '{}' ({} field(s) set total)",
                    added,
                    input.key,
                    input.fields.len()
                )))
            },
        )
        .build()
}

/// Input for HDEL command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HdelInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
    /// Fields to delete
    pub fields: Vec<String>,
}

/// Build the hdel tool
pub fn hdel(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hdel")
        .description(
            "Delete one or more fields from a hash. Returns the number of fields that were removed.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, HdelInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HdelInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("HDEL");
                cmd.arg(&input.key);
                for field in &input.fields {
                    cmd.arg(field);
                }

                let removed: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("HDEL failed")?;

                Ok(CallToolResult::text(format!(
                    "Deleted {} of {} field(s) from hash '{}'",
                    removed,
                    input.fields.len(),
                    input.key
                )))
            },
        )
        .build()
}

/// Input for LPUSH command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LpushInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// List key
    pub key: String,
    /// Elements to push to the head of the list
    pub elements: Vec<String>,
}

/// Build the lpush tool
pub fn lpush(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_lpush")
        .description(
            "Push one or more elements to the head (left) of a list. Creates the list \
             if it does not exist. Returns the new list length.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, LpushInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LpushInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("LPUSH");
                cmd.arg(&input.key);
                for elem in &input.elements {
                    cmd.arg(elem);
                }

                let length: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("LPUSH failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - pushed {} element(s) to '{}', new length: {}",
                    input.elements.len(),
                    input.key,
                    length
                )))
            },
        )
        .build()
}

/// Input for RPUSH command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RpushInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// List key
    pub key: String,
    /// Elements to push to the tail of the list
    pub elements: Vec<String>,
}

/// Build the rpush tool
pub fn rpush(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_rpush")
        .description(
            "Push one or more elements to the tail (right) of a list. Creates the list \
             if it does not exist. Returns the new list length.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RpushInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RpushInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("RPUSH");
                cmd.arg(&input.key);
                for elem in &input.elements {
                    cmd.arg(elem);
                }

                let length: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("RPUSH failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - pushed {} element(s) to '{}', new length: {}",
                    input.elements.len(),
                    input.key,
                    length
                )))
            },
        )
        .build()
}

/// Input for LPOP command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LpopInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// List key
    pub key: String,
    /// Number of elements to pop (default: 1)
    #[serde(default)]
    pub count: Option<u64>,
}

/// Build the lpop tool
pub fn lpop(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_lpop")
        .description(
            "Pop one or more elements from the head (left) of a list. Returns the \
             popped element(s), or nil if the list is empty.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, LpopInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LpopInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("LPOP");
                cmd.arg(&input.key);
                if let Some(count) = input.count {
                    cmd.arg(count);
                }

                let result: redis::Value = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("LPOP failed")?;

                Ok(CallToolResult::text(format!(
                    "LPOP '{}': {}",
                    input.key,
                    super::format_value(&result)
                )))
            },
        )
        .build()
}

/// Input for RPOP command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RpopInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// List key
    pub key: String,
    /// Number of elements to pop (default: 1)
    #[serde(default)]
    pub count: Option<u64>,
}

/// Build the rpop tool
pub fn rpop(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_rpop")
        .description(
            "Pop one or more elements from the tail (right) of a list. Returns the \
             popped element(s), or nil if the list is empty.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RpopInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RpopInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("RPOP");
                cmd.arg(&input.key);
                if let Some(count) = input.count {
                    cmd.arg(count);
                }

                let result: redis::Value = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("RPOP failed")?;

                Ok(CallToolResult::text(format!(
                    "RPOP '{}': {}",
                    input.key,
                    super::format_value(&result)
                )))
            },
        )
        .build()
}

/// Input for SADD command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaddInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Set key
    pub key: String,
    /// Members to add to the set
    pub members: Vec<String>,
}

/// Build the sadd tool
pub fn sadd(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_sadd")
        .description(
            "Add one or more members to a set. Creates the set if it does not exist. \
             Returns the number of members that were added (not already present).",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, SaddInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SaddInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("SADD");
                cmd.arg(&input.key);
                for member in &input.members {
                    cmd.arg(member);
                }

                let added: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("SADD failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - added {} of {} member(s) to set '{}'",
                    added,
                    input.members.len(),
                    input.key
                )))
            },
        )
        .build()
}

/// Input for SREM command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SremInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Set key
    pub key: String,
    /// Members to remove from the set
    pub members: Vec<String>,
}

/// Build the srem tool
pub fn srem(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_srem")
        .description(
            "Remove one or more members from a set. Returns the number of members \
             that were removed.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, SremInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SremInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("SREM");
                cmd.arg(&input.key);
                for member in &input.members {
                    cmd.arg(member);
                }

                let removed: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("SREM failed")?;

                Ok(CallToolResult::text(format!(
                    "Removed {} of {} member(s) from set '{}'",
                    removed,
                    input.members.len(),
                    input.key
                )))
            },
        )
        .build()
}

/// A score-member pair for sorted set operations
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScoreMember {
    /// Score value
    pub score: f64,
    /// Member value
    pub member: String,
}

/// Input for ZADD command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZaddInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Score-member pairs to add
    pub members: Vec<ScoreMember>,
    /// Only add new elements, do not update existing ones
    #[serde(default)]
    pub nx: bool,
    /// Only update existing elements, do not add new ones
    #[serde(default)]
    pub xx: bool,
    /// Only update elements whose new score is greater than current score
    #[serde(default)]
    pub gt: bool,
    /// Only update elements whose new score is less than current score
    #[serde(default)]
    pub lt: bool,
    /// Return the number of elements changed (added + updated) instead of only added
    #[serde(default)]
    pub ch: bool,
}

/// Build the zadd tool
pub fn zadd(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zadd")
        .description(
            "Add one or more members to a sorted set with scores. Creates the set if it \
             does not exist. Supports NX (only add new), XX (only update existing), \
             GT/LT (score comparison), and CH (count changed) flags.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ZaddInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZaddInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("ZADD");
                cmd.arg(&input.key);

                if input.nx {
                    cmd.arg("NX");
                }
                if input.xx {
                    cmd.arg("XX");
                }
                if input.gt {
                    cmd.arg("GT");
                }
                if input.lt {
                    cmd.arg("LT");
                }
                if input.ch {
                    cmd.arg("CH");
                }

                for sm in &input.members {
                    cmd.arg(sm.score).arg(&sm.member);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("ZADD failed")?;

                let verb = if input.ch { "changed" } else { "added" };
                Ok(CallToolResult::text(format!(
                    "OK - {} {} member(s) in sorted set '{}'",
                    count, verb, input.key
                )))
            },
        )
        .build()
}

/// Input for ZREM command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZremInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Members to remove
    pub members: Vec<String>,
}

/// Build the zrem tool
pub fn zrem(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zrem")
        .description(
            "Remove one or more members from a sorted set. Returns the number of \
             members that were removed.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ZremInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZremInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("ZREM");
                cmd.arg(&input.key);
                for member in &input.members {
                    cmd.arg(member);
                }

                let removed: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("ZREM failed")?;

                Ok(CallToolResult::text(format!(
                    "Removed {} of {} member(s) from sorted set '{}'",
                    removed,
                    input.members.len(),
                    input.key
                )))
            },
        )
        .build()
}

/// Input for XADD command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct XaddInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Stream key
    pub key: String,
    /// Entry ID (default: "*" for auto-generated)
    #[serde(default)]
    pub id: Option<String>,
    /// Field-value pairs for the stream entry
    pub fields: HashMap<String, String>,
    /// Do not create stream if it does not exist
    #[serde(default)]
    pub nomkstream: bool,
    /// Cap stream to a maximum length
    #[serde(default)]
    pub maxlen: Option<u64>,
    /// Cap stream entries older than this ID
    #[serde(default)]
    pub minid: Option<String>,
    /// Use approximate trimming (~) for better performance
    #[serde(default)]
    pub approximate: bool,
}

/// Build the xadd tool
pub fn xadd(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_xadd")
        .description(
            "Append an entry to a stream. Auto-generates an ID by default (\"*\"). \
             Supports NOMKSTREAM, MAXLEN, and MINID trimming options. \
             Returns the ID of the added entry.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, XaddInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XaddInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("XADD");
                cmd.arg(&input.key);

                if input.nomkstream {
                    cmd.arg("NOMKSTREAM");
                }

                if let Some(maxlen) = input.maxlen {
                    cmd.arg("MAXLEN");
                    if input.approximate {
                        cmd.arg("~");
                    }
                    cmd.arg(maxlen);
                } else if let Some(ref minid) = input.minid {
                    cmd.arg("MINID");
                    if input.approximate {
                        cmd.arg("~");
                    }
                    cmd.arg(minid);
                }

                let id = input.id.as_deref().unwrap_or("*");
                cmd.arg(id);

                for (field, value) in &input.fields {
                    cmd.arg(field).arg(value);
                }

                let entry_id: String = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("XADD failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - added entry {} to stream '{}'",
                    entry_id, input.key
                )))
            },
        )
        .build()
}

/// Input for XTRIM command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct XtrimInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Stream key
    pub key: String,
    /// Trimming strategy: "MAXLEN" or "MINID"
    pub strategy: String,
    /// Threshold value (count for MAXLEN, ID for MINID)
    pub threshold: String,
    /// Use approximate trimming (~) for better performance
    #[serde(default)]
    pub approximate: bool,
}

/// Build the xtrim tool
pub fn xtrim(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_xtrim")
        .description(
            "Trim a stream to a given length (MAXLEN) or minimum ID (MINID). \
             Use approximate=true for better performance with near-exact trimming. \
             Returns the number of entries removed.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, XtrimInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XtrimInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("XTRIM");
                cmd.arg(&input.key);
                cmd.arg(&input.strategy);

                if input.approximate {
                    cmd.arg("~");
                }
                cmd.arg(&input.threshold);

                let trimmed: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("XTRIM failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - trimmed {} entries from stream '{}'",
                    trimmed, input.key
                )))
            },
        )
        .build()
}
