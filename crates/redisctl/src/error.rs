//! Error types for redisctl
//!
//! Defines structured error types using thiserror for better error handling and user experience.

#![allow(dead_code)] // Foundation code - will be used in future PRs

use thiserror::Error;

/// Main error type for the redisctl application
#[derive(Error, Debug)]
pub enum RedisCtlError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Profile '{name}' not found")]
    ProfileNotFound { name: String },

    #[error("Profile '{name}' is type '{actual_type}' but command requires '{expected_type}'")]
    ProfileTypeMismatch {
        name: String,
        actual_type: String,
        expected_type: String,
    },

    #[error("No profile configured. Use 'redisctl profile set' to configure a profile.")]
    NoProfileConfigured,

    #[error("Missing credentials for profile '{name}'")]
    MissingCredentials { name: String },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("API error: {message}")]
    ApiError { message: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Command not supported for deployment type '{deployment_type}'")]
    UnsupportedDeploymentType { deployment_type: String },
    #[error("File error for '{path}': {message}")]
    FileError { path: String, message: String },

    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Timeout: {message}")]
    Timeout { message: String },

    #[error("Output formatting error: {message}")]
    OutputError { message: String },
}

/// Result type for redisctl operations
pub type Result<T> = std::result::Result<T, RedisCtlError>;

impl RedisCtlError {
    /// Get helpful suggestions for resolving this error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            RedisCtlError::ProfileNotFound { name } => vec![
                format!("List available profiles: redisctl profile list"),
                format!("Create profile '{}': redisctl profile set {}", name, name),
                format!("Check profile name spelling"),
            ],
            RedisCtlError::NoProfileConfigured => vec![
                "Create a Cloud profile: redisctl profile set mycloud cloud --api-key <key> --api-secret <secret>".to_string(),
                "Create an Enterprise profile: redisctl profile set myenterprise enterprise --url <url> --username <user>".to_string(),
                "View profile documentation: redisctl profile --help".to_string(),
            ],
            RedisCtlError::MissingCredentials { name } => vec![
                format!("Update profile credentials: redisctl profile set {}", name),
                format!("Check profile details: redisctl profile show {}", name),
                "Verify environment variables are set correctly".to_string(),
            ],
            RedisCtlError::AuthenticationFailed { .. } => vec![
                "Check your credentials: redisctl profile show <profile>".to_string(),
                "For Cloud: Verify API key and secret are correct".to_string(),
                "For Enterprise: Verify username and password are correct".to_string(),
                "Ensure the API endpoint URL is correct".to_string(),
            ],
            RedisCtlError::ConnectionError { message } if message.contains("certificate") || message.contains("SSL") => vec![
                "For Enterprise: Try with --insecure flag for self-signed certificates".to_string(),
                "Update profile with insecure option: redisctl profile set <name> enterprise --insecure".to_string(),
                "Check that the server URL is correct and reachable".to_string(),
            ],
            RedisCtlError::ConnectionError { .. } => vec![
                "Check network connectivity".to_string(),
                "Verify the server URL is correct: redisctl profile show <profile>".to_string(),
                "Ensure firewall allows connections to the API endpoint".to_string(),
            ],
            RedisCtlError::ApiError { message } if message.contains("404") => vec![
                "Verify the resource ID is correct".to_string(),
                "List available resources to find the correct ID".to_string(),
                "Check that you're using the correct profile".to_string(),
            ],
            RedisCtlError::ProfileTypeMismatch { expected_type, .. } => vec![
                format!("Use a {} profile for this command", expected_type),
                format!("List profiles: redisctl profile list"),
                format!("Create a {} profile: redisctl profile set <name> {}", expected_type, expected_type.to_lowercase()),
            ],
            RedisCtlError::UnsupportedDeploymentType { .. } => vec![
                "Check the command documentation: redisctl <command> --help".to_string(),
                "Use the appropriate command for your deployment type".to_string(),
            ],
            RedisCtlError::InvalidInput { .. } => vec![
                "Check the command syntax: redisctl <command> --help".to_string(),
                "Verify input file format is correct (JSON/YAML)".to_string(),
            ],
            RedisCtlError::FileError { path, .. } => vec![
                format!("Check that file exists: {}", path),
                "Verify file permissions are correct".to_string(),
                "Ensure file path is correct (use absolute path if needed)".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Format error with suggestions for display
    pub fn display_with_suggestions(&self) -> String {
        let mut output = format!("Error: {}", self);

        let suggestions = self.suggestions();
        if !suggestions.is_empty() {
            output.push_str("\n\nTry:\n");
            for suggestion in suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }

        output
    }
}

