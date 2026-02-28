//! User, role, ACL, and LDAP tools

use std::sync::Arc;

use redis_enterprise::ldap_mappings::LdapMappingHandler;
use redis_enterprise::redis_acls::{CreateRedisAclRequest, RedisAclHandler};
use redis_enterprise::roles::{CreateRoleRequest, RolesHandler};
use redis_enterprise::users::{CreateUserRequest, UpdateUserRequest, UserHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;
use crate::tools::wrap_list;

// ============================================================================
// User tools
// ============================================================================

/// Input for listing users
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListUsersInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_users tool
pub fn list_users(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_users")
        .description("List all users in the Redis Enterprise cluster")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListUsersInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListUsersInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = UserHandler::new(client);
                let users = handler.list().await.tool_context("Failed to list users")?;

                wrap_list("users", &users)
            },
        )
        .build()
}

/// Input for getting a specific user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUserInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// User UID
    pub uid: u32,
}

/// Build the get_user tool
pub fn get_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_user")
        .description(
            "Get detailed information about a specific user in the Redis Enterprise cluster",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetUserInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetUserInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = UserHandler::new(client);
                let user = handler
                    .get(input.uid)
                    .await
                    .tool_context("Failed to get user")?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

/// Input for creating a user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateEnterpriseUserInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// User email address (used as login)
    pub email: String,
    /// User password
    pub password: String,
    /// Role name: "admin", "cluster_member", "cluster_viewer", "db_member", "db_viewer", or "none"
    pub role: String,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
    /// Whether the user receives email alerts
    #[serde(default)]
    pub email_alerts: Option<bool>,
    /// Role UIDs to assign (for custom role-based access)
    #[serde(default)]
    pub role_uids: Option<Vec<u32>>,
}

/// Build the create_enterprise_user tool
pub fn create_enterprise_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_enterprise_user")
        .description(
            "Create a new user in the Redis Enterprise cluster. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateEnterpriseUserInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateEnterpriseUserInput>| async move {
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

                let request = CreateUserRequest {
                    email: input.email,
                    password: input.password,
                    role: input.role,
                    name: input.name,
                    email_alerts: input.email_alerts,
                    bdbs_email_alerts: None,
                    role_uids: input.role_uids,
                    auth_method: None,
                };

                let handler = UserHandler::new(client);
                let user = handler
                    .create(request)
                    .await
                    .tool_context("Failed to create user")?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

/// Input for updating a user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateEnterpriseUserInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// User UID to update
    pub uid: u32,
    /// New password
    #[serde(default)]
    pub password: Option<String>,
    /// New role: "admin", "cluster_member", "cluster_viewer", "db_member", "db_viewer", or "none"
    #[serde(default)]
    pub role: Option<String>,
    /// New email address
    #[serde(default)]
    pub email: Option<String>,
    /// New display name
    #[serde(default)]
    pub name: Option<String>,
    /// Whether the user receives email alerts
    #[serde(default)]
    pub email_alerts: Option<bool>,
    /// Role UIDs to assign (for custom role-based access)
    #[serde(default)]
    pub role_uids: Option<Vec<u32>>,
}

/// Build the update_enterprise_user tool
pub fn update_enterprise_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_user")
        .description(
            "Update an existing user in the Redis Enterprise cluster. \
             Only specified fields will be modified. Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateEnterpriseUserInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateEnterpriseUserInput>| async move {
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

                let request = UpdateUserRequest {
                    password: input.password,
                    role: input.role,
                    email: input.email,
                    name: input.name,
                    email_alerts: input.email_alerts,
                    bdbs_email_alerts: None,
                    role_uids: input.role_uids,
                    auth_method: None,
                };

                let handler = UserHandler::new(client);
                let user = handler
                    .update(input.uid, request)
                    .await
                    .tool_context("Failed to update user")?;

                CallToolResult::from_serialize(&user)
            },
        )
        .build()
}

/// Input for deleting a user
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteEnterpriseUserInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// User UID to delete
    pub uid: u32,
}

/// Build the delete_enterprise_user tool
pub fn delete_enterprise_user(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_enterprise_user")
        .description(
            "DANGEROUS: Permanently deletes a user from the Redis Enterprise cluster. \
             Active sessions using this user will be terminated. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteEnterpriseUserInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteEnterpriseUserInput>| async move {
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

                let handler = UserHandler::new(client);
                handler
                    .delete(input.uid)
                    .await
                    .tool_context("Failed to delete user")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "User deleted successfully",
                    "uid": input.uid
                }))
            },
        )
        .build()
}

// ============================================================================
// Role tools
// ============================================================================

/// Input for listing roles (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRolesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_roles tool
pub fn list_roles(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_roles")
        .description(
            "List all roles in the Redis Enterprise cluster. Returns role names, \
             permissions (management, data_access), and database-specific role assignments.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListRolesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListRolesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = RolesHandler::new(client);
                let roles = handler.list().await.tool_context("Failed to list roles")?;

                wrap_list("roles", &roles)
            },
        )
        .build()
}

