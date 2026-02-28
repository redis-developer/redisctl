//! Alerts, logs, aggregate stats, shards, debug info, and module tools

use std::sync::Arc;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::debuginfo::DebugInfoHandler;
use redis_enterprise::logs::{LogsHandler, LogsQuery};
use redis_enterprise::modules::ModuleHandler;
use redis_enterprise::shards::ShardHandler;
use redis_enterprise::stats::StatsHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, Tool, ToolBuilder, ToolError};

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
        .description("List all active alerts in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ListAlertsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListAlertsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ListLogsInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to list logs: {}", e)))?;

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
        .description(
            "Get current statistics for all nodes in the Redis Enterprise cluster in a single \
             call. Returns aggregated stats per node including CPU, memory, and network metrics.",
        )
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetAllNodesStatsInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get all nodes stats: {}", e))
                    })?;

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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetAllDatabasesStatsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAllDatabasesStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetShardStatsInput>(
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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetAllShardsStatsInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get all shards stats: {}", e))
                    })?;

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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ListShardsInput>(
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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetShardInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get shard: {}", e)))?;

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
        .description(
            "List all debug info collection tasks in the Redis Enterprise cluster. Returns task \
             IDs, statuses (queued, running, completed, failed), and download URLs for completed \
             collections.",
        )
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ListDebugInfoTasksInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to list debug info tasks: {}", e))
                    })?;

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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetDebugInfoStatusInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get debug info status: {}", e))
                    })?;

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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ListModulesInput>(
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
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetModuleInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get module: {}", e)))?;

                CallToolResult::from_serialize(&module)
            },
        )
        .build()
}

pub(super) const INSTRUCTIONS: &str = r#"
### Redis Enterprise - Alerts & Logs
- list_alerts: List all active alerts
- list_logs: List cluster event logs (with time range and pagination)

### Redis Enterprise - Aggregate Stats
- get_all_nodes_stats: Get stats for all nodes in one call
- get_all_databases_stats: Get stats for all databases in one call
- get_shard_stats: Get stats for a specific shard
- get_all_shards_stats: Get stats for all shards in one call

### Redis Enterprise - Shards
- list_shards: List database shards (with optional database filter)
- get_shard: Get shard details by UID

### Redis Enterprise - Debug Info
- list_debug_info_tasks: List debug info collection tasks
- get_debug_info_status: Get status of a debug info collection task

### Redis Enterprise - Modules
- list_modules: List installed Redis modules (RedisJSON, RediSearch, etc.)
- get_module: Get details about a specific module
"#;

/// Build an MCP sub-router containing observability tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Alerts
        .tool(list_alerts(state.clone()))
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
        // Debug Info
        .tool(list_debug_info_tasks(state.clone()))
        .tool(get_debug_info_status(state.clone()))
        // Modules
        .tool(list_modules(state.clone()))
        .tool(get_module(state.clone()))
}
