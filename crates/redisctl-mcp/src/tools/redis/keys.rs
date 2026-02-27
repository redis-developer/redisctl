//! Key-level Redis tools (keys, scan, get, key_type, ttl, exists, memory_usage, object_encoding,
//! object_freq, object_idletime, object_help, set, del, expire, rename)

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

pub(super) const INSTRUCTIONS: &str = "\
### Redis Database - Keys\n\
- redis_keys: List keys matching pattern (SCAN)\n\
- redis_scan: Scan keys with type filter (string, list, set, zset, hash, stream)\n\
- redis_get: Get string value\n\
- redis_type: Get key type\n\
- redis_ttl: Get key TTL\n\
- redis_exists: Check key existence\n\
- redis_memory_usage: Get key memory usage\n\
- redis_object_encoding: Get key internal encoding\n\
- redis_object_freq: Get LFU access frequency counter\n\
- redis_object_idletime: Get key idle time in seconds\n\
- redis_object_help: Get available OBJECT subcommands\n\
- redis_set: Set key to string value with optional expiry and conditional flags [write]\n\
- redis_del: Delete one or more keys [write]\n\
- redis_expire: Set TTL on a key in seconds [write]\n\
- redis_rename: Rename a key [write]\n\
";

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
        .tool(rename(state))
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
        .description(
            "List keys matching a pattern using SCAN (production-safe, non-blocking). \
             Returns up to 'limit' keys.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, KeysInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<KeysInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

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
                        .map_err(|e| ToolError::new(format!("SCAN failed: {}", e)))?;

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
        .description("Get the value of a key from Redis")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let value: Option<String> = redis::cmd("GET")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("GET failed: {}", e)))?;

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
        .description("Get the type of a key (string, list, set, zset, hash, stream)")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, TypeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<TypeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let key_type: String = redis::cmd("TYPE")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("TYPE failed: {}", e)))?;

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
        .description("Get the time-to-live (TTL) of a key in seconds. Returns -1 if no expiry, -2 if key doesn't exist.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, TtlInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<TtlInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let ttl: i64 = redis::cmd("TTL")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("TTL failed: {}", e)))?;

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
        .description("Check if one or more keys exist. Returns the count of keys that exist.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ExistsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ExistsInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let mut cmd = redis::cmd("EXISTS");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("EXISTS failed: {}", e)))?;

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
        .description("Get the memory usage of a key in bytes")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, MemoryUsageInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<MemoryUsageInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let bytes: Option<i64> = redis::cmd("MEMORY")
                    .arg("USAGE")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("MEMORY USAGE failed: {}", e)))?;

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
            "Scan keys with optional type filter. More efficient than redis_keys when filtering \
             by type (string, list, set, zset, hash, stream).",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ScanInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ScanInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

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

                    let (new_cursor, keys): (u64, Vec<String>) =
                        cmd.query_async(&mut conn)
                            .await
                            .map_err(|e| ToolError::new(format!("SCAN failed: {}", e)))?;

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
            "Get the internal encoding of a key (e.g., embstr, int, raw, quicklist, listpack, \
             hashtable, intset, skiplist). Useful for understanding memory usage patterns.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ObjectEncodingInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ObjectEncodingInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let encoding: Option<String> = redis::cmd("OBJECT")
                    .arg("ENCODING")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("OBJECT ENCODING failed: {}", e)))?;

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
            "Get the LFU access frequency counter for a key using OBJECT FREQ. \
             Only works when maxmemory-policy is set to allkeys-lfu or volatile-lfu.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ObjectFreqInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ObjectFreqInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let freq: i64 = redis::cmd("OBJECT")
                    .arg("FREQ")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("OBJECT FREQ failed: {}", e)))?;

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
        .description(
            "Get the idle time of a key in seconds using OBJECT IDLETIME. \
             Shows how long since the key was last accessed.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ObjectIdletimeInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ObjectIdletimeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let idle: i64 = redis::cmd("OBJECT")
                    .arg("IDLETIME")
                    .arg(&input.key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("OBJECT IDLETIME failed: {}", e)))?;

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
        .description("Get available OBJECT subcommands using OBJECT HELP")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ObjectHelpInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ObjectHelpInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let result: Vec<String> = redis::cmd("OBJECT")
                    .arg("HELP")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("OBJECT HELP failed: {}", e)))?;

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
            "Set a key to a string value with optional expiry and conditional flags. \
             Use EX for seconds, PX for milliseconds expiry. Use NX to only set if \
             the key does not exist, XX to only set if it exists.",
        )
        .extractor_handler_typed::<_, _, _, SetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SetInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

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
                    .map_err(|e| ToolError::new(format!("SET failed: {}", e)))?;

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
        .description("Delete one or more keys. Returns the number of keys that were removed.")
        .extractor_handler_typed::<_, _, _, DelInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DelInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let mut cmd = redis::cmd("DEL");
                for key in &input.keys {
                    cmd.arg(key);
                }

                let count: i64 = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("DEL failed: {}", e)))?;

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
        .description(
            "Set a timeout on a key in seconds. The key will be automatically deleted \
             after the timeout expires. Returns whether the timeout was set.",
        )
        .extractor_handler_typed::<_, _, _, ExpireInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ExpireInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let result: bool = redis::cmd("EXPIRE")
                    .arg(&input.key)
                    .arg(input.seconds)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("EXPIRE failed: {}", e)))?;

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
        .description(
            "Rename a key. Returns an error if the source key does not exist. \
             If the destination key already exists, it is overwritten.",
        )
        .extractor_handler_typed::<_, _, _, RenameInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RenameInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .map_err(|e| ToolError::new(format!("Invalid URL: {}", e)))?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| ToolError::new(format!("Connection failed: {}", e)))?;

                let _: () = redis::cmd("RENAME")
                    .arg(&input.key)
                    .arg(&input.newkey)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("RENAME failed: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "OK - renamed '{}' to '{}'",
                    input.key, input.newkey
                )))
            },
        )
        .build()
}
