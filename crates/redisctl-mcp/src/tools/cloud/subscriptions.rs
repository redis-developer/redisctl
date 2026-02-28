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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
        .non_destructive()
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
            "DANGEROUS: Permanently deletes a database and all its data. This action cannot be undone. \
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
        .non_destructive()
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
            "DANGEROUS: Permanently deletes a subscription. All databases must be deleted first. \
             This action cannot be undone. Requires write permission.",
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
            "DANGEROUS: Removes all data from a database. This action cannot be undone. \
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
        .non_destructive()
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

/// Input for updating a subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSubscriptionInput {
    /// Subscription ID to update
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_subscription tool
pub fn update_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_subscription")
        .description(
            "Update a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateSubscriptionInput>| async move {
                use redis_cloud::flexible::subscriptions::BaseSubscriptionUpdateRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = BaseSubscriptionUpdateRequest {
                    subscription_id: None,
                    command_type: None,
                };

                let handler = SubscriptionHandler::new(client);
                let result = handler
                    .update_subscription(input.subscription_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update subscription: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting subscription pricing
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSubscriptionPricingInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_subscription_pricing tool
pub fn get_subscription_pricing(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_subscription_pricing")
        .description(
            "Get pricing details for a Redis Cloud subscription.",
        )
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetSubscriptionPricingInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetSubscriptionPricingInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let pricing = handler
                    .get_subscription_pricing(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get subscription pricing: {}", e))
                    })?;

                CallToolResult::from_serialize(&pricing)
            },
        )
        .build()
}

/// Input for getting available Redis versions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRedisVersionsInput {
    /// Optional subscription ID to filter versions
    #[serde(default)]
    pub subscription_id: Option<i32>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_redis_versions tool
pub fn get_redis_versions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_redis_versions")
        .description(
            "Get available Redis versions. Optionally filter by subscription ID.",
        )
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetRedisVersionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetRedisVersionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let versions = handler
                    .get_redis_versions(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get Redis versions: {}", e))
                    })?;

                CallToolResult::from_serialize(&versions)
            },
        )
        .build()
}

/// Input for getting subscription CIDR allowlist
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSubscriptionCidrAllowlistInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_subscription_cidr_allowlist tool
pub fn get_subscription_cidr_allowlist(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_subscription_cidr_allowlist")
        .description("Get the CIDR allowlist for a Redis Cloud subscription.")
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetSubscriptionCidrAllowlistInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetSubscriptionCidrAllowlistInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let allowlist = handler
                    .get_cidr_allowlist(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get subscription CIDR allowlist: {}", e))
                    })?;

                CallToolResult::from_serialize(&allowlist)
            },
        )
        .build()
}

/// Input for updating subscription CIDR allowlist
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSubscriptionCidrAllowlistInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// List of CIDR IP ranges to allow (e.g., ["192.168.1.0/24", "10.0.0.0/8"])
    #[serde(default)]
    pub cidr_ips: Option<Vec<String>>,
    /// List of security group IDs to allow (AWS only)
    #[serde(default)]
    pub security_group_ids: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_subscription_cidr_allowlist tool
