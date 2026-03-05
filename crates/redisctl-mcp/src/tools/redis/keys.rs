//! Key-level Redis tools (keys, scan, get, key_type, ttl, exists, memory_usage, object_encoding,
//! object_freq, object_idletime, object_help, set, del, expire, rename, mget, mset, persist,
//! unlink, copy, dump, restore, randomkey, touch, incr, decr, append, strlen, getrange, setrange,
//! setnx)

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use super::format_value;

use crate::state::AppState;

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
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
    "redis_mget",
    "redis_mset",
    "redis_persist",
    "redis_unlink",
    "redis_copy",
    "redis_dump",
    "redis_restore",
    "redis_randomkey",
    "redis_touch",
    "redis_incr",
    "redis_decr",
    "redis_append",
    "redis_strlen",
    "redis_getrange",
    "redis_setrange",
    "redis_setnx",
];

/// Build a sub-router containing all key-level Redis tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(keys(state.clone()))
        .tool(scan(state.clone()))
        .tool(get(state.clone()))
        .tool(key_type(state.clone()))
        .tool(ttl(state.clone()))
        .tool(exists(state.clone()))
        .tool(memory_usage(state.clone()))
        .tool(object_encoding(state.clone()))
        .tool(object_freq(state.clone()))
        .tool(object_idletime(state.clone()))
        .tool(object_help(state.clone()))
        .tool(set(state.clone()))
        .tool(del(state.clone()))
        .tool(expire(state.clone()))
        .tool(rename(state.clone()))
        .tool(mget(state.clone()))
        .tool(mset(state.clone()))
        .tool(persist(state.clone()))
        .tool(unlink(state.clone()))
        .tool(copy(state.clone()))
        .tool(dump(state.clone()))
        .tool(restore(state.clone()))
        .tool(randomkey(state.clone()))
        .tool(touch(state.clone()))
        .tool(incr(state.clone()))
        .tool(decr(state.clone()))
        .tool(append(state.clone()))
        .tool(strlen(state.clone()))
        .tool(getrange(state.clone()))
        .tool(setrange(state.clone()))
        .tool(setnx(state))
}

/// Input for keys command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeysInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key pattern to match (default: "*")
    #[serde(default = "default_pattern")]
    pub pattern: String,
    /// Maximum number of keys to return (default: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_pattern() -> String {
    "*".to_string()
}

fn default_limit() -> usize {
    100
}

/// Build the keys tool
pub fn keys(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_keys")
        .description("List keys matching a pattern using SCAN (production-safe, non-blocking).")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<KeysInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                // Use SCAN to safely iterate keys
                let mut cursor: u64 = 0;
                let mut all_keys: Vec<String> = Vec::new();

                loop {
                    let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                        .arg(cursor)
                        .arg("MATCH")
                        .arg(&input.pattern)
                        .arg("COUNT")
                        .arg(100)
                        .query_async(&mut conn)
                        .await
                        .tool_context("SCAN failed")?;

                    all_keys.extend(keys);
                    cursor = new_cursor;

                    if cursor == 0 || all_keys.len() >= input.limit {
                        break;
                    }
                }

                // Truncate to limit
                all_keys.truncate(input.limit);

                let output = if all_keys.is_empty() {
                    format!("No keys found matching pattern '{}'", input.pattern)
                } else {
                    format!(
                        "Found {} key(s) matching '{}'\n\n{}",
                        all_keys.len(),
                        input.pattern,
                        all_keys.join("\n")
                    )
                };

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// Input for GET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to get
    pub key: String,
}

/// Build the get tool
pub fn get(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_get")
        .description("Get the value of a key.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let value: Option<String> = redis::cmd("GET")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("GET failed")?;

                match value {
                    Some(v) => Ok(CallToolResult::text(v)),
                    None => Ok(CallToolResult::text(format!(
                        "(nil) - key '{}' not found",
                        input.key
                    ))),
                }
            },
        )
        .build()
}

/// Input for TYPE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TypeInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to check type
    pub key: String,
}

