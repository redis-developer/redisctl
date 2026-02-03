//! Redis Cloud API tools

use std::sync::Arc;

use redis_cloud::flexible::{DatabaseHandler, SubscriptionHandler};
use redis_cloud::{AccountHandler, AclHandler, TaskHandler, UserHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Tool, ToolBuilder, ToolError};

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
