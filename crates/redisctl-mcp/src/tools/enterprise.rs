//! Redis Enterprise API tools

use std::sync::Arc;
use std::time::Duration;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::bdb::{CreateDatabaseRequest, DatabaseHandler};
use redis_enterprise::cluster::ClusterHandler;
use redis_enterprise::debuginfo::DebugInfoHandler;
use redis_enterprise::license::{LicenseHandler, LicenseUpdateRequest};
use redis_enterprise::logs::{LogsHandler, LogsQuery};
use redis_enterprise::modules::ModuleHandler;
use redis_enterprise::nodes::NodeHandler;
use redis_enterprise::redis_acls::RedisAclHandler;
use redis_enterprise::roles::RolesHandler;
use redis_enterprise::shards::ShardHandler;
use redis_enterprise::stats::{StatsHandler, StatsQuery};
use redis_enterprise::users::UserHandler;
use redisctl_core::enterprise::{
    backup_database_and_wait, flush_database_and_wait, import_database_and_wait,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;
use crate::tools::wrap_list;

/// Input for getting cluster info (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cluster tool
pub fn get_cluster(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cluster")
        .description(
            "Get Redis Enterprise cluster information including name, version, and configuration",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetClusterInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let cluster = handler
                    .info()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get cluster info: {}", e)))?;

                CallToolResult::from_serialize(&cluster)
            },
        )
        .build()
}

// ============================================================================
// License tools
// ============================================================================

/// Input for getting license info
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLicenseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_license tool
pub fn get_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_license")
        .description(
            "Get Redis Enterprise cluster license information including type, expiration date, \
             cluster name, owner, and enabled features",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetLicenseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetLicenseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let license = handler
                    .get()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get license: {}", e)))?;

                CallToolResult::from_serialize(&license)
            },
        )
        .build()
}

/// Input for getting license usage
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLicenseUsageInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_license_usage tool
pub fn get_license_usage(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_license_usage")
        .description(
            "Get Redis Enterprise cluster license utilization statistics including shards, \
             nodes, and RAM usage against license limits",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetLicenseUsageInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetLicenseUsageInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let usage = handler
                    .usage()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get license usage: {}", e)))?;

                CallToolResult::from_serialize(&usage)
            },
        )
        .build()
}

// ============================================================================
// Logs tools
// ============================================================================

/// Input for listing cluster logs
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListLogsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, ListLogsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListLogsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

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

                wrap_list("logs", &logs)
            },
        )
        .build()
}

// ============================================================================
// Database tools
// ============================================================================

/// Input for listing databases
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabasesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Optional filter by database name (case-insensitive substring match)
    #[serde(default)]
    pub name_filter: Option<String>,
    /// Optional filter by database status (e.g., "active", "pending", "creation-failed")
    #[serde(default)]
    pub status_filter: Option<String>,
}

/// Build the list_databases tool
pub fn list_databases(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_databases")
        .description(
            "List all databases on the Redis Enterprise cluster. Supports filtering by name \
             (case-insensitive substring match) and status.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListDatabasesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListDatabasesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let databases = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list databases: {}", e)))?;

                // Apply name filter
                let filtered: Vec<_> = databases
                    .into_iter()
                    .filter(|db| {
                        if let Some(filter) = &input.name_filter {
                            db.name.to_lowercase().contains(&filter.to_lowercase())
                        } else {
                            true
                        }
                    })
                    .filter(|db| {
                        if let Some(filter) = &input.status_filter {
                            db.status
                                .as_ref()
                                .map(|s| s.to_lowercase() == filter.to_lowercase())
                                .unwrap_or(false)
                        } else {
                            true
                        }
                    })
                    .collect();

                wrap_list("databases", &filtered)
            },
        )
        .build()
}

/// Input for getting a specific database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID
    pub uid: u32,
}

/// Build the get_database tool
pub fn get_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_database")
        .description("Get detailed information about a specific Redis Enterprise database")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetDatabaseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .get(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

/// Input for listing nodes
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNodesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_nodes tool
pub fn list_nodes(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_nodes")
        .description("List all nodes in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListNodesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListNodesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let nodes = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list nodes: {}", e)))?;

                wrap_list("nodes", &nodes)
            },
        )
        .build()
}

