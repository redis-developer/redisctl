//! Error types for configuration operations

use thiserror::Error;

/// Errors that can occur during configuration operations
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config from {path}: {source}")]
    LoadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to save config to {path}: {source}")]
    SaveError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Profile '{name}' not found")]
    ProfileNotFound { name: String },

    #[error("No {deployment_type} profiles configured. {suggestion}")]
    NoProfilesOfType {
        deployment_type: String,
        suggestion: String,
    },

    #[error("Failed to resolve credential: {0}")]
    CredentialError(String),

    #[cfg(feature = "secure-storage")]
    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Environment variable expansion failed: {0}")]
    EnvExpansionError(String),

    #[error("Failed to determine config directory")]
    ConfigDirError,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;
