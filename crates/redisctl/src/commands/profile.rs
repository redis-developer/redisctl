//! Profile management command implementations

#![allow(dead_code)] // Functions called from bin target

use crate::cli::{OutputFormat, ProfileCommands};
use crate::connection::ConnectionManager;
use crate::error::RedisCtlError;
use crate::output;
use anyhow::Context;
use redisctl_config::{self, Config};
use tracing::{debug, info, trace};

/// Handle profile management commands
pub async fn handle_profile_command(
    profile_cmd: &ProfileCommands,
    conn_mgr: &ConnectionManager,
    output_format: OutputFormat,
) -> Result<(), RedisCtlError> {
    use ProfileCommands::*;

    match profile_cmd {
        List => handle_list(conn_mgr, output_format).await,
        Path => handle_path(output_format).await,
        Show { name } => handle_show(conn_mgr, name, output_format).await,
        Set {
            name,
            r#type,
            api_key,
            api_secret,
            api_url,
            url,
            username,
            password,
            insecure,
            ca_cert,
            host,
            port,
            no_tls,
            db,
            #[cfg(feature = "secure-storage")]
            use_keyring,
        } => {
            handle_set(
                conn_mgr,
                name,
                r#type,
                api_key,
                api_secret,
                api_url,
                url,
                username,
                password,
                insecure,
                ca_cert,
                host,
                port,
                no_tls,
                db,
                #[cfg(feature = "secure-storage")]
                use_keyring,
            )
            .await
        }
        Remove { name } => handle_remove(conn_mgr, name).await,
        DefaultEnterprise { name } => handle_default_enterprise(conn_mgr, name).await,
        DefaultCloud { name } => handle_default_cloud(conn_mgr, name).await,
        DefaultDatabase { name } => handle_default_database(conn_mgr, name).await,
        Validate => handle_validate(conn_mgr).await,
    }
}