// ============================================================================
// Node details
// ============================================================================

/// Input for getting a specific node
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetNodeInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetNodeInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let node = handler
                    .get(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get node: {}", e)))?;

                CallToolResult::from_serialize(&node)
            },
        )
        .build()
}

// ============================================================================
// User tools
// ============================================================================

/// Input for listing users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListUsersInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_users tool
pub fn list_users(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_users")
        .description("List all users in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListUsersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListUsersInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = UserHandler::new(client);
                let users = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list users: {}", e)))?;

                wrap_list("users", &users)
            },
        )
        .build()
}

/// Input for getting a specific user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUserInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetUserInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetUserInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = UserHandler::new(client);
                let user = handler
                    .get(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get user: {}", e)))?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

// ============================================================================
// Alert tools
// ============================================================================

/// Input for listing alerts
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAlertsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_alerts tool
pub fn list_alerts(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_alerts")
        .description("List all active alerts in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListAlertsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListAlertsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = AlertHandler::new(client);
                let alerts = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list alerts: {}", e)))?;

                wrap_list("alerts", &alerts)
            },
        )
        .build()
}

/// Input for listing database alerts
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabaseAlertsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID
    pub uid: u32,
}

/// Build the list_database_alerts tool
pub fn list_database_alerts(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_database_alerts")
        .description("List all alerts for a specific database in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListDatabaseAlertsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListDatabaseAlertsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = AlertHandler::new(client);
                let alerts = handler
                    .list_by_database(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list database alerts: {}", e)))?;

                wrap_list("alerts", &alerts)
            },
        )
        .build()
}

// ============================================================================
// Stats tools
// ============================================================================

/// Input for getting cluster stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetClusterStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetClusterStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

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
            },
        )
        .build()
}

/// Input for getting database stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetDatabaseStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetDatabaseStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

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
            },
        )
        .build()
}

/// Input for getting node stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetNodeStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetNodeStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);

                if input.interval.is_some()
                    || input.start_time.is_some()
                    || input.end_time.is_some()
                {
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
            },
        )
        .build()
}

/// Input for getting all nodes stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAllNodesStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_all_nodes_stats tool
pub fn get_all_nodes_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_all_nodes_stats")
        .description(
            "Get current statistics for all nodes in the Redis Enterprise cluster in a single \
             call. Returns aggregated stats per node including CPU, memory, and network metrics.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAllNodesStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetAllNodesStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler
                    .nodes_last()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get all nodes stats: {}", e)))?;

                CallToolResult::from_serialize(&stats)
            },
        )
        .build()
}

/// Input for getting all databases stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAllDatabasesStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

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
        .extractor_handler_typed::<_, _, _, GetAllDatabasesStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetAllDatabasesStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler.databases_last().await.map_err(|e| {
                    ToolError::new(format!("Failed to get all databases stats: {}", e))
                })?;

                CallToolResult::from_serialize(&stats)
            },
        )
        .build()
}

/// Input for getting shard stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetShardStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Shard UID
    pub uid: u32,
}

/// Build the get_shard_stats tool
pub fn get_shard_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_shard_stats")
        .description("Get current statistics for a specific shard in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetShardStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetShardStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler
                    .shard(input.uid, None)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get shard stats: {}", e)))?;

                CallToolResult::from_serialize(&stats)
            },
        )
        .build()
}

/// Input for getting all shards stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAllShardsStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_all_shards_stats tool
pub fn get_all_shards_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_all_shards_stats")
        .description(
            "Get current statistics for all shards in the Redis Enterprise cluster in a single \
             call. Returns aggregated stats per shard.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAllShardsStatsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetAllShardsStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler
                    .shards(None)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get all shards stats: {}", e)))?;

                CallToolResult::from_serialize(&stats)
            },
        )
        .build()
}

// ============================================================================
// Shard tools
// ============================================================================

/// Input for listing shards
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListShardsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, ListShardsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListShardsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

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

                wrap_list("shards", &shards)
            },
        )
        .build()
}

/// Input for getting a specific shard
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetShardInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Shard UID (e.g., "1" or "2")
    pub uid: String,
}

