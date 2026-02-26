//! Account, ACL, task, and user tools for Redis Cloud

use std::sync::Arc;

use redis_cloud::acl::{
    AclRedisRuleCreateRequest, AclRedisRuleUpdateRequest, AclRoleCreateRequest,
    AclRoleDatabaseSpec, AclRoleRedisRuleSpec, AclRoleUpdateRequest, AclUserCreateRequest,
    AclUserUpdateRequest,
};
use redis_cloud::{
    AccountHandler, AclHandler, CostReportCreateRequest, CostReportHandler, TaskHandler,
    UserHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

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
// ACL write operations (require write permission)
// ============================================================================

/// Input for creating an ACL user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAclUserInput {
    /// Access control user name
    pub name: String,
    /// Name of the database access role to assign. Use list_acl_roles to get available roles.
    pub role: String,
    /// Database password for the user
    pub password: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_acl_user tool
pub fn create_acl_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_acl_user")
        .description(
            "Create a new ACL user with the assigned database access role. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateAclUserInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<CreateAclUserInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let request = AclUserCreateRequest {
                    name: input.name,
                    role: input.role,
                    password: input.password,
                    command_type: None,
                };
                let result = handler
                    .create_user(&request)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to create ACL user: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating an ACL user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAclUserInput {
    /// ACL user ID to update
    pub user_id: i32,
    /// New database access role name (optional)
    #[serde(default)]
    pub role: Option<String>,
    /// New database password (optional)
    #[serde(default)]
    pub password: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_acl_user tool
pub fn update_acl_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_acl_user")
        .description(
            "Update an ACL user's role or password. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateAclUserInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateAclUserInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let request = AclUserUpdateRequest {
                    user_id: None,
                    role: input.role,
                    password: input.password,
                    command_type: None,
                };
                let result = handler
                    .update_acl_user(input.user_id, &request)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update ACL user: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting an ACL user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAclUserInput {
    /// ACL user ID to delete
    pub user_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_acl_user tool
pub fn delete_acl_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_acl_user")
        .description(
            "Delete an ACL user. This is a destructive operation. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteAclUserInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DeleteAclUserInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let result = handler
                    .delete_user(input.user_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to delete ACL user: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Database specification for ACL role assignment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DatabaseSpec {
    /// Subscription ID for the database
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Optional list of regions for Active-Active databases
    #[serde(default)]
    pub regions: Option<Vec<String>>,
}

/// Redis rule specification for role creation/update
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RedisRuleSpec {
    /// Redis ACL rule name. Use list_redis_rules to get available rules.
    pub rule_name: String,
    /// List of databases where this rule applies
    pub databases: Vec<DatabaseSpec>,
}

/// Input for creating an ACL role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAclRoleInput {
    /// Database access role name
    pub name: String,
    /// List of Redis ACL rules to assign to this role
    pub redis_rules: Vec<RedisRuleSpec>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_acl_role tool
pub fn create_acl_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_acl_role")
        .description(
            "Create a new ACL role with assigned Redis rules and database associations. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateAclRoleInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<CreateAclRoleInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let request = AclRoleCreateRequest {
                    name: input.name,
                    redis_rules: input
                        .redis_rules
                        .into_iter()
                        .map(|r| AclRoleRedisRuleSpec {
                            rule_name: r.rule_name,
                            databases: r
                                .databases
                                .into_iter()
                                .map(|d| AclRoleDatabaseSpec {
                                    subscription_id: d.subscription_id,
                                    database_id: d.database_id,
                                    regions: d.regions,
                                })
                                .collect(),
                        })
                        .collect(),
                    command_type: None,
                };
                let result = handler
                    .create_role(&request)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to create ACL role: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating an ACL role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAclRoleInput {
    /// ACL role ID to update
    pub role_id: i32,
    /// New role name (optional)
    #[serde(default)]
    pub name: Option<String>,
    /// New list of Redis ACL rules (optional)
    #[serde(default)]
    pub redis_rules: Option<Vec<RedisRuleSpec>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_acl_role tool
pub fn update_acl_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_acl_role")
        .description(
            "Update an ACL role's name or Redis rule assignments. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateAclRoleInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateAclRoleInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let request = AclRoleUpdateRequest {
                    name: input.name,
                    redis_rules: input.redis_rules.map(|rules| {
                        rules
                            .into_iter()
                            .map(|r| AclRoleRedisRuleSpec {
                                rule_name: r.rule_name,
                                databases: r
                                    .databases
                                    .into_iter()
                                    .map(|d| AclRoleDatabaseSpec {
                                        subscription_id: d.subscription_id,
                                        database_id: d.database_id,
                                        regions: d.regions,
                                    })
                                    .collect(),
                            })
                            .collect()
                    }),
                    role_id: None,
                    command_type: None,
                };
                let result = handler
                    .update_role(input.role_id, &request)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to update ACL role: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting an ACL role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAclRoleInput {
    /// ACL role ID to delete
    pub role_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_acl_role tool
pub fn delete_acl_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_acl_role")
        .description(
            "Delete an ACL role. This is a destructive operation. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteAclRoleInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DeleteAclRoleInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let result = handler
                    .delete_acl_role(input.role_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to delete ACL role: {}", e)))?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating a Redis ACL rule
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateRedisRuleInput {
    /// Redis ACL rule name
    pub name: String,
    /// Redis ACL rule pattern (e.g., "+@all ~*" or "+@read ~cache:*")
    pub redis_rule: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_redis_rule tool
pub fn create_redis_rule(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_redis_rule")
        .description(
            "Create a new Redis ACL rule defining command permissions. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateRedisRuleInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateRedisRuleInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let request = AclRedisRuleCreateRequest {
                    name: input.name,
                    redis_rule: input.redis_rule,
                    command_type: None,
                };
                let result = handler
                    .create_redis_rule(&request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to create Redis rule: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating a Redis ACL rule
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateRedisRuleInput {
    /// Redis ACL rule ID to update
    pub rule_id: i32,
    /// New rule name
    pub name: String,
    /// New Redis ACL rule pattern
    pub redis_rule: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_redis_rule tool
pub fn update_redis_rule(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_redis_rule")
        .description(
            "Update a Redis ACL rule's name or pattern. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateRedisRuleInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateRedisRuleInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let request = AclRedisRuleUpdateRequest {
                    redis_rule_id: None,
                    name: input.name,
                    redis_rule: input.redis_rule,
                    command_type: None,
                };
                let result = handler
                    .update_redis_rule(input.rule_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update Redis rule: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a Redis ACL rule
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteRedisRuleInput {
    /// Redis ACL rule ID to delete
    pub rule_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_redis_rule tool
pub fn delete_redis_rule(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_redis_rule")
        .description(
            "Delete a Redis ACL rule. This is a destructive operation. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteRedisRuleInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteRedisRuleInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AclHandler::new(client);
                let result = handler
                    .delete_redis_rule(input.rule_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to delete Redis rule: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Cost report tools
// ============================================================================

/// Input for generating a cost report
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateCostReportInput {
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// End date in YYYY-MM-DD format (max 40 days from start_date)
    pub end_date: String,
    /// Output format: "csv" or "json" (default: "csv")
    #[serde(default)]
    pub format: Option<String>,
    /// Filter by subscription IDs
    #[serde(default)]
    pub subscription_ids: Option<Vec<i32>>,
    /// Filter by database IDs
    #[serde(default)]
    pub database_ids: Option<Vec<i32>>,
    /// Filter by subscription type: "pro" or "essentials"
    #[serde(default)]
    pub subscription_type: Option<String>,
    /// Filter by cloud regions
    #[serde(default)]
    pub regions: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the generate_cost_report tool
pub fn generate_cost_report(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("generate_cost_report")
        .description(
            "Generate a cost report in FOCUS format for the specified date range. \
             Returns a task ID to track generation progress. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, GenerateCostReportInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GenerateCostReportInput>| async move {
                use redis_cloud::{CostReportFormat, SubscriptionType};

                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CostReportHandler::new(client);
                let format = input.format.as_deref().map(|f| match f {
                    "json" => CostReportFormat::Json,
                    _ => CostReportFormat::Csv,
                });
                let subscription_type = input.subscription_type.as_deref().map(|t| match t {
                    "essentials" => SubscriptionType::Essentials,
                    _ => SubscriptionType::Pro,
                });
                let request = CostReportCreateRequest {
                    start_date: input.start_date,
                    end_date: input.end_date,
                    format,
                    subscription_ids: input.subscription_ids,
                    database_ids: input.database_ids,
                    subscription_type,
                    regions: input.regions,
                    tags: None,
                };
                let result = handler
                    .generate_cost_report(request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to generate cost report: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for downloading a cost report
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DownloadCostReportInput {
    /// Cost report ID from a completed generation task
    pub cost_report_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the download_cost_report tool
pub fn download_cost_report(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("download_cost_report")
        .description(
            "Download a previously generated cost report by ID. \
             Returns the report content (CSV or JSON).",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, DownloadCostReportInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DownloadCostReportInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CostReportHandler::new(client);
                let bytes = handler
                    .download_cost_report(&input.cost_report_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to download cost report: {}", e))
                    })?;

                let content = String::from_utf8(bytes).unwrap_or_else(|e| {
                    format!("<binary data, {} bytes>", e.into_bytes().len())
                });
                CallToolResult::from_serialize(&content)
            },
        )
        .build()
}

// ============================================================================
// Payment method tools
// ============================================================================

/// Input for listing payment methods
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPaymentMethodsInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_payment_methods tool
pub fn list_payment_methods(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_payment_methods")
        .description("List all payment methods for the Redis Cloud account.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListPaymentMethodsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListPaymentMethodsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = AccountHandler::new(client);
                let methods = handler
                    .get_account_payment_methods()
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to list payment methods: {}", e))
                    })?;

                CallToolResult::from_serialize(&methods)
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
- list_payment_methods: List account payment methods

### Redis Cloud - ACL Write Operations (require --read-only=false)
- create_acl_user: Create a new ACL user with a role and password
- update_acl_user: Update an ACL user's role or password
- delete_acl_user: Delete an ACL user
- create_acl_role: Create a new ACL role with Redis rules and database associations
- update_acl_role: Update an ACL role's name or rule assignments
- delete_acl_role: Delete an ACL role
- create_redis_rule: Create a new Redis ACL rule pattern
- update_redis_rule: Update a Redis ACL rule's name or pattern
- delete_redis_rule: Delete a Redis ACL rule

### Redis Cloud - Cost Reports
- generate_cost_report: Generate a FOCUS cost report (require --read-only=false)
- download_cost_report: Download a generated cost report

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
        .tool(list_payment_methods(state.clone()))
        // ACL Write Operations
        .tool(create_acl_user(state.clone()))
        .tool(update_acl_user(state.clone()))
        .tool(delete_acl_user(state.clone()))
        .tool(create_acl_role(state.clone()))
        .tool(update_acl_role(state.clone()))
        .tool(delete_acl_role(state.clone()))
        .tool(create_redis_rule(state.clone()))
        .tool(update_redis_rule(state.clone()))
        .tool(delete_redis_rule(state.clone()))
        // Cost Reports
        .tool(generate_cost_report(state.clone()))
        .tool(download_cost_report(state.clone()))
        // Logs
        .tool(get_system_logs(state.clone()))
        .tool(get_session_logs(state.clone()))
        // Tasks
        .tool(list_tasks(state.clone()))
        .tool(get_task(state.clone()))
}
