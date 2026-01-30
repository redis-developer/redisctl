//! Direct Redis database tools

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder, ToolError};

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
        .handler_with_state(state, |state, input: PingInput| async move {
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
        })
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
        .handler_with_state(state, |state, input: InfoInput| async move {
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
        })
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
        .handler_with_state(state, |state, input: KeysInput| async move {
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
        })
        .build()
        .expect("valid tool")
}
