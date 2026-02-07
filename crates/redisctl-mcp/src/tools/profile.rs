//! Profile management tools for redisctl configuration

use std::sync::Arc;

use redisctl_core::{Config, DeploymentType, ProfileCredentials};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

// ============================================================================
// Read Operations
// ============================================================================

/// Input for listing profiles (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListProfilesInput {}

/// Profile summary for list output
#[derive(Debug, Serialize)]
struct ProfileSummary {
    name: String,
    deployment_type: String,
    is_default: bool,
}

/// Build the profile_list tool
pub fn list_profiles(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_list")
        .description("List all configured redisctl profiles with their types and default status")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListProfilesInput>(
            state,
            |State(_state): State<Arc<AppState>>, Json(_input): Json<ListProfilesInput>| async move {
                let config = Config::load()
                    .map_err(|e| ToolError::new(format!("Failed to load config: {}", e)))?;

                let profiles: Vec<ProfileSummary> = config
                    .list_profiles()
                    .iter()
                    .map(|(name, profile)| {
                        let deployment_type = match profile.deployment_type {
                            DeploymentType::Cloud => "cloud",
                            DeploymentType::Enterprise => "enterprise",
                            DeploymentType::Database => "database",
                        };

                        let is_default = match profile.deployment_type {
                            DeploymentType::Cloud => config.default_cloud.as_ref() == Some(name),
                            DeploymentType::Enterprise => {
                                config.default_enterprise.as_ref() == Some(name)
                            }
                            DeploymentType::Database => config.default_database.as_ref() == Some(name),
                        };

                        ProfileSummary {
                            name: (*name).clone(),
                            deployment_type: deployment_type.to_string(),
                            is_default,
                        }
                    })
                    .collect();

                if profiles.is_empty() {
                    return Ok(CallToolResult::text(
                        "No profiles configured. Use 'redisctl profile set' to create one.",
                    ));
                }

                // Format as a nice table-like output
                let mut output = format!("Found {} profile(s):\n\n", profiles.len());
                for p in &profiles {
                    let default_marker = if p.is_default { " (default)" } else { "" };
                    output.push_str(&format!(
                        "- {}: {}{}\n",
                        p.name, p.deployment_type, default_marker
                    ));
                }

                Ok(CallToolResult::text(output))
            },
        )
        .build()
}

/// Input for showing a specific profile
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShowProfileInput {
    /// Name of the profile to show
    pub name: String,
}

/// Masked profile details for output
#[derive(Debug, Serialize)]
struct MaskedProfileDetails {
    name: String,
    deployment_type: String,
    is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    cloud: Option<MaskedCloudCredentials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enterprise: Option<MaskedEnterpriseCredentials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    database: Option<MaskedDatabaseCredentials>,
}

#[derive(Debug, Serialize)]
struct MaskedCloudCredentials {
    api_key: String,
    api_secret: String,
    api_url: String,
}

#[derive(Debug, Serialize)]
struct MaskedEnterpriseCredentials {
    url: String,
    username: String,
    password: String,
    insecure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ca_cert: Option<String>,
}

#[derive(Debug, Serialize)]
struct MaskedDatabaseCredentials {
    host: String,
    port: u16,
    password: String,
    tls: bool,
    username: String,
    database: u8,
}

/// Mask a credential value, showing only first/last chars
fn mask_credential(value: &str) -> String {
    if value.is_empty() {
        return "(not set)".to_string();
    }
    if value.starts_with("keyring:") || value.starts_with("${") {
        // Show reference type but not the actual reference
        if value.starts_with("keyring:") {
            return "(keyring)".to_string();
        }
        return "(env var)".to_string();
    }
    if value.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &value[..2], &value[value.len() - 2..])
}

