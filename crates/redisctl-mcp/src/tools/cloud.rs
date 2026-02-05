//! Redis Cloud API tools

use std::sync::Arc;
use std::time::Duration;

use redis_cloud::databases::DatabaseCreateRequest;
use redis_cloud::flexible::{DatabaseHandler, SubscriptionHandler};
use redis_cloud::{AccountHandler, AclHandler, TaskHandler, UserHandler};
use redisctl_core::cloud::{
    backup_database_and_wait, create_database_and_wait, delete_database_and_wait,
    delete_subscription_and_wait, flush_database_and_wait, import_database_and_wait,
    update_database_and_wait,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

/// Input for listing subscriptions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSubscriptionsInput {}

/// Build the list_subscriptions tool
pub fn list_subscriptions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_subscriptions")
        .description("List all Redis Cloud subscriptions accessible with the current credentials. Returns JSON with subscription details.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListSubscriptionsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<ListSubscriptionsInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = SubscriptionHandler::new(client);
                let account_subs = handler
                    .get_all_subscriptions()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list subscriptions: {}", e)))?;

                CallToolResult::from_serialize(&account_subs)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting a specific subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSubscriptionInput {
    /// Subscription ID
    pub subscription_id: i32,
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = SubscriptionHandler::new(client);
                let subscription = handler
                    .get_subscription_by_id(input.subscription_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get subscription: {}", e)))?;

                CallToolResult::from_serialize(&subscription)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for listing databases
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabasesInput {
    /// Subscription ID
    pub subscription_id: i32,
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = DatabaseHandler::new(client);
                let databases = handler
                    .get_subscription_databases(input.subscription_id, None, None)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list databases: {}", e)))?;

                CallToolResult::from_serialize(&databases)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting a specific database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = DatabaseHandler::new(client);
                let database = handler
                    .get_subscription_database_by_id(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get database: {}", e)))?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
        .expect("valid tool")
}

// ============================================================================
// Account tools
// ============================================================================

/// Input for getting current account
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAccountInput {}

/// Build the get_account tool
pub fn get_account(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_account")
        .description("Get information about the current Redis Cloud account including name, ID, and settings.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAccountInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<GetAccountInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AccountHandler::new(client);
                let account = handler
                    .get_current_account()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get account: {}", e)))?;

                CallToolResult::from_serialize(&account)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting account system logs
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSystemLogsInput {
    /// Number of entries to skip (for pagination)
    #[serde(default)]
    pub offset: Option<i32>,
    /// Maximum number of entries to return
    #[serde(default)]
    pub limit: Option<i32>,
}

/// Build the get_system_logs tool
pub fn get_system_logs(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_system_logs")
        .description(
            "Get system audit logs for the Redis Cloud account. Includes events like \
             subscription changes, database modifications, and user actions.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetSystemLogsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetSystemLogsInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AccountHandler::new(client);
                let logs = handler
                    .get_account_system_logs(input.offset, input.limit)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get system logs: {}", e)))?;

                CallToolResult::from_serialize(&logs)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting account session logs
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSessionLogsInput {
    /// Number of entries to skip (for pagination)
    #[serde(default)]
    pub offset: Option<i32>,
    /// Maximum number of entries to return
    #[serde(default)]
    pub limit: Option<i32>,
}

/// Build the get_session_logs tool
pub fn get_session_logs(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_session_logs")
        .description(
            "Get session activity logs for the Redis Cloud account. Includes user login/logout \
             events and session information.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetSessionLogsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetSessionLogsInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AccountHandler::new(client);
                let logs = handler
                    .get_account_session_logs(input.offset, input.limit)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get session logs: {}", e)))?;

                CallToolResult::from_serialize(&logs)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting supported regions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRegionsInput {
    /// Optional cloud provider filter (e.g., "AWS", "GCP", "Azure")
    #[serde(default)]
    pub provider: Option<String>,
}

/// Build the get_regions tool
pub fn get_regions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_regions")
        .description(
            "Get supported cloud regions for Redis Cloud. Optionally filter by provider (AWS, GCP, Azure).",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetRegionsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetRegionsInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AccountHandler::new(client);
                let regions = handler
                    .get_supported_regions(input.provider)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get regions: {}", e)))?;

                CallToolResult::from_serialize(&regions)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting database modules
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetModulesInput {}

/// Build the get_modules tool
pub fn get_modules(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_modules")
        .description(
            "Get supported Redis database modules (e.g., Search, JSON, TimeSeries, Bloom).",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetModulesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<GetModulesInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AccountHandler::new(client);
                let modules = handler
                    .get_supported_database_modules()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get modules: {}", e)))?;

                CallToolResult::from_serialize(&modules)
            },
        )
        .build()
        .expect("valid tool")
}

// ============================================================================
// Task tools
// ============================================================================

/// Input for listing tasks
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListTasksInput {}

/// Build the list_tasks tool
pub fn list_tasks(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_tasks")
        .description("List all async tasks in the Redis Cloud account. Tasks track long-running operations like database creation.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListTasksInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<ListTasksInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = TaskHandler::new(client);
                let tasks = handler
                    .get_all_tasks()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list tasks: {}", e)))?;

                CallToolResult::from_serialize(&tasks)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting a specific task
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTaskInput {
    /// Task ID
    pub task_id: String,
}

/// Build the get_task tool
pub fn get_task(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_task")
        .description("Get status and details of a specific async task by ID.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetTaskInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetTaskInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = TaskHandler::new(client);
                let task = handler
                    .get_task_by_id(input.task_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get task: {}", e)))?;

                CallToolResult::from_serialize(&task)
            },
        )
        .build()
        .expect("valid tool")
}

// ============================================================================
// Account Users tools
// ============================================================================

/// Input for listing account users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAccountUsersInput {}

/// Build the list_account_users tool
pub fn list_account_users(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_account_users")
        .description(
            "List all users in the Redis Cloud account (team members with console access).",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListAccountUsersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<ListAccountUsersInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = UserHandler::new(client);
                let users = handler
                    .get_all_users()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list users: {}", e)))?;

                CallToolResult::from_serialize(&users)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting a specific account user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAccountUserInput {
    /// Account user ID
    pub user_id: i32,
}

/// Build the get_account_user tool
pub fn get_account_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_account_user")
        .description("Get detailed information about a specific account user (team member) by ID.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAccountUserInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAccountUserInput>| async move {
                let client = state.cloud_client().await.map_err(|e| {
                    ToolError::new(format!("Failed to get Cloud client: {}", e))
                })?;

                let handler = UserHandler::new(client);
                let user = handler
                    .get_user_by_id(input.user_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get account user: {}", e)))?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
        .expect("valid tool")
}

// ============================================================================
// ACL tools (database-level access control)
// ============================================================================

/// Input for listing ACL users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAclUsersInput {}

/// Build the list_acl_users tool
pub fn list_acl_users(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_acl_users")
        .description("List all ACL users (database-level Redis users for authentication).")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListAclUsersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<ListAclUsersInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AclHandler::new(client);
                let users = handler
                    .get_all_acl_users()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list ACL users: {}", e)))?;

                CallToolResult::from_serialize(&users)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting a specific ACL user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAclUserInput {
    /// ACL user ID
    pub user_id: i32,
}

/// Build the get_acl_user tool
pub fn get_acl_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_acl_user")
        .description("Get detailed information about a specific ACL user by ID.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAclUserInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetAclUserInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AclHandler::new(client);
                let user = handler
                    .get_user_by_id(input.user_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get ACL user: {}", e)))?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for listing ACL roles
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAclRolesInput {}

/// Build the list_acl_roles tool
pub fn list_acl_roles(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_acl_roles")
        .description("List all ACL roles (permission templates for database access).")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListAclRolesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<ListAclRolesInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AclHandler::new(client);
                let roles = handler
                    .get_roles()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list ACL roles: {}", e)))?;

                CallToolResult::from_serialize(&roles)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for listing Redis rules
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRedisRulesInput {}

/// Build the list_redis_rules tool
pub fn list_redis_rules(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_redis_rules")
        .description("List all Redis ACL rules (command permissions for Redis users).")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListRedisRulesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(_input): Json<ListRedisRulesInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = AclHandler::new(client);
                let rules = handler
                    .get_all_redis_rules()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list Redis rules: {}", e)))?;

                CallToolResult::from_serialize(&rules)
            },
        )
        .build()
        .expect("valid tool")
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
}

/// Build the get_backup_status tool
pub fn get_backup_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_backup_status")
        .description("Get backup status and history for a Redis Cloud database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetBackupStatusInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetBackupStatusInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = DatabaseHandler::new(client);
                let log = handler
                    .get_slow_log(input.subscription_id, input.database_id, input.region_name)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get slow log: {}", e)))?;

                CallToolResult::from_serialize(&log)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting database tags
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTagsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = DatabaseHandler::new(client);
                let tags = handler
                    .get_tags(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get tags: {}", e)))?;

                CallToolResult::from_serialize(&tags)
            },
        )
        .build()
        .expect("valid tool")
}

/// Input for getting database certificate
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCertificateInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
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
            |State(state): State<Arc<AppState>>, Json(input): Json<GetCertificateInput>| async move {
                let client = state
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

                let handler = DatabaseHandler::new(client);
                let cert = handler
                    .get_subscription_database_certificate(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get certificate: {}", e)))?;

                CallToolResult::from_serialize(&cert)
            },
        )
        .build()
        .expect("valid tool")
}

// ============================================================================
// Write operations (require write permission)
// ============================================================================

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
}

fn default_replication() -> bool {
    true
}

fn default_protocol() -> String {
    "redis".to_string()
}

fn default_timeout() -> u64 {
    600
}

/// Build the create_database tool
///
/// This tool uses Layer 2 (redisctl-core) to create a database and wait for completion.
/// It demonstrates the full 3-layer architecture:
/// - Layer 1: DatabaseCreateRequest (redis-cloud)
/// - Layer 2: create_database_and_wait (redisctl-core)
/// - Layer 3: MCP tool (this crate)
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}

// ============================================================================
// Update database
// ============================================================================

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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}

// ============================================================================
// Delete database
// ============================================================================

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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}

// ============================================================================
// Backup database
// ============================================================================

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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}

// ============================================================================
// Import database
// ============================================================================

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
}

fn default_import_timeout() -> u64 {
    1800 // Imports can take longer
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}

// ============================================================================
// Delete subscription
// ============================================================================

/// Input for deleting a subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteSubscriptionInput {
    /// Subscription ID to delete
    pub subscription_id: i32,
    /// Timeout in seconds (default: 600)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}

// ============================================================================
// Flush database
// ============================================================================

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
}

fn default_flush_timeout() -> u64 {
    300
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
                    .cloud_client()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

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
        .expect("valid tool")
}
