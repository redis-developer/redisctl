//! Redis Enterprise API tools

use std::sync::Arc;

use redis_enterprise::bdb::DatabaseHandler;
use redis_enterprise::cluster::ClusterHandler;
use redis_enterprise::nodes::NodeHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

/// Input for getting cluster info (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterInput {}

/// Build the get_cluster tool
pub fn get_cluster(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cluster")
        .description(
            "Get Redis Enterprise cluster information including name, version, and configuration",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: GetClusterInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = ClusterHandler::new(client);
            let cluster = handler
                .info()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get cluster info: {}", e)))?;

            let output = serde_json::to_string_pretty(&cluster)
                .map_err(|e| ToolError::new(format!("Failed to serialize: {}", e)))?;

            Ok(CallToolResult::text(output))
        })
        .build()
        .expect("valid tool")
}

/// Input for listing databases
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabasesInput {
    /// Optional filter by database name
    #[serde(default)]
    pub name_filter: Option<String>,
}

/// Build the list_databases tool
pub fn list_databases(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_databases")
        .description("List all databases on the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: ListDatabasesInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = DatabaseHandler::new(client);
            let databases = handler
                .list()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list databases: {}", e)))?;

            let filtered: Vec<_> = if let Some(filter) = &input.name_filter {
                databases
                    .into_iter()
                    .filter(|db| db.name.to_lowercase().contains(&filter.to_lowercase()))
                    .collect()
            } else {
                databases
            };

            let output = filtered
                .iter()
                .map(|db| {
                    format!(
                        "- {} (UID: {}): {} shards",
                        db.name,
                        db.uid,
                        db.shards_count
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "?".to_string())
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let summary = format!("Found {} database(s)\n\n{}", filtered.len(), output);
            Ok(CallToolResult::text(summary))
        })
        .build()
        .expect("valid tool")
}

/// Input for getting a specific database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseInput {
    /// Database UID
    pub uid: u32,
}

/// Build the get_database tool
pub fn get_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_database")
        .description("Get detailed information about a specific Redis Enterprise database")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetDatabaseInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = DatabaseHandler::new(client);
            let database = handler
                .get(input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get database: {}", e)))?;

            let output = serde_json::to_string_pretty(&database)
                .map_err(|e| ToolError::new(format!("Failed to serialize: {}", e)))?;

            Ok(CallToolResult::text(output))
        })
        .build()
        .expect("valid tool")
}

/// Input for listing nodes
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNodesInput {}

/// Build the list_nodes tool
pub fn list_nodes(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_nodes")
        .description("List all nodes in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: ListNodesInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = NodeHandler::new(client);
            let nodes = handler
                .list()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list nodes: {}", e)))?;

            let output = nodes
                .iter()
                .map(|node| {
                    format!(
                        "- Node {} ({}): {}",
                        node.uid,
                        node.addr.as_deref().unwrap_or("unknown"),
                        node.status
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let summary = format!("Found {} node(s)\n\n{}", nodes.len(), output);
            Ok(CallToolResult::text(summary))
        })
        .build()
        .expect("valid tool")
}