/// Build the get_shard tool
pub fn get_shard(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_shard")
        .description(
            "Get detailed information about a specific shard in the Redis Enterprise cluster \
             including role (master/replica), status, and assigned node.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetShardInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetShardInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ShardHandler::new(client);
                let shard = handler
                    .get(&input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get shard: {}", e)))?;

                CallToolResult::from_serialize(&shard)
            },
        )
        .build()
}

// ============================================================================
// Database endpoints
// ============================================================================

/// Input for getting database endpoints
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseEndpointsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetDatabaseEndpointsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetDatabaseEndpointsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let endpoints = handler
                    .endpoints(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get endpoints: {}", e)))?;

                wrap_list("endpoints", &endpoints)
            },
        )
        .build()
}

// ============================================================================
// Debug Info tools
// ============================================================================

/// Input for listing debug info tasks
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDebugInfoTasksInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

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
        .extractor_handler_typed::<_, _, _, ListDebugInfoTasksInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListDebugInfoTasksInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DebugInfoHandler::new(client);
                let tasks = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list debug info tasks: {}", e)))?;

                wrap_list("tasks", &tasks)
            },
        )
        .build()
}

/// Input for getting debug info task status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDebugInfoStatusInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetDebugInfoStatusInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetDebugInfoStatusInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DebugInfoHandler::new(client);
                let status = handler
                    .status(&input.task_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get debug info status: {}", e)))?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

// ============================================================================
// Module tools
// ============================================================================

/// Input for listing modules
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListModulesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

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
        .extractor_handler_typed::<_, _, _, ListModulesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListModulesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ModuleHandler::new(client);
                let modules = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list modules: {}", e)))?;

                wrap_list("modules", &modules)
            },
        )
        .build()
}

/// Input for getting a specific module
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetModuleInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
        .extractor_handler_typed::<_, _, _, GetModuleInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetModuleInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ModuleHandler::new(client);
                let module = handler
                    .get(&input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get module: {}", e)))?;

                CallToolResult::from_serialize(&module)
            },
        )
        .build()
}

// ============================================================================
// Database Write Operations
// ============================================================================

fn default_enterprise_timeout() -> u64 {
    600
}

/// Input for backing up an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BackupDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to backup
    pub bdb_uid: u32,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_enterprise_timeout")]
    pub timeout_seconds: u64,
}

/// Build the backup_enterprise_database tool
pub fn backup_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("backup_enterprise_database")
        .description(
            "Trigger a backup of a Redis Enterprise database and wait for completion. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, BackupDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<BackupDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                // Use Layer 2 workflow
                backup_database_and_wait(
                    &client,
                    input.bdb_uid,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to backup database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Backup completed successfully",
                    "bdb_uid": input.bdb_uid
                }))
            },
        )
        .build()
}

/// Input for importing data into an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to import into
    pub bdb_uid: u32,
    /// Import location (file path or URL)
    pub import_location: String,
    /// Whether to flush the database before import (default: false)
    #[serde(default)]
    pub flush: bool,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_enterprise_timeout")]
    pub timeout_seconds: u64,
}

/// Build the import_enterprise_database tool
pub fn import_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("import_enterprise_database")
        .description(
            "Import data into a Redis Enterprise database from an external source and wait for completion. \
             WARNING: If flush is true, existing data will be deleted before import. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, ImportDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ImportDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                // Use Layer 2 workflow
                import_database_and_wait(
                    &client,
                    input.bdb_uid,
                    &input.import_location,
                    input.flush,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to import database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Import completed successfully",
                    "bdb_uid": input.bdb_uid,
                    "import_location": input.import_location
                }))
            },
        )
        .build()
}

/// Input for creating an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateEnterpriseDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database name
    pub name: String,
    /// Memory size in bytes (e.g., 1073741824 for 1GB)
    pub memory_size: Option<u64>,
    /// Port number (optional, cluster will assign if not specified)
    pub port: Option<u16>,
    /// Enable replication for high availability
    #[serde(default)]
    pub replication: Option<bool>,
    /// Persistence mode: "disabled", "aof", "snapshot", "aof_and_snapshot"
    pub persistence: Option<String>,
    /// Eviction policy: "noeviction", "allkeys-lru", "volatile-lru", etc.
    pub eviction_policy: Option<String>,
    /// Enable sharding (clustering)
    #[serde(default)]
    pub sharding: Option<bool>,
    /// Number of shards (if sharding is enabled)
    pub shards_count: Option<u32>,
}

