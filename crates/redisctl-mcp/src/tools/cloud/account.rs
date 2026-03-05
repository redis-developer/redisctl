//! Account, ACL, task, cloud account (BYOC), and user tools for Redis Cloud

use std::sync::Arc;

use redis_cloud::acl::{
    AclRedisRuleCreateRequest, AclRedisRuleUpdateRequest, AclRoleCreateRequest,
    AclRoleDatabaseSpec, AclRoleRedisRuleSpec, AclRoleUpdateRequest, AclUserCreateRequest,
    AclUserUpdateRequest,
};
use redis_cloud::cloud_accounts::{CloudAccountCreateRequest, CloudAccountUpdateRequest};
use redis_cloud::users::AccountUserUpdateRequest;
use redis_cloud::{
    AccountHandler, AclHandler, CloudAccountHandler, CostReportCreateRequest, CostReportHandler,
    TaskHandler, UserHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

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
        .description("Get current account information.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get account")?;

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
        .description("Get system audit logs.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get system logs")?;

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
        .description("Get session activity logs.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get session logs")?;

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
        .description("Get supported cloud regions. Optionally filter by provider.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get regions")?;

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
        .description("Get supported database modules.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get modules")?;

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
        .description("List all account users (team members with console access).")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to list users")?;

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
        .description("Get an account user by ID.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get account user")?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

/// Input for updating an account user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAccountUserInput {
    /// Account user ID to update
    pub user_id: i32,
    /// Updated name for the user
    pub name: String,
    /// Updated role (e.g., "owner", "member", "viewer")
    #[serde(default)]
    pub role: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_account_user tool
pub fn update_account_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_account_user")
        .description("Update an account user's name or role.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateAccountUserInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = UserHandler::new(client);
                let request = AccountUserUpdateRequest {
                    user_id: None,
                    name: input.name,
                    role: input.role,
                    command_type: None,
                };
                let result = handler
                    .update_user(input.user_id, &request)
                    .await
                    .tool_context("Failed to update account user")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting an account user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAccountUserInput {
    /// Account user ID to delete
    pub user_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_account_user tool
pub fn delete_account_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_account_user")
        .description("DANGEROUS: Delete an account user. The user will lose all access.")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteAccountUserInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = UserHandler::new(client);
                let result = handler
                    .delete_user_by_id(input.user_id)
                    .await
                    .tool_context("Failed to delete account user")?;

                CallToolResult::from_serialize(&result)
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
        .description("List all ACL users.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to list ACL users")?;

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
        .description("Get an ACL user by ID.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get ACL user")?;

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
        .description("List all ACL roles.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to list ACL roles")?;

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
        .description("List all Redis ACL rules.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to list Redis rules")?;

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
        .description("Create a new ACL user with a database access role.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to create ACL user")?;

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
        .description("Update an ACL user's role or password.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to update ACL user")?;

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
        .description("DANGEROUS: Delete an ACL user. Active sessions will be terminated.")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DeleteAclUserInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
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
                    .tool_context("Failed to delete ACL user")?;

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
        .description("Create a new ACL role with Redis rules and database associations.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to create ACL role")?;

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
        .description("Update an ACL role's name or Redis rule assignments.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to update ACL role")?;

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
        .description("DANGEROUS: Delete an ACL role. Assigned users will lose their permissions.")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DeleteAclRoleInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
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
                    .tool_context("Failed to delete ACL role")?;

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
        .description("Create a new Redis ACL rule defining command permissions.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to create Redis rule")?;

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
        .description("Update a Redis ACL rule's name or pattern.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to update Redis rule")?;

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
        .description("DANGEROUS: Delete a Redis ACL rule. Roles using it will lose those permissions.")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteRedisRuleInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
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
                    .tool_context("Failed to delete Redis rule")?;

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
        .description("Generate a FOCUS cost report for the specified date range.")
        .non_destructive()
        .extractor_handler(
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
                    .tool_context("Failed to generate cost report")?;

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
        .description("Download a previously generated cost report by ID.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to download cost report")?;

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
        .description("List all payment methods.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to list payment methods")?;

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
        .description("List all async tasks.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to list tasks")?;

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
        .description("Get task status by ID.")
        .read_only_safe()
        .extractor_handler(
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
                    .tool_context("Failed to get task")?;

                CallToolResult::from_serialize(&task)
            },
        )
        .build()
}

/// Input for waiting on a cloud task
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WaitForCloudTaskInput {
    /// Task ID to wait for
    pub task_id: String,
    /// Maximum time to wait in seconds (default: 300)
    #[serde(default = "default_task_timeout")]
    pub timeout_seconds: u64,
    /// Polling interval in seconds (default: 5)
    #[serde(default = "default_task_interval")]
    pub interval_seconds: u64,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

fn default_task_timeout() -> u64 {
    300
}

fn default_task_interval() -> u64 {
    5
}

/// Build the wait_for_cloud_task tool
pub fn wait_for_cloud_task(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("wait_for_cloud_task")
        .description("Poll an async task until it reaches a terminal state. Useful for multi-step workflows.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<WaitForCloudTaskInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TaskHandler::new(client);
                let deadline =
                    tokio::time::Instant::now() + std::time::Duration::from_secs(input.timeout_seconds);
                let interval = std::time::Duration::from_secs(input.interval_seconds);

                loop {
                    let task = handler
                        .get_task_by_id(input.task_id.clone())
                        .await
                        .tool_context("Failed to get task status")?;

                    // Check for terminal states
                    if let Some(ref status) = task.status {
                        let s = status.to_lowercase();
                        if matches!(
                            s.as_str(),
                            "completed"
                                | "failed"
                                | "error"
                                | "success"
                                | "cancelled"
                                | "aborted"
                        ) {
                            return CallToolResult::from_serialize(&task);
                        }
                    }

                    // Check timeout
                    if tokio::time::Instant::now() >= deadline {
                        return CallToolResult::from_serialize(&serde_json::json!({
                            "timeout": true,
                            "message": format!(
                                "Task {} did not complete within {} seconds",
                                input.task_id, input.timeout_seconds
                            ),
                            "last_status": task,
                        }));
                    }

                    tokio::time::sleep(interval).await;
                }
            },
        )
        .build()
}

// ============================================================================
// Cloud Account (BYOC) tools
// ============================================================================

/// Input for listing cloud accounts
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCloudAccountsInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_cloud_accounts tool
pub fn list_cloud_accounts(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_cloud_accounts")
        .description("List all cloud provider accounts (BYOC).")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListCloudAccountsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CloudAccountHandler::new(client);
                let result = handler
                    .get_cloud_accounts()
                    .await
                    .tool_context("Failed to list cloud accounts")?;

                let accounts = result.cloud_accounts.unwrap_or_default();
                wrap_list("cloud_accounts", &accounts)
            },
        )
        .build()
}

