//! Server-level Redis tools (ping, info, dbsize, client_list, cluster_info, slowlog,
//! config_get, memory_stats, latency_history, acl_list, acl_whoami, module_list,
//! config_set, flushdb)

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;

/// Build a sub-router containing all server-level Redis tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(ping(state.clone()))
        .tool(info(state.clone()))
        .tool(dbsize(state.clone()))
        .tool(client_list(state.clone()))
        .tool(cluster_info(state.clone()))
        .tool(slowlog(state.clone()))
        .tool(config_get(state.clone()))
        .tool(memory_stats(state.clone()))
        .tool(latency_history(state.clone()))
        .tool(acl_list(state.clone()))
        .tool(acl_whoami(state.clone()))
        .tool(module_list(state.clone()))
        .tool(config_set(state.clone()))
        .tool(flushdb(state))
}

/// Input for ping command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PingInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the ping tool
pub fn ping(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_ping")
        .description("Test connectivity to a Redis database by sending a PING command")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, PingInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<PingInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let response: String = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await
                    .tool_context("PING failed")?;

                Ok(CallToolResult::text(format!(
                    "Connected successfully. Response: {}",
                    response
                )))
            },
        )
        .build()
}

/// Input for info command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct InfoInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Optional section to retrieve (e.g., "server", "memory", "stats")
    #[serde(default)]
    pub section: Option<String>,
}

/// Build the info tool
pub fn info(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_info")
        .description("Get Redis server information using the INFO command")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, InfoInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<InfoInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let mut cmd = redis::cmd("INFO");
                if let Some(section) = &input.section {
                    cmd.arg(section);
                }

                let info: String = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("INFO failed")?;

                Ok(CallToolResult::text(info))
            },
        )
        .build()
}

/// Input for DBSIZE command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbsizeInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the dbsize tool
pub fn dbsize(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_dbsize")
        .description("Get the number of keys in the currently selected database")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, DbsizeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DbsizeInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let size: i64 = redis::cmd("DBSIZE")
                    .query_async(&mut conn)
                    .await
                    .tool_context("DBSIZE failed")?;

                Ok(CallToolResult::text(format!(
                    "Database contains {} keys",
                    size
                )))
            },
        )
        .build()
}

/// Input for CLIENT LIST command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClientListInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the client_list tool
pub fn client_list(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_client_list")
        .description("Get list of client connections to the Redis server")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ClientListInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ClientListInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let clients: String = redis::cmd("CLIENT")
                    .arg("LIST")
                    .query_async(&mut conn)
                    .await
                    .tool_context("CLIENT LIST failed")?;

                let count = clients.lines().count();
                Ok(CallToolResult::text(format!(
                    "{} connected client(s):\n\n{}",
                    count, clients
                )))
            },
        )
        .build()
}

/// Input for CLUSTER INFO command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClusterInfoInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the cluster_info tool
pub fn cluster_info(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_cluster_info")
        .description("Get Redis Cluster information (only works on cluster-enabled databases)")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ClusterInfoInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ClusterInfoInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let info: String = redis::cmd("CLUSTER")
                    .arg("INFO")
                    .query_async(&mut conn)
                    .await
                    .tool_context("CLUSTER INFO failed")?;

                Ok(CallToolResult::text(info))
            },
        )
        .build()
}

/// Input for SLOWLOG GET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SlowlogInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, SlowlogInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SlowlogInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                // SLOWLOG GET returns nested arrays
                let entries: Vec<Vec<redis::Value>> = redis::cmd("SLOWLOG")
                    .arg("GET")
                    .arg(input.count)
                    .query_async(&mut conn)
                    .await
                    .tool_context("SLOWLOG GET failed")?;

                if entries.is_empty() {
                    return Ok(CallToolResult::text("No slow queries recorded"));
                }

                let mut output = format!("Slow log ({} entries):\n\n", entries.len());

                for entry in entries {
                    // Each entry is: [id, timestamp, duration_us, command_args, ...]
                    if entry.len() >= 4 {
                        let id = super::format_value(&entry[0]);
                        let duration_us = super::format_value(&entry[2]);
                        let command = if let redis::Value::Array(args) = &entry[3] {
                            args.iter()
                                .map(super::format_value)
                                .collect::<Vec<_>>()
                                .join(" ")
                        } else {
                            super::format_value(&entry[3])
                        };

                        output.push_str(&format!("#{} - {} us: {}\n", id, duration_us, command));
                    }
                }

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// Input for CONFIG GET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigGetInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Configuration parameter pattern (e.g. "maxmemory", "save", "*")
    pub parameter: String,
}

/// Build the config_get tool
pub fn config_get(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_config_get")
        .description(
            "Get Redis configuration parameter values using CONFIG GET. \
             Supports glob-style patterns (e.g. \"maxmemory\", \"*memory*\", \"*\").",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ConfigGetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ConfigGetInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: Vec<(String, String)> = redis::cmd("CONFIG")
                    .arg("GET")
                    .arg(&input.parameter)
                    .query_async(&mut conn)
                    .await
                    .tool_context("CONFIG GET failed")?;

                if result.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "No configuration parameters matching '{}'",
                        input.parameter
                    )));
                }

                let output = result
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(CallToolResult::text(format!(
                    "Configuration ({} parameter(s)):\n{}",
                    result.len(),
                    output
                )))
            },
        )
        .build()
}

/// Input for MEMORY STATS command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryStatsInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the memory_stats tool
pub fn memory_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_memory_stats")
        .description(
            "Get detailed memory allocator statistics using MEMORY STATS. \
             Shows memory usage breakdown by category.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, MemoryStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<MemoryStatsInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: redis::Value = redis::cmd("MEMORY")
                    .arg("STATS")
                    .query_async(&mut conn)
                    .await
                    .tool_context("MEMORY STATS failed")?;

                Ok(CallToolResult::text(super::format_value(&result)))
            },
        )
        .build()
}

