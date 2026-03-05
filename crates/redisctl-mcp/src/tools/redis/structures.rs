//! Data structure Redis tools (hgetall, lrange, smembers, zrange, xinfo_stream, xrange, xlen,
//! pubsub_channels, pubsub_numsub, hset, hdel, lpush, rpush, lpop, rpop, sadd, srem, zadd,
//! zrem, xadd, xtrim, hget, hmget, hlen, hexists, hkeys, hvals, hincrby, scard, sismember,
//! sunion, sinter, sdiff, zcard, zscore, zrank, zcount, zrangebyscore, llen, lindex)

use std::collections::HashMap;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
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
    "redis_hget",
    "redis_hmget",
    "redis_hlen",
    "redis_hexists",
    "redis_hkeys",
    "redis_hvals",
    "redis_hincrby",
    "redis_scard",
    "redis_sismember",
    "redis_sunion",
    "redis_sinter",
    "redis_sdiff",
    "redis_zcard",
    "redis_zscore",
    "redis_zrank",
    "redis_zcount",
    "redis_zrangebyscore",
    "redis_llen",
    "redis_lindex",
];

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
        .tool(xtrim(state.clone()))
        .tool(hget(state.clone()))
        .tool(hmget(state.clone()))
        .tool(hlen(state.clone()))
        .tool(hexists(state.clone()))
        .tool(hkeys(state.clone()))
        .tool(hvals(state.clone()))
        .tool(hincrby(state.clone()))
        .tool(scard(state.clone()))
        .tool(sismember(state.clone()))
        .tool(sunion(state.clone()))
        .tool(sinter(state.clone()))
        .tool(sdiff(state.clone()))
        .tool(zcard(state.clone()))
        .tool(zscore(state.clone()))
        .tool(zrank(state.clone()))
        .tool(zcount(state.clone()))
        .tool(zrangebyscore(state.clone()))
        .tool(llen(state.clone()))
        .tool(lindex(state))
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
        .description("Get all fields and values of a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HgetallInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HgetallInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get a range of elements from a list (start=0, stop=-1 for all).")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, LrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LrangeInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get all members of a set.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SmembersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SmembersInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get a range of members from a sorted set by index.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZrangeInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get stream metadata including length, consumer groups, and entry details (XINFO STREAM).")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, XinfoStreamInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XinfoStreamInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get stream entries in a range. Use \"-\" to \"+\" for all entries.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, XrangeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XrangeInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get the number of entries in a stream.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, XlenInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<XlenInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("List active pub/sub channels, optionally filtered by pattern.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, PubsubChannelsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<PubsubChannelsInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Get subscriber counts for pub/sub channels.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, PubsubNumsubInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<PubsubNumsubInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Set one or more field-value pairs in a hash. Creates the hash if needed.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, HsetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HsetInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Delete one or more fields from a hash.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, HdelInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HdelInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Push elements to the head (left) of a list. Creates the list if needed.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, LpushInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LpushInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Push elements to the tail (right) of a list. Creates the list if needed.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RpushInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RpushInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Pop elements from the head (left) of a list.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, LpopInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LpopInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Pop elements from the tail (right) of a list.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RpopInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RpopInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Add one or more members to a set. Creates the set if needed.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, SaddInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SaddInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Remove one or more members from a set.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, SremInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SremInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
            "Add members with scores to a sorted set. Creates the set if needed. \
             Supports NX, XX, GT, LT, and CH flags.",
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

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
        .description("Remove one or more members from a sorted set.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ZremInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZremInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
            "Append an entry to a stream. Supports NOMKSTREAM, MAXLEN, and MINID trimming.",
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

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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
            "Trim a stream by length (MAXLEN) or minimum ID (MINID). \
             Use approximate=true for better performance.",
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

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

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

// --- P1 Hash read tools ---

/// Input for HGET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HgetInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
    /// Field to get
    pub field: String,
}

/// Build the hget tool
pub fn hget(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hget")
        .description("Get the value of a single field in a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HgetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HgetInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let value: Option<String> = redis::cmd("HGET")
                    .arg(&input.key)
                    .arg(&input.field)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HGET failed")?;

                match value {
                    Some(v) => Ok(CallToolResult::text(v)),
                    None => Ok(CallToolResult::text(format!(
                        "(nil) - field '{}' not found in '{}'",
                        input.field, input.key
                    ))),
                }
            },
        )
        .build()
}