async fn handle_list(
    conn_mgr: &ConnectionManager,
    output_format: OutputFormat,
) -> Result<(), RedisCtlError> {
    debug!("Listing all configured profiles");
    let profiles = conn_mgr.config.list_profiles();
    trace!("Found {} profiles", profiles.len());

    match output_format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let config_path = conn_mgr
                .config_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| {
                    Config::config_path()
                        .ok()
                        .and_then(|p| p.to_str().map(String::from))
                });

            let profile_list: Vec<serde_json::Value> = profiles
                .iter()
                .map(|(name, profile)| {
                    let is_default_enterprise =
                        conn_mgr.config.default_enterprise.as_deref() == Some(name);
                    let is_default_cloud = conn_mgr.config.default_cloud.as_deref() == Some(name);

                    let mut obj = serde_json::json!({
                        "name": name,
                        "deployment_type": profile.deployment_type.to_string(),
                        "is_default_enterprise": is_default_enterprise,
                        "is_default_cloud": is_default_cloud,
                    });

                    match profile.deployment_type {
                        redisctl_config::DeploymentType::Cloud => {
                            if let Some((_, _, url)) = profile.cloud_credentials() {
                                obj["api_url"] = serde_json::json!(url);
                            }
                        }
                        redisctl_config::DeploymentType::Enterprise => {
                            if let Some((url, username, _, insecure, ca_cert)) =
                                profile.enterprise_credentials()
                            {
                                obj["url"] = serde_json::json!(url);
                                obj["username"] = serde_json::json!(username);
                                obj["insecure"] = serde_json::json!(insecure);
                                if let Some(cert_path) = ca_cert {
                                    obj["ca_cert"] = serde_json::json!(cert_path);
                                }
                            }
                        }
                        redisctl_config::DeploymentType::Database => {
                            if let Some((host, port, _, tls, username, database)) =
                                profile.database_credentials()
                            {
                                obj["host"] = serde_json::json!(host);
                                obj["port"] = serde_json::json!(port);
                                obj["tls"] = serde_json::json!(tls);
                                obj["username"] = serde_json::json!(username);
                                obj["database"] = serde_json::json!(database);
                            }
                        }
                    }

                    obj
                })
                .collect();

            let output_data = serde_json::json!({
                "config_path": config_path,
                "profiles": profile_list,
                "count": profiles.len()
            });

            let fmt = match output_format {
                OutputFormat::Json => output::OutputFormat::Json,
                OutputFormat::Yaml => output::OutputFormat::Yaml,
                _ => output::OutputFormat::Json,
            };

            output::print_output(&output_data, fmt, None)?;
        }
        _ => {
            // Show config file path at the top
            if let Some(ref path) = conn_mgr.config_path {
                println!("Configuration file: {}", path.display());
                println!();
            } else if let Ok(config_path) = Config::config_path() {
                println!("Configuration file: {}", config_path.display());
                println!();
            }

            if profiles.is_empty() {
                info!("No profiles configured");
                println!("No profiles configured.");
                println!("Use 'redisctl profile set' to create a profile.");
                return Ok(());
            }

            println!("{:<15} {:<12} DETAILS", "NAME", "TYPE");
            println!("{:-<15} {:-<12} {:-<30}", "", "", "");

            for (name, profile) in profiles {
                let mut details = String::new();
                match profile.deployment_type {
                    redisctl_config::DeploymentType::Cloud => {
                        if let Some((_, _, url)) = profile.cloud_credentials() {
                            details = format!("URL: {}", url);
                        }
                    }
                    redisctl_config::DeploymentType::Enterprise => {
                        if let Some((url, username, _, insecure, _ca_cert)) =
                            profile.enterprise_credentials()
                        {
                            details = format!(
                                "URL: {}, User: {}{}",
                                url,
                                username,
                                if insecure { " (insecure)" } else { "" }
                            );
                        }
                    }
                    redisctl_config::DeploymentType::Database => {
                        if let Some((host, port, _, tls, _, _)) = profile.database_credentials() {
                            details = format!(
                                "{}:{} {}",
                                host,
                                port,
                                if tls { "(TLS)" } else { "(no TLS)" }
                            );
                        }
                    }
                }

                let is_default_enterprise =
                    conn_mgr.config.default_enterprise.as_deref() == Some(name);
                let is_default_cloud = conn_mgr.config.default_cloud.as_deref() == Some(name);
                let is_default_database = conn_mgr.config.default_database.as_deref() == Some(name);
                let name_display =
                    if is_default_enterprise || is_default_cloud || is_default_database {
                        format!("{}*", name)
                    } else {
                        name.to_string()
                    };

                println!(
                    "{:<15} {:<12} {}",
                    name_display, profile.deployment_type, details
                );
            }
        }
    }

    Ok(())
}

async fn handle_path(output_format: OutputFormat) -> Result<(), RedisCtlError> {
    let config_path = Config::config_path()?;

    match output_format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let output_data = serde_json::json!({
                "config_path": config_path.to_str()
            });

            let fmt = match output_format {
                OutputFormat::Json => output::OutputFormat::Json,
                OutputFormat::Yaml => output::OutputFormat::Yaml,
                _ => output::OutputFormat::Json,
            };

            output::print_output(&output_data, fmt, None)?;
        }
        _ => {
            println!("{}", config_path.display());
        }
    }
    Ok(())
}

