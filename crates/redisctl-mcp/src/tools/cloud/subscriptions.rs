//! Subscription and database tools for Redis Cloud

use std::sync::Arc;
use std::time::Duration;

use redis_cloud::databases::DatabaseCreateRequest;
use redis_cloud::flexible::{DatabaseHandler, SubscriptionHandler};
use redisctl_core::cloud::{
    backup_database_and_wait, create_database_and_wait, delete_database_and_wait,
    delete_subscription_and_wait, flush_database_and_wait, import_database_and_wait,
    update_database_and_wait,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

/// Input for listing subscriptions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSubscriptionsInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_subscriptions tool
pub fn list_subscriptions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_subscriptions")
        .description("List all Redis Cloud subscriptions accessible with the current credentials. Returns JSON with subscription details.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListSubscriptionsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListSubscriptionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let account_subs = handler
                    .get_all_subscriptions()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list subscriptions: {}", e)))?;

                CallToolResult::from_serialize(&account_subs)
            },
        )
        .build()
}

/// Input for getting a specific subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSubscriptionInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_subscription tool
pub fn get_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_subscription")
        .description("Get detailed information about a specific Redis Cloud subscription. Returns JSON with full subscription details.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetSubscriptionInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let subscription = handler
                    .get_subscription_by_id(input.subscription_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get subscription: {}", e)))?;

                CallToolResult::from_serialize(&subscription)
            },
        )
        .build()
}

/// Input for listing databases
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabasesInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_databases tool
pub fn list_databases(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_databases")
        .description(
            "List all databases in a Redis Cloud subscription. Returns JSON with database details.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListDatabasesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListDatabasesInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let databases = handler
                    .get_subscription_databases(input.subscription_id, None, None)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list databases: {}", e)))?;

                CallToolResult::from_serialize(&databases)
            },
        )
        .build()
}

/// Input for getting a specific database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_database tool
pub fn get_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database")
        .description("Get detailed information about a specific Redis Cloud database. Returns JSON with full database configuration.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetDatabaseInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .get_subscription_database_by_id(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

// ============================================================================
// Database operations tools
// ============================================================================

/// Input for getting database backup status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBackupStatusInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Optional region name for Active-Active databases
    #[serde(default)]
    pub region_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_backup_status tool
pub fn get_backup_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_backup_status")
        .description("Get backup status and history for a Redis Cloud database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetBackupStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetBackupStatusInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let status = handler
                    .get_database_backup_status(
                        input.subscription_id,
                        input.database_id,
                        input.region_name,
                    )
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get backup status: {}", e)))?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for getting slow log
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSlowLogInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Optional region name for Active-Active databases
    #[serde(default)]
    pub region_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_slow_log tool
pub fn get_slow_log(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_slow_log")
        .description(
            "Get slow log entries for a Redis Cloud database. Shows slow queries for debugging.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetSlowLogInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetSlowLogInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let log = handler
                    .get_slow_log(input.subscription_id, input.database_id, input.region_name)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get slow log: {}", e)))?;

                CallToolResult::from_serialize(&log)
            },
        )
        .build()
}

/// Input for getting database tags
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTagsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_tags tool
pub fn get_tags(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_tags")
        .description("Get tags attached to a Redis Cloud database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetTagsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetTagsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let tags = handler
                    .get_tags(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get tags: {}", e)))?;

                CallToolResult::from_serialize(&tags)
            },
        )
        .build()
}

/// Input for getting database certificate
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCertificateInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_database_certificate tool
pub fn get_database_certificate(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_certificate")
        .description(
            "Get the TLS/SSL certificate for a Redis Cloud database. \
             Returns the public certificate in PEM format for TLS connections.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetCertificateInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetCertificateInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let cert = handler
                    .get_subscription_database_certificate(
                        input.subscription_id,
                        input.database_id,
                    )
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get certificate: {}", e)))?;

                CallToolResult::from_serialize(&cert)
            },
        )
        .build()
}

// ============================================================================
// Write operations (require write permission)
// ============================================================================

fn default_replication() -> bool {
    true
}

fn default_protocol() -> String {
    "redis".to_string()
}

fn default_timeout() -> u64 {
    600
}