pub fn update_subscription_cidr_allowlist(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_subscription_cidr_allowlist")
        .description(
            "Update the CIDR allowlist for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateSubscriptionCidrAllowlistInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateSubscriptionCidrAllowlistInput>| async move {
                use redis_cloud::flexible::subscriptions::CidrAllowlistUpdateRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = CidrAllowlistUpdateRequest {
                    subscription_id: None,
                    cidr_ips: input.cidr_ips,
                    security_group_ids: input.security_group_ids,
                    command_type: None,
                };

                let handler = SubscriptionHandler::new(client);
                let result = handler
                    .update_subscription_cidr_allowlist(input.subscription_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to update subscription CIDR allowlist: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting subscription maintenance windows
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSubscriptionMaintenanceWindowsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_subscription_maintenance_windows tool
pub fn get_subscription_maintenance_windows(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_subscription_maintenance_windows")
        .description("Get maintenance windows for a Redis Cloud subscription.")
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetSubscriptionMaintenanceWindowsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetSubscriptionMaintenanceWindowsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let windows = handler
                    .get_subscription_maintenance_windows(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to get subscription maintenance windows: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&windows)
            },
        )
        .build()
}

/// Input for a maintenance window specification
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaintenanceWindowInput {
    /// Start hour (0-23 UTC)
    pub start_hour: i32,
    /// Duration in hours
    pub duration_in_hours: i32,
    /// Days of the week (e.g., ["Monday", "Wednesday", "Friday"])
    pub days: Vec<String>,
}

/// Input for updating subscription maintenance windows
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSubscriptionMaintenanceWindowsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Maintenance mode: "manual" or "automatic"
    pub mode: String,
    /// Maintenance windows (required when mode is "manual")
    #[serde(default)]
    pub windows: Option<Vec<MaintenanceWindowInput>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_subscription_maintenance_windows tool
pub fn update_subscription_maintenance_windows(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_subscription_maintenance_windows")
        .description(
            "Update maintenance windows for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateSubscriptionMaintenanceWindowsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateSubscriptionMaintenanceWindowsInput>| async move {
                use redis_cloud::flexible::subscriptions::{
                    MaintenanceWindowSpec, SubscriptionMaintenanceWindowsSpec,
                };

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let windows = input.windows.map(|ws| {
                    ws.into_iter()
                        .map(|w| MaintenanceWindowSpec {
                            start_hour: w.start_hour,
                            duration_in_hours: w.duration_in_hours,
                            days: w.days,
                        })
                        .collect()
                });

                let request = SubscriptionMaintenanceWindowsSpec {
                    mode: input.mode,
                    windows,
                };

                let handler = SubscriptionHandler::new(client);
                let result = handler
                    .update_subscription_maintenance_windows(input.subscription_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to update subscription maintenance windows: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting Active-Active subscription regions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetActiveActiveRegionsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_active_active_regions tool
pub fn get_active_active_regions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_active_active_regions")
        .description(
            "Get regions from an Active-Active Redis Cloud subscription.",
        )
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetActiveActiveRegionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetActiveActiveRegionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = SubscriptionHandler::new(client);
                let regions = handler
                    .get_regions_from_active_active_subscription(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to get Active-Active regions: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&regions)
            },
        )
        .build()
}

/// Input for adding a region to an Active-Active subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddActiveActiveRegionInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Deployment CIDR for the new region (e.g., "10.0.0.0/24")
    pub deployment_cidr: String,
    /// Region name (e.g., "us-east-1")
    #[serde(default)]
    pub region: Option<String>,
    /// VPC ID for the region
    #[serde(default)]
    pub vpc_id: Option<String>,
    /// Whether to perform a dry run without making changes
    #[serde(default)]
    pub dry_run: Option<bool>,
    /// RESP version for the region
    #[serde(default)]
    pub resp_version: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the add_active_active_region tool
pub fn add_active_active_region(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("add_active_active_region")
        .description(
            "Add a new region to an Active-Active Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, AddActiveActiveRegionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<AddActiveActiveRegionInput>| async move {
                use redis_cloud::flexible::subscriptions::ActiveActiveRegionCreateRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = ActiveActiveRegionCreateRequest {
                    subscription_id: None,
                    region: input.region,
                    vpc_id: input.vpc_id,
                    deployment_cidr: input.deployment_cidr,
                    dry_run: input.dry_run,
                    databases: None,
                    resp_version: input.resp_version,
                    customer_managed_key_resource_name: None,
                    command_type: None,
                };

                let handler = SubscriptionHandler::new(client);
                let result = handler
                    .add_new_region_to_active_active_subscription(
                        input.subscription_id,
                        &request,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to add Active-Active region: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for a region to delete from an Active-Active subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActiveActiveRegionToDeleteInput {
    /// Region name to delete (e.g., "us-east-1")
    pub region: String,
}

/// Input for deleting regions from an Active-Active subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteActiveActiveRegionsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Regions to delete
    pub regions: Vec<ActiveActiveRegionToDeleteInput>,
    /// Whether to perform a dry run without making changes
    #[serde(default)]
    pub dry_run: Option<bool>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_active_active_regions tool
pub fn delete_active_active_regions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_active_active_regions")
        .description(
            "DANGEROUS: Permanently removes regions from an Active-Active subscription. \
             This may cause data loss in the removed regions. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteActiveActiveRegionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteActiveActiveRegionsInput>| async move {
                use redis_cloud::flexible::subscriptions::{
                    ActiveActiveRegionDeleteRequest, ActiveActiveRegionToDelete,
                };

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let regions = input
                    .regions
                    .into_iter()
                    .map(|r| ActiveActiveRegionToDelete {
                        region: Some(r.region),
                    })
                    .collect();

                let request = ActiveActiveRegionDeleteRequest {
                    subscription_id: None,
                    regions: Some(regions),
                    dry_run: input.dry_run,
                    command_type: None,
                };

                let handler = SubscriptionHandler::new(client);
                let result = handler
                    .delete_regions_from_active_active_subscription(input.subscription_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to delete Active-Active regions: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting available database versions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAvailableDatabaseVersionsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_available_database_versions tool
pub fn get_available_database_versions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_available_database_versions")
        .description("Get available target Redis versions for upgrading a database.")
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetAvailableDatabaseVersionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAvailableDatabaseVersionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let versions = handler
                    .get_available_target_versions(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get available database versions: {}", e))
                    })?;

                CallToolResult::from_serialize(&versions)
            },
        )
        .build()
}

/// Input for upgrading a database Redis version
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpgradeDatabaseRedisVersionInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Target Redis version to upgrade to (e.g., "7.2")
    pub target_redis_version: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the upgrade_database_redis_version tool
pub fn upgrade_database_redis_version(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("upgrade_database_redis_version")
        .description(
            "Upgrade the Redis version of a database. \
             Use get_available_database_versions to find valid target versions. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpgradeDatabaseRedisVersionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpgradeDatabaseRedisVersionInput>| async move {
                use redis_cloud::databases::DatabaseUpgradeRedisVersionRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = DatabaseUpgradeRedisVersionRequest {
                    database_id: None,
                    subscription_id: None,
                    target_redis_version: input.target_redis_version,
                    command_type: None,
                };

                let handler = DatabaseHandler::new(client);
                let result = handler
                    .upgrade_database_redis_version(
                        input.subscription_id,
                        input.database_id,
                        &request,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to upgrade database Redis version: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting database upgrade status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseUpgradeStatusInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_database_upgrade_status tool
pub fn get_database_upgrade_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_upgrade_status")
        .description("Get the Redis version upgrade status for a database.")
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetDatabaseUpgradeStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetDatabaseUpgradeStatusInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let status = handler
                    .get_database_redis_version_upgrade_status(
                        input.subscription_id,
                        input.database_id,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get database upgrade status: {}", e))
                    })?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for getting database import status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseImportStatusInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_database_import_status tool
pub fn get_database_import_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database_import_status")
        .description("Get the import status for a Redis Cloud database.")
        .read_only()
        .idempotent()
        .non_destructive()
        .extractor_handler_typed::<_, _, _, GetDatabaseImportStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetDatabaseImportStatusInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let status = handler
                    .get_database_import_status(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get database import status: {}", e))
                    })?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for creating a database tag
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDatabaseTagInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tag key
    pub key: String,
    /// Tag value
    pub value: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_database_tag tool
pub fn create_database_tag(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_database_tag")
        .description(
            "Create a tag on a Redis Cloud database. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateDatabaseTagInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateDatabaseTagInput>| async move {
                use redis_cloud::databases::DatabaseTagCreateRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = DatabaseTagCreateRequest {
                    key: input.key,
                    value: input.value,
                    subscription_id: None,
                    database_id: None,
                    command_type: None,
                };

                let handler = DatabaseHandler::new(client);
                let tag = handler
                    .create_tag(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to create database tag: {}", e))
                    })?;

                CallToolResult::from_serialize(&tag)
            },
        )
        .build()
}

/// Input for updating a database tag
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateDatabaseTagInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tag key to update
    pub tag_key: String,
    /// New tag value
    pub value: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_database_tag tool
pub fn update_database_tag(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_database_tag")
        .description(
            "Update a tag on a Redis Cloud database. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateDatabaseTagInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateDatabaseTagInput>| async move {
                use redis_cloud::databases::DatabaseTagUpdateRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = DatabaseTagUpdateRequest {
                    subscription_id: None,
                    database_id: None,
                    key: None,
                    value: input.value,
                    command_type: None,
                };

                let handler = DatabaseHandler::new(client);
                let tag = handler
                    .update_tag(
                        input.subscription_id,
                        input.database_id,
                        input.tag_key,
                        &request,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update database tag: {}", e))
                    })?;

                CallToolResult::from_serialize(&tag)
            },
        )
        .build()
}