/// Input for HMGET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HmgetInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
    /// Fields to get
    pub fields: Vec<String>,
}

/// Build the hmget tool
pub fn hmget(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hmget")
        .description("Get the values of multiple fields in a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HmgetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HmgetInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let mut cmd = redis::cmd("HMGET");
                cmd.arg(&input.key);
                for field in &input.fields {
                    cmd.arg(field);
                }

                let values: Vec<redis::Value> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("HMGET failed")?;

                let output = input
                    .fields
                    .iter()
                    .zip(values.iter())
                    .map(|(f, v)| format!("{}: {}", f, super::format_value(v)))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// Input for HLEN command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HlenInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
}

/// Build the hlen tool
pub fn hlen(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hlen")
        .description("Get the number of fields in a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HlenInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HlenInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let count: i64 = redis::cmd("HLEN")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HLEN failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {} fields",
                    input.key, count
                )))
            },
        )
        .build()
}

/// Input for HEXISTS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HexistsInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
    /// Field to check
    pub field: String,
}

/// Build the hexists tool
pub fn hexists(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hexists")
        .description("Check if a field exists in a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HexistsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HexistsInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let exists: bool = redis::cmd("HEXISTS")
                    .arg(&input.key)
                    .arg(&input.field)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HEXISTS failed")?;

                Ok(CallToolResult::text(format!(
                    "{}.{}: {}",
                    input.key,
                    input.field,
                    if exists { "exists" } else { "does not exist" }
                )))
            },
        )
        .build()
}

/// Input for HKEYS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HkeysInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
}

/// Build the hkeys tool
pub fn hkeys(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hkeys")
        .description("Get all field names in a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HkeysInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HkeysInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let fields: Vec<String> = redis::cmd("HKEYS")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HKEYS failed")?;

                if fields.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "(empty hash or key '{}' not found)",
                        input.key
                    )));
                }

                Ok(CallToolResult::text(format!(
                    "Hash '{}' ({} fields):\n{}",
                    input.key,
                    fields.len(),
                    fields.join("\n")
                )))
            },
        )
        .build()
}

/// Input for HVALS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HvalsInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
}

/// Build the hvals tool
pub fn hvals(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hvals")
        .description("Get all values in a hash.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, HvalsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HvalsInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let values: Vec<String> = redis::cmd("HVALS")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HVALS failed")?;

                if values.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "(empty hash or key '{}' not found)",
                        input.key
                    )));
                }

                Ok(CallToolResult::text(format!(
                    "Hash '{}' ({} values):\n{}",
                    input.key,
                    values.len(),
                    values.join("\n")
                )))
            },
        )
        .build()
}

/// Input for HINCRBY command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HincrbyInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Hash key
    pub key: String,
    /// Field to increment
    pub field: String,
    /// Increment value (can be negative)
    pub increment: i64,
}

/// Build the hincrby tool
pub fn hincrby(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_hincrby")
        .description("Increment the integer value of a hash field by the given amount.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, HincrbyInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<HincrbyInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let value: i64 = redis::cmd("HINCRBY")
                    .arg(&input.key)
                    .arg(&input.field)
                    .arg(input.increment)
                    .query_async(&mut conn)
                    .await
                    .tool_context("HINCRBY failed")?;

                Ok(CallToolResult::text(format!(
                    "{}.{}: {}",
                    input.key, input.field, value
                )))
            },
        )
        .build()
}

// --- P1 Set read tools ---

/// Input for SCARD command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScardInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Set key
    pub key: String,
}

/// Build the scard tool
pub fn scard(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_scard")
        .description("Get the number of members in a set (cardinality).")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ScardInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ScardInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let count: i64 = redis::cmd("SCARD")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("SCARD failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {} members",
                    input.key, count
                )))
            },
        )
        .build()
}

/// Input for SISMEMBER command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SismemberInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Set key
    pub key: String,
    /// Member to check
    pub member: String,
}

/// Build the sismember tool
pub fn sismember(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_sismember")
        .description("Check if a value is a member of a set.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SismemberInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SismemberInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let is_member: bool = redis::cmd("SISMEMBER")
                    .arg(&input.key)
                    .arg(&input.member)
                    .query_async(&mut conn)
                    .await
                    .tool_context("SISMEMBER failed")?;

                Ok(CallToolResult::text(format!(
                    "'{}' {} a member of '{}'",
                    input.member,
                    if is_member { "is" } else { "is not" },
                    input.key
                )))
            },
        )
        .build()
}