async fn handle_show(
    conn_mgr: &ConnectionManager,
    name: &str,
    output_format: OutputFormat,
) -> Result<(), RedisCtlError> {
    match conn_mgr.config.profiles.get(name) {
        Some(profile) => {
            let is_default_enterprise = conn_mgr.config.default_enterprise.as_deref() == Some(name);
            let is_default_cloud = conn_mgr.config.default_cloud.as_deref() == Some(name);

            match output_format {
                OutputFormat::Json | OutputFormat::Yaml => {
                    let mut output_data = serde_json::json!({
                        "name": name,
                        "deployment_type": profile.deployment_type.to_string(),
                        "is_default_enterprise": is_default_enterprise,
                        "is_default_cloud": is_default_cloud,
                    });

                    match profile.deployment_type {
                        redisctl_config::DeploymentType::Cloud => {
                            if let Some((api_key, _, api_url)) = profile.cloud_credentials() {
                                output_data["api_key_preview"] = serde_json::json!(format!(
                                    "{}...",
                                    &api_key[..std::cmp::min(8, api_key.len())]
                                ));
                                output_data["api_url"] = serde_json::json!(api_url);
                            }
                        }
                        redisctl_config::DeploymentType::Enterprise => {
                            if let Some((url, username, has_password, insecure, ca_cert)) =
                                profile.enterprise_credentials()
                            {
                                output_data["url"] = serde_json::json!(url);
                                output_data["username"] = serde_json::json!(username);
                                output_data["password_configured"] =
                                    serde_json::json!(has_password.is_some());
                                output_data["insecure"] = serde_json::json!(insecure);
                                if let Some(cert_path) = ca_cert {
                                    output_data["ca_cert"] = serde_json::json!(cert_path);
                                }
                            }
                        }
                        redisctl_config::DeploymentType::Database => {
                            if let Some((host, port, has_password, tls, username, database)) =
                                profile.database_credentials()
                            {
                                output_data["host"] = serde_json::json!(host);
                                output_data["port"] = serde_json::json!(port);
                                output_data["password_configured"] =
                                    serde_json::json!(has_password.is_some());
                                output_data["tls"] = serde_json::json!(tls);
                                output_data["username"] = serde_json::json!(username);
                                output_data["database"] = serde_json::json!(database);
                            }
                        }
                    }

                    let fmt = match output_format {
                        OutputFormat::Json => output::OutputFormat::Json,
                        OutputFormat::Yaml => output::OutputFormat::Yaml,
                        _ => output::OutputFormat::Json,
                    };

                    output::print_output(&output_data, fmt, None)?;
                }
                _ => {
                    println!("Profile: {}", name);
                    println!("Type: {}", profile.deployment_type);

                    match profile.deployment_type {
                        redisctl_config::DeploymentType::Cloud => {
                            if let Some((api_key, _, api_url)) = profile.cloud_credentials() {
                                println!(
                                    "API Key: {}...",
                                    &api_key[..std::cmp::min(8, api_key.len())]
                                );
                                println!("API URL: {}", api_url);
                            }
                        }
                        redisctl_config::DeploymentType::Enterprise => {
                            if let Some((url, username, has_password, insecure, ca_cert)) =
                                profile.enterprise_credentials()
                            {
                                println!("URL: {}", url);
                                println!("Username: {}", username);
                                println!(
                                    "Password: {}",
                                    if has_password.is_some() {
                                        "configured"
                                    } else {
                                        "not set"
                                    }
                                );
                                println!("Insecure: {}", insecure);
                                if let Some(cert_path) = ca_cert {
                                    println!("CA Cert: {}", cert_path);
                                }
                            }
                        }
                        redisctl_config::DeploymentType::Database => {
                            if let Some((host, port, has_password, tls, username, database)) =
                                profile.database_credentials()
                            {
                                println!("Host: {}", host);
                                println!("Port: {}", port);
                                println!("Username: {}", username);
                                println!(
                                    "Password: {}",
                                    if has_password.is_some() {
                                        "configured"
                                    } else {
                                        "not set"
                                    }
                                );
                                println!("TLS: {}", tls);
                                println!("Database: {}", database);
                            }
                        }
                    }

                    if is_default_enterprise {
                        println!("Default for enterprise: yes");
                    }
                    if is_default_cloud {
                        println!("Default for cloud: yes");
                    }
                    if conn_mgr.config.default_database.as_deref() == Some(name) {
                        println!("Default for database: yes");
                    }
                }
            }

            Ok(())
        }
        None => Err(RedisCtlError::ProfileNotFound { name: name.into() }),
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_set(
    conn_mgr: &ConnectionManager,
    name: &str,
    deployment: &redisctl_config::DeploymentType,
    api_key: &Option<String>,
    api_secret: &Option<String>,
    api_url: &str,
    url: &Option<String>,
    username: &Option<String>,
    password: &Option<String>,
    insecure: &bool,
    ca_cert: &Option<String>,
    host: &Option<String>,
    port: &Option<u16>,
    no_tls: &bool,
    db: &Option<u8>,
    #[cfg(feature = "secure-storage")] use_keyring: &bool,
) -> Result<(), RedisCtlError> {
    debug!("Setting profile: {}", name);

    // Check if profile already exists
    if conn_mgr.config.profiles.contains_key(name) {
        // Ask for confirmation before overwriting
        println!("Profile '{}' already exists.", name);
        print!("Do you want to overwrite it? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Profile update cancelled.");
            return Ok(());
        }
    }

    // Create the profile based on deployment type
    let profile = match deployment {
        redisctl_config::DeploymentType::Cloud => {
            let api_key = api_key
                .clone()
                .ok_or_else(|| anyhow::anyhow!("API key is required for Cloud profiles"))?;
            let api_secret = api_secret
                .clone()
                .ok_or_else(|| anyhow::anyhow!("API secret is required for Cloud profiles"))?;

            // Handle keyring storage if requested
            #[cfg(feature = "secure-storage")]
            let (stored_key, stored_secret) = if *use_keyring {
                use redisctl_config::CredentialStore;
                let store = CredentialStore::new();

                // Store credentials in keyring and get references
                let key_ref = store
                    .store_credential(&format!("{}-api-key", name), &api_key)
                    .context("Failed to store API key in keyring")?;
                let secret_ref = store
                    .store_credential(&format!("{}-api-secret", name), &api_secret)
                    .context("Failed to store API secret in keyring")?;

                println!("Credentials stored securely in OS keyring");
                (key_ref, secret_ref)
            } else {
                (api_key.clone(), api_secret.clone())
            };

            #[cfg(not(feature = "secure-storage"))]
            let (stored_key, stored_secret) = (api_key.clone(), api_secret.clone());

            redisctl_config::Profile {
                deployment_type: redisctl_config::DeploymentType::Cloud,
                credentials: redisctl_config::ProfileCredentials::Cloud {
                    api_key: stored_key,
                    api_secret: stored_secret,
                    api_url: api_url.to_string(),
                },
                files_api_key: None,
                resilience: None,
            }
        }
        redisctl_config::DeploymentType::Enterprise => {
            let url = url
                .clone()
                .ok_or_else(|| anyhow::anyhow!("URL is required for Enterprise profiles"))?;
            let username = username
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Username is required for Enterprise profiles"))?;

            // Prompt for password if not provided
            let password = match password {
                Some(p) => Some(p.clone()),
                None => {
                    let pass = rpassword::prompt_password("Enter password: ")
                        .context("Failed to read password")?;
                    Some(pass)
                }
            };

            // Handle keyring storage if requested
            #[cfg(feature = "secure-storage")]
            let (stored_username, stored_password) = if *use_keyring {
                use redisctl_config::CredentialStore;
                let store = CredentialStore::new();

                // Store credentials in keyring and get references
                let user_ref = store
                    .store_credential(&format!("{}-username", name), &username)
                    .context("Failed to store username in keyring")?;

                let pass_ref = if let Some(ref p) = password {
                    Some(
                        store
                            .store_credential(&format!("{}-password", name), p)
                            .context("Failed to store password in keyring")?,
                    )
                } else {
                    None
                };

                println!("Credentials stored securely in OS keyring");
                (user_ref, pass_ref)
            } else {
                (username.clone(), password.clone())
            };

            #[cfg(not(feature = "secure-storage"))]
            let (stored_username, stored_password) = (username.clone(), password.clone());

            redisctl_config::Profile {
                deployment_type: redisctl_config::DeploymentType::Enterprise,
                credentials: redisctl_config::ProfileCredentials::Enterprise {
                    url: url.clone(),
                    username: stored_username,
                    password: stored_password,
                    insecure: *insecure,
                    ca_cert: ca_cert.clone(),
                },
                files_api_key: None,
                resilience: None,
            }
        }
        redisctl_config::DeploymentType::Database => {
            let host = host
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Host is required for Database profiles"))?;
            let port =
                port.ok_or_else(|| anyhow::anyhow!("Port is required for Database profiles"))?;

            // Prompt for password if not provided (optional for database profiles)
            let password = match password {
                Some(p) if !p.is_empty() => Some(p.clone()),
                _ => {
                    print!("Enter password (press Enter for none): ");
                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                    let pass = rpassword::read_password().context("Failed to read password")?;
                    if pass.is_empty() { None } else { Some(pass) }
                }
            };

            // Use username if provided, otherwise default to "default"
            let username = username.clone().unwrap_or_else(|| "default".to_string());

            // Handle keyring storage if requested
            #[cfg(feature = "secure-storage")]
            let stored_password = if *use_keyring {
                if let Some(ref p) = password {
                    use redisctl_config::CredentialStore;
                    let store = CredentialStore::new();
                    let pass_ref = store
                        .store_credential(&format!("{}-password", name), p)
                        .context("Failed to store password in keyring")?;
                    println!("Password stored securely in OS keyring");
                    Some(pass_ref)
                } else {
                    None
                }
            } else {
                password.clone()
            };

            #[cfg(not(feature = "secure-storage"))]
            let stored_password = password.clone();

            redisctl_config::Profile {
                deployment_type: redisctl_config::DeploymentType::Database,
                credentials: redisctl_config::ProfileCredentials::Database {
                    host,
                    port,
                    password: stored_password,
                    tls: !*no_tls,
                    username,
                    database: db.unwrap_or(0),
                },
                files_api_key: None,
                resilience: None,
            }
        }
    };

    // Update the configuration
    let mut config = conn_mgr.config.clone();
    config.profiles.insert(name.to_string(), profile);

    // Save the configuration to the appropriate location
    if let Some(ref path) = conn_mgr.config_path {
        config
            .save_to_path(path)
            .context("Failed to save configuration")?;
        println!("Profile '{}' saved successfully to:", name);
        println!("  {}", path.display());
    } else {
        config.save().context("Failed to save configuration")?;
        if let Ok(config_path) = Config::config_path() {
            println!("Profile '{}' saved successfully to:", name);
            println!("  {}", config_path.display());
        } else {
            println!("Profile '{}' saved successfully.", name);
        }
    }

    // Suggest setting as default if it's the only profile of its type
    let profiles_of_type = config.get_profiles_of_type(*deployment);
    if profiles_of_type.len() == 1 {
        println!();
        match deployment {
            redisctl_config::DeploymentType::Enterprise => {
                println!("Tip: Set as default for enterprise commands with:");
                println!("  redisctl profile default-enterprise {}", name);
            }
            redisctl_config::DeploymentType::Cloud => {
                println!("Tip: Set as default for cloud commands with:");
                println!("  redisctl profile default-cloud {}", name);
            }
            redisctl_config::DeploymentType::Database => {
                println!("Tip: Set as default for database commands with:");
                println!("  redisctl profile default-database {}", name);
            }
        }
    }

    Ok(())
}

async fn handle_remove(conn_mgr: &ConnectionManager, name: &str) -> Result<(), RedisCtlError> {
    debug!("Removing profile: {}", name);

    // Check if profile exists
    if !conn_mgr.config.profiles.contains_key(name) {
        return Err(RedisCtlError::ProfileNotFound { name: name.into() });
    }

    // Check if it's a default profile
    let is_default_enterprise = conn_mgr.config.default_enterprise.as_deref() == Some(name);
    let is_default_cloud = conn_mgr.config.default_cloud.as_deref() == Some(name);
    if is_default_enterprise {
        println!(
            "Warning: '{}' is the default profile for enterprise commands.",
            name
        );
    }
    if is_default_cloud {
        println!(
            "Warning: '{}' is the default profile for cloud commands.",
            name
        );
    }

    // Ask for confirmation
    print!(
        "Are you sure you want to remove profile '{}'? (y/N): ",
        name
    );
    use std::io::{self, Write};
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if input != "y" && input != "yes" {
        println!("Profile removal cancelled.");
        return Ok(());
    }

    // Remove the profile
    let mut config = conn_mgr.config.clone();
    config.profiles.remove(name);

    // Clear defaults if this was a default profile
    if is_default_enterprise {
        config.default_enterprise = None;
        println!("Default enterprise profile cleared.");
    }
    if is_default_cloud {
        config.default_cloud = None;
        println!("Default cloud profile cleared.");
    }

    // Save the configuration to the appropriate location
    if let Some(ref path) = conn_mgr.config_path {
        config
            .save_to_path(path)
            .context("Failed to save configuration")?;
    } else {
        config.save().context("Failed to save configuration")?;
    }

    println!("Profile '{}' removed successfully.", name);
    Ok(())
}

async fn handle_default_enterprise(
    conn_mgr: &ConnectionManager,
    name: &str,
) -> Result<(), RedisCtlError> {
    debug!("Setting default enterprise profile: {}", name);

    // Check if profile exists and is an enterprise profile
    match conn_mgr.config.profiles.get(name) {
        Some(profile) => {
            if profile.deployment_type != redisctl_config::DeploymentType::Enterprise {
                return Err(anyhow::anyhow!(
                    "Profile '{}' is a cloud profile, not an enterprise profile",
                    name
                )
                .into());
            }
        }
        None => return Err(RedisCtlError::ProfileNotFound { name: name.into() }),
    }

    // Update the configuration
    let mut config = conn_mgr.config.clone();
    config.default_enterprise = Some(name.to_string());

    // Save the configuration to the appropriate location
    if let Some(ref path) = conn_mgr.config_path {
        config
            .save_to_path(path)
            .context("Failed to save configuration")?;
    } else {
        config.save().context("Failed to save configuration")?;
    }

    println!("Default enterprise profile set to '{}'.", name);
    Ok(())
}

async fn handle_default_cloud(
    conn_mgr: &ConnectionManager,
    name: &str,
) -> Result<(), RedisCtlError> {
    debug!("Setting default cloud profile: {}", name);

    // Check if profile exists and is a cloud profile
    match conn_mgr.config.profiles.get(name) {
        Some(profile) => {
            if profile.deployment_type != redisctl_config::DeploymentType::Cloud {
                return Err(anyhow::anyhow!(
                    "Profile '{}' is an enterprise profile, not a cloud profile",
                    name
                )
                .into());
            }
        }
        None => return Err(RedisCtlError::ProfileNotFound { name: name.into() }),
    }

    // Update the configuration
    let mut config = conn_mgr.config.clone();
    config.default_cloud = Some(name.to_string());

    // Save the configuration to the appropriate location
    if let Some(ref path) = conn_mgr.config_path {
        config
            .save_to_path(path)
            .context("Failed to save configuration")?;
    } else {
        config.save().context("Failed to save configuration")?;
    }

    println!("Default cloud profile set to '{}'.", name);
    Ok(())
}

async fn handle_default_database(
    conn_mgr: &ConnectionManager,
    name: &str,
) -> Result<(), RedisCtlError> {
    debug!("Setting default database profile: {}", name);

    // Check if profile exists and is a database profile
    match conn_mgr.config.profiles.get(name) {
        Some(profile) => {
            if profile.deployment_type != redisctl_config::DeploymentType::Database {
                return Err(anyhow::anyhow!("Profile '{}' is not a database profile", name).into());
            }
        }
        None => return Err(RedisCtlError::ProfileNotFound { name: name.into() }),
    }

    // Update the configuration
    let mut config = conn_mgr.config.clone();
    config.default_database = Some(name.to_string());

    // Save the configuration to the appropriate location
    if let Some(ref path) = conn_mgr.config_path {
        config
            .save_to_path(path)
            .context("Failed to save configuration")?;
    } else {
        config.save().context("Failed to save configuration")?;
    }

    println!("Default database profile set to '{}'.", name);
    Ok(())
}

async fn handle_validate(conn_mgr: &ConnectionManager) -> Result<(), RedisCtlError> {
    debug!("Validating configuration");

    // Configuration file validation
    let config_path = Config::config_path()?;
    println!("Configuration file: {}", config_path.display());

    if !config_path.exists() {
        println!("✗ Configuration file does not exist");
        println!("\nTry:");
        println!("  • Create a profile: redisctl profile set <name> <type>");
        return Ok(());
    }

    println!("✓ Configuration file exists and is readable");

    // Profile validation
    let profiles = conn_mgr.config.list_profiles();
    println!("✓ Found {} profile(s)", profiles.len());

    if profiles.is_empty() {
        println!("\n⚠ No profiles configured");
        println!("\nTry:");
        println!(
            "  • Create a Cloud profile: redisctl profile set mycloud cloud --api-key <key> --api-secret <secret>"
        );
        println!(
            "  • Create an Enterprise profile: redisctl profile set myenterprise enterprise --url <url> --username <user>"
        );
        return Ok(());
    }

    println!();

    let mut has_warnings = false;
    let mut has_errors = false;

    for (name, profile) in profiles {
        print!("Profile '{}' ({}): ", name, profile.deployment_type);

        match profile.deployment_type {
            redisctl_config::DeploymentType::Cloud => match profile.cloud_credentials() {
                Some((api_key, api_secret, api_url)) => {
                    if api_key.is_empty() || api_secret.is_empty() {
                        println!("✗ Missing credentials");
                        has_errors = true;
                    } else {
                        println!("✓ Valid");
                    }

                    // Check for valid URL
                    if !api_url.starts_with("http://") && !api_url.starts_with("https://") {
                        println!("  ⚠ API URL should start with http:// or https://");
                        has_warnings = true;
                    }
                }
                None => {
                    println!("✗ Missing Cloud credentials");
                    has_errors = true;
                }
            },
            redisctl_config::DeploymentType::Enterprise => match profile.enterprise_credentials() {
                Some((url, username, password, _insecure, _ca_cert)) => {
                    if username.is_empty() {
                        println!("✗ Missing username");
                        has_errors = true;
                    } else if password.is_none()
                        || password.as_ref().is_none_or(|p: &&str| p.is_empty())
                    {
                        println!("⚠ Missing password (will be prompted)");
                        has_warnings = true;
                    } else {
                        println!("✓ Valid");
                    }

                    // Check for valid URL
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        println!("  ⚠ URL should start with http:// or https://");
                        has_warnings = true;
                    }
                }
                None => {
                    println!("✗ Missing Enterprise credentials");
                    has_errors = true;
                }
            },
            redisctl_config::DeploymentType::Database => match profile.database_credentials() {
                Some((host, port, password, _tls, _username, _database)) => {
                    if host.is_empty() {
                        println!("✗ Missing host");
                        has_errors = true;
                    } else if port == 0 {
                        println!("✗ Invalid port");
                        has_errors = true;
                    } else if password.is_none() || password.as_ref().is_none_or(|p| p.is_empty()) {
                        println!("⚠ No password configured");
                        has_warnings = true;
                    } else {
                        println!("✓ Valid");
                    }
                }
                None => {
                    println!("✗ Missing Database credentials");
                    has_errors = true;
                }
            },
        }
    }

    // Check default profiles
    println!();
    if let Some(default_ent) = &conn_mgr.config.default_enterprise {
        if conn_mgr.config.profiles.contains_key(default_ent) {
            println!("✓ Default enterprise profile: {}", default_ent);
        } else {
            println!("✗ Default enterprise profile '{}' not found", default_ent);
            has_errors = true;
        }
    }

    if let Some(default_cloud) = &conn_mgr.config.default_cloud {
        if conn_mgr.config.profiles.contains_key(default_cloud) {
            println!("✓ Default cloud profile: {}", default_cloud);
        } else {
            println!("✗ Default cloud profile '{}' not found", default_cloud);
            has_errors = true;
        }
    }

    if let Some(default_db) = &conn_mgr.config.default_database {
        if conn_mgr.config.profiles.contains_key(default_db) {
            println!("✓ Default database profile: {}", default_db);
        } else {
            println!("✗ Default database profile '{}' not found", default_db);
            has_errors = true;
        }
    }

    println!();
    if has_errors {
        println!("⚠ Configuration has errors. Fix them before using affected profiles.");
    } else if has_warnings {
        println!("⚠ Configuration has warnings but should work.");
    } else {
        println!("✓ Configuration is valid");
    }

    Ok(())
}