/// Input for creating a database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDatabaseInput {
    /// Subscription ID to create the database in
    pub subscription_id: i32,
    /// Database name
    pub name: String,
    /// Memory limit in GB (e.g., 1.0, 2.5, 10.0)
    pub memory_limit_in_gb: f64,
    /// Enable replication for high availability (default: true)
    #[serde(default = "default_replication")]
    pub replication: bool,
    /// Protocol: "redis" (RESP2), "stack" (RESP2 with modules), or "memcached"
    #[serde(default = "default_protocol")]
    pub protocol: String,
    /// Data persistence: "none", "aof-every-1-second", "aof-every-write", "snapshot-every-1-hour", etc.
    #[serde(default)]
    pub data_persistence: Option<String>,
    /// Timeout in seconds to wait for database creation (default: 600)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_database tool
pub fn create_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_database")
        .description(
            "Create a new Redis Cloud database and wait for it to be ready. \
             Returns the created database details. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Build the request using Layer 1's TypedBuilder
                let request = match (input.protocol.as_str(), input.data_persistence.as_ref()) {
                    ("redis", None) => DatabaseCreateRequest::builder()
                        .name(&input.name)
                        .memory_limit_in_gb(input.memory_limit_in_gb)
                        .replication(input.replication)
                        .build(),
                    ("redis", Some(persistence)) => DatabaseCreateRequest::builder()
                        .name(&input.name)
                        .memory_limit_in_gb(input.memory_limit_in_gb)
                        .replication(input.replication)
                        .data_persistence(persistence)
                        .build(),
                    (protocol, None) => DatabaseCreateRequest::builder()
                        .name(&input.name)
                        .memory_limit_in_gb(input.memory_limit_in_gb)
                        .replication(input.replication)
                        .protocol(protocol)
                        .build(),
                    (protocol, Some(persistence)) => DatabaseCreateRequest::builder()
                        .name(&input.name)
                        .memory_limit_in_gb(input.memory_limit_in_gb)
                        .replication(input.replication)
                        .protocol(protocol)
                        .data_persistence(persistence)
                        .build(),
                };

                // Use Layer 2 workflow - no progress callback needed for MCP
                let database = create_database_and_wait(
                    &client,
                    input.subscription_id,
                    &request,
                    Duration::from_secs(input.timeout_seconds),
                    None, // MCP doesn't need progress callbacks
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to create database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

/// Input for updating a database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateDatabaseInput {
    /// Subscription ID containing the database
    pub subscription_id: i32,
    /// Database ID to update
    pub database_id: i32,
    /// New database name (optional)
    #[serde(default)]
    pub name: Option<String>,
    /// New memory limit in GB (optional)
    #[serde(default)]
    pub memory_limit_in_gb: Option<f64>,
    /// Change replication setting (optional)
    #[serde(default)]
    pub replication: Option<bool>,
    /// Change data persistence (optional)
    #[serde(default)]
    pub data_persistence: Option<String>,
    /// Change eviction policy (optional)
    #[serde(default)]
    pub data_eviction_policy: Option<String>,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_database tool
pub fn update_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_database")
        .description(
            "Update an existing Redis Cloud database configuration. \
             Returns the updated database details. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateDatabaseInput>| async move {
                use redis_cloud::databases::DatabaseUpdateRequest;

                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Build the update request
                let mut request = DatabaseUpdateRequest::builder().build();
                request.name = input.name;
                request.memory_limit_in_gb = input.memory_limit_in_gb;
                request.replication = input.replication;
                request.data_persistence = input.data_persistence;
                request.data_eviction_policy = input.data_eviction_policy;

                // Validate at least one field is set
                if request.name.is_none()
                    && request.memory_limit_in_gb.is_none()
                    && request.replication.is_none()
                    && request.data_persistence.is_none()
                    && request.data_eviction_policy.is_none()
                {
                    return Err(McpError::tool(
                        "At least one update field is required",
                    ));
                }

                // Use Layer 2 workflow
                let database = update_database_and_wait(
                    &client,
                    input.subscription_id,
                    input.database_id,
                    &request,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to update database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

/// Input for deleting a database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDatabaseInput {
    /// Subscription ID containing the database
    pub subscription_id: i32,
    /// Database ID to delete
    pub database_id: i32,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_database tool
pub fn delete_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_database")
        .description(
            "Delete a Redis Cloud database. This is a destructive operation. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Use Layer 2 workflow
                delete_database_and_wait(
                    &client,
                    input.subscription_id,
                    input.database_id,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to delete database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Database deleted successfully",
                    "subscription_id": input.subscription_id,
                    "database_id": input.database_id
                }))
            },
        )
        .build()
}