/// Input for SUNION command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SunionInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Set keys to compute union of
    pub keys: Vec<String>,
}

/// Build the sunion tool
pub fn sunion(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_sunion")
        .description("Return the union of multiple sets.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SunionInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SunionInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let mut cmd = redis::cmd("SUNION");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let members: Vec<String> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("SUNION failed")?;

                if members.is_empty() {
                    return Ok(CallToolResult::text("(empty set)"));
                }

                Ok(CallToolResult::text(format!(
                    "Union ({} members):\n{}",
                    members.len(),
                    members.join("\n")
                )))
            },
        )
        .build()
}

/// Input for SINTER command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SinterInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Set keys to compute intersection of
    pub keys: Vec<String>,
}

/// Build the sinter tool
pub fn sinter(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_sinter")
        .description("Return the intersection of multiple sets.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SinterInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SinterInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let mut cmd = redis::cmd("SINTER");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let members: Vec<String> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("SINTER failed")?;

                if members.is_empty() {
                    return Ok(CallToolResult::text("(empty set)"));
                }

                Ok(CallToolResult::text(format!(
                    "Intersection ({} members):\n{}",
                    members.len(),
                    members.join("\n")
                )))
            },
        )
        .build()
}

/// Input for SDIFF command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SdiffInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Set keys (first set minus all subsequent sets)
    pub keys: Vec<String>,
}

/// Build the sdiff tool
pub fn sdiff(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_sdiff")
        .description("Return the difference between the first set and all subsequent sets.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SdiffInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SdiffInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let mut cmd = redis::cmd("SDIFF");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let members: Vec<String> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("SDIFF failed")?;

                if members.is_empty() {
                    return Ok(CallToolResult::text("(empty set)"));
                }

                Ok(CallToolResult::text(format!(
                    "Difference ({} members):\n{}",
                    members.len(),
                    members.join("\n")
                )))
            },
        )
        .build()
}

// --- P1 Sorted Set read tools ---

/// Input for ZCARD command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZcardInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
}

/// Build the zcard tool
pub fn zcard(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zcard")
        .description("Get the number of members in a sorted set (cardinality).")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZcardInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZcardInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let count: i64 = redis::cmd("ZCARD")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("ZCARD failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {} members",
                    input.key, count
                )))
            },
        )
        .build()
}

/// Input for ZSCORE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZscoreInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Member to get score for
    pub member: String,
}

/// Build the zscore tool
pub fn zscore(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zscore")
        .description("Get the score of a member in a sorted set.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZscoreInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZscoreInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let score: Option<f64> = redis::cmd("ZSCORE")
                    .arg(&input.key)
                    .arg(&input.member)
                    .query_async(&mut conn)
                    .await
                    .tool_context("ZSCORE failed")?;

                match score {
                    Some(s) => Ok(CallToolResult::text(format!(
                        "{}.{}: {}",
                        input.key, input.member, s
                    ))),
                    None => Ok(CallToolResult::text(format!(
                        "(nil) - '{}' not found in '{}'",
                        input.member, input.key
                    ))),
                }
            },
        )
        .build()
}

/// Input for ZRANK command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZrankInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Member to get rank for
    pub member: String,
}

/// Build the zrank tool
pub fn zrank(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zrank")
        .description(
            "Get the rank (0-based index) of a member in a sorted set, ordered low to high.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZrankInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZrankInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let rank: Option<i64> = redis::cmd("ZRANK")
                    .arg(&input.key)
                    .arg(&input.member)
                    .query_async(&mut conn)
                    .await
                    .tool_context("ZRANK failed")?;

                match rank {
                    Some(r) => Ok(CallToolResult::text(format!(
                        "{}.{}: rank {}",
                        input.key, input.member, r
                    ))),
                    None => Ok(CallToolResult::text(format!(
                        "(nil) - '{}' not found in '{}'",
                        input.member, input.key
                    ))),
                }
            },
        )
        .build()
}

/// Input for ZCOUNT command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZcountInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Minimum score (use "-inf" for no lower bound)
    pub min: String,
    /// Maximum score (use "+inf" for no upper bound)
    pub max: String,
}