/// Input for getting a cloud account by ID
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCloudAccountInput {
    /// Cloud account ID
    pub cloud_account_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cloud_account tool
pub fn get_cloud_account(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cloud_account")
        .description("Get a cloud provider account (BYOC) by ID.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetCloudAccountInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CloudAccountHandler::new(client);
                let account = handler
                    .get_cloud_account_by_id(input.cloud_account_id)
                    .await
                    .tool_context("Failed to get cloud account")?;

                CallToolResult::from_serialize(&account)
            },
        )
        .build()
}

/// Input for creating a cloud account
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateCloudAccountInput {
    /// Cloud account display name
    pub name: String,
    /// Cloud provider (e.g., "AWS", "GCP", "Azure"). Defaults to "AWS" if not specified.
    #[serde(default)]
    pub provider: Option<String>,
    /// Cloud provider access key
    pub access_key_id: String,
    /// Cloud provider secret key
    pub access_secret_key: String,
    /// Cloud provider management console username
    pub console_username: String,
    /// Cloud provider management console password
    pub console_password: String,
    /// Cloud provider management console login URL
    pub sign_in_login_url: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_cloud_account tool
pub fn create_cloud_account(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_cloud_account")
        .description("Create a new cloud provider account (BYOC).")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateCloudAccountInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CloudAccountHandler::new(client);
                let request = CloudAccountCreateRequest {
                    name: input.name,
                    provider: input.provider,
                    access_key_id: input.access_key_id,
                    access_secret_key: input.access_secret_key,
                    console_username: input.console_username,
                    console_password: input.console_password,
                    sign_in_login_url: input.sign_in_login_url,
                    command_type: None,
                };
                let result = handler
                    .create_cloud_account(&request)
                    .await
                    .tool_context("Failed to create cloud account")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating a cloud account
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateCloudAccountInput {
    /// Cloud account ID to update
    pub cloud_account_id: i32,
    /// New display name (optional)
    #[serde(default)]
    pub name: Option<String>,
    /// Cloud provider access key
    pub access_key_id: String,
    /// Cloud provider secret key
    pub access_secret_key: String,
    /// Cloud provider management console username
    pub console_username: String,
    /// Cloud provider management console password
    pub console_password: String,
    /// Cloud provider management console login URL (optional)
    #[serde(default)]
    pub sign_in_login_url: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_cloud_account tool
pub fn update_cloud_account(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_cloud_account")
        .description("Update a cloud provider account (BYOC) configuration.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateCloudAccountInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CloudAccountHandler::new(client);
                let request = CloudAccountUpdateRequest {
                    name: input.name,
                    cloud_account_id: None,
                    access_key_id: input.access_key_id,
                    access_secret_key: input.access_secret_key,
                    console_username: input.console_username,
                    console_password: input.console_password,
                    sign_in_login_url: input.sign_in_login_url,
                    command_type: None,
                };
                let result = handler
                    .update_cloud_account(input.cloud_account_id, &request)
                    .await
                    .tool_context("Failed to update cloud account")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a cloud account
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteCloudAccountInput {
    /// Cloud account ID to delete
    pub cloud_account_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_cloud_account tool
pub fn delete_cloud_account(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_cloud_account")
        .description("DANGEROUS: Delete a cloud provider account (BYOC).")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteCloudAccountInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = CloudAccountHandler::new(client);
                let result = handler
                    .delete_cloud_account(input.cloud_account_id)
                    .await
                    .tool_context("Failed to delete cloud account")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
    "get_account",
    "get_system_logs",
    "get_session_logs",
    "get_regions",
    "get_modules",
    "list_account_users",
    "get_account_user",
    "update_account_user",
    "delete_account_user",
    "list_acl_users",
    "get_acl_user",
    "list_acl_roles",
    "list_redis_rules",
    "create_acl_user",
    "update_acl_user",
    "delete_acl_user",
    "create_acl_role",
    "update_acl_role",
    "delete_acl_role",
    "create_redis_rule",
    "update_redis_rule",
    "delete_redis_rule",
    "generate_cost_report",
    "download_cost_report",
    "list_payment_methods",
    "list_tasks",
    "get_task",
    "wait_for_cloud_task",
    "list_cloud_accounts",
    "get_cloud_account",
    "create_cloud_account",
    "update_cloud_account",
    "delete_cloud_account",
];

/// Build an MCP sub-router containing account, ACL, cloud account, and task tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Account & Configuration
        .tool(get_account(state.clone()))
        .tool(get_regions(state.clone()))
        .tool(get_modules(state.clone()))
        .tool(list_account_users(state.clone()))
        .tool(get_account_user(state.clone()))
        .tool(update_account_user(state.clone()))
        .tool(delete_account_user(state.clone()))
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
        // Cloud Accounts (BYOC)
        .tool(list_cloud_accounts(state.clone()))
        .tool(get_cloud_account(state.clone()))
        .tool(create_cloud_account(state.clone()))
        .tool(update_cloud_account(state.clone()))
        .tool(delete_cloud_account(state.clone()))
        // Cost Reports
        .tool(generate_cost_report(state.clone()))
        .tool(download_cost_report(state.clone()))
        // Logs
        .tool(get_system_logs(state.clone()))
        .tool(get_session_logs(state.clone()))
        // Tasks
        .tool(list_tasks(state.clone()))
        .tool(get_task(state.clone()))
        .tool(wait_for_cloud_task(state.clone()))
}