/// Input for getting a specific role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRoleInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Role UID
    pub uid: u32,
}

/// Build the get_role tool
pub fn get_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_role")
        .description(
            "Get detailed information about a specific role including permissions \
             and database role assignments.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetRoleInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetRoleInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = RolesHandler::new(client);
                let role = handler
                    .get(input.uid)
                    .await
                    .tool_context("Failed to get role")?;

                CallToolResult::from_serialize(&role)
            },
        )
        .build()
}

/// Input for creating a role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateEnterpriseRoleInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Role name
    pub name: String,
    /// Management permission level: "admin", "db_member", "db_viewer", "cluster_member", "cluster_viewer", or "none"
    #[serde(default)]
    pub management: Option<String>,
    /// Data access permission level: "redis_acl" or "none"
    #[serde(default)]
    pub data_access: Option<String>,
}

/// Build the create_enterprise_role tool
pub fn create_enterprise_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_enterprise_role")
        .description(
            "Create a new role in the Redis Enterprise cluster. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateEnterpriseRoleInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateEnterpriseRoleInput>| async move {
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

                let request = CreateRoleRequest {
                    name: input.name,
                    management: input.management,
                    data_access: input.data_access,
                    bdb_roles: None,
                    cluster_roles: None,
                };

                let handler = RolesHandler::new(client);
                let role = handler
                    .create(request)
                    .await
                    .tool_context("Failed to create role")?;

                CallToolResult::from_serialize(&role)
            },
        )
        .build()
}

/// Input for updating a role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateEnterpriseRoleInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Role UID to update
    pub uid: u32,
    /// Role name
    pub name: String,
    /// Management permission level: "admin", "db_member", "db_viewer", "cluster_member", "cluster_viewer", or "none"
    #[serde(default)]
    pub management: Option<String>,
    /// Data access permission level: "redis_acl" or "none"
    #[serde(default)]
    pub data_access: Option<String>,
}

/// Build the update_enterprise_role tool
pub fn update_enterprise_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_role")
        .description(
            "Update an existing role in the Redis Enterprise cluster. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateEnterpriseRoleInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateEnterpriseRoleInput>| async move {
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

                let request = CreateRoleRequest {
                    name: input.name,
                    management: input.management,
                    data_access: input.data_access,
                    bdb_roles: None,
                    cluster_roles: None,
                };

                let handler = RolesHandler::new(client);
                let role = handler
                    .update(input.uid, request)
                    .await
                    .tool_context("Failed to update role")?;

                CallToolResult::from_serialize(&role)
            },
        )
        .build()
}

/// Input for deleting a role
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteEnterpriseRoleInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Role UID to delete
    pub uid: u32,
}

/// Build the delete_enterprise_role tool
pub fn delete_enterprise_role(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_enterprise_role")
        .description(
            "DANGEROUS: Permanently deletes a role from the Redis Enterprise cluster. \
             Users assigned to this role will lose their permissions. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteEnterpriseRoleInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteEnterpriseRoleInput>| async move {
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

                let handler = RolesHandler::new(client);
                handler
                    .delete(input.uid)
                    .await
                    .tool_context("Failed to delete role")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Role deleted successfully",
                    "uid": input.uid
                }))
            },
        )
        .build()
}

// ============================================================================
// Redis ACL tools
// ============================================================================

/// Input for listing Redis ACLs (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRedisAclsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_redis_acls tool
pub fn list_redis_acls(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_acls")
        .description(
            "List all Redis ACLs in the Redis Enterprise cluster. Returns ACL names, \
             rules, and associated databases.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListRedisAclsInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListRedisAclsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = RedisAclHandler::new(client);
                let acls = handler.list().await.tool_context("Failed to list ACLs")?;

                wrap_list("acls", &acls)
            },
        )
        .build()
}

/// Input for getting a specific Redis ACL
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRedisAclInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// ACL UID
    pub uid: u32,
}

/// Build the get_redis_acl tool
pub fn get_redis_acl(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_acl")
        .description(
            "Get detailed information about a specific Redis ACL including the ACL rule string \
             and associated databases.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetRedisAclInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetRedisAclInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = RedisAclHandler::new(client);
                let acl = handler
                    .get(input.uid)
                    .await
                    .tool_context("Failed to get ACL")?;

                CallToolResult::from_serialize(&acl)
            },
        )
        .build()
}

/// Input for creating a Redis ACL
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateEnterpriseAclInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// ACL name
    pub name: String,
    /// ACL rule string (e.g., "+@all ~*" or "+get +set ~cache:*")
    pub acl: String,
    /// Description of the ACL
    #[serde(default)]
    pub description: Option<String>,
}

