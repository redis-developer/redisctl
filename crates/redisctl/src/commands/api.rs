//! Raw API access commands for direct REST endpoint calls

use crate::cli::{HttpMethod, OutputFormat};
use crate::connection::ConnectionManager;
use crate::error::Result as CliResult;
use crate::output::print_output;
use anyhow::Context;
use redisctl_core::{Config, DeploymentType};
use serde_json::Value;

/// Parameters for API command execution
#[allow(dead_code)] // Used by binary target
pub struct ApiCommandParams {
    pub config: Config,
    pub config_path: Option<std::path::PathBuf>,
    pub profile_name: Option<String>,
    pub deployment: DeploymentType,
    pub method: HttpMethod,
    pub path: String,
    pub data: Option<String>,
    pub query: Option<String>,
    pub output_format: OutputFormat,
}

/// Handle raw API commands
#[allow(dead_code)] // Used by binary target
pub async fn handle_api_command(params: ApiCommandParams) -> CliResult<()> {
    let connection_manager = ConnectionManager::with_config_path(params.config, params.config_path);

    match params.deployment {
        DeploymentType::Cloud => {
            handle_cloud_api(
                connection_manager,
                params.profile_name.as_deref(),
                params.method,
                params.path,
                params.data,
                params.query,
                params.output_format,
            )
            .await
        }
        DeploymentType::Enterprise => {
            handle_enterprise_api(
                connection_manager,
                params.profile_name.as_deref(),
                params.method,
                params.path,
                params.data,
                params.query,
                params.output_format,
            )
            .await
        }
        DeploymentType::Database => Err(anyhow::anyhow!(
            "Raw API access is not supported for database profiles. Database profiles are for direct Redis connections."
        ).into()),
    }
}

/// Handle Cloud API calls
#[allow(dead_code)] // Used by binary target
async fn handle_cloud_api(
    connection_manager: ConnectionManager,
    profile_name: Option<&str>,
    method: HttpMethod,
    path: String,
    data: Option<String>,
    query: Option<String>,
    output_format: OutputFormat,
) -> CliResult<()> {
    let client = connection_manager.create_cloud_client(profile_name).await?;

    // Ensure path starts with /
    let normalized_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };

    // Parse request body if provided
    let body: Option<Value> = if let Some(data_str) = data {
        if let Some(file_path) = data_str.strip_prefix('@') {
            // Read from file
            let content = std::fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read file: {}", file_path))?;
            Some(
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?,
            )
        } else {
            // Parse as JSON string
            Some(
                serde_json::from_str(&data_str)
                    .context("Failed to parse JSON from data parameter")?,
            )
        }
    } else {
        None
    };

    // Execute the API call based on HTTP method
    let result: std::result::Result<Value, _> = match method {
        HttpMethod::Get => client.get_raw(&normalized_path).await,
        HttpMethod::Post => {
            let body = body.unwrap_or(serde_json::json!({}));
            client.post_raw(&normalized_path, body).await
        }
        HttpMethod::Put => {
            let body = body.unwrap_or(serde_json::json!({}));
            client.put_raw(&normalized_path, body).await
        }
        HttpMethod::Patch => {
            let body = body.unwrap_or(serde_json::json!({}));
            client.patch_raw(&normalized_path, body).await
        }
        HttpMethod::Delete => client.delete_raw(&normalized_path).await,
    };

    match result {
        Ok(response) => {
            // Convert CLI OutputFormat to output::OutputFormat
            let format = match output_format {
                crate::cli::OutputFormat::Auto | crate::cli::OutputFormat::Json => {
                    crate::output::OutputFormat::Json
                }
                crate::cli::OutputFormat::Yaml => crate::output::OutputFormat::Yaml,
                crate::cli::OutputFormat::Table => crate::output::OutputFormat::Table,
            };

            print_output(response, format, query.as_deref()).map_err(|e| {
                crate::error::RedisCtlError::OutputError {
                    message: e.to_string(),
                }
            })?;
            Ok(())
        }
        Err(e) => {
            // Format error nicely
            eprintln!("API Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle Enterprise API calls
#[allow(dead_code)] // Used by binary target
async fn handle_enterprise_api(
    connection_manager: ConnectionManager,
    profile_name: Option<&str>,
    method: HttpMethod,
    path: String,
    data: Option<String>,
    query: Option<String>,
    output_format: OutputFormat,
) -> CliResult<()> {
    let client = connection_manager
        .create_enterprise_client(profile_name)
        .await?;

    // Normalize path with smart v1 prefixing for Enterprise
    let normalized_path = if path.starts_with('/') {
        // Path has leading slash - check if it has version
        if path.starts_with("/v")
            && path
                .chars()
                .nth(2)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            // Already has version (e.g., /v1/cluster, /v2/bdbs)
            path
        } else if path == "/" {
            // Just root path - prefix with /v1
            "/v1".to_string()
        } else {
            // Has leading slash but no version - prefix with /v1
            format!("/v1{}", path)
        }
    } else {
        // No leading slash - check if it starts with version
        if path.starts_with("v")
            && path
                .chars()
                .nth(1)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            // Starts with version (e.g., v1/cluster) - just add leading slash
            format!("/{}", path)
        } else {
            // No version - prefix with /v1/
            format!("/v1/{}", path)
        }
    };

    // Parse request body if provided
    let body: Option<Value> = if let Some(data_str) = data {
        if let Some(file_path) = data_str.strip_prefix('@') {
            // Read from file
            let content = std::fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read file: {}", file_path))?;
            Some(
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?,
            )
        } else {
            // Parse as JSON string
            Some(
                serde_json::from_str(&data_str)
                    .context("Failed to parse JSON from data parameter")?,
            )
        }
    } else {
        None
    };

    // Execute the API call based on HTTP method
    let result: std::result::Result<Value, _> = match method {
        HttpMethod::Get => client.get_raw(&normalized_path).await,
        HttpMethod::Post => {
            let body = body.unwrap_or(serde_json::json!({}));
            client.post_raw(&normalized_path, body).await
        }
        HttpMethod::Put => {
            let body = body.unwrap_or(serde_json::json!({}));
            client.put_raw(&normalized_path, body).await
        }
        HttpMethod::Patch => {
            let body = body.unwrap_or(serde_json::json!({}));
            client.patch_raw(&normalized_path, body).await
        }
        HttpMethod::Delete => client.delete_raw(&normalized_path).await,
    };

    match result {
        Ok(response) => {
            // Convert CLI OutputFormat to output::OutputFormat
            let format = match output_format {
                crate::cli::OutputFormat::Auto | crate::cli::OutputFormat::Json => {
                    crate::output::OutputFormat::Json
                }
                crate::cli::OutputFormat::Yaml => crate::output::OutputFormat::Yaml,
                crate::cli::OutputFormat::Table => crate::output::OutputFormat::Table,
            };

            print_output(response, format, query.as_deref()).map_err(|e| {
                crate::error::RedisCtlError::OutputError {
                    message: e.to_string(),
                }
            })?;
            Ok(())
        }
        Err(e) => {
            // Format error nicely
            eprintln!("API Error: {}", e);
            std::process::exit(1);
        }
    }
}