impl From<redis_cloud::CloudError> for RedisCtlError {
    fn from(err: redis_cloud::CloudError) -> Self {
        match err {
            redis_cloud::CloudError::AuthenticationFailed { message } => {
                RedisCtlError::AuthenticationFailed { message }
            }
            redis_cloud::CloudError::ConnectionError(message) => {
                RedisCtlError::ConnectionError { message }
            }
            _ => RedisCtlError::ApiError {
                message: err.to_string(),
            },
        }
    }
}

impl From<redis_enterprise::RestError> for RedisCtlError {
    fn from(err: redis_enterprise::RestError) -> Self {
        match err {
            redis_enterprise::RestError::AuthenticationFailed => {
                RedisCtlError::AuthenticationFailed {
                    message: "Authentication failed".to_string(),
                }
            }
            redis_enterprise::RestError::Unauthorized => RedisCtlError::AuthenticationFailed {
                message: "401 Unauthorized: Invalid username or password. Check your credentials."
                    .to_string(),
            },
            redis_enterprise::RestError::NotFound => RedisCtlError::ApiError {
                message: "404 Not Found: The requested resource does not exist".to_string(),
            },
            redis_enterprise::RestError::ApiError { code, message } => RedisCtlError::ApiError {
                message: format!("HTTP {}: {}", code, message),
            },
            redis_enterprise::RestError::ServerError(msg) => RedisCtlError::ApiError {
                message: format!("Server error (5xx): {}", msg),
            },
            redis_enterprise::RestError::RequestFailed(reqwest_err) => {
                RedisCtlError::ConnectionError {
                    message: reqwest_err.to_string(),
                }
            }
            redis_enterprise::RestError::ConnectionError(msg) => {
                RedisCtlError::ConnectionError { message: msg }
            }
            redis_enterprise::RestError::ValidationError(msg) => {
                RedisCtlError::InvalidInput { message: msg }
            }
            _ => RedisCtlError::ApiError {
                message: err.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for RedisCtlError {
    fn from(err: serde_json::Error) -> Self {
        RedisCtlError::OutputError {
            message: format!("JSON error: {}", err),
        }
    }
}

impl From<std::io::Error> for RedisCtlError {
    fn from(err: std::io::Error) -> Self {
        RedisCtlError::OutputError {
            message: format!("IO error: {}", err),
        }
    }
}

impl From<anyhow::Error> for RedisCtlError {
    fn from(err: anyhow::Error) -> Self {
        RedisCtlError::Config(err.to_string())
    }
}

impl From<redisctl_core::ConfigError> for RedisCtlError {
    fn from(err: redisctl_core::ConfigError) -> Self {
        RedisCtlError::Configuration(err.to_string())
    }
}

impl From<redisctl_core::error::CoreError> for RedisCtlError {
    fn from(err: redisctl_core::error::CoreError) -> Self {
        match err {
            redisctl_core::error::CoreError::TaskTimeout(duration) => RedisCtlError::Timeout {
                message: format!("Operation timed out after {} seconds", duration.as_secs()),
            },
            redisctl_core::error::CoreError::TaskFailed(msg) => RedisCtlError::ApiError {
                message: format!("Task failed: {}", msg),
            },
            redisctl_core::error::CoreError::Validation(msg) => {
                RedisCtlError::InvalidInput { message: msg }
            }
            redisctl_core::error::CoreError::Config(msg) => RedisCtlError::Configuration(msg),
            redisctl_core::error::CoreError::Cloud(cloud_err) => RedisCtlError::from(cloud_err),
            redisctl_core::error::CoreError::Enterprise(enterprise_err) => {
                RedisCtlError::from(enterprise_err)
            }
        }
    }
}