/// Build the create_enterprise_acl tool
pub fn create_enterprise_acl(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_enterprise_acl")
        .description(
            "Create a new Redis ACL in the Redis Enterprise cluster. \
             The ACL rule string follows Redis ACL syntax (e.g., \"+@all ~*\"). \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateEnterpriseAclInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateEnterpriseAclInput>| async move {
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

                let request = CreateRedisAclRequest {
                    name: input.name,
                    acl: input.acl,
                    description: input.description,
                };

                let handler = RedisAclHandler::new(client);
                let acl = handler
                    .create(request)
                    .await
                    .tool_context("Failed to create ACL")?;

                CallToolResult::from_serialize(&acl)
            },
        )
        .build()
}

/// Input for updating a Redis ACL
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateEnterpriseAclInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// ACL UID to update
    pub uid: u32,
    /// ACL name
    pub name: String,
    /// ACL rule string (e.g., "+@all ~*" or "+get +set ~cache:*")
    pub acl: String,
    /// Description of the ACL
    #[serde(default)]
    pub description: Option<String>,
}

/// Build the update_enterprise_acl tool
pub fn update_enterprise_acl(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_acl")
        .description(
            "Update an existing Redis ACL in the Redis Enterprise cluster. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateEnterpriseAclInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateEnterpriseAclInput>| async move {
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

                let request = CreateRedisAclRequest {
                    name: input.name,
                    acl: input.acl,
                    description: input.description,
                };

                let handler = RedisAclHandler::new(client);
                let acl = handler
                    .update(input.uid, request)
                    .await
                    .tool_context("Failed to update ACL")?;

                CallToolResult::from_serialize(&acl)
            },
        )
        .build()
}

/// Input for deleting a Redis ACL
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteEnterpriseAclInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// ACL UID to delete
    pub uid: u32,
}

/// Build the delete_enterprise_acl tool
pub fn delete_enterprise_acl(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_enterprise_acl")
        .description(
            "DANGEROUS: Permanently deletes a Redis ACL from the cluster. \
             Databases using this ACL will lose those access controls. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteEnterpriseAclInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteEnterpriseAclInput>| async move {
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

                let handler = RedisAclHandler::new(client);
                handler
                    .delete(input.uid)
                    .await
                    .tool_context("Failed to delete ACL")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "ACL deleted successfully",
                    "uid": input.uid
                }))
            },
        )
        .build()
}

// ============================================================================
// LDAP tools
// ============================================================================

/// Input for getting LDAP config (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLdapConfigInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_enterprise_ldap_config tool
pub fn get_enterprise_ldap_config(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_ldap_config")
        .description(
            "Get the LDAP configuration for the Redis Enterprise cluster including \
             server settings, bind DN, and query suffixes.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetLdapConfigInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetLdapConfigInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = LdapMappingHandler::new(client);
                let config = handler
                    .get_config()
                    .await
                    .tool_context("Failed to get LDAP config")?;

                CallToolResult::from_serialize(&config)
            },
        )
        .build()
}

/// Input for updating LDAP config
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateLdapConfigInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// LDAP configuration as a JSON object. Fields: enabled (bool), servers (array of {host, port, use_tls, starttls}),
    /// cache_refresh_interval, authentication_query_suffix, authorization_query_suffix, bind_dn, bind_pass
    pub config: Value,
}

/// Build the update_enterprise_ldap_config tool
pub fn update_enterprise_ldap_config(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_ldap_config")
        .description(
            "Update the LDAP configuration for the Redis Enterprise cluster. \
             Accepts a JSON object with LDAP settings. Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateLdapConfigInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateLdapConfigInput>| async move {
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

                let config = serde_json::from_value(input.config)
                    .tool_context("Invalid LDAP config")?;

                let handler = LdapMappingHandler::new(client);
                let result = handler
                    .update_config(config)
                    .await
                    .tool_context("Failed to update LDAP config")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Build an MCP sub-router containing RBAC and LDAP tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Users
        .tool(list_users(state.clone()))
        .tool(get_user(state.clone()))
        .tool(create_enterprise_user(state.clone()))
        .tool(update_enterprise_user(state.clone()))
        .tool(delete_enterprise_user(state.clone()))
        // Roles
        .tool(list_roles(state.clone()))
        .tool(get_role(state.clone()))
        .tool(create_enterprise_role(state.clone()))
        .tool(update_enterprise_role(state.clone()))
        .tool(delete_enterprise_role(state.clone()))
        // ACLs
        .tool(list_redis_acls(state.clone()))
        .tool(get_redis_acl(state.clone()))
        .tool(create_enterprise_acl(state.clone()))
        .tool(update_enterprise_acl(state.clone()))
        .tool(delete_enterprise_acl(state.clone()))
        // LDAP
        .tool(get_enterprise_ldap_config(state.clone()))
        .tool(update_enterprise_ldap_config(state.clone()))
}
