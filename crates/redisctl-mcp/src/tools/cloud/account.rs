//! Account, ACL, task, and user tools for Redis Cloud

use std::sync::Arc;

use redis_cloud::{AccountHandler, AclHandler, TaskHandler, UserHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;
use crate::tools::wrap_list;

// ============================================================================
// Account tools
// ============================================================================

/// Input for getting current account
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAccountInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_account tool
pub fn get_account(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_account")
        .description("Get information about the current Redis Cloud account including name, ID, and settings.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAccountInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetAccountInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AccountHandler::new(client);
                let account = handler
                    .get_current_account()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get account: {}", e)))?;

                CallToolResult::from_serialize(&account)
            },
        )
        .build()
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
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AccountHandler::new(client);
                let logs = handler
                    .get_account_system_logs(input.offset, input.limit)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get system logs: {}", e)))?;

                CallToolResult::from_serialize(&logs)
            },
        )
        .build()
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
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetSessionLogsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AccountHandler::new(client);
                let logs = handler
                    .get_account_session_logs(input.offset, input.limit)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get session logs: {}", e)))?;

                CallToolResult::from_serialize(&logs)
            },
        )
        .build()
}

/// Input for getting supported regions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRegionsInput {
    /// Optional cloud provider filter (e.g., "AWS", "GCP", "Azure")
    #[serde(default)]
    pub provider: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AccountHandler::new(client);
                let regions = handler
                    .get_supported_regions(input.provider)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get regions: {}", e)))?;

                CallToolResult::from_serialize(&regions)
            },
        )
        .build()
}

/// Input for getting database modules
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetModulesInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

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
            |State(state): State<Arc<AppState>>, Json(input): Json<GetModulesInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AccountHandler::new(client);
                let modules = handler
                    .get_supported_database_modules()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get modules: {}", e)))?;

                CallToolResult::from_serialize(&modules)
            },
        )
        .build()
}

// ============================================================================
// Account Users tools
// ============================================================================

/// Input for listing account users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAccountUsersInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

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
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListAccountUsersInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = UserHandler::new(client);
                let users = handler
                    .get_all_users()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list users: {}", e)))?;

                CallToolResult::from_serialize(&users)
            },
        )
        .build()
}

/// Input for getting a specific account user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAccountUserInput {
    /// Account user ID
    pub user_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_account_user tool
pub fn get_account_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_account_user")
        .description(
            "Get detailed information about a specific account user (team member) by ID.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetAccountUserInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAccountUserInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = UserHandler::new(client);
                let user = handler
                    .get_user_by_id(input.user_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get account user: {}", e)))?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

// ============================================================================
// ACL tools (database-level access control)
// ============================================================================

/// Input for listing ACL users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAclUsersInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_acl_users tool
pub fn list_acl_users(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_acl_users")
        .description("List all ACL users (database-level Redis users for authentication).")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListAclUsersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListAclUsersInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let users = handler
                    .get_all_acl_users()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list ACL users: {}", e)))?;

                CallToolResult::from_serialize(&users)
            },
        )
        .build()
}

/// Input for getting a specific ACL user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAclUserInput {
    /// ACL user ID
    pub user_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let user = handler
                    .get_user_by_id(input.user_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get ACL user: {}", e)))?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

/// Input for listing ACL roles
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAclRolesInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_acl_roles tool
pub fn list_acl_roles(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_acl_roles")
        .description("List all ACL roles (permission templates for database access).")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListAclRolesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListAclRolesInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let roles = handler
                    .get_roles()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list ACL roles: {}", e)))?;

                CallToolResult::from_serialize(&roles)
            },
        )
        .build()
}

/// Input for listing Redis rules
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRedisRulesInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_redis_rules tool
pub fn list_redis_rules(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_redis_rules")
        .description("List all Redis ACL rules (command permissions for Redis users).")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListRedisRulesInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListRedisRulesInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let rules = handler
                    .get_all_redis_rules()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list Redis rules: {}", e)))?;

                CallToolResult::from_serialize(&rules)
            },
        )
        .build()
}

// ============================================================================
// Task tools
// ============================================================================

/// Input for listing tasks
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListTasksInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_tasks tool
pub fn list_tasks(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_tasks")
        .description("List all async tasks in the Redis Cloud account. Tasks track long-running operations like database creation.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListTasksInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListTasksInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TaskHandler::new(client);
                let tasks = handler
                    .get_all_tasks()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list tasks: {}", e)))?;

                wrap_list("tasks", &tasks)
            },
        )
        .build()
}

/// Input for getting a specific task
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTaskInput {
    /// Task ID
    pub task_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
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
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TaskHandler::new(client);
                let task = handler
                    .get_task_by_id(input.task_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get task: {}", e)))?;

                CallToolResult::from_serialize(&task)
            },
        )
        .build()
}

pub(super) const INSTRUCTIONS: &str = r#"
### Redis Cloud - Account & Configuration
- get_account: Get current account information
- get_regions: Get supported cloud regions
- get_modules: Get supported Redis modules
- list_account_users: List team members
- get_account_user: Get team member details by ID
- list_acl_users: List database ACL users
- get_acl_user: Get ACL user details by ID
- list_acl_roles: List ACL roles
- list_redis_rules: List Redis ACL rules

### Redis Cloud - Logs
- get_system_logs: Get system audit logs (subscription/database changes)
- get_session_logs: Get session activity logs (login/logout events)

### Redis Cloud - Tasks
- list_tasks: List async operations
- get_task: Get task status
"#;

/// Build an MCP sub-router containing account, ACL, and task tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Account & Configuration
        .tool(get_account(state.clone()))
        .tool(get_regions(state.clone()))
        .tool(get_modules(state.clone()))
        .tool(list_account_users(state.clone()))
        .tool(get_account_user(state.clone()))
        .tool(list_acl_users(state.clone()))
        .tool(get_acl_user(state.clone()))
        .tool(list_acl_roles(state.clone()))
        .tool(list_redis_rules(state.clone()))
        // Logs
        .tool(get_system_logs(state.clone()))
        .tool(get_session_logs(state.clone()))
        // Tasks
        .tool(list_tasks(state.clone()))
        .tool(get_task(state.clone()))
}
