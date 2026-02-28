//! Database, CRDB, and database alert tools

use std::sync::Arc;
use std::time::Duration;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::bdb::{CreateDatabaseRequest, DatabaseHandler};
use redis_enterprise::crdb::CrdbHandler;
use redis_enterprise::stats::{StatsHandler, StatsQuery};
use redisctl_core::enterprise::{
    backup_database_and_wait, flush_database_and_wait, import_database_and_wait,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;
use crate::tools::wrap_list;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListDatabasesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListDatabasesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let databases = handler
                    .list()
                    .await
                    .tool_context("Failed to list databases")?;

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetDatabaseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .get(input.uid)
                    .await
                    .tool_context("Failed to get database")?;

                CallToolResult::from_serialize(&database)
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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetDatabaseStatsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetDatabaseStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

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
                        .database(input.uid, Some(query))
                        .await
                        .tool_context("Failed to get database stats")?;
                    CallToolResult::from_serialize(&stats)
                } else {
                    let stats = handler
                        .database_last(input.uid)
                        .await
                        .tool_context("Failed to get database stats")?;
                    CallToolResult::from_serialize(&stats)
                }
            },
        )
        .build()
}

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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetDatabaseEndpointsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetDatabaseEndpointsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let endpoints = handler
                    .endpoints(input.uid)
                    .await
                    .tool_context("Failed to get endpoints")?;

                wrap_list("endpoints", &endpoints)
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
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListDatabaseAlertsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListDatabaseAlertsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = AlertHandler::new(client);
                let alerts = handler
                    .list_by_database(input.uid)
                    .await
                    .tool_context("Failed to list database alerts")?;

                wrap_list("alerts", &alerts)
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
        .non_destructive()
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
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                // Use Layer 2 workflow
                backup_database_and_wait(
                    &client,
                    input.bdb_uid,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .tool_context("Failed to backup database")?;

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
        .non_destructive()
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
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

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
                .tool_context("Failed to import database")?;

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
        .non_destructive()
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
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

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
                    .tool_context("Failed to create database")?;

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
        .non_destructive()
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
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .update(input.uid, input.updates)
                    .await
                    .tool_context("Failed to update database")?;

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
            "DANGEROUS: Permanently deletes a database from the Redis Enterprise cluster \
             and all its data. This action cannot be undone. Requires write permission.",
        )
        .destructive()
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
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                handler
                    .delete(input.uid)
                    .await
                    .tool_context("Failed to delete database")?;

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
            "DANGEROUS: Removes all data from a Redis Enterprise database. \
             This action cannot be undone. Requires write permission.",
        )
        .destructive()
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
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                // Use Layer 2 workflow
                flush_database_and_wait(
                    &client,
                    input.bdb_uid,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .tool_context("Failed to flush database")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Database flushed successfully",
                    "bdb_uid": input.bdb_uid
                }))
            },
        )
        .build()
}

// ============================================================================
// CRDB (Active-Active) tools
// ============================================================================

/// Input for listing CRDBs (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCrdbsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_enterprise_crdbs tool
pub fn list_enterprise_crdbs(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_crdbs")
        .description(
            "List all Active-Active (CRDB) databases in the Redis Enterprise cluster. \
             Returns database names, GUIDs, status, and instance information.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListCrdbsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListCrdbsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = CrdbHandler::new(client);
                let crdbs = handler.list().await.tool_context("Failed to list CRDBs")?;

                wrap_list("crdbs", &crdbs)
            },
        )
        .build()
}

/// Input for getting a specific CRDB
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrdbInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// CRDB GUID (globally unique identifier)
    pub guid: String,
}

/// Build the get_enterprise_crdb tool
pub fn get_enterprise_crdb(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_crdb")
        .description(
            "Get detailed information about a specific Active-Active (CRDB) database \
             including instances, replication status, and configuration.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetCrdbInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetCrdbInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = CrdbHandler::new(client);
                let crdb = handler
                    .get(&input.guid)
                    .await
                    .tool_context("Failed to get CRDB")?;

                CallToolResult::from_serialize(&crdb)
            },
        )
        .build()
}

/// Input for getting CRDB tasks
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrdbTasksInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// CRDB GUID (globally unique identifier)
    pub guid: String,
}

/// Build the get_enterprise_crdb_tasks tool
pub fn get_enterprise_crdb_tasks(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_crdb_tasks")
        .description(
            "Get tasks for a specific Active-Active (CRDB) database. \
             Returns pending and completed tasks related to CRDB operations.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetCrdbTasksInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetCrdbTasksInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = CrdbHandler::new(client);
                let tasks = handler
                    .tasks(&input.guid)
                    .await
                    .tool_context("Failed to get CRDB tasks")?;

                CallToolResult::from_serialize(&tasks)
            },
        )
        .build()
}

/// Build an MCP sub-router containing database tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Databases
        .tool(list_databases(state.clone()))
        .tool(get_database(state.clone()))
        .tool(get_database_stats(state.clone()))
        .tool(get_database_endpoints(state.clone()))
        .tool(list_database_alerts(state.clone()))
        // CRDBs (Active-Active)
        .tool(list_enterprise_crdbs(state.clone()))
        .tool(get_enterprise_crdb(state.clone()))
        .tool(get_enterprise_crdb_tasks(state.clone()))
        // Database Write Operations
        .tool(backup_enterprise_database(state.clone()))
        .tool(import_enterprise_database(state.clone()))
        .tool(create_enterprise_database(state.clone()))
        .tool(update_enterprise_database(state.clone()))
        .tool(delete_enterprise_database(state.clone()))
        .tool(flush_enterprise_database(state.clone()))
}