/// Build the profile_show tool
pub fn show_profile(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_show")
        .description("Show details of a specific profile. Credentials are masked for security.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ShowProfileInput>(
            state,
            |State(_state): State<Arc<AppState>>, Json(input): Json<ShowProfileInput>| async move {
                let config = Config::load()
                    .map_err(|e| ToolError::new(format!("Failed to load config: {}", e)))?;

                let profile = config
                    .profiles
                    .get(&input.name)
                    .ok_or_else(|| ToolError::new(format!("Profile '{}' not found", input.name)))?;

                let deployment_type = match profile.deployment_type {
                    DeploymentType::Cloud => "cloud",
                    DeploymentType::Enterprise => "enterprise",
                    DeploymentType::Database => "database",
                };

                let is_default = match profile.deployment_type {
                    DeploymentType::Cloud => config.default_cloud.as_ref() == Some(&input.name),
                    DeploymentType::Enterprise => {
                        config.default_enterprise.as_ref() == Some(&input.name)
                    }
                    DeploymentType::Database => {
                        config.default_database.as_ref() == Some(&input.name)
                    }
                };

                let (cloud, enterprise, database) = match &profile.credentials {
                    ProfileCredentials::Cloud {
                        api_key,
                        api_secret,
                        api_url,
                    } => (
                        Some(MaskedCloudCredentials {
                            api_key: mask_credential(api_key),
                            api_secret: mask_credential(api_secret),
                            api_url: api_url.clone(),
                        }),
                        None,
                        None,
                    ),
                    ProfileCredentials::Enterprise {
                        url,
                        username,
                        password,
                        insecure,
                        ca_cert,
                    } => (
                        None,
                        Some(MaskedEnterpriseCredentials {
                            url: url.clone(),
                            username: username.clone(),
                            password: password
                                .as_ref()
                                .map(|p| mask_credential(p))
                                .unwrap_or_else(|| "(not set)".to_string()),
                            insecure: *insecure,
                            ca_cert: ca_cert.clone(),
                        }),
                        None,
                    ),
                    ProfileCredentials::Database {
                        host,
                        port,
                        password,
                        tls,
                        username,
                        database,
                    } => (
                        None,
                        None,
                        Some(MaskedDatabaseCredentials {
                            host: host.clone(),
                            port: *port,
                            password: password
                                .as_ref()
                                .map(|p| mask_credential(p))
                                .unwrap_or_else(|| "(not set)".to_string()),
                            tls: *tls,
                            username: username.clone(),
                            database: *database,
                        }),
                    ),
                };

                let details = MaskedProfileDetails {
                    name: input.name,
                    deployment_type: deployment_type.to_string(),
                    is_default,
                    cloud,
                    enterprise,
                    database,
                };

                CallToolResult::from_serialize(&details)
            },
        )
        .build()
}

/// Input for getting config path (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigPathInput {}

/// Build the profile_path tool
pub fn config_path(_state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_path")
        .description("Show the path to the redisctl configuration file")
        .read_only()
        .idempotent()
        .handler(|_input: ConfigPathInput| async move {
            let path = Config::config_path()
                .map_err(|e| ToolError::new(format!("Failed to get config path: {}", e)))?;

            let exists = path.exists();
            let output = format!(
                "Configuration file: {}\nExists: {}",
                path.display(),
                if exists { "yes" } else { "no" }
            );

            Ok(CallToolResult::text(output))
        })
        .build()
}

/// Input for validating config (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateConfigInput {}

