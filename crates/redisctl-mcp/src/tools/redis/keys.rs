//! Key-level Redis tools (keys, scan, get, key_type, ttl, exists, memory_usage, object_encoding)

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, Tool, ToolBuilder, ToolError};

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
}

/// Input for keys command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeysInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
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
