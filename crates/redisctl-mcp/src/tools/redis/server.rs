//! Server-level Redis tools (ping, info, dbsize, client_list, cluster_info, slowlog)

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

pub(super) const INSTRUCTIONS: &str = "\
### Redis Database - Connection\n\
- redis_ping: Test connectivity\n\
- redis_info: Get server information\n\
- redis_dbsize: Get key count\n\
- redis_client_list: Get connected clients\n\
- redis_cluster_info: Get cluster info (if clustered)\n\
- redis_slowlog: Get slow query log entries\n\
";

/// Build a sub-router containing all server-level Redis tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(ping(state.clone()))
        .tool(info(state.clone()))
        .tool(dbsize(state.clone()))
        .tool(client_list(state.clone()))
        .tool(cluster_info(state.clone()))
        .tool(slowlog(state.clone()))
}

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
}

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
}

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
