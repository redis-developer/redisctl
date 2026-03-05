//! Database, CRDB, and database alert tools

use std::sync::Arc;
use std::time::Duration;

use redis_enterprise::alerts::AlertHandler;
use redis_enterprise::bdb::{CreateDatabaseRequest, DatabaseHandler, DatabaseUpgradeRequest};
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
        .description("List all databases. Supports filtering by name and status.")
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
        .description("Get database details by UID.")
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
            "Get statistics for a specific database. Optionally specify interval and time range \
             for historical data.",
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
        .description("Get connection endpoints for a specific database.")
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
        .description("List all alerts for a specific database.")
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
        .description("Trigger a database backup and wait for completion.")
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
            "Import data into a database from an external source and wait for completion. \
             WARNING: If flush is true, existing data will be deleted before import.",
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
            "Create a new database on the Enterprise cluster. \
             Prerequisites: 1) get_cluster -- verify the cluster is healthy and has capacity. \
             2) list_enterprise_databases -- review existing databases.",
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
        .description("Update database configuration. Pass fields to update as JSON.")
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
        .description("DANGEROUS: Delete a database and all its data.")
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteEnterpriseDatabaseInput>| async move {
                // Check destructive permission
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
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
        .description("DANGEROUS: Flush all data from a database.")
        .destructive()
        .extractor_handler_typed::<_, _, _, FlushEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<FlushEnterpriseDatabaseInput>| async move {
                // Check destructive permission
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
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

/// Input for exporting an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportEnterpriseDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to export
    pub uid: u32,
    /// Export location (e.g., S3 URL or FTP path)
    pub export_location: String,
}

/// Build the export_enterprise_database tool
pub fn export_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("export_enterprise_database")
        .description("Export a database to a specified location (e.g., S3, FTP).")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ExportEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ExportEnterpriseDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let response = handler
                    .export(input.uid, &input.export_location)
                    .await
                    .tool_context("Failed to export database")?;

                CallToolResult::from_serialize(&response)
            },
        )
        .build()
}

/// Input for restoring an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestoreEnterpriseDatabaseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to restore
    pub uid: u32,
    /// Optional backup UID to restore from (uses latest if not specified)
    #[serde(default)]
    pub backup_uid: Option<String>,
}

/// Build the restore_enterprise_database tool
pub fn restore_enterprise_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("restore_enterprise_database")
        .description("Restore a database from a backup.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RestoreEnterpriseDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RestoreEnterpriseDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = DatabaseHandler::new(client);
                let response = handler
                    .restore(input.uid, input.backup_uid.as_deref())
                    .await
                    .tool_context("Failed to restore database")?;

                CallToolResult::from_serialize(&response)
            },
        )
        .build()
}

/// Input for upgrading Redis version of an Enterprise database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpgradeEnterpriseDatabaseRedisInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Database UID to upgrade
    pub uid: u32,
    /// Target Redis version (defaults to latest if not specified)
    #[serde(default)]
    pub redis_version: Option<String>,
    /// Restart shards even if no version change
    #[serde(default)]
    pub force_restart: Option<bool>,
    /// Allow data loss in non-replicated, non-persistent databases
    #[serde(default)]
    pub may_discard_data: Option<bool>,
}

/// Build the upgrade_enterprise_database_redis tool
pub fn upgrade_enterprise_database_redis(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("upgrade_enterprise_database_redis")
        .description("Upgrade the Redis version of a database.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpgradeEnterpriseDatabaseRedisInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpgradeEnterpriseDatabaseRedisInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let request = DatabaseUpgradeRequest {
                    redis_version: input.redis_version,
                    force_restart: input.force_restart,
                    may_discard_data: input.may_discard_data,
                    ..Default::default()
                };

                let handler = DatabaseHandler::new(client);
                let response = handler
                    .upgrade_redis_version(input.uid, request)
                    .await
                    .tool_context("Failed to upgrade database Redis version")?;

                CallToolResult::from_serialize(&response)
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
        .description("List all Active-Active (CRDB) databases.")
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
        .description("Get details of a specific Active-Active (CRDB) database by GUID.")
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
        .description("Get tasks for a specific Active-Active (CRDB) database.")
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

/// Input for creating an Active-Active (CRDB) database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateEnterpriseCrdbInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Full CRDB configuration as JSON (name, memory_size, instances, etc.)
    pub request: Value,
}

/// Build the create_enterprise_crdb tool
pub fn create_enterprise_crdb(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_enterprise_crdb")
        .description("Create a new Active-Active (CRDB) database. Pass full configuration as JSON.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateEnterpriseCrdbInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateEnterpriseCrdbInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let crdb: Value = client
                    .post("/v1/crdbs", &input.request)
                    .await
                    .tool_context("Failed to create CRDB")?;

                CallToolResult::from_serialize(&crdb)
            },
        )
        .build()
}

/// Input for updating an Active-Active (CRDB) database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateEnterpriseCrdbInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// CRDB GUID (globally unique identifier)
    pub guid: String,
    /// JSON object with fields to update
    pub updates: Value,
}

/// Build the update_enterprise_crdb tool
pub fn update_enterprise_crdb(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_crdb")
        .description("Update an Active-Active (CRDB) database. Pass fields to update as JSON.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateEnterpriseCrdbInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateEnterpriseCrdbInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = CrdbHandler::new(client);
                let crdb = handler
                    .update(&input.guid, input.updates)
                    .await
                    .tool_context("Failed to update CRDB")?;

                CallToolResult::from_serialize(&crdb)
            },
        )
        .build()
}

/// Input for deleting an Active-Active (CRDB) database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteEnterpriseCrdbInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// CRDB GUID (globally unique identifier) to delete
    pub guid: String,
}

/// Build the delete_enterprise_crdb tool
pub fn delete_enterprise_crdb(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_enterprise_crdb")
        .description("DANGEROUS: Delete an Active-Active (CRDB) database across all participating clusters.")
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteEnterpriseCrdbInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteEnterpriseCrdbInput>| async move {
                // Check destructive permission
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = CrdbHandler::new(client);
                handler
                    .delete(&input.guid)
                    .await
                    .tool_context("Failed to delete CRDB")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "CRDB deleted successfully",
                    "guid": input.guid
                }))
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
    "list_enterprise_databases",
    "get_enterprise_database",
    "get_database_stats",
    "get_database_endpoints",
    "list_database_alerts",
    "backup_enterprise_database",
    "import_enterprise_database",
    "create_enterprise_database",
    "update_enterprise_database",
    "delete_enterprise_database",
    "flush_enterprise_database",
    "export_enterprise_database",
    "restore_enterprise_database",
    "upgrade_enterprise_database_redis",
    "list_enterprise_crdbs",
    "get_enterprise_crdb",
    "get_enterprise_crdb_tasks",
    "create_enterprise_crdb",
    "update_enterprise_crdb",
    "delete_enterprise_crdb",
];

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
        // CRDB Write Operations
        .tool(create_enterprise_crdb(state.clone()))
        .tool(update_enterprise_crdb(state.clone()))
        .tool(delete_enterprise_crdb(state.clone()))
        // Database Write Operations
        .tool(backup_enterprise_database(state.clone()))
        .tool(import_enterprise_database(state.clone()))
        .tool(create_enterprise_database(state.clone()))
        .tool(update_enterprise_database(state.clone()))
        .tool(delete_enterprise_database(state.clone()))
        .tool(flush_enterprise_database(state.clone()))
        .tool(export_enterprise_database(state.clone()))
        .tool(restore_enterprise_database(state.clone()))
        .tool(upgrade_enterprise_database_redis(state.clone()))
}