/// Build the type tool
pub fn key_type(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_type")
        .description("Get the data type of a key.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<TypeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let key_type: String = redis::cmd("TYPE")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("TYPE failed")?;

                Ok(CallToolResult::text(format!("{}: {}", input.key, key_type)))
            },
        )
        .build()
}

/// Input for TTL command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TtlInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to check TTL
    pub key: String,
}

/// Build the ttl tool
pub fn ttl(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_ttl")
        .description("Get the TTL of a key in seconds (-1 = no expiry, -2 = missing).")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<TtlInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let ttl: i64 = redis::cmd("TTL")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("TTL failed")?;

                let message = match ttl {
                    -2 => format!("{}: key does not exist", input.key),
                    -1 => format!("{}: no expiry set", input.key),
                    _ => format!("{}: {} seconds remaining", input.key, ttl),
                };

                Ok(CallToolResult::text(message))
            },
        )
        .build()
}

/// Input for EXISTS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExistsInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Keys to check existence
    pub keys: Vec<String>,
}

/// Build the exists tool
pub fn exists(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_exists")
        .description("Check if one or more keys exist.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ExistsInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("EXISTS");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("EXISTS failed")?;

                Ok(CallToolResult::text(format!(
                    "{} of {} key(s) exist",
                    count,
                    input.keys.len()
                )))
            },
        )
        .build()
}

/// Input for MEMORY USAGE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryUsageInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to check memory usage
    pub key: String,
}

/// Build the memory_usage tool
pub fn memory_usage(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_memory_usage")
        .description("Get memory usage of a key in bytes (MEMORY USAGE).")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<MemoryUsageInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let bytes: Option<i64> = redis::cmd("MEMORY")
                    .arg("USAGE")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("MEMORY USAGE failed")?;

                match bytes {
                    Some(b) => Ok(CallToolResult::text(format!("{}: {} bytes", input.key, b))),
                    None => Ok(CallToolResult::text(format!(
                        "{}: key does not exist",
                        input.key
                    ))),
                }
            },
        )
        .build()
}

/// Input for SCAN with type filter
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScanInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key pattern to match (default: "*")
    #[serde(default = "default_pattern")]
    pub pattern: String,
    /// Filter by key type (e.g., "string", "list", "set", "zset", "hash", "stream")
    #[serde(default)]
    pub key_type: Option<String>,
    /// Maximum number of keys to return (default: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Build the scan tool with type filter
pub fn scan(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_scan")
        .description(
            "Scan keys with optional type filter. Prefer over redis_keys when filtering by type.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ScanInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cursor: u64 = 0;
                let mut all_keys: Vec<String> = Vec::new();

                loop {
                    let mut cmd = redis::cmd("SCAN");
                    cmd.arg(cursor)
                        .arg("MATCH")
                        .arg(&input.pattern)
                        .arg("COUNT")
                        .arg(100);

                    // Add TYPE filter if specified
                    if let Some(ref key_type) = input.key_type {
                        cmd.arg("TYPE").arg(key_type);
                    }

                    let (new_cursor, keys): (u64, Vec<String>) = cmd
                        .query_async(&mut conn)
                        .await
                        .tool_context("SCAN failed")?;

                    all_keys.extend(keys);
                    cursor = new_cursor;

                    if cursor == 0 || all_keys.len() >= input.limit {
                        break;
                    }
                }

                all_keys.truncate(input.limit);

                let type_info = input
                    .key_type
                    .as_ref()
                    .map(|t| format!(" of type '{}'", t))
                    .unwrap_or_default();

                let output = if all_keys.is_empty() {
                    format!(
                        "No keys{} found matching pattern '{}'",
                        type_info, input.pattern
                    )
                } else {
                    format!(
                        "Found {} key(s){} matching '{}'\n\n{}",
                        all_keys.len(),
                        type_info,
                        input.pattern,
                        all_keys.join("\n")
                    )
                };

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// Input for OBJECT ENCODING command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ObjectEncodingInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to check encoding
    pub key: String,
}

/// Build the object_encoding tool
pub fn object_encoding(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_object_encoding")
        .description(
            "Get the internal encoding of a key. Useful for understanding memory usage patterns.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ObjectEncodingInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let encoding: Option<String> = redis::cmd("OBJECT")
                    .arg("ENCODING")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("OBJECT ENCODING failed")?;

                match encoding {
                    Some(enc) => Ok(CallToolResult::text(format!("{}: {}", input.key, enc))),
                    None => Ok(CallToolResult::text(format!(
                        "{}: key does not exist",
                        input.key
                    ))),
                }
            },
        )
        .build()
}