/// Build the profile_validate tool
pub fn validate_config(_state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_validate")
        .description("Validate the redisctl configuration file and check for common issues")
        .read_only()
        .idempotent()
        .handler(|_input: ValidateConfigInput| async move {
            let path = Config::config_path()
                .map_err(|e| ToolError::new(format!("Failed to get config path: {}", e)))?;

            if !path.exists() {
                return Ok(CallToolResult::text(format!(
                    "Configuration file not found at: {}\n\nThis is normal if you haven't created any profiles yet.\nUse 'redisctl profile set' to create a profile.",
                    path.display()
                )));
            }

            // Try to load the config
            let config = match Config::load() {
                Ok(c) => c,
                Err(e) => {
                    return Ok(CallToolResult::text(format!(
                        "Configuration file is INVALID:\n\nPath: {}\nError: {}",
                        path.display(),
                        e
                    )));
                }
            };

            // Check for issues
            let mut issues: Vec<String> = Vec::new();
            let mut warnings: Vec<String> = Vec::new();

            // Check if defaults reference valid profiles
            if let Some(ref default) = config.default_cloud
                && !config.profiles.contains_key(default)
            {
                issues.push(format!(
                    "default_cloud '{}' references non-existent profile",
                    default
                ));
            }
            if let Some(ref default) = config.default_enterprise
                && !config.profiles.contains_key(default)
            {
                issues.push(format!(
                    "default_enterprise '{}' references non-existent profile",
                    default
                ));
            }
            if let Some(ref default) = config.default_database
                && !config.profiles.contains_key(default)
            {
                issues.push(format!(
                    "default_database '{}' references non-existent profile",
                    default
                ));
            }

            // Check for profiles without passwords (warning only)
            for (name, profile) in &config.profiles {
                if !profile.has_password() {
                    match profile.deployment_type {
                        DeploymentType::Enterprise => {
                            warnings.push(format!(
                                "Profile '{}' has no password set (will prompt interactively)",
                                name
                            ));
                        }
                        DeploymentType::Database => {
                            // Database without password might be intentional
                        }
                        DeploymentType::Cloud => {
                            // Cloud uses API key/secret, not password
                        }
                    }
                }
            }

            // Build output
            let mut output = format!(
                "Configuration file: {}\nStatus: VALID\n\nProfiles: {}\n",
                path.display(),
                config.profiles.len()
            );

            if !issues.is_empty() {
                output.push_str("\nIssues:\n");
                for issue in &issues {
                    output.push_str(&format!("  - {}\n", issue));
                }
            }

            if !warnings.is_empty() {
                output.push_str("\nWarnings:\n");
                for warning in &warnings {
                    output.push_str(&format!("  - {}\n", warning));
                }
            }

            if issues.is_empty() && warnings.is_empty() {
                output.push_str("\nNo issues found.");
            }

            Ok(CallToolResult::text(output))
        })
        .build()
}

// ============================================================================
// Write Operations (require !read_only)
// ============================================================================

/// Input for setting default cloud profile
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDefaultCloudInput {
    /// Name of the profile to set as default cloud profile
    pub name: String,
}

/// Build the profile_set_default_cloud tool
pub fn set_default_cloud(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_set_default_cloud")
        .description("Set the default profile for Cloud commands. Requires write access.")
        .idempotent()
        .extractor_handler_typed::<_, _, _, SetDefaultCloudInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SetDefaultCloudInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool("Write operations require --read-only=false"));
                }

                let mut config = Config::load()
                    .map_err(|e| ToolError::new(format!("Failed to load config: {}", e)))?;

                // Verify profile exists and is a cloud profile
                let profile = config
                    .profiles
                    .get(&input.name)
                    .ok_or_else(|| ToolError::new(format!("Profile '{}' not found", input.name)))?;

                if !matches!(profile.deployment_type, DeploymentType::Cloud) {
                    return Err(McpError::tool(format!(
                        "Profile '{}' is not a cloud profile (type: {:?})",
                        input.name, profile.deployment_type
                    )));
                }

                config.default_cloud = Some(input.name.clone());
                config
                    .save()
                    .map_err(|e| ToolError::new(format!("Failed to save config: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Default cloud profile set to '{}'",
                    input.name
                )))
            },
        )
        .build()
}

/// Input for setting default enterprise profile
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDefaultEnterpriseInput {
    /// Name of the profile to set as default enterprise profile
    pub name: String,
}