/// Build the create_enterprise_database tool
pub fn create_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_enterprise_database")
        .description(
            "Create a new database in the Redis Enterprise cluster. \
             Returns the created database details. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateEnterpriseDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                // Build the request using struct construction (all Option fields have defaults)
                let request = CreateDatabaseRequest {
                    name: input.name.clone(),
                    memory_size: input.memory_size,
                    port: input.port,
                    replication: input.replication,
                    persistence: input.persistence.clone(),
                    eviction_policy: input.eviction_policy.clone(),
                    sharding: input.sharding,
                    shards_count: input.shards_count,
                    shard_count: None,
                    proxy_policy: None,
                    rack_aware: None,
                    module_list: None,
                    crdt: None,
                    authentication_redis_pass: None,
                };

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .create(request)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to create database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

/// Input for updating an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateEnterpriseDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to update
    pub uid: u32,
    /// JSON object with fields to update (e.g., {"memory_size": 2147483648, "replication": true})
    pub updates: Value,
}

/// Build the update_enterprise_database tool
pub fn update_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_database")
        .description(
            "Update configuration of an existing Redis Enterprise database. \
             Pass a JSON object with the fields to update. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateEnterpriseDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .update(input.uid, input.updates)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

/// Input for deleting an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteEnterpriseDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to delete
    pub uid: u32,
}

/// Build the delete_enterprise_database tool
pub fn delete_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_enterprise_database")
        .description(
            "Delete a database from the Redis Enterprise cluster. \
             WARNING: This permanently deletes the database and all its data! \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteEnterpriseDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                handler
                    .delete(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to delete database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Database deleted successfully",
                    "uid": input.uid
                }))
            },
        )
        .build()
}

/// Input for flushing an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FlushEnterpriseDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to flush
    pub bdb_uid: u32,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_enterprise_timeout")]
    pub timeout_seconds: u64,
}

/// Build the flush_enterprise_database tool
pub fn flush_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("flush_enterprise_database")
        .description(
            "Flush all data from a Redis Enterprise database and wait for completion. \
             WARNING: This permanently deletes ALL data in the database! \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, FlushEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<FlushEnterpriseDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                // Use Layer 2 workflow
                flush_database_and_wait(
                    &client,
                    input.bdb_uid,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to flush database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Database flushed successfully",
                    "bdb_uid": input.bdb_uid
                }))
            },
        )
        .build()
}

// ============================================================================
// Role tools
// ============================================================================

/// Input for listing roles (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRolesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_roles tool
pub fn list_roles(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_roles")
        .description(
            "List all roles in the Redis Enterprise cluster. Returns role names, \
             permissions (management, data_access), and database-specific role assignments.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListRolesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListRolesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = RolesHandler::new(client);
                let roles = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list roles: {}", e)))?;

                wrap_list("roles", &roles)
            },
        )
        .build()
}

/// Input for getting a specific role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRoleInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Role UID
    pub uid: u32,
}

/// Build the get_role tool
pub fn get_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_role")
        .description(
            "Get detailed information about a specific role including permissions \
             and database role assignments.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetRoleInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetRoleInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = RolesHandler::new(client);
                let role = handler
                    .get(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get role: {}", e)))?;

                CallToolResult::from_serialize(&role)
            },
        )
        .build()
}

// ============================================================================
// Redis ACL tools
// ============================================================================

/// Input for listing Redis ACLs (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRedisAclsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_redis_acls tool
pub fn list_redis_acls(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_acls")
        .description(
            "List all Redis ACLs in the Redis Enterprise cluster. Returns ACL names, \
             rules, and associated databases.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListRedisAclsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListRedisAclsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = RedisAclHandler::new(client);
                let acls = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list ACLs: {}", e)))?;

                wrap_list("acls", &acls)
            },
        )
        .build()
}