/// Input for OBJECT FREQ command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ObjectFreqInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to get LFU access frequency for
    pub key: String,
}

/// Build the object_freq tool
pub fn object_freq(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_object_freq")
        .description(
            "Get the LFU access frequency counter for a key. \
             Only works with allkeys-lfu or volatile-lfu eviction policy.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ObjectFreqInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let freq: i64 = redis::cmd("OBJECT")
                    .arg("FREQ")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("OBJECT FREQ failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: LFU frequency counter = {}",
                    input.key, freq
                )))
            },
        )
        .build()
}

/// Input for OBJECT IDLETIME command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ObjectIdletimeInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to get idle time for
    pub key: String,
}

/// Build the object_idletime tool
pub fn object_idletime(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_object_idletime")
        .description("Get idle time of a key in seconds since last access.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ObjectIdletimeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let idle: i64 = redis::cmd("OBJECT")
                    .arg("IDLETIME")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("OBJECT IDLETIME failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: idle for {} seconds",
                    input.key, idle
                )))
            },
        )
        .build()
}

/// Input for OBJECT HELP command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ObjectHelpInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the object_help tool
pub fn object_help(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_object_help")
        .description("Get available OBJECT subcommands.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ObjectHelpInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: Vec<String> = redis::cmd("OBJECT")
                    .arg("HELP")
                    .query_async(&mut conn)
                    .await
                    .tool_context("OBJECT HELP failed")?;

                Ok(CallToolResult::text(format!(
                    "OBJECT subcommands:\n{}",
                    result.join("\n")
                )))
            },
        )
        .build()
}

// --- Write tools ---

/// Input for SET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to set
    pub key: String,
    /// Value to set
    pub value: String,
    /// Expire time in seconds
    #[serde(default)]
    pub ex: Option<u64>,
    /// Expire time in milliseconds
    #[serde(default)]
    pub px: Option<u64>,
    /// Only set if key does not already exist
    #[serde(default)]
    pub nx: bool,
    /// Only set if key already exists
    #[serde(default)]
    pub xx: bool,
}

/// Build the set tool
pub fn set(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_set")
        .description(
            "Set a key to a string value with optional expiry and conditional flags (NX/XX).",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SetInput>| async move {
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

                let mut cmd = redis::cmd("SET");
                cmd.arg(&input.key).arg(&input.value);

                if let Some(ex) = input.ex {
                    cmd.arg("EX").arg(ex);
                }
                if let Some(px) = input.px {
                    cmd.arg("PX").arg(px);
                }
                if input.nx {
                    cmd.arg("NX");
                }
                if input.xx {
                    cmd.arg("XX");
                }

                let result: Option<String> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("SET failed")?;

                match result {
                    Some(_) => Ok(CallToolResult::text(format!(
                        "OK - set '{}' successfully",
                        input.key
                    ))),
                    None => Ok(CallToolResult::text(format!(
                        "Key '{}' not set (condition not met: {})",
                        input.key,
                        if input.nx {
                            "NX - key already exists"
                        } else {
                            "XX - key does not exist"
                        }
                    ))),
                }
            },
        )
        .build()
}

/// Input for DEL command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DelInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Keys to delete
    pub keys: Vec<String>,
}

/// Build the del tool
pub fn del(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_del")
        .description("DANGEROUS: Delete one or more keys.")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DelInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("DEL");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("DEL failed")?;

                Ok(CallToolResult::text(format!(
                    "Deleted {} of {} key(s)",
                    count,
                    input.keys.len()
                )))
            },
        )
        .build()
}