/// Build the profile_set_default_enterprise tool
pub fn set_default_enterprise(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_set_default_enterprise")
        .description("Set the default profile for Enterprise commands. Requires write access.")
        .idempotent()
        .extractor_handler_typed::<_, _, _, SetDefaultEnterpriseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<SetDefaultEnterpriseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool("Write operations require --read-only=false"));
                }

                let mut config = Config::load()
                    .map_err(|e| ToolError::new(format!("Failed to load config: {}", e)))?;

                // Verify profile exists and is an enterprise profile
                let profile = config
                    .profiles
                    .get(&input.name)
                    .ok_or_else(|| ToolError::new(format!("Profile '{}' not found", input.name)))?;

                if !matches!(profile.deployment_type, DeploymentType::Enterprise) {
                    return Err(McpError::tool(format!(
                        "Profile '{}' is not an enterprise profile (type: {:?})",
                        input.name, profile.deployment_type
                    )));
                }

                config.default_enterprise = Some(input.name.clone());
                config
                    .save()
                    .map_err(|e| ToolError::new(format!("Failed to save config: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Default enterprise profile set to '{}'",
                    input.name
                )))
            },
        )
        .build()
}

/// Input for deleting a profile
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteProfileInput {
    /// Name of the profile to delete
    pub name: String,
}

/// Build the profile_delete tool
pub fn delete_profile(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("profile_delete")
        .description("Delete a profile from the configuration. Requires write access.")
        .extractor_handler_typed::<_, _, _, DeleteProfileInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<DeleteProfileInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool("Write operations require --read-only=false"));
                }

                let mut config = Config::load()
                    .map_err(|e| ToolError::new(format!("Failed to load config: {}", e)))?;

                // Check if profile exists
                if !config.profiles.contains_key(&input.name) {
                    return Err(McpError::tool(format!(
                        "Profile '{}' not found",
                        input.name
                    )));
                }

                // Remove the profile (also clears defaults if this was a default)
                config.remove_profile(&input.name);

                config
                    .save()
                    .map_err(|e| ToolError::new(format!("Failed to save config: {}", e)))?;

                Ok(CallToolResult::text(format!(
                    "Profile '{}' deleted",
                    input.name
                )))
            },
        )
        .build()
}

/// Instructions text describing all App-level tools, resources, and prompts
pub fn instructions() -> &'static str {
    r#"
### Profile Management - Read
- profile_list: List all configured profiles
- profile_show: Show profile details (credentials masked)
- profile_path: Show configuration file path
- profile_validate: Validate configuration file

### Profile Management - Write (requires --read-only=false)
- profile_set_default_cloud: Set default Cloud profile
- profile_set_default_enterprise: Set default Enterprise profile
- profile_delete: Delete a profile

## Resources

Read-only data accessible via URI:
- redis://config/path - Configuration file path
- redis://profiles - List of configured profiles
- redis://help - Usage instructions and help

## Prompts

Pre-built templates for common workflows:
- troubleshoot_database - Diagnose database issues
- analyze_performance - Analyze performance metrics
- capacity_planning - Help with capacity planning
- migration_planning - Plan Redis migrations
"#
}

/// Build an MCP sub-router containing all App-level tools, resources, and prompts
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Profile Tools - Read
        .tool(list_profiles(state.clone()))
        .tool(show_profile(state.clone()))
        .tool(config_path(state.clone()))
        .tool(validate_config(state.clone()))
        // Profile Tools - Write
        .tool(set_default_cloud(state.clone()))
        .tool(set_default_enterprise(state.clone()))
        .tool(delete_profile(state.clone()))
        // Resources
        .resource(crate::resources::config_path_resource())
        .resource(crate::resources::profiles_resource())
        .resource(crate::resources::help_resource())
        // Prompts
        .prompt(crate::prompts::troubleshoot_database_prompt())
        .prompt(crate::prompts::analyze_performance_prompt())
        .prompt(crate::prompts::capacity_planning_prompt())
        .prompt(crate::prompts::migration_planning_prompt())
}