/// Input for getting a specific Redis ACL
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRedisAclInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// ACL UID
    pub uid: u32,
}

/// Build the get_redis_acl tool
pub fn get_redis_acl(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_acl")
        .description(
            "Get detailed information about a specific Redis ACL including the ACL rule string \
             and associated databases.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetRedisAclInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetRedisAclInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = RedisAclHandler::new(client);
                let acl = handler
                    .get(input.uid)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get ACL: {}", e)))?;

                CallToolResult::from_serialize(&acl)
            },
        )
        .build()
}

// ============================================================================
// License Write Operations
// ============================================================================

/// Input for updating license
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateLicenseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// The license key string to install
    pub license_key: String,
}

/// Build the update_license tool
pub fn update_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_license")
        .description(
            "Update the Redis Enterprise cluster license with a new license key. \
             This applies a new license to the cluster. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateLicenseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateLicenseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let request = LicenseUpdateRequest {
                    license: input.license_key,
                };
                let license = handler
                    .update(request)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update license: {}", e)))?;

                CallToolResult::from_serialize(&license)
            },
        )
        .build()
}

/// Input for validating license
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateLicenseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// The license key string to validate
    pub license_key: String,
}

/// Build the validate_license tool
pub fn validate_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("validate_enterprise_license")
        .description(
            "Validate a license key before applying it to the Redis Enterprise cluster. \
             Returns license information if valid, or an error if invalid. \
             This is a dry-run that does not modify the cluster.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ValidateLicenseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ValidateLicenseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let license = handler
                    .validate(&input.license_key)
                    .await
                    .map_err(|e| ToolError::new(format!("License validation failed: {}", e)))?;

                CallToolResult::from_serialize(&license)
            },
        )
        .build()
}

// ============================================================================
// Cluster Configuration Operations
// ============================================================================

/// Input for updating cluster configuration
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateClusterInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// JSON object with cluster settings to update (e.g., {"name": "my-cluster", "email_alerts": true})
    pub updates: Value,
}

/// Build the update_cluster tool
pub fn update_cluster(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_cluster")
        .description(
            "Update Redis Enterprise cluster configuration settings. \
             Pass a JSON object with the fields to update (e.g., name, email_alerts, rack_aware). \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateClusterInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateClusterInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let result = handler
                    .update(input.updates)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update cluster: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting cluster policy (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterPolicyInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cluster_policy tool
pub fn get_cluster_policy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_cluster_policy")
        .description(
            "Get Redis Enterprise cluster policy settings including default shards placement, \
             rack awareness, default Redis version, and other cluster-wide defaults.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterPolicyInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetClusterPolicyInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let policy = handler
                    .policy()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get cluster policy: {}", e)))?;

                CallToolResult::from_serialize(&policy)
            },
        )
        .build()
}

/// Input for updating cluster policy
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateClusterPolicyInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// JSON object with policy settings to update
    /// (e.g., {"default_shards_placement": "sparse", "rack_aware": true, "default_provisioned_redis_version": "7.2"})
    pub policy: Value,
}

/// Build the update_cluster_policy tool
pub fn update_cluster_policy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_cluster_policy")
        .description(
            "Update Redis Enterprise cluster policy settings. \
             Common settings: default_shards_placement (dense/sparse), rack_aware, \
             default_provisioned_redis_version, persistent_node_removal. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateClusterPolicyInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateClusterPolicyInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let result = handler
                    .policy_update(input.policy)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update cluster policy: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Maintenance Mode Operations
// ============================================================================

/// Input for enabling maintenance mode (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnableMaintenanceModeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the enable_maintenance_mode tool
pub fn enable_maintenance_mode(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("enable_enterprise_maintenance_mode")
        .description(
            "Enable maintenance mode on the Redis Enterprise cluster. \
             When enabled, cluster configuration changes are blocked, allowing safe \
             maintenance operations like upgrades. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, EnableMaintenanceModeInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<EnableMaintenanceModeInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                // Enable maintenance mode by setting block_cluster_changes to true
                let result = handler
                    .update(serde_json::json!({"block_cluster_changes": true}))
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to enable maintenance mode: {}", e))
                    })?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Maintenance mode enabled",
                    "result": result
                }))
            },
        )
        .build()
}