/// Input for deleting a database tag
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDatabaseTagInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tag key to delete
    pub tag_key: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_database_tag tool
pub fn delete_database_tag(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_database_tag")
        .description(
            "DANGEROUS: Permanently deletes a tag from a database. This action cannot be undone. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteDatabaseTagInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteDatabaseTagInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = DatabaseHandler::new(client);
                let result = handler
                    .delete_tag(input.subscription_id, input.database_id, input.tag_key)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to delete database tag: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for a tag key-value pair
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TagInput {
    /// Tag key
    pub key: String,
    /// Tag value
    pub value: String,
}

/// Input for updating all database tags
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateDatabaseTagsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tags to set on the database (replaces all existing tags)
    pub tags: Vec<TagInput>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_database_tags tool
pub fn update_database_tags(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_database_tags")
        .description(
            "Update all tags on a Redis Cloud database (replaces existing tags). \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateDatabaseTagsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateDatabaseTagsInput>| async move {
                use redis_cloud::databases::{DatabaseTagsUpdateRequest, Tag};

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let tags = input
                    .tags
                    .into_iter()
                    .map(|t| Tag {
                        key: t.key,
                        value: t.value,
                        command_type: None,
                    })
                    .collect();

                let request = DatabaseTagsUpdateRequest {
                    subscription_id: None,
                    database_id: None,
                    tags,
                    command_type: None,
                };

                let handler = DatabaseHandler::new(client);
                let result = handler
                    .update_tags(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update database tags: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating CRDB local properties
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateCrdbLocalPropertiesInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Updated database name
    #[serde(default)]
    pub name: Option<String>,
    /// Whether to perform a dry run without making changes
    #[serde(default)]
    pub dry_run: Option<bool>,
    /// Total memory limit in GB including replication overhead
    #[serde(default)]
    pub memory_limit_in_gb: Option<f64>,
    /// Maximum dataset size in GB
    #[serde(default)]
    pub dataset_size_in_gb: Option<f64>,
    /// Enable OSS Cluster API support
    #[serde(default)]
    pub support_oss_cluster_api: Option<bool>,
    /// Use external endpoint for OSS Cluster API
    #[serde(default)]
    pub use_external_endpoint_for_oss_cluster_api: Option<bool>,
    /// Enable TLS for connections
    #[serde(default)]
    pub enable_tls: Option<bool>,
    /// Global data persistence setting for all regions
    #[serde(default)]
    pub global_data_persistence: Option<String>,
    /// Global password for all regions
    #[serde(default)]
    pub global_password: Option<String>,
    /// Global source IP allowlist for all regions
    #[serde(default)]
    pub global_source_ip: Option<Vec<String>>,
    /// Data eviction policy
    #[serde(default)]
    pub data_eviction_policy: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_crdb_local_properties tool
pub fn update_crdb_local_properties(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_crdb_local_properties")
        .description(
            "Update local properties of an Active-Active (CRDB) database. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateCrdbLocalPropertiesInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateCrdbLocalPropertiesInput>| async move {
                use redis_cloud::databases::CrdbUpdatePropertiesRequest;

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = CrdbUpdatePropertiesRequest {
                    subscription_id: None,
                    database_id: None,
                    name: input.name,
                    dry_run: input.dry_run,
                    memory_limit_in_gb: input.memory_limit_in_gb,
                    dataset_size_in_gb: input.dataset_size_in_gb,
                    support_oss_cluster_api: input.support_oss_cluster_api,
                    use_external_endpoint_for_oss_cluster_api: input
                        .use_external_endpoint_for_oss_cluster_api,
                    client_ssl_certificate: None,
                    client_tls_certificates: None,
                    enable_tls: input.enable_tls,
                    global_data_persistence: input.global_data_persistence,
                    global_password: input.global_password,
                    global_source_ip: input.global_source_ip,
                    global_alerts: None,
                    regions: None,
                    data_eviction_policy: input.data_eviction_policy,
                    command_type: None,
                };

                let handler = DatabaseHandler::new(client);
                let result = handler
                    .update_crdb_local_properties(
                        input.subscription_id,
                        input.database_id,
                        &request,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update CRDB local properties: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

pub(super) const INSTRUCTIONS: &str = r#"
### Redis Cloud - Subscriptions & Databases
- list_subscriptions: List all Cloud subscriptions
- get_subscription: Get subscription details
- get_subscription_pricing: Get pricing details for a subscription
- get_redis_versions: Get available Redis versions
- get_subscription_cidr_allowlist: Get CIDR allowlist for a subscription
- get_subscription_maintenance_windows: Get maintenance windows for a subscription
- get_active_active_regions: Get regions from an Active-Active subscription
- list_databases: List databases in a subscription
- get_database: Get database details
- get_backup_status: Get database backup status
- get_slow_log: Get slow query log
- get_database_tags: Get tags for a database
- get_database_certificate: Get TLS/SSL certificate for a database
- get_available_database_versions: Get available target Redis versions for upgrade
- get_database_upgrade_status: Get Redis version upgrade status
- get_database_import_status: Get database import status

### Redis Cloud - Write Operations
- create_database: Create a new database and wait for it to be ready [write]
- update_database: Update a database configuration [write]
- backup_database: Trigger a manual backup [write]
- import_database: Import data into a database [write]
- create_subscription: Create a new subscription [write]
- update_subscription: Update a subscription [write]
- update_subscription_cidr_allowlist: Update CIDR allowlist for a subscription [write]
- update_subscription_maintenance_windows: Update maintenance windows [write]
- add_active_active_region: Add a region to an Active-Active subscription [write]
- upgrade_database_redis_version: Upgrade the Redis version of a database [write]
- create_database_tag: Create a tag on a database [write]
- update_database_tag: Update a tag on a database [write]
- update_database_tags: Update all tags on a database [write]
- update_crdb_local_properties: Update local properties of an Active-Active database [write]
- delete_database: Permanently delete a database and all its data [destructive]
- delete_subscription: Permanently delete a subscription [destructive]
- flush_database: Remove all data from a database [destructive]
- delete_active_active_regions: Remove regions from an Active-Active subscription [destructive]
- delete_database_tag: Delete a tag from a database [destructive]
"#;

/// Build an MCP sub-router containing subscription and database tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Subscriptions & Databases
        .tool(list_subscriptions(state.clone()))
        .tool(get_subscription(state.clone()))
        .tool(get_subscription_pricing(state.clone()))
        .tool(get_redis_versions(state.clone()))
        .tool(get_subscription_cidr_allowlist(state.clone()))
        .tool(get_subscription_maintenance_windows(state.clone()))
        .tool(get_active_active_regions(state.clone()))
        .tool(list_databases(state.clone()))
        .tool(get_database(state.clone()))
        .tool(get_backup_status(state.clone()))
        .tool(get_slow_log(state.clone()))
        .tool(get_tags(state.clone()))
        .tool(get_database_certificate(state.clone()))
        .tool(get_available_database_versions(state.clone()))
        .tool(get_database_upgrade_status(state.clone()))
        .tool(get_database_import_status(state.clone()))
        // Write Operations
        .tool(create_database(state.clone()))
        .tool(update_database(state.clone()))
        .tool(delete_database(state.clone()))
        .tool(backup_database(state.clone()))
        .tool(import_database(state.clone()))
        .tool(delete_subscription(state.clone()))
        .tool(flush_database(state.clone()))
        .tool(create_subscription(state.clone()))
        .tool(update_subscription(state.clone()))
        .tool(update_subscription_cidr_allowlist(state.clone()))
        .tool(update_subscription_maintenance_windows(state.clone()))
        .tool(add_active_active_region(state.clone()))
        .tool(delete_active_active_regions(state.clone()))
        .tool(upgrade_database_redis_version(state.clone()))
        .tool(create_database_tag(state.clone()))
        .tool(update_database_tag(state.clone()))
        .tool(delete_database_tag(state.clone()))
        .tool(update_database_tags(state.clone()))
        .tool(update_crdb_local_properties(state.clone()))
}