/// Input for LATENCY HISTORY command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LatencyHistoryInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
    /// Latency event name (e.g. "command", "fast-command")
    pub event: String,
}

/// Build the latency_history tool
pub fn latency_history(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_latency_history")
        .description(
            "Get latency history for a specific event using LATENCY HISTORY. \
             Returns timestamp and latency pairs. Events include \"command\", \
             \"fast-command\", etc. May return empty if latency monitoring is not enabled \
             (CONFIG SET latency-monitor-threshold <ms>).",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, LatencyHistoryInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<LatencyHistoryInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str())
                    .tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: Vec<Vec<redis::Value>> = redis::cmd("LATENCY")
                    .arg("HISTORY")
                    .arg(&input.event)
                    .query_async(&mut conn)
                    .await
                    .tool_context("LATENCY HISTORY failed")?;

                if result.is_empty() {
                    return Ok(CallToolResult::text(format!(
                        "No latency history for event '{}'. \
                         Latency monitoring may not be enabled \
                         (CONFIG SET latency-monitor-threshold <ms>).",
                        input.event
                    )));
                }

                let mut output = format!(
                    "Latency history for '{}' ({} entries):\n\n",
                    input.event,
                    result.len()
                );

                for entry in &result {
                    if entry.len() >= 2 {
                        let timestamp = super::format_value(&entry[0]);
                        let latency_ms = super::format_value(&entry[1]);
                        output.push_str(&format!("  {} - {} ms\n", timestamp, latency_ms));
                    }
                }

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// Input for ACL LIST command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AclListInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the acl_list tool
pub fn acl_list(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_acl_list")
        .description("List all ACL rules configured on the Redis server using ACL LIST")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, AclListInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<AclListInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let rules: Vec<String> = redis::cmd("ACL")
                    .arg("LIST")
                    .query_async(&mut conn)
                    .await
                    .tool_context("ACL LIST failed")?;

                if rules.is_empty() {
                    return Ok(CallToolResult::text("No ACL rules configured"));
                }

                Ok(CallToolResult::text(format!(
                    "ACL rules ({}):\n{}",
                    rules.len(),
                    rules.join("\n")
                )))
            },
        )
        .build()
}

/// Input for ACL WHOAMI command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AclWhoamiInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the acl_whoami tool
pub fn acl_whoami(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_acl_whoami")
        .description("Get the username of the current authenticated connection using ACL WHOAMI")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, AclWhoamiInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<AclWhoamiInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let username: String = redis::cmd("ACL")
                    .arg("WHOAMI")
                    .query_async(&mut conn)
                    .await
                    .tool_context("ACL WHOAMI failed")?;

                Ok(CallToolResult::text(format!("Current user: {}", username)))
            },
        )
        .build()
}

/// Input for MODULE LIST command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ModuleListInput {
    /// Optional Redis URL (overrides profile, uses configured URL if not provided)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from (uses default profile if not set)
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the module_list tool
pub fn module_list(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_module_list")
        .description("List loaded Redis modules with their names and versions using MODULE LIST")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ModuleListInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ModuleListInput>| async move {
                let url = super::resolve_redis_url(input.url, input.profile.as_deref(), &state)?;

                let client = redis::Client::open(url.as_str()).tool_context("Invalid URL")?;

                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .tool_context("Connection failed")?;

                let result: redis::Value = redis::cmd("MODULE")
                    .arg("LIST")
                    .query_async(&mut conn)
                    .await
                    .tool_context("MODULE LIST failed")?;

                let formatted = super::format_value(&result);
                if formatted == "[]" {
                    return Ok(CallToolResult::text("No modules loaded"));
                }

                Ok(CallToolResult::text(format!(
                    "Loaded modules:\n{}",
                    formatted
                )))
            },
        )
        .build()
}

// --- Write tools ---

/// Input for CONFIG SET command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigSetInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Configuration parameter name
    pub parameter: String,
    /// Configuration parameter value
    pub value: String,
}

/// Build the config_set tool
pub fn config_set(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_config_set")
        .description(
            "Set a Redis configuration parameter at runtime using CONFIG SET. \
             Changes may not persist across restarts unless CONFIG REWRITE is called.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ConfigSetInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ConfigSetInput>| async move {
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

                let _: () = redis::cmd("CONFIG")
                    .arg("SET")
                    .arg(&input.parameter)
                    .arg(&input.value)
                    .query_async(&mut conn)
                    .await
                    .tool_context("CONFIG SET failed")?;

                Ok(CallToolResult::text(format!(
                    "OK - set {} = {}",
                    input.parameter, input.value
                )))
            },
        )
        .build()
}

/// Input for FLUSHDB command
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FlushdbInput {
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name for connection resolution
    #[serde(default)]
    pub profile: Option<String>,
    /// Use asynchronous flush (non-blocking, default: false)
    #[serde(default)]
    pub async_flush: bool,
}

/// Build the flushdb tool
pub fn flushdb(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_flushdb")
        .description(
            "DANGEROUS: Flush all keys from the current database. This permanently deletes \
             all data. Use with extreme caution. Set async_flush=true for non-blocking operation.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, FlushdbInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<FlushdbInput>| async move {
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

                let mut cmd = redis::cmd("FLUSHDB");
                if input.async_flush {
                    cmd.arg("ASYNC");
                }

                let _: () = cmd
                    .query_async(&mut conn)
                    .await
                    .tool_context("FLUSHDB failed")?;

                let mode = if input.async_flush { " (async)" } else { "" };
                Ok(CallToolResult::text(format!(
                    "OK - database flushed{}",
                    mode
                )))
            },
        )
        .build()
}