/// Input for disabling maintenance mode (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DisableMaintenanceModeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the disable_maintenance_mode tool
pub fn disable_maintenance_mode(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("disable_enterprise_maintenance_mode")
        .description(
            "Disable maintenance mode on the Redis Enterprise cluster. \
             This re-enables cluster configuration changes after maintenance is complete. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DisableMaintenanceModeInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DisableMaintenanceModeInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                // Disable maintenance mode by setting block_cluster_changes to false
                let result = handler
                    .update(serde_json::json!({"block_cluster_changes": false}))
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to disable maintenance mode: {}", e))
                    })?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Maintenance mode disabled",
                    "result": result
                }))
            },
        )
        .build()
}

// ============================================================================
// Certificate Operations
// ============================================================================

/// Input for getting cluster certificates (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterCertificatesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cluster_certificates tool
pub fn get_cluster_certificates(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_cluster_certificates")
        .description(
            "Get all certificates configured on the Redis Enterprise cluster including \
             proxy certificates, syncer certificates, and API certificates.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterCertificatesInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetClusterCertificatesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let certificates = handler
                    .certificates()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get certificates: {}", e)))?;

                CallToolResult::from_serialize(&certificates)
            },
        )
        .build()
}

/// Input for rotating cluster certificates (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RotateClusterCertificatesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the rotate_cluster_certificates tool
pub fn rotate_cluster_certificates(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("rotate_enterprise_cluster_certificates")
        .description(
            "Rotate all certificates on the Redis Enterprise cluster. \
             This generates new certificates and replaces the existing ones. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, RotateClusterCertificatesInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RotateClusterCertificatesInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let result = handler
                    .certificates_rotate()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to rotate certificates: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Certificate rotation initiated",
                    "result": result
                }))
            },
        )
        .build()
}

/// Input for updating cluster certificates
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateClusterCertificatesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Certificate name (e.g., "proxy", "syncer", "api")
    pub name: String,
    /// PEM-encoded certificate content
    pub certificate: String,
    /// PEM-encoded private key content
    pub key: String,
}

/// Build the update_cluster_certificates tool
pub fn update_cluster_certificates(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_cluster_certificates")
        .description(
            "Update a specific certificate on the Redis Enterprise cluster. \
             Provide the certificate name (proxy, syncer, api), the PEM-encoded certificate, \
             and the PEM-encoded private key. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateClusterCertificatesInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateClusterCertificatesInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| super::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let body = serde_json::json!({
                    "name": input.name,
                    "certificate": input.certificate,
                    "key": input.key
                });
                let result = handler
                    .update_cert(body)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update certificate: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Certificate updated successfully",
                    "name": input.name,
                    "result": result
                }))
            },
        )
        .build()
}

/// Instructions text describing all Enterprise tools
pub fn instructions() -> &'static str {
    r#"
### Redis Enterprise - Cluster
- get_cluster: Get cluster information
- get_cluster_stats: Get cluster statistics
- update_enterprise_cluster: Update cluster configuration (write)
- get_enterprise_cluster_policy: Get cluster policy settings
- update_enterprise_cluster_policy: Update cluster policy (write)
- enable_enterprise_maintenance_mode: Enable maintenance mode (write)
- disable_enterprise_maintenance_mode: Disable maintenance mode (write)
- get_enterprise_cluster_certificates: Get cluster certificates
- rotate_enterprise_cluster_certificates: Rotate all certificates (write)
- update_enterprise_cluster_certificates: Update a specific certificate (write)

### Redis Enterprise - License
- get_license: Get license information (type, expiration, features)
- get_license_usage: Get license utilization (shards, nodes, RAM vs limits)
- update_enterprise_license: Update cluster license with a new key (write)
- validate_enterprise_license: Validate a license key before applying

### Redis Enterprise - Logs
- list_logs: List cluster event logs (with time range and pagination)

### Redis Enterprise - Databases
- list_enterprise_databases: List all databases
- get_enterprise_database: Get database details
- get_database_stats: Get database statistics
- get_database_endpoints: Get connection endpoints
- list_database_alerts: Get alerts for a database

