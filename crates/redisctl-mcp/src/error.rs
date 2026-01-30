//! Error types for the MCP server

use thiserror::Error;

/// Errors that can occur in the MCP server
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum McpError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Cloud API error
    #[error("Cloud API error: {0}")]
    CloudApi(String),

    /// Enterprise API error
    #[error("Enterprise API error: {0}")]
    EnterpriseApi(String),

    /// Redis database error
    #[error("Redis error: {0}")]
    Redis(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    /// Operation not permitted in read-only mode
    #[error("Operation not permitted: server is in read-only mode")]
    ReadOnlyMode,
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::ToolExecution(err.to_string())
    }
}

impl From<redisctl_config::ConfigError> for McpError {
    fn from(err: redisctl_config::ConfigError) -> Self {
        McpError::Configuration(err.to_string())
    }
}

impl From<redis::RedisError> for McpError {
    fn from(err: redis::RedisError) -> Self {
        McpError::Redis(err.to_string())
    }
}
