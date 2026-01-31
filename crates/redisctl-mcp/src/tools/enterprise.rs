//! Redis Enterprise API tools

use std::sync::Arc;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::bdb::DatabaseHandler;
use redis_enterprise::cluster::ClusterHandler;
use redis_enterprise::debuginfo::DebugInfoHandler;
use redis_enterprise::license::LicenseHandler;
use redis_enterprise::logs::{LogsHandler, LogsQuery};
use redis_enterprise::modules::ModuleHandler;
use redis_enterprise::nodes::NodeHandler;
use redis_enterprise::shards::ShardHandler;
use redis_enterprise::stats::{StatsHandler, StatsQuery};
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

// ============================================================================
// License tools
// ============================================================================

/// Input for getting license info
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLicenseInput {}

/// Build the get_license tool
pub fn get_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_license")
        .description(
            "Get Redis Enterprise cluster license information including type, expiration date, \
             cluster name, owner, and enabled features",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: GetLicenseInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = LicenseHandler::new(client);
            let license = handler
                .get()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get license: {}", e)))?;

            CallToolResult::from_serialize(&license)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting license usage
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLicenseUsageInput {}

/// Build the get_license_usage tool
pub fn get_license_usage(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_license_usage")
        .description(
            "Get Redis Enterprise cluster license utilization statistics including shards, \
             nodes, and RAM usage against license limits",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: GetLicenseUsageInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = LicenseHandler::new(client);
            let usage = handler
                .usage()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get license usage: {}", e)))?;

            CallToolResult::from_serialize(&usage)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Logs tools
// ============================================================================

/// Input for listing cluster logs
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListLogsInput {
    /// Start time - only return events after this time (ISO 8601 format, e.g., "2024-01-15T10:00:00Z")
    #[serde(default)]
    pub start_time: Option<String>,
    /// End time - only return events before this time (ISO 8601 format)
    #[serde(default)]
    pub end_time: Option<String>,
    /// Sort order: "asc" (oldest first) or "desc" (newest first, default)
    #[serde(default)]
    pub order: Option<String>,
    /// Maximum number of log entries to return
    #[serde(default)]
    pub limit: Option<u32>,
    /// Number of entries to skip (for pagination)
    #[serde(default)]
    pub offset: Option<u32>,
}

/// Build the list_logs tool
pub fn list_logs(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_logs")
        .description(
            "List cluster event logs from Redis Enterprise. Logs include events like database \
             changes, node status updates, configuration modifications, and alerts. Supports \
             filtering by time range and pagination.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: ListLogsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let query = if input.start_time.is_some()
                || input.end_time.is_some()
                || input.order.is_some()
                || input.limit.is_some()
                || input.offset.is_some()
            {
                Some(LogsQuery {
                    stime: input.start_time,
                    etime: input.end_time,
                    order: input.order,
                    limit: input.limit,
                    offset: input.offset,
                })
            } else {
                None
            };

            let handler = LogsHandler::new(client);
            let logs = handler
                .list(query)
                .await
                .map_err(|e| ToolError::new(format!("Failed to list logs: {}", e)))?;

            CallToolResult::from_serialize(&logs)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Database tools
// ============================================================================

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
pub struct GetClusterStatsInput {
    /// Time interval for aggregation: "1sec", "10sec", "5min", "15min", "1hour", "12hour", "1week"
    #[serde(default)]
    pub interval: Option<String>,
    /// Start time for historical query (ISO 8601 format, e.g., "2024-01-15T10:00:00Z")
    #[serde(default)]
    pub start_time: Option<String>,
    /// End time for historical query (ISO 8601 format)
    #[serde(default)]
    pub end_time: Option<String>,
}

/// Build the get_cluster_stats tool
pub fn get_cluster_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cluster_stats")
        .description(
            "Get statistics for the Redis Enterprise cluster. By default returns the latest \
             stats. Optionally specify interval and time range for historical data.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetClusterStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);

            // If any query params provided, get historical stats
            if input.interval.is_some() || input.start_time.is_some() || input.end_time.is_some() {
                let query = StatsQuery {
                    interval: input.interval,
                    stime: input.start_time,
                    etime: input.end_time,
                    metrics: None,
                };
                let stats = handler
                    .cluster(Some(query))
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get cluster stats: {}", e)))?;
                CallToolResult::from_serialize(&stats)
            } else {
                let stats = handler
                    .cluster_last()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get cluster stats: {}", e)))?;
                CallToolResult::from_serialize(&stats)
            }
        })
        .build()
        .expect("valid tool")
}

/// Input for getting database stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseStatsInput {
    /// Database UID
    pub uid: u32,
    /// Time interval for aggregation: "1sec", "10sec", "5min", "15min", "1hour", "12hour", "1week"
    #[serde(default)]
    pub interval: Option<String>,
    /// Start time for historical query (ISO 8601 format, e.g., "2024-01-15T10:00:00Z")
    #[serde(default)]
    pub start_time: Option<String>,
    /// End time for historical query (ISO 8601 format)
    #[serde(default)]
    pub end_time: Option<String>,
}

/// Build the get_database_stats tool
pub fn get_database_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_stats")
        .description(
            "Get statistics for a specific database. By default returns the latest stats. \
             Optionally specify interval and time range for historical data.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetDatabaseStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);

            if input.interval.is_some() || input.start_time.is_some() || input.end_time.is_some() {
                let query = StatsQuery {
                    interval: input.interval,
                    stime: input.start_time,
                    etime: input.end_time,
                    metrics: None,
                };
                let stats = handler
                    .database(input.uid, Some(query))
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get database stats: {}", e)))?;
                CallToolResult::from_serialize(&stats)
            } else {
                let stats = handler
                    .database_last(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get database stats: {}", e)))?;
                CallToolResult::from_serialize(&stats)
            }
        })
        .build()
        .expect("valid tool")
}

/// Input for getting node stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeStatsInput {
    /// Node UID
    pub uid: u32,
    /// Time interval for aggregation: "1sec", "10sec", "5min", "15min", "1hour", "12hour", "1week"
    #[serde(default)]
    pub interval: Option<String>,
    /// Start time for historical query (ISO 8601 format, e.g., "2024-01-15T10:00:00Z")
    #[serde(default)]
    pub start_time: Option<String>,
    /// End time for historical query (ISO 8601 format)
    #[serde(default)]
    pub end_time: Option<String>,
}