/// Input for EXPIRE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExpireInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to set expiry on
    pub key: String,
    /// TTL in seconds
    pub seconds: i64,
}

/// Build the expire tool
pub fn expire(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_expire")
        .description("Set a timeout on a key in seconds. Key auto-deletes after expiry.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ExpireInput>| async move {
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

                let result: bool = redis::cmd("EXPIRE")
                    .arg(&input.key)
                    .arg(input.seconds)
                    .query_async(&mut conn)
                    .await
                    .tool_context("EXPIRE failed")?;

                if result {
                    Ok(CallToolResult::text(format!(
                        "OK - TTL set to {} seconds on '{}'",
                        input.seconds, input.key
                    )))
                } else {
                    Ok(CallToolResult::text(format!(
                        "Key '{}' does not exist or timeout could not be set",
                        input.key
                    )))
                }
            },
        )
        .build()
}

/// Input for RENAME command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Current key name
    pub key: String,
    /// New key name
    pub newkey: String,
}

/// Build the rename tool
pub fn rename(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_rename")
        .description("Rename a key. Overwrites the destination key if it exists.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RenameInput>| async move {
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

                let _: () = redis::cmd("RENAME")
                    .arg(&input.key)
                    .arg(&input.newkey)
                    .query_async(&mut conn)
                    .await
                    .tool_context("RENAME failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - renamed '{}' to '{}'",
                    input.key, input.newkey
                )))
            },
        )
        .build()
}

// --- P0 Key Operations ---

/// Input for MGET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MgetInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Keys to get
    pub keys: Vec<String>,
}

/// Build the mget tool
pub fn mget(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_mget")
        .description("Get the values of multiple keys in a single call.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<MgetInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("MGET");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let values: Vec<redis::Value> = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("MGET failed")?;

                let output = input
                    .keys
                    .iter()
                    .zip(values.iter())
                    .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// A key-value pair for MSET
#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeyValuePair {
    /// Key name
    pub key: String,
    /// Value to set
    pub value: String,
}

/// Input for MSET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MsetInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key-value pairs to set
    pub entries: Vec<KeyValuePair>,
}

/// Build the mset tool
pub fn mset(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_mset")
        .description("Set multiple key-value pairs in a single atomic call.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<MsetInput>| async move {
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

                let mut cmd = redis::cmd("MSET");
                for entry in &input.entries {
                    cmd.arg(&entry.key).arg(&entry.value);
                }

                let _: () = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("MSET failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - set {} key(s)",
                    input.entries.len()
                )))
            },
        )
        .build()
}

/// Input for PERSIST command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PersistInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to remove expiry from
    pub key: String,
}

/// Build the persist tool
pub fn persist(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_persist")
        .description("Remove the expiry from a key, making it persistent.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<PersistInput>| async move {
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

                let result: bool = redis::cmd("PERSIST")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("PERSIST failed")?;

                if result {
                    Ok(CallToolResult::text(format!(
                        "OK - expiry removed from '{}'",
                        input.key
                    )))
                } else {
                    Ok(CallToolResult::text(format!(
                        "Key '{}' does not exist or has no expiry",
                        input.key
                    )))
                }
            },
        )
        .build()
}

/// Input for UNLINK command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnlinkInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Keys to unlink (async delete)
    pub keys: Vec<String>,
}

/// Build the unlink tool
pub fn unlink(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_unlink")
        .description(
            "DANGEROUS: Asynchronously delete one or more keys (non-blocking version of DEL).",
        )
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UnlinkInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("UNLINK");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("UNLINK failed")?;

                Ok(CallToolResult::text(format!(
                    "Unlinked {} of {} key(s)",
                    count,
                    input.keys.len()
                )))
            },
        )
        .build()
}

/// Input for COPY command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CopyInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Source key
    pub source: String,
    /// Destination key
    pub destination: String,
    /// Replace destination key if it already exists
    #[serde(default)]
    pub replace: bool,
}

