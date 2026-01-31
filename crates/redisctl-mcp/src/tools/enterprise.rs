//! Redis Enterprise API tools

use std::sync::Arc;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::bdb::DatabaseHandler;
use redis_enterprise::cluster::ClusterHandler;
use redis_enterprise::nodes::NodeHandler;
use redis_enterprise::shards::ShardHandler;
use redis_enterprise::stats::StatsHandler;
use redis_enterprise::users::UserHandler;
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

            CallToolResult::from_serialize(&cluster)
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

            CallToolResult::from_serialize(&database)
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

// ============================================================================
// Node details
// ============================================================================

/// Input for getting a specific node
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeInput {
    /// Node UID
    pub uid: u32,
}

/// Build the get_node tool
pub fn get_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_node")
        .description(
            "Get detailed information about a specific node in the Redis Enterprise cluster",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetNodeInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = NodeHandler::new(client);
            let node = handler
                .get(input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get node: {}", e)))?;

            CallToolResult::from_serialize(&node)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// User tools
// ============================================================================

/// Input for listing users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListUsersInput {}

/// Build the list_users tool
pub fn list_users(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_users")
        .description("List all users in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: ListUsersInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = UserHandler::new(client);
            let users = handler
                .list()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list users: {}", e)))?;

            let output = users
                .iter()
                .map(|user| {
                    format!(
                        "- {} (UID: {}): {}",
                        user.name.as_deref().unwrap_or("(unnamed)"),
                        user.uid,
                        user.email
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let summary = format!("Found {} user(s)\n\n{}", users.len(), output);
            Ok(CallToolResult::text(summary))
        })
        .build()
        .expect("valid tool")
}

/// Input for getting a specific user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUserInput {
    /// User UID
    pub uid: u32,
}

/// Build the get_user tool
pub fn get_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_user")
        .description(
            "Get detailed information about a specific user in the Redis Enterprise cluster",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetUserInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = UserHandler::new(client);
            let user = handler
                .get(input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get user: {}", e)))?;

            CallToolResult::from_serialize(&user)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Alert tools
// ============================================================================

/// Input for listing alerts
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAlertsInput {}

/// Build the list_alerts tool
pub fn list_alerts(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_alerts")
        .description("List all active alerts in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: ListAlertsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = AlertHandler::new(client);
            let alerts = handler
                .list()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list alerts: {}", e)))?;

            CallToolResult::from_serialize(&alerts)
        })
        .build()
        .expect("valid tool")
}

/// Input for listing database alerts
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabaseAlertsInput {
    /// Database UID
    pub uid: u32,
}

/// Build the list_database_alerts tool
pub fn list_database_alerts(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_database_alerts")
        .description("List all alerts for a specific database in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: ListDatabaseAlertsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = AlertHandler::new(client);
            let alerts = handler
                .list_by_database(input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to list database alerts: {}", e)))?;

            CallToolResult::from_serialize(&alerts)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Stats tools
// ============================================================================

/// Input for getting cluster stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterStatsInput {}

/// Build the get_cluster_stats tool
pub fn get_cluster_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cluster_stats")
        .description("Get current statistics for the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: GetClusterStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);
            let stats = handler
                .cluster_last()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get cluster stats: {}", e)))?;

            CallToolResult::from_serialize(&stats)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting database stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseStatsInput {
    /// Database UID
    pub uid: u32,
}

/// Build the get_database_stats tool
pub fn get_database_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_stats")
        .description(
            "Get current statistics for a specific database in the Redis Enterprise cluster",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetDatabaseStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);
            let stats = handler
                .database_last(input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get database stats: {}", e)))?;

            CallToolResult::from_serialize(&stats)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting node stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeStatsInput {
    /// Node UID
    pub uid: u32,
}

/// Build the get_node_stats tool
pub fn get_node_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_node_stats")
        .description("Get current statistics for a specific node in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetNodeStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);
            let stats = handler
                .node_last(input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get node stats: {}", e)))?;

            CallToolResult::from_serialize(&stats)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Shard tools
// ============================================================================

/// Input for listing shards
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListShardsInput {
    /// Optional database UID to filter by
    #[serde(default)]
    pub database_uid: Option<u32>,
}

/// Build the list_shards tool
pub fn list_shards(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_shards")
        .description(
            "List all shards in the Redis Enterprise cluster. Optionally filter by database UID.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: ListShardsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = ShardHandler::new(client);
            let shards = if let Some(db_uid) = input.database_uid {
                handler
                    .list_by_database(db_uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list shards: {}", e)))?
            } else {
                handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list shards: {}", e)))?
            };

            CallToolResult::from_serialize(&shards)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Database endpoints
// ============================================================================

/// Input for getting database endpoints
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseEndpointsInput {
    /// Database UID
    pub uid: u32,
}

/// Build the get_database_endpoints tool
pub fn get_database_endpoints(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_endpoints")
        .description(
            "Get connection endpoints for a specific database in the Redis Enterprise cluster",
        )
        .read_only()
        .idempotent()
        .handler_with_state(
            state,
            |state, input: GetDatabaseEndpointsInput| async move {
                let client = state.enterprise_client().await.map_err(|e| {
                    ToolError::new(format!("Failed to get Enterprise client: {}", e))
                })?;

                let handler = DatabaseHandler::new(client);
                let endpoints = handler
                    .endpoints(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get endpoints: {}", e)))?;

                CallToolResult::from_serialize(&endpoints)
            },
        )
        .build()
        .expect("valid tool")
}