/// Build the get_node_stats tool
pub fn get_node_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_node_stats")
        .description(
            "Get statistics for a specific node. By default returns the latest stats. \
             Optionally specify interval and time range for historical data.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetNodeStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);

            if input.interval.is_some() || input.start_time.is_some() || input.end_time.is_some() {
                let query = StatsQuery {
                    interval: input.interval,
                    stime: input.start_time,
                    etime: input.end_time,
                    metrics: None,
                };
                let stats = handler
                    .node(input.uid, Some(query))
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get node stats: {}", e)))?;
                CallToolResult::from_serialize(&stats)
            } else {
                let stats = handler
                    .node_last(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get node stats: {}", e)))?;
                CallToolResult::from_serialize(&stats)
            }
        })
        .build()
        .expect("valid tool")
}

/// Input for getting all nodes stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAllNodesStatsInput {}

/// Build the get_all_nodes_stats tool
pub fn get_all_nodes_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_all_nodes_stats")
        .description(
            "Get current statistics for all nodes in the Redis Enterprise cluster in a single \
             call. Returns aggregated stats per node including CPU, memory, and network metrics.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: GetAllNodesStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);
            let stats = handler
                .nodes_last()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get all nodes stats: {}", e)))?;

            CallToolResult::from_serialize(&stats)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting all databases stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAllDatabasesStatsInput {}

/// Build the get_all_databases_stats tool
pub fn get_all_databases_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_all_databases_stats")
        .description(
            "Get current statistics for all databases in the Redis Enterprise cluster in a \
             single call. Returns aggregated stats per database including latency, throughput, \
             and memory usage.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(
            state,
            |state, _input: GetAllDatabasesStatsInput| async move {
                let client = state.enterprise_client().await.map_err(|e| {
                    ToolError::new(format!("Failed to get Enterprise client: {}", e))
                })?;

                let handler = StatsHandler::new(client);
                let stats = handler.databases_last().await.map_err(|e| {
                    ToolError::new(format!("Failed to get all databases stats: {}", e))
                })?;

                CallToolResult::from_serialize(&stats)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting shard stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetShardStatsInput {
    /// Shard UID
    pub uid: u32,
}

/// Build the get_shard_stats tool
pub fn get_shard_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_shard_stats")
        .description("Get current statistics for a specific shard in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetShardStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);
            let stats = handler
                .shard(input.uid, None)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get shard stats: {}", e)))?;

            CallToolResult::from_serialize(&stats)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting all shards stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAllShardsStatsInput {}

/// Build the get_all_shards_stats tool
pub fn get_all_shards_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_all_shards_stats")
        .description(
            "Get current statistics for all shards in the Redis Enterprise cluster in a single \
             call. Returns aggregated stats per shard.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: GetAllShardsStatsInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = StatsHandler::new(client);
            let stats = handler
                .shards(None)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get all shards stats: {}", e)))?;

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

// ============================================================================
// Debug Info tools
// ============================================================================

/// Input for listing debug info tasks
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDebugInfoTasksInput {}

/// Build the list_debug_info_tasks tool
pub fn list_debug_info_tasks(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_debug_info_tasks")
        .description(
            "List all debug info collection tasks in the Redis Enterprise cluster. Returns task \
             IDs, statuses (queued, running, completed, failed), and download URLs for completed \
             collections.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: ListDebugInfoTasksInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = DebugInfoHandler::new(client);
            let tasks = handler
                .list()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list debug info tasks: {}", e)))?;

            CallToolResult::from_serialize(&tasks)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting debug info task status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDebugInfoStatusInput {
    /// The task ID returned when debug info collection was started
    pub task_id: String,
}

/// Build the get_debug_info_status tool
pub fn get_debug_info_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_debug_info_status")
        .description(
            "Get the status of a debug info collection task. Returns status (queued, running, \
             completed, failed), progress percentage, download URL (when completed), and error \
             message (if failed).",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetDebugInfoStatusInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = DebugInfoHandler::new(client);
            let status = handler
                .status(&input.task_id)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get debug info status: {}", e)))?;

            CallToolResult::from_serialize(&status)
        })
        .build()
        .expect("valid tool")
}

// ============================================================================
// Module tools
// ============================================================================

/// Input for listing modules
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListModulesInput {}

/// Build the list_modules tool
pub fn list_modules(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_modules")
        .description(
            "List all Redis modules installed on the Redis Enterprise cluster. Returns module \
             names, versions, descriptions, and capabilities (e.g., RedisJSON, RediSearch, \
             RedisTimeSeries).",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: ListModulesInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = ModuleHandler::new(client);
            let modules = handler
                .list()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list modules: {}", e)))?;

            CallToolResult::from_serialize(&modules)
        })
        .build()
        .expect("valid tool")
}

/// Input for getting a specific module
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetModuleInput {
    /// Module UID
    pub uid: String,
}

/// Build the get_module tool
pub fn get_module(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_module")
        .description(
            "Get detailed information about a specific Redis module including version, \
             description, author, license, capabilities, and platform compatibility.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetModuleInput| async move {
            let client = state
                .enterprise_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Enterprise client: {}", e)))?;

            let handler = ModuleHandler::new(client);
            let module = handler
                .get(&input.uid)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get module: {}", e)))?;

            CallToolResult::from_serialize(&module)
        })
        .build()
        .expect("valid tool")
}