/// Build the copy tool
pub fn copy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_copy")
        .description("Copy a key to a new key. Use replace=true to overwrite the destination.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<CopyInput>| async move {
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

                let mut cmd = redis::cmd("COPY");
                cmd.arg(&input.source).arg(&input.destination);
                if input.replace {
                    cmd.arg("REPLACE");
                }

                let result: bool = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("COPY failed")?;

                if result {
                    Ok(CallToolResult::text(format!(
                        "OK - copied '{}' to '{}'",
                        input.source, input.destination
                    )))
                } else {
                    Ok(CallToolResult::text(format!(
                        "COPY failed: destination '{}' already exists (use replace=true to overwrite)",
                        input.destination
                    )))
                }
            },
        )
        .build()
}

/// Input for DUMP command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DumpInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to dump
    pub key: String,
}

/// Build the dump tool
pub fn dump(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_dump")
        .description(
            "Serialize a key's value using Redis internal format. Returns hex-encoded bytes \
             for use with RESTORE.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DumpInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let value: redis::Value = redis::cmd("DUMP")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("DUMP failed")?;

                match value {
                    redis::Value::BulkString(bytes) => {
                        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                        Ok(CallToolResult::text(format!(
                            "{}: {} bytes\n{}",
                            input.key,
                            bytes.len(),
                            hex
                        )))
                    }
                    redis::Value::Nil => Ok(CallToolResult::text(format!(
                        "(nil) - key '{}' not found",
                        input.key
                    ))),
                    _ => Ok(CallToolResult::text(format_value(&value))),
                }
            },
        )
        .build()
}

/// Input for RESTORE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestoreInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to restore
    pub key: String,
    /// TTL in milliseconds (0 = no expiry)
    pub ttl_ms: u64,
    /// Hex-encoded serialized value from DUMP
    pub serialized_value: String,
}

/// Build the restore tool
pub fn restore(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_restore")
        .description(
            "Restore a key from a serialized value (from DUMP). \
             The serialized_value must be hex-encoded.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RestoreInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                // Decode hex string to bytes
                let bytes: Result<Vec<u8>, _> = (0..input.serialized_value.len())
                    .step_by(2)
                    .map(|i| {
                        u8::from_str_radix(
                            &input.serialized_value[i..i.min(input.serialized_value.len()) + 2],
                            16,
                        )
                    })
                    .collect();

                let bytes =
                    bytes.map_err(|_| McpError::tool("Invalid hex string in serialized_value"))?;

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let _: () = redis::cmd("RESTORE")
                    .arg(&input.key)
                    .arg(input.ttl_ms)
                    .arg(bytes.as_slice())
                    .query_async(&mut conn)
                    .await
                    .tool_context("RESTORE failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - restored key '{}'",
                    input.key
                )))
            },
        )
        .build()
}

/// Input for RANDOMKEY command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RandomkeyInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the randomkey tool
pub fn randomkey(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_randomkey")
        .description("Return a random key from the database.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RandomkeyInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let key: Option<String> = redis::cmd("RANDOMKEY")
                    .query_async(&mut conn)
                    .await
                    .tool_context("RANDOMKEY failed")?;

                match key {
                    Some(k) => Ok(CallToolResult::text(k)),
                    None => Ok(CallToolResult::text("(empty) - database has no keys")),
                }
            },
        )
        .build()
}

/// Input for TOUCH command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TouchInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Keys to touch (update last access time)
    pub keys: Vec<String>,
}

/// Build the touch tool
pub fn touch(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_touch")
        .description("Update the last access time of one or more keys without modifying them.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<TouchInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("TOUCH");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("TOUCH failed")?;

                Ok(CallToolResult::text(format!(
                    "Touched {} of {} key(s)",
                    count,
                    input.keys.len()
                )))
            },
        )
        .build()
}

// --- P1 String Operations ---

/// Input for INCR command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct IncrInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to increment
    pub key: String,
}

