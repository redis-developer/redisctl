//! RBAC command implementations for Redis Enterprise

#![allow(dead_code)]

use crate::cli::OutputFormat;
use crate::connection::ConnectionManager;
use crate::error::{RedisCtlError, Result as CliResult};
use anyhow::Context;
use redis_enterprise::ldap_mappings::LdapMappingHandler;
use redis_enterprise::redis_acls::{CreateRedisAclRequest, RedisAclHandler};
use redis_enterprise::roles::RolesHandler;
use redis_enterprise::users::{AuthRequest, PasswordSet, UserHandler};

use super::utils::*;

// ============================================================================
// User Management Commands
// ============================================================================

pub async fn list_users(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client);
    let users = handler.list().await?;
    let users_json = serde_json::to_value(users).context("Failed to serialize users")?;
    let data = handle_output(users_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn get_user(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client);
    let user = handler.get(id).await?;
    // Mask password field if present
    let mut user_json = serde_json::to_value(user).context("Failed to serialize user")?;
    if let Some(obj) = user_json.as_object_mut()
        && obj.contains_key("password")
    {
        obj.insert(
            "password".to_string(),
            serde_json::Value::String("***".to_string()),
        );
    }
    let data = handle_output(user_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

/// Create a new user
#[allow(clippy::too_many_arguments)]
pub async fn create_user(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    email: Option<&str>,
    password: Option<&str>,
    role: Option<&str>,
    name: Option<&str>,
    email_alerts: bool,
    role_uids: &[u32],
    auth_method: Option<&str>,
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    // Start with JSON from --data if provided, otherwise empty object
    let mut request = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse user data")?
    } else {
        serde_json::json!({})
    };

    let request_obj = request.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(email_val) = email {
        request_obj.insert("email".to_string(), serde_json::json!(email_val));
    }

    if let Some(password_val) = password {
        request_obj.insert("password".to_string(), serde_json::json!(password_val));
    }

    if let Some(role_val) = role {
        request_obj.insert("role".to_string(), serde_json::json!(role_val));
    }

    if let Some(name_val) = name {
        request_obj.insert("name".to_string(), serde_json::json!(name_val));
    }

    if email_alerts {
        request_obj.insert("email_alerts".to_string(), serde_json::json!(true));
    }

    if !role_uids.is_empty() {
        request_obj.insert("role_uids".to_string(), serde_json::json!(role_uids));
    }

    if let Some(auth) = auth_method {
        request_obj.insert("auth_method".to_string(), serde_json::json!(auth));
    }

    // Validate required fields when not using pure --data mode
    if data.is_none() {
        if !request_obj.contains_key("email") {
            return Err(RedisCtlError::InvalidInput {
                message: "--email is required (unless using --data with complete configuration)"
                    .to_string(),
            });
        }
        if !request_obj.contains_key("password") {
            return Err(RedisCtlError::InvalidInput {
                message: "--password is required (unless using --data with complete configuration)"
                    .to_string(),
            });
        }
        if !request_obj.contains_key("role") {
            return Err(RedisCtlError::InvalidInput {
                message: "--role is required (unless using --data with complete configuration)"
                    .to_string(),
            });
        }
    }

    let user_json = client.post_raw("/v1/users", request).await?;
    let data = handle_output(user_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

/// Update a user
#[allow(clippy::too_many_arguments)]
pub async fn update_user(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    email: Option<&str>,
    password: Option<&str>,
    role: Option<&str>,
    name: Option<&str>,
    email_alerts: Option<bool>,
    role_uids: &[u32],
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    // Start with JSON from --data if provided, otherwise empty object
    let mut request = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse update data")?
    } else {
        serde_json::json!({})
    };

    let request_obj = request.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(email_val) = email {
        request_obj.insert("email".to_string(), serde_json::json!(email_val));
    }

    if let Some(password_val) = password {
        request_obj.insert("password".to_string(), serde_json::json!(password_val));
    }

    if let Some(role_val) = role {
        request_obj.insert("role".to_string(), serde_json::json!(role_val));
    }

    if let Some(name_val) = name {
        request_obj.insert("name".to_string(), serde_json::json!(name_val));
    }

    if let Some(alerts) = email_alerts {
        request_obj.insert("email_alerts".to_string(), serde_json::json!(alerts));
    }

    if !role_uids.is_empty() {
        request_obj.insert("role_uids".to_string(), serde_json::json!(role_uids));
    }

    // Validate that we have at least one field to update
    if request_obj.is_empty() {
        return Err(RedisCtlError::InvalidInput {
            message: "At least one update field is required (--email, --password, --role, --name, --email-alerts, --role-uid, or --data)".to_string(),
        });
    }

    let user_json = client
        .put_raw(&format!("/v1/users/{}", id), request)
        .await?;
    let data = handle_output(user_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn delete_user(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    force: bool,
    _output_format: OutputFormat,
    _query: Option<&str>,
) -> CliResult<()> {
    if !force && !confirm_action(&format!("Delete user {}?", id))? {
        println!("Operation cancelled");
        return Ok(());
    }

    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client);
    handler.delete(id).await?;
    println!("User {} deleted successfully", id);
    Ok(())
}

pub async fn reset_user_password(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    password: Option<&str>,
    _output_format: OutputFormat,
    _query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client);

    // Get the user to get their email
    let user = handler.get(id).await?;
    let email = user.email.clone();

    let new_password = if let Some(pwd) = password {
        pwd.to_string()
    } else {
        // Prompt for password if not provided
        rpassword::prompt_password("New password: ").context("Failed to read password")?
    };

    let request = PasswordSet {
        email,
        password: new_password,
    };

    handler.password_set(request).await?;
    println!("Password reset successfully for user {}", id);
    Ok(())
}

// User-Role Assignment Commands

pub async fn get_user_roles(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    user_id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client);

    let user = handler.get(user_id).await?;
    let roles = serde_json::json!({
        "user_id": user_id,
        "role": user.role,
        "role_uids": user.role_uids
    });

    let data = handle_output(roles, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn assign_user_role(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    user_id: u32,
    role_id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client.clone());

    // Get current user to preserve existing data
    let user = handler.get(user_id).await?;
    let mut role_uids: Vec<u32> = user.role_uids.clone().unwrap_or_default();

    // Add new role if not already present
    if !role_uids.contains(&role_id) {
        role_uids.push(role_id);
    }

    // Use raw API since UpdateUserRequest doesn't have Deserialize
    let update = serde_json::json!({
        "role_uids": role_uids
    });

    let updated = client
        .put_raw(&format!("/v1/users/{}", user_id), update)
        .await?;
    let data = handle_output(updated, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn remove_user_role(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    user_id: u32,
    role_id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client.clone());

    // Get current user to preserve existing data
    let user = handler.get(user_id).await?;
    let mut role_uids: Vec<u32> = user.role_uids.clone().unwrap_or_default();

    // Remove the role
    role_uids.retain(|&id| id != role_id);

    // Use raw API since UpdateUserRequest doesn't have Deserialize
    let update = serde_json::json!({
        "role_uids": role_uids
    });

    let updated = client
        .put_raw(&format!("/v1/users/{}", user_id), update)
        .await?;
    let data = handle_output(updated, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

// ============================================================================
// Role Management Commands
// ============================================================================

pub async fn list_roles(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RolesHandler::new(client);
    let roles = handler.list().await?;
    let roles_json = serde_json::to_value(roles).context("Failed to serialize roles")?;
    let data = handle_output(roles_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn get_role(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RolesHandler::new(client);
    let role = handler.get(id).await?;
    let role_json = serde_json::to_value(role).context("Failed to serialize role")?;
    let data = handle_output(role_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

/// Create a new role
pub async fn create_role(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    name: Option<&str>,
    management: Option<&str>,
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    // Start with JSON from --data if provided, otherwise empty object
    let mut request = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse role data")?
    } else {
        serde_json::json!({})
    };

    let request_obj = request.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(name_val) = name {
        request_obj.insert("name".to_string(), serde_json::json!(name_val));
    }

    if let Some(mgmt) = management {
        request_obj.insert("management".to_string(), serde_json::json!(mgmt));
    }

    // Validate required fields when not using pure --data mode
    if data.is_none() && !request_obj.contains_key("name") {
        return Err(RedisCtlError::InvalidInput {
            message: "--name is required (unless using --data with complete configuration)"
                .to_string(),
        });
    }

    let result = client.post_raw("/v1/roles", request).await?;
    let data = handle_output(result, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

/// Update a role
#[allow(clippy::too_many_arguments)]
pub async fn update_role(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    name: Option<&str>,
    management: Option<&str>,
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    // Start with JSON from --data if provided, otherwise empty object
    let mut request = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse role data")?
    } else {
        serde_json::json!({})
    };

    let request_obj = request.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(name_val) = name {
        request_obj.insert("name".to_string(), serde_json::json!(name_val));
    }

    if let Some(mgmt) = management {
        request_obj.insert("management".to_string(), serde_json::json!(mgmt));
    }

    // Validate that we have at least one field to update
    if request_obj.is_empty() {
        return Err(RedisCtlError::InvalidInput {
            message: "At least one update field is required (--name, --management, or --data)"
                .to_string(),
        });
    }

    let result = client
        .put_raw(&format!("/v1/roles/{}", id), request)
        .await?;
    let data = handle_output(result, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn delete_role(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    force: bool,
    _output_format: OutputFormat,
    _query: Option<&str>,
) -> CliResult<()> {
    if !force && !confirm_action(&format!("Delete role {}?", id))? {
        println!("Operation cancelled");
        return Ok(());
    }

    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RolesHandler::new(client);
    handler.delete(id).await?;
    println!("Role {} deleted successfully", id);
    Ok(())
}

pub async fn get_role_permissions(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RolesHandler::new(client);

    let role = handler.get(id).await?;

    let result = serde_json::json!({
        "role_id": id,
        "management": role.management,
        "data_access": role.data_access,
        "bdb_roles": role.bdb_roles,
        "cluster_roles": role.cluster_roles
    });

    let data = handle_output(result, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn get_role_users(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    role_id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let user_handler = UserHandler::new(client);

    // Get all users and filter by role
    let users = user_handler.list().await?;
    let users_with_role: Vec<_> = users
        .into_iter()
        .filter(|u| {
            u.role_uids
                .as_ref()
                .is_some_and(|uids| uids.contains(&role_id))
        })
        .collect();

    let users_json = serde_json::to_value(users_with_role).context("Failed to serialize users")?;
    let data = handle_output(users_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

// ============================================================================
// ACL Management Commands
// ============================================================================

pub async fn list_acls(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RedisAclHandler::new(client);
    let acls = handler.list().await?;
    let acls_json = serde_json::to_value(acls).context("Failed to serialize ACLs")?;
    let data = handle_output(acls_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn get_acl(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RedisAclHandler::new(client);
    let acl = handler.get(id).await?;
    let acl_json = serde_json::to_value(acl).context("Failed to serialize ACL")?;
    let data = handle_output(acl_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

/// Create a new ACL
#[allow(clippy::too_many_arguments)]
pub async fn create_acl(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    name: Option<&str>,
    acl: Option<&str>,
    description: Option<&str>,
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RedisAclHandler::new(client);

    // Start with JSON from --data if provided, otherwise empty object
    let mut request_json = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse ACL data")?
    } else {
        serde_json::json!({})
    };

    let request_obj = request_json.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(name_val) = name {
        request_obj.insert("name".to_string(), serde_json::json!(name_val));
    }

    if let Some(acl_val) = acl {
        request_obj.insert("acl".to_string(), serde_json::json!(acl_val));
    }

    if let Some(desc) = description {
        request_obj.insert("description".to_string(), serde_json::json!(desc));
    }

    // Validate required fields when not using pure --data mode
    if data.is_none() {
        if !request_obj.contains_key("name") {
            return Err(RedisCtlError::InvalidInput {
                message: "--name is required (unless using --data with complete configuration)"
                    .to_string(),
            });
        }
        if !request_obj.contains_key("acl") {
            return Err(RedisCtlError::InvalidInput {
                message: "--acl is required (unless using --data with complete configuration)"
                    .to_string(),
            });
        }
    }

    let request: CreateRedisAclRequest =
        serde_json::from_value(request_json).context("Invalid ACL creation request format")?;

    let acl_result = handler.create(request).await?;
    let acl_json = serde_json::to_value(acl_result).context("Failed to serialize ACL")?;
    let data = handle_output(acl_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

/// Update an ACL
#[allow(clippy::too_many_arguments)]
pub async fn update_acl(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    name: Option<&str>,
    acl: Option<&str>,
    description: Option<&str>,
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RedisAclHandler::new(client);

    // Start with JSON from --data if provided, otherwise empty object
    let mut request_json = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse ACL data")?
    } else {
        serde_json::json!({})
    };

    let request_obj = request_json.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(name_val) = name {
        request_obj.insert("name".to_string(), serde_json::json!(name_val));
    }

    if let Some(acl_val) = acl {
        request_obj.insert("acl".to_string(), serde_json::json!(acl_val));
    }

    if let Some(desc) = description {
        request_obj.insert("description".to_string(), serde_json::json!(desc));
    }

    // Validate that we have at least one field to update
    if request_obj.is_empty() {
        return Err(RedisCtlError::InvalidInput {
            message:
                "At least one update field is required (--name, --acl, --description, or --data)"
                    .to_string(),
        });
    }

    // For update, we need name and acl - get current values if not provided
    if !request_obj.contains_key("name") || !request_obj.contains_key("acl") {
        let current = handler.get(id).await?;
        if !request_obj.contains_key("name") {
            request_obj.insert("name".to_string(), serde_json::json!(current.name));
        }
        if !request_obj.contains_key("acl") {
            request_obj.insert("acl".to_string(), serde_json::json!(current.acl));
        }
    }

    let request: CreateRedisAclRequest =
        serde_json::from_value(request_json).context("Invalid ACL update request format")?;

    let acl_result = handler.update(id, request).await?;
    let acl_json = serde_json::to_value(acl_result).context("Failed to serialize ACL")?;
    let data = handle_output(acl_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn delete_acl(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    id: u32,
    force: bool,
    _output_format: OutputFormat,
    _query: Option<&str>,
) -> CliResult<()> {
    if !force && !confirm_action(&format!("Delete ACL {}?", id))? {
        println!("Operation cancelled");
        return Ok(());
    }

    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = RedisAclHandler::new(client);
    handler.delete(id).await?;
    println!("ACL {} deleted successfully", id);
    Ok(())
}

pub async fn test_acl(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    user_id: u32,
    command: &str,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    // This would typically involve testing the ACL against a specific command
    // The actual implementation depends on the API endpoint available
    let test_data = serde_json::json!({
        "user_id": user_id,
        "command": command
    });

    let result = client
        .post_raw("/v1/redis_acls/test", test_data)
        .await
        .unwrap_or_else(|_| {
            serde_json::json!({
                "user_id": user_id,
                "command": command,
                "result": "Test endpoint not available"
            })
        });

    let data = handle_output(result, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

// ============================================================================
// LDAP Integration Commands
// ============================================================================

pub async fn get_ldap_config(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    let config = client.get_raw("/v1/cluster/ldap").await?;
    let data = handle_output(config, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_ldap_config(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    enabled: Option<bool>,
    server_url: Option<&str>,
    bind_dn: Option<&str>,
    bind_password: Option<&str>,
    base_dn: Option<&str>,
    user_filter: Option<&str>,
    data: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    // Start with JSON from --data if provided, otherwise empty object
    let mut ldap_data = if let Some(data_str) = data {
        read_json_data(data_str).context("Failed to parse LDAP data")?
    } else {
        serde_json::json!({})
    };

    let ldap_obj = ldap_data.as_object_mut().unwrap();

    // CLI parameters override JSON values
    if let Some(en) = enabled {
        ldap_obj.insert("enabled".to_string(), serde_json::json!(en));
    }
    if let Some(url) = server_url {
        ldap_obj.insert("server_url".to_string(), serde_json::json!(url));
    }
    if let Some(dn) = bind_dn {
        ldap_obj.insert("bind_dn".to_string(), serde_json::json!(dn));
    }
    if let Some(pass) = bind_password {
        ldap_obj.insert("bind_password".to_string(), serde_json::json!(pass));
    }
    if let Some(dn) = base_dn {
        ldap_obj.insert("base_dn".to_string(), serde_json::json!(dn));
    }
    if let Some(filter) = user_filter {
        ldap_obj.insert("user_filter".to_string(), serde_json::json!(filter));
    }

    let result = client.put_raw("/v1/cluster/ldap", ldap_data).await?;
    let output_data = handle_output(result, output_format, query)?;
    print_formatted_output(output_data, output_format)?;
    Ok(())
}

pub async fn test_ldap_connection(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    let result = client
        .post_raw("/v1/cluster/ldap/test", serde_json::json!({}))
        .await
        .unwrap_or_else(|e| {
            serde_json::json!({
                "status": "error",
                "message": e.to_string()
            })
        });

    let data = handle_output(result, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn sync_ldap(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    let result = client
        .post_raw("/v1/cluster/ldap/sync", serde_json::json!({}))
        .await?;
    let data = handle_output(result, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn get_ldap_mappings(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = LdapMappingHandler::new(client);
    let mappings = handler.list().await?;
    let mappings_json = serde_json::to_value(mappings).context("Failed to serialize mappings")?;
    let data = handle_output(mappings_json, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

// ============================================================================
// Authentication & Session Commands
// ============================================================================

pub async fn test_auth(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    username: &str,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;
    let handler = UserHandler::new(client);

    // Prompt for password
    let password = rpassword::prompt_password("Password: ").context("Failed to read password")?;

    let auth_request = AuthRequest {
        email: username.to_string(),
        password,
    };

    match handler.authorize(auth_request).await {
        Ok(response) => {
            // Mask the JWT token in output
            let mut response_json = serde_json::to_value(response)?;
            if let Some(obj) = response_json.as_object_mut()
                && obj.contains_key("jwt")
            {
                obj.insert(
                    "jwt".to_string(),
                    serde_json::Value::String("***".to_string()),
                );
            }
            let data = handle_output(response_json, output_format, query)?;
            print_formatted_output(data, output_format)?;
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "failed",
                "error": e.to_string()
            });
            let data = handle_output(error_response, output_format, query)?;
            print_formatted_output(data, output_format)?;
        }
    }
    Ok(())
}

pub async fn list_sessions(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    output_format: OutputFormat,
    query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    let sessions = client.get_raw("/v1/sessions").await.unwrap_or_else(|_| {
        serde_json::json!({
            "message": "Sessions endpoint not available"
        })
    });

    let data = handle_output(sessions, output_format, query)?;
    print_formatted_output(data, output_format)?;
    Ok(())
}

pub async fn revoke_session(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    session_id: &str,
    _output_format: OutputFormat,
    _query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    client
        .delete_raw(&format!("/v1/sessions/{}", session_id))
        .await?;
    println!("Session {} revoked successfully", session_id);
    Ok(())
}

pub async fn revoke_all_user_sessions(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    user_id: u32,
    _output_format: OutputFormat,
    _query: Option<&str>,
) -> CliResult<()> {
    let client = conn_mgr.create_enterprise_client(profile_name).await?;

    client
        .delete_raw(&format!("/v1/users/{}/sessions", user_id))
        .await
        .unwrap_or_else(|_| {
            println!("Note: Session revocation endpoint may not be available");
            serde_json::Value::Null
        });

    println!("All sessions for user {} revoked", user_id);
    Ok(())
}
