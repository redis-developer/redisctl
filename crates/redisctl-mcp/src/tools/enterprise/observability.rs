//! Alerts, logs, aggregate stats, shards, debug info, and module tools

use std::sync::Arc;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::debuginfo::{DebugInfoHandler, DebugInfoRequest};
use redis_enterprise::logs::{LogsHandler, LogsQuery};
use redis_enterprise::modules::ModuleHandler;
use redis_enterprise::shards::ShardHandler;
use redis_enterprise::stats::StatsHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;
use crate::tools::wrap_list;

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
        .description("List all active alerts.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListAlertsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = AlertHandler::new(client);
                let alerts = handler.list().await.tool_context("Failed to list alerts")?;

                wrap_list("alerts", &alerts)
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
        .description("List cluster event logs. Supports filtering by time range and pagination.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListLogsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

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
                    .tool_context("Failed to list logs")?;

                wrap_list("logs", &logs)
            },
        )
        .build()
}

// ============================================================================
// Aggregate Stats tools
// ============================================================================

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
        .description("Get current statistics for all nodes including CPU, memory, and network metrics.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAllNodesStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler
                    .nodes_last()
                    .await
                    .tool_context("Failed to get all nodes stats")?;

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
            "Get current statistics for all databases including latency, throughput, and memory usage.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAllDatabasesStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler.databases_last().await.tool_context("Failed to get all databases stats")?;

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
        .description("Get current statistics for a specific shard.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetShardStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler
                    .shard(input.uid, None)
                    .await
                    .tool_context("Failed to get shard stats")?;

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
        .description("Get current statistics for all shards.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAllShardsStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);
                let stats = handler
                    .shards(None)
                    .await
                    .tool_context("Failed to get all shards stats")?;

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
        .description("List all shards. Optionally filter by database UID.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListShardsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ShardHandler::new(client);
                let shards = if let Some(db_uid) = input.database_uid {
                    handler
                        .list_by_database(db_uid)
                        .await
                        .tool_context("Failed to list shards")?
                } else {
                    handler.list().await.tool_context("Failed to list shards")?
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
            "Get shard details including role (master/replica), status, and assigned node.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetShardInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ShardHandler::new(client);
                let shard = handler
                    .get(&input.uid)
                    .await
                    .tool_context("Failed to get shard")?;

                CallToolResult::from_serialize(&shard)
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
        .description("List all debug info collection tasks and their statuses.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListDebugInfoTasksInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DebugInfoHandler::new(client);
                let tasks = handler
                    .list()
                    .await
                    .tool_context("Failed to list debug info tasks")?;

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
        .description("Get the status of a debug info collection task by ID.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetDebugInfoStatusInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DebugInfoHandler::new(client);
                let status = handler
                    .status(&input.task_id)
                    .await
                    .tool_context("Failed to get debug info status")?;

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
        .description("List all installed Redis modules.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListModulesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ModuleHandler::new(client);
                let modules = handler
                    .list()
                    .await
                    .tool_context("Failed to list modules")?;

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
        .description("Get details of a specific Redis module by UID.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetModuleInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ModuleHandler::new(client);
                let module = handler
                    .get(&input.uid)
                    .await
                    .tool_context("Failed to get module")?;

                CallToolResult::from_serialize(&module)
            },
        )
        .build()
}

// ============================================================================
// Filtered Shard tools
// ============================================================================

/// Input for listing shards by database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListShardsByDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to list shards for
    pub bdb_uid: u32,
}

/// Build the list_shards_by_database tool
pub fn list_shards_by_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_shards_by_database")
        .description("List all shards for a specific database.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListShardsByDatabaseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ShardHandler::new(client);
                let shards = handler
                    .list_by_database(input.bdb_uid)
                    .await
                    .tool_context("Failed to list shards by database")?;

                wrap_list("shards", &shards)
            },
        )
        .build()
}

/// Input for listing shards by node
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListShardsByNodeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Node UID to list shards for
    pub node_uid: u32,
}

/// Build the list_shards_by_node tool
pub fn list_shards_by_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_shards_by_node")
        .description("List all shards on a specific node.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListShardsByNodeInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ShardHandler::new(client);
                let shards = handler
                    .list_by_node(input.node_uid)
                    .await
                    .tool_context("Failed to list shards by node")?;

                wrap_list("shards", &shards)
            },
        )
        .build()
}

// ============================================================================
// Alert Acknowledge
// ============================================================================

/// Input for acknowledging an alert
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AcknowledgeAlertInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Alert UID to acknowledge
    pub alert_uid: String,
}

/// Build the acknowledge_enterprise_alert tool
pub fn acknowledge_enterprise_alert(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("acknowledge_enterprise_alert")
        .description("Acknowledge (clear) a specific alert by ID.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<AcknowledgeAlertInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = AlertHandler::new(client);
                handler
                    .clear(&input.alert_uid)
                    .await
                    .tool_context("Failed to acknowledge alert")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Alert acknowledged successfully",
                    "alert_uid": input.alert_uid
                }))
            },
        )
        .build()
}

// ============================================================================
// Debug Info Create
// ============================================================================

/// Input for creating a debug info collection task
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDebugInfoInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// List of node UIDs to collect debug info from (if not specified, collects from all nodes)
    #[serde(default)]
    pub node_uids: Option<Vec<u32>>,
    /// List of database UIDs to collect debug info for (if not specified, collects for all databases)
    #[serde(default)]
    pub bdb_uids: Option<Vec<u32>>,
}

/// Build the create_debug_info tool
pub fn create_debug_info(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_debug_info")
        .description("Start a debug info collection task. Optionally scope to specific nodes or databases.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateDebugInfoInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let request = DebugInfoRequest {
                    node_uids: input.node_uids,
                    bdb_uids: input.bdb_uids,
                    include_logs: None,
                    include_metrics: None,
                    include_configs: None,
                    time_range: None,
                };

                let handler = DebugInfoHandler::new(client);
                let status = handler
                    .create(request)
                    .await
                    .tool_context("Failed to create debug info collection")?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
    "list_alerts",
    "acknowledge_enterprise_alert",
    "list_logs",
    "get_all_nodes_stats",
    "get_all_databases_stats",
    "get_shard_stats",
    "get_all_shards_stats",
    "list_shards",
    "get_shard",
    "list_shards_by_database",
    "list_shards_by_node",
    "list_debug_info_tasks",
    "get_debug_info_status",
    "create_debug_info",
    "list_modules",
    "get_module",
];

/// Build an MCP sub-router containing observability tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Alerts
        .tool(list_alerts(state.clone()))
        .tool(acknowledge_enterprise_alert(state.clone()))
        // Logs
        .tool(list_logs(state.clone()))
        // Aggregate Stats
        .tool(get_all_nodes_stats(state.clone()))
        .tool(get_all_databases_stats(state.clone()))
        .tool(get_shard_stats(state.clone()))
        .tool(get_all_shards_stats(state.clone()))
        // Shards
        .tool(list_shards(state.clone()))
        .tool(get_shard(state.clone()))
        .tool(list_shards_by_database(state.clone()))
        .tool(list_shards_by_node(state.clone()))
        // Debug Info
        .tool(list_debug_info_tasks(state.clone()))
        .tool(get_debug_info_status(state.clone()))
        .tool(create_debug_info(state.clone()))
        // Modules
        .tool(list_modules(state.clone()))
        .tool(get_module(state.clone()))
}