/// Input for backing up a database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BackupDatabaseInput {
    /// Subscription ID containing the database
    pub subscription_id: i32,
    /// Database ID to backup
    pub database_id: i32,
    /// Region name (required for Active-Active databases)
    #[serde(default)]
    pub region_name: Option<String>,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the backup_database tool
pub fn backup_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("backup_database")
        .description(
            "Trigger a manual backup of a Redis Cloud database. \
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
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Use Layer 2 workflow
                backup_database_and_wait(
                    &client,
                    input.subscription_id,
                    input.database_id,
                    input.region_name.as_deref(),
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to backup database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Backup completed successfully",
                    "subscription_id": input.subscription_id,
                    "database_id": input.database_id
                }))
            },
        )
        .build()
}

fn default_import_timeout() -> u64 {
    1800 // Imports can take longer
}

/// Input for importing data into a database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportDatabaseInput {
    /// Subscription ID containing the database
    pub subscription_id: i32,
    /// Database ID to import into
    pub database_id: i32,
    /// Source type: "http", "redis", "ftp", "aws-s3", "azure-blob-storage", "google-blob-storage"
    pub source_type: String,
    /// URI to import from
    pub import_from_uri: String,
    /// Timeout in seconds (default: 1800 for imports)
    #[serde(default = "default_import_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the import_database tool
pub fn import_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("import_database")
        .description(
            "Import data into a Redis Cloud database from an external source. \
             WARNING: This will overwrite existing data. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, ImportDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ImportDatabaseInput>| async move {
                use redis_cloud::databases::DatabaseImportRequest;

                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Build the import request
                let request = DatabaseImportRequest::builder()
                    .source_type(&input.source_type)
                    .import_from_uri(vec![input.import_from_uri.clone()])
                    .build();

                // Use Layer 2 workflow
                import_database_and_wait(
                    &client,
                    input.subscription_id,
                    input.database_id,
                    &request,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to import database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Import completed successfully",
                    "subscription_id": input.subscription_id,
                    "database_id": input.database_id
                }))
            },
        )
        .build()
}

/// Input for deleting a subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteSubscriptionInput {
    /// Subscription ID to delete
    pub subscription_id: i32,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_subscription tool
pub fn delete_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_subscription")
        .description(
            "Delete a Redis Cloud subscription. WARNING: All databases in the subscription \
             must be deleted first. This is a destructive operation. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteSubscriptionInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Use Layer 2 workflow
                delete_subscription_and_wait(
                    &client,
                    input.subscription_id,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to delete subscription: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Subscription deleted successfully",
                    "subscription_id": input.subscription_id
                }))
            },
        )
        .build()
}

fn default_flush_timeout() -> u64 {
    300
}

/// Input for flushing a database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FlushDatabaseInput {
    /// Subscription ID containing the database
    pub subscription_id: i32,
    /// Database ID to flush
    pub database_id: i32,
    /// Timeout in seconds (default: 300)
    #[serde(default = "default_flush_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the flush_database tool
pub fn flush_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("flush_database")
        .description(
            "Flush all data from a Redis Cloud database and wait for completion. \
             WARNING: This permanently deletes ALL data in the database! \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, FlushDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<FlushDatabaseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Use Layer 2 workflow
                flush_database_and_wait(
                    &client,
                    input.subscription_id,
                    input.database_id,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| ToolError::new(format!("Failed to flush database: {}", e)))?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Database flushed successfully",
                    "subscription_id": input.subscription_id,
                    "database_id": input.database_id
                }))
            },
        )
        .build()
}

fn default_cloud_account_id() -> i32 {
    1 // Default internal account
}

fn default_subscription_timeout() -> u64 {
    1800 // Subscriptions can take a while
}

