//! Direct Redis database tools

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

/// Input for ping command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PingInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
}

/// Build the ping tool
pub fn ping(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_ping")
        .description("Test connectivity to a Redis database by sending a PING command")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, PingInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<PingInput>| async move {
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

                let response: String = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("PING failed: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Connected successfully. Response: {}",
                    response
                )))
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for info command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct InfoInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional section to retrieve (e.g., "server", "memory", "stats")
    #[serde(default)]
    pub section: Option<String>,
}

/// Build the info tool
pub fn info(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_info")
        .description("Get Redis server information using the INFO command")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, InfoInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<InfoInput>| async move {
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

                let mut cmd = redis::cmd("INFO");
                if let Some(section) = &input.section {
                    cmd.arg(section);
                }

                let info: String = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("INFO failed: {}", e)))?;

                Ok(CallToolResult::text(info))
            },
        )
        .build()
        .expect("valid tool")
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
        .expect("valid tool")
}

// ============================================================================
// Key operations
// ============================================================================

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
        .expect("valid tool")
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
        .expect("valid tool")
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
        .expect("valid tool")
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
        .expect("valid tool")
}

/// Input for DBSIZE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbsizeInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
}

/// Build the dbsize tool
pub fn dbsize(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_dbsize")
        .description("Get the number of keys in the currently selected database")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, DbsizeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DbsizeInput>| async move {
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

                let size: i64 = redis::cmd("DBSIZE")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("DBSIZE failed: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Database contains {} keys",
                    size
                )))
            },
        )
        .build()
        .expect("valid tool")
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
        .expect("valid tool")
}

// ============================================================================
// Hash operations
// ============================================================================

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
        .expect("valid tool")
}

// ============================================================================
// List operations
// ============================================================================

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
        .expect("valid tool")
}

// ============================================================================
// Set operations
// ============================================================================

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
        .expect("valid tool")
}

// ============================================================================
// Sorted Set operations
// ============================================================================

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
        .expect("valid tool")
}

// ============================================================================
// Cluster info
// ============================================================================

/// Input for CLUSTER INFO command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClusterInfoInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
}

/// Build the cluster_info tool
pub fn cluster_info(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_cluster_info")
        .description("Get Redis Cluster information (only works on cluster-enabled databases)")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ClusterInfoInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ClusterInfoInput>| async move {
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

                let info: String = redis::cmd("CLUSTER")
                    .arg("INFO")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("CLUSTER INFO failed: {}", e)))?;

                Ok(CallToolResult::text(info))
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for CLIENT LIST command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClientListInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
}

/// Build the client_list tool
pub fn client_list(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_client_list")
        .description("Get list of client connections to the Redis server")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ClientListInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ClientListInput>| async move {
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

                let clients: String = redis::cmd("CLIENT")
                    .arg("LIST")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("CLIENT LIST failed: {}", e)))?;

                let count = clients.lines().count();
                Ok(CallToolResult::text(format!(
                    "{} connected client(s):\n\n{}",
                    count, clients
                )))
            },
        )
        .build()
        .expect("valid tool")
}

// ============================================================================
// SCAN with type filter
// ============================================================================

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
        .expect("valid tool")
}

// ============================================================================
// Object encoding
// ============================================================================

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
        .expect("valid tool")
}

// ============================================================================
// Slowlog
// ============================================================================

/// Input for SLOWLOG GET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SlowlogInput {
    /// Optional Redis URL (uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Number of entries to return (default: 10)
    #[serde(default = "default_slowlog_count")]
    pub count: usize,
}

fn default_slowlog_count() -> usize {
    10
}

/// Build the slowlog tool
pub fn slowlog(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_slowlog")
        .description(
            "Get slow query log entries. Useful for identifying slow commands affecting performance.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, SlowlogInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SlowlogInput>| async move {
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

                // SLOWLOG GET returns nested arrays
                let entries: Vec<Vec<redis::Value>> = redis::cmd("SLOWLOG")
                    .arg("GET")
                    .arg(input.count)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ToolError::new(format!("SLOWLOG GET failed: {}", e)))?;

                if entries.is_empty() {
                    return Ok(CallToolResult::text("No slow queries recorded"));
                }

                let mut output = format!("Slow log ({} entries):\n\n", entries.len());

                for entry in entries {
                    // Each entry is: [id, timestamp, duration_us, command_args, ...]
                    if entry.len() >= 4 {
                        let id = format_value(&entry[0]);
                        let duration_us = format_value(&entry[2]);
                        let command = if let redis::Value::Array(args) = &entry[3] {
                            args.iter()
                                .map(format_value)
                                .collect::<Vec<_>>()
                                .join(" ")
                        } else {
                            format_value(&entry[3])
                        };

                        output.push_str(&format!("#{} - {} us: {}\n", id, duration_us, command));
                    }
                }

                Ok(CallToolResult::text(output))
            },
        )
        .build()
        .expect("valid tool")
}

fn format_value(v: &redis::Value) -> String {
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

/// Instructions text describing all Redis database tools
pub fn instructions() -> &'static str {
    r#"
### Redis Database - Connection
- redis_ping: Test connectivity
- redis_info: Get server information
- redis_dbsize: Get key count
- redis_client_list: Get connected clients
- redis_cluster_info: Get cluster info (if clustered)
- redis_slowlog: Get slow query log entries

### Redis Database - Keys
- redis_keys: List keys matching pattern (SCAN)
- redis_scan: Scan keys with type filter (string, list, set, zset, hash, stream)
- redis_get: Get string value
- redis_type: Get key type
- redis_ttl: Get key TTL
- redis_exists: Check key existence
- redis_memory_usage: Get key memory usage
- redis_object_encoding: Get key internal encoding

### Redis Database - Data Structures
- redis_hgetall: Get all hash fields
- redis_lrange: Get list range
- redis_smembers: Get set members
- redis_zrange: Get sorted set range
"#
}

/// Build an MCP sub-router containing all Redis database tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Connection
        .tool(ping(state.clone()))
        .tool(info(state.clone()))
        .tool(dbsize(state.clone()))
        .tool(client_list(state.clone()))
        .tool(cluster_info(state.clone()))
        .tool(slowlog(state.clone()))
        // Keys
        .tool(keys(state.clone()))
        .tool(scan(state.clone()))
        .tool(get(state.clone()))
        .tool(key_type(state.clone()))
        .tool(ttl(state.clone()))
        .tool(exists(state.clone()))
        .tool(memory_usage(state.clone()))
        .tool(object_encoding(state.clone()))
        // Data Structures
        .tool(hgetall(state.clone()))
        .tool(lrange(state.clone()))
        .tool(smembers(state.clone()))
        .tool(zrange(state.clone()))
}