/// Build the incr tool
pub fn incr(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_incr")
        .description("Increment the integer value of a key by 1. Creates the key with value 1 if it does not exist.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<IncrInput>| async move {
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

                let value: i64 = redis::cmd("INCR")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("INCR failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {}",
                    input.key, value
                )))
            },
        )
        .build()
}

/// Input for DECR command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DecrInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to decrement
    pub key: String,
}

/// Build the decr tool
pub fn decr(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_decr")
        .description("Decrement the integer value of a key by 1. Creates the key with value -1 if it does not exist.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DecrInput>| async move {
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

                let value: i64 = redis::cmd("DECR")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("DECR failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {}",
                    input.key, value
                )))
            },
        )
        .build()
}

/// Input for APPEND command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to append to
    pub key: String,
    /// Value to append
    pub value: String,
}

/// Build the append tool
pub fn append(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_append")
        .description("Append a value to a key. Creates the key if it does not exist. Returns the new string length.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<AppendInput>| async move {
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

                let length: i64 = redis::cmd("APPEND")
                    .arg(&input.key)
                    .arg(&input.value)
                    .query_async(&mut conn)
                    .await
                    .tool_context("APPEND failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - '{}' new length: {}",
                    input.key, length
                )))
            },
        )
        .build()
}

/// Input for STRLEN command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StrlenInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to get string length of
    pub key: String,
}

/// Build the strlen tool
pub fn strlen(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_strlen")
        .description("Get the length of the string value stored at a key.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<StrlenInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let length: i64 = redis::cmd("STRLEN")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .tool_context("STRLEN failed")?;

                Ok(CallToolResult::text(format!(
                    "{}: {} bytes",
                    input.key, length
                )))
            },
        )
        .build()
}

/// Input for GETRANGE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetrangeInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to get substring from
    pub key: String,
    /// Start offset (0-based, negative counts from end)
    pub start: i64,
    /// End offset (inclusive, negative counts from end)
    pub end: i64,
}

/// Build the getrange tool
pub fn getrange(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_getrange")
        .description(
            "Get a substring of the string value at a key by start and end offsets (inclusive).",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetrangeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let value: String = redis::cmd("GETRANGE")
                    .arg(&input.key)
                    .arg(input.start)
                    .arg(input.end)
                    .query_async(&mut conn)
                    .await
                    .tool_context("GETRANGE failed")?;

                Ok(CallToolResult::text(value))
            },
        )
        .build()
}

/// Input for SETRANGE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetrangeInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to overwrite substring in
    pub key: String,
    /// Byte offset to start overwriting at
    pub offset: u64,
    /// Value to write at the offset
    pub value: String,
}

/// Build the setrange tool
pub fn setrange(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_setrange")
        .description("Overwrite part of a string value at the given byte offset. Returns the new string length.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SetrangeInput>| async move {
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

                let length: i64 = redis::cmd("SETRANGE")
                    .arg(&input.key)
                    .arg(input.offset)
                    .arg(&input.value)
                    .query_async(&mut conn)
                    .await
                    .tool_context("SETRANGE failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - '{}' new length: {}",
                    input.key, length
                )))
            },
        )
        .build()
}

/// Input for SETNX command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetnxInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Key to set
    pub key: String,
    /// Value to set
    pub value: String,
}

/// Build the setnx tool
pub fn setnx(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_setnx")
        .description(
            "Set a key only if it does not already exist. Returns whether the key was set.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SetnxInput>| async move {
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

                let was_set: bool = redis::cmd("SETNX")
                    .arg(&input.key)
                    .arg(&input.value)
                    .query_async(&mut conn)
                    .await
                    .tool_context("SETNX failed")?;

                if was_set {
                    Ok(CallToolResult::text(format!(
                        "OK - set '{}' (key was new)",
                        input.key
                    )))
                } else {
                    Ok(CallToolResult::text(format!(
                        "Key '{}' already exists, not set",
                        input.key
                    )))
                }
            },
        )
        .build()
}