/// Input for creating a subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSubscriptionInput {
    /// Subscription name
    pub name: String,
    /// Cloud provider: "AWS", "GCP", or "Azure"
    pub cloud_provider: String,
    /// Cloud region (e.g., "us-east-1" for AWS, "us-central1" for GCP)
    pub region: String,
    /// Cloud account ID (use list_cloud_accounts to find available accounts, or use 1 for internal account)
    #[serde(default = "default_cloud_account_id")]
    pub cloud_account_id: i32,
    /// Database name for the initial database
    pub database_name: String,
    /// Memory limit in GB for the initial database
    pub memory_limit_in_gb: f64,
    /// Database protocol: "redis" (default), "stack", or "memcached"
    #[serde(default = "default_protocol")]
    pub protocol: String,
    /// Enable replication for high availability (default: true)
    #[serde(default = "default_replication")]
    pub replication: bool,
    /// Timeout in seconds (default: 1800 - subscriptions take longer)
    #[serde(default = "default_subscription_timeout")]
    pub timeout_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_subscription tool
pub fn create_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_subscription")
        .description(
            "Create a new Redis Cloud Pro subscription with an initial database. \
             This is a simplified interface for common subscription creation scenarios. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateSubscriptionInput>| async move {
                use redis_cloud::flexible::subscriptions::{
                    SubscriptionCreateRequest, SubscriptionDatabaseSpec, SubscriptionRegionSpec,
                    SubscriptionSpec,
                };
                use redisctl_core::cloud::create_subscription_and_wait;

                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                // Build the subscription request
                let request = SubscriptionCreateRequest::builder()
                    .name(&input.name)
                    .cloud_providers(vec![SubscriptionSpec {
                        provider: Some(input.cloud_provider.clone()),
                        cloud_account_id: Some(input.cloud_account_id),
                        regions: vec![SubscriptionRegionSpec {
                            region: input.region.clone(),
                            multiple_availability_zones: None,
                            preferred_availability_zones: None,
                            networking: None,
                        }],
                    }])
                    .databases(vec![SubscriptionDatabaseSpec {
                        name: input.database_name.clone(),
                        protocol: input.protocol.clone(),
                        memory_limit_in_gb: Some(input.memory_limit_in_gb),
                        dataset_size_in_gb: None,
                        support_oss_cluster_api: None,
                        data_persistence: None,
                        replication: Some(input.replication),
                        throughput_measurement: None,
                        local_throughput_measurement: None,
                        modules: None,
                        quantity: None,
                        average_item_size_in_bytes: None,
                        resp_version: None,
                        redis_version: None,
                        sharding_type: None,
                        query_performance_factor: None,
                    }])
                    .build();

                // Use Layer 2 workflow
                let subscription = create_subscription_and_wait(
                    &client,
                    &request,
                    Duration::from_secs(input.timeout_seconds),
                    None,
                )
                .await
                .map_err(|e| {
                    ToolError::new(format!("Failed to create subscription: {}", e))
                })?;

                CallToolResult::from_serialize(&subscription)
            },
        )
        .build()
}

pub(super) const INSTRUCTIONS: &str = r#"
### Redis Cloud - Subscriptions & Databases
- list_subscriptions: List all Cloud subscriptions
- get_subscription: Get subscription details
- list_databases: List databases in a subscription
- get_database: Get database details
- get_backup_status: Get database backup status
- get_slow_log: Get slow query log
- get_database_tags: Get tags for a database
- get_database_certificate: Get TLS/SSL certificate for a database

### Redis Cloud - Write Operations (require --read-only=false)
- create_database: Create a new database and wait for it to be ready
- update_database: Update a database configuration
- delete_database: Delete a database
- backup_database: Trigger a manual backup
- import_database: Import data into a database
- delete_subscription: Delete a subscription (all databases must be deleted first)
- flush_database: Flush all data from a database
"#;

/// Build an MCP sub-router containing subscription and database tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Subscriptions & Databases
        .tool(list_subscriptions(state.clone()))
        .tool(get_subscription(state.clone()))
        .tool(list_databases(state.clone()))
        .tool(get_database(state.clone()))
        .tool(get_backup_status(state.clone()))
        .tool(get_slow_log(state.clone()))
        .tool(get_tags(state.clone()))
        .tool(get_database_certificate(state.clone()))
        // Write Operations
        .tool(create_database(state.clone()))
        .tool(update_database(state.clone()))
        .tool(delete_database(state.clone()))
        .tool(backup_database(state.clone()))
        .tool(import_database(state.clone()))
        .tool(delete_subscription(state.clone()))
        .tool(flush_database(state.clone()))
        .tool(create_subscription(state.clone()))
}