### Redis Enterprise - Nodes
- list_nodes: List cluster nodes
- get_node: Get node details
- get_node_stats: Get node statistics

### Redis Enterprise - Users & Alerts
- list_enterprise_users: List cluster users
- get_enterprise_user: Get user details
- list_alerts: List all active alerts

### Redis Enterprise - Shards
- list_shards: List database shards (with optional database filter)
- get_shard: Get shard details by UID

### Redis Enterprise - Aggregate Stats
- get_all_nodes_stats: Get stats for all nodes in one call
- get_all_databases_stats: Get stats for all databases in one call
- get_shard_stats: Get stats for a specific shard
- get_all_shards_stats: Get stats for all shards in one call

### Redis Enterprise - Debug Info
- list_debug_info_tasks: List debug info collection tasks
- get_debug_info_status: Get status of a debug info collection task

### Redis Enterprise - Modules
- list_modules: List installed Redis modules (RedisJSON, RediSearch, etc.)
- get_module: Get details about a specific module

### Redis Enterprise - Roles
- list_enterprise_roles: List all roles in the cluster
- get_enterprise_role: Get role details and permissions

### Redis Enterprise - ACLs
- list_enterprise_acls: List all Redis ACLs
- get_enterprise_acl: Get ACL details and rules

### Redis Enterprise - Write Operations (require --read-only=false)
- backup_enterprise_database: Trigger a database backup and wait for completion
- import_enterprise_database: Import data into a database and wait for completion
- create_enterprise_database: Create a new database
- update_enterprise_database: Update database configuration
- delete_enterprise_database: Delete a database
- flush_enterprise_database: Flush all data from a database
"#
}

/// Build an MCP sub-router containing all Enterprise tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Cluster
        .tool(get_cluster(state.clone()))
        .tool(get_cluster_stats(state.clone()))
        .tool(update_cluster(state.clone()))
        .tool(get_cluster_policy(state.clone()))
        .tool(update_cluster_policy(state.clone()))
        .tool(enable_maintenance_mode(state.clone()))
        .tool(disable_maintenance_mode(state.clone()))
        .tool(get_cluster_certificates(state.clone()))
        .tool(rotate_cluster_certificates(state.clone()))
        .tool(update_cluster_certificates(state.clone()))
        // License
        .tool(get_license(state.clone()))
        .tool(get_license_usage(state.clone()))
        .tool(update_license(state.clone()))
        .tool(validate_license(state.clone()))
        // Logs
        .tool(list_logs(state.clone()))
        // Databases
        .tool(list_databases(state.clone()))
        .tool(get_database(state.clone()))
        .tool(get_database_stats(state.clone()))
        .tool(get_database_endpoints(state.clone()))
        .tool(list_database_alerts(state.clone()))
        // Nodes
        .tool(list_nodes(state.clone()))
        .tool(get_node(state.clone()))
        .tool(get_node_stats(state.clone()))
        // Users & Alerts
        .tool(list_users(state.clone()))
        .tool(get_user(state.clone()))
        .tool(list_alerts(state.clone()))
        // Shards
        .tool(list_shards(state.clone()))
        .tool(get_shard(state.clone()))
        // Aggregate Stats
        .tool(get_all_nodes_stats(state.clone()))
        .tool(get_all_databases_stats(state.clone()))
        .tool(get_shard_stats(state.clone()))
        .tool(get_all_shards_stats(state.clone()))
        // Debug Info
        .tool(list_debug_info_tasks(state.clone()))
        .tool(get_debug_info_status(state.clone()))
        // Modules
        .tool(list_modules(state.clone()))
        .tool(get_module(state.clone()))
        // Roles
        .tool(list_roles(state.clone()))
        .tool(get_role(state.clone()))
        // ACLs
        .tool(list_redis_acls(state.clone()))
        .tool(get_redis_acl(state.clone()))
        // Write Operations
        .tool(backup_enterprise_database(state.clone()))
        .tool(import_enterprise_database(state.clone()))
        .tool(create_enterprise_database(state.clone()))
        .tool(update_enterprise_database(state.clone()))
        .tool(delete_enterprise_database(state.clone()))
        .tool(flush_enterprise_database(state.clone()))
}