/// Build the zcount tool
pub fn zcount(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zcount")
        .description("Count members in a sorted set with scores between min and max (inclusive). Use \"-inf\"/\"+inf\" for unbounded.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZcountInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ZcountInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let count: i64 = redis::cmd("ZCOUNT")
                    .arg(&input.key)
                    .arg(&input.min)
                    .arg(&input.max)
                    .query_async(&mut conn)
                    .await
                    .tool_context("ZCOUNT failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {} members in score range [{}, {}]",
                    input.key, count, input.min, input.max
                )))
            },
        )
        .build()
}

/// Input for ZRANGEBYSCORE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZrangebyscoreInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Sorted set key
    pub key: String,
    /// Minimum score (use "-inf" for no lower bound)
    pub min: String,
    /// Maximum score (use "+inf" for no upper bound)
    pub max: String,
    /// Include scores in output
    #[serde(default)]
    pub withscores: bool,
    /// Offset for pagination (requires count)
    #[serde(default)]
    pub offset: Option<i64>,
    /// Maximum number of results (requires offset)
    #[serde(default)]
    pub count: Option<i64>,
}

/// Build the zrangebyscore tool
pub fn zrangebyscore(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_zrangebyscore")
        .description("Get members from a sorted set with scores in the given range. Use \"-inf\"/\"+inf\" for unbounded.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ZrangebyscoreInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ZrangebyscoreInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let mut cmd = redis::cmd("ZRANGEBYSCORE");
                cmd.arg(&input.key).arg(&input.min).arg(&input.max);

                if input.withscores {
                    cmd.arg("WITHSCORES");
                }

                if let (Some(offset), Some(count)) = (input.offset, input.count) {
                    cmd.arg("LIMIT").arg(offset).arg(count);
                }

                if input.withscores {
                    let result: Vec<(String, f64)> = cmd
                        .query_async(&mut conn)
                        .await
                        .tool_context("ZRANGEBYSCORE failed")?;

                    if result.is_empty() {
                        return Ok(CallToolResult::text(format!(
                            "No members in '{}' with scores in [{}, {}]",
                            input.key, input.min, input.max
                        )));
                    }

                    let output = result
                        .iter()
                        .map(|(member, score)| format!("{} (score: {})", member, score))
                        .collect::<Vec<_>>()
                        .join("\n");

                    Ok(CallToolResult::text(format!(
                        "'{}' ({} members in [{}, {}]):\n{}",
                        input.key,
                        result.len(),
                        input.min,
                        input.max,
                        output
                    )))
                } else {
                    let result: Vec<String> = cmd
                        .query_async(&mut conn)
                        .await
                        .tool_context("ZRANGEBYSCORE failed")?;

                    if result.is_empty() {
                        return Ok(CallToolResult::text(format!(
                            "No members in '{}' with scores in [{}, {}]",
                            input.key, input.min, input.max
                        )));
                    }

                    Ok(CallToolResult::text(format!(
                        "'{}' ({} members in [{}, {}]):\n{}",
                        input.key,
                        result.len(),
                        input.min,
                        input.max,
                        result.join("\n")
                    )))
                }
            },
        )
        .build()
}

// --- P1 List read tools ---

/// Input for LLEN command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlenInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// List key
    pub key: String,
}

/// Build the llen tool
pub fn llen(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_llen")
        .description("Get the length of a list.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, LlenInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LlenInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let length: i64 = redis::cmd("LLEN")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("LLEN failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {} elements",
                    input.key, length
                )))
            },
        )
        .build()
}

/// Input for LINDEX command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LindexInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// List key
    pub key: String,
    /// Index (0-based, negative counts from end)
    pub index: i64,
}

/// Build the lindex tool
pub fn lindex(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_lindex")
        .description("Get an element from a list by its index (0-based, negative counts from end).")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, LindexInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<LindexInput>| async move {
                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let value: Option<String> = redis::cmd("LINDEX")
                    .arg(&input.key)
                    .arg(input.index)
                    .query_async(&mut conn)
                    .await
                    .tool_context("LINDEX failed")?;

                match value {
                    Some(v) => Ok(CallToolResult::text(v)),
                    None => Ok(CallToolResult::text(format!(
                        "(nil) - index {} out of range or key '{}' not found",
                        input.index, input.key
                    ))),
                }
            },
        )
        .build()
}
