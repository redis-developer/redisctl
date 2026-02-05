//! Unified error handling for redisctl-core
//!
//! Wraps both Cloud and Enterprise errors with consistent helper methods.
//!
//! # Example
//!
//! ```rust
//! use redisctl_core::{CoreError, Result};
//! use redis_cloud::CloudError;
//!
//! fn handle_error(err: CoreError) {
//!     if err.is_not_found() {
//!         println!("Resource not found");
//!     } else if err.is_retryable() {
//!         println!("Temporary error, can retry");
//!     }
//! }
//!
//! // Cloud errors are automatically converted
//! let cloud_err = CloudError::NotFound { message: "DB not found".to_string() };
//! let core_err: CoreError = cloud_err.into();
//! assert!(core_err.is_not_found());
//! ```

use std::time::Duration;
use thiserror::Error;

/// Core error type wrapping both platform errors
#[derive(Error, Debug)]
pub enum CoreError {
    /// Error from Redis Cloud API
    #[error("Cloud API error: {0}")]
    Cloud(#[from] redis_cloud::CloudError),

    /// Error from Redis Enterprise API
    #[error("Enterprise API error: {0}")]
    Enterprise(#[from] redis_enterprise::RestError),

    /// Task timed out waiting for completion
    #[error("Task timed out after {0:?}")]
    TaskTimeout(Duration),

    /// Task failed during async operation
    #[error("Task failed: {0}")]
    TaskFailed(String),

    /// Validation error (e.g., module resolution)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for core operations
pub type Result<T> = std::result::Result<T, CoreError>;

impl CoreError {
    /// Returns true if this is a "not found" error (404)
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_not_found(),
            CoreError::Enterprise(e) => e.is_not_found(),
            _ => false,
        }
    }

    /// Returns true if this is an authentication/authorization error (401/403)
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_unauthorized(),
            CoreError::Enterprise(e) => e.is_unauthorized(),
            _ => false,
        }
    }

    /// Returns true if this is a server error (5xx)
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_server_error(),
            CoreError::Enterprise(e) => e.is_server_error(),
            _ => false,
        }
    }

    /// Returns true if this is a timeout error
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_timeout(),
            CoreError::Enterprise(e) => e.is_timeout(),
            CoreError::TaskTimeout(_) => true,
            _ => false,
        }
    }

    /// Returns true if this is a rate limiting error (429)
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_rate_limited(),
            CoreError::Enterprise(e) => e.is_rate_limited(),
            _ => false,
        }
    }

    /// Returns true if this is a conflict/precondition error (409/412)
    #[must_use]
    pub fn is_conflict(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_conflict(),
            CoreError::Enterprise(e) => e.is_conflict(),
            _ => false,
        }
    }

    /// Returns true if this is a bad request error (400)
    #[must_use]
    pub fn is_bad_request(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_bad_request(),
            CoreError::Enterprise(e) => e.is_bad_request(),
            CoreError::Validation(_) => true,
            _ => false,
        }
    }

    /// Returns true if this error is potentially retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            CoreError::Cloud(e) => e.is_retryable(),
            CoreError::Enterprise(e) => e.is_retryable(),
            CoreError::TaskTimeout(_) => true, // Timeout might succeed on retry
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis_cloud::CloudError;
    use redis_enterprise::RestError;

    #[test]
    fn test_core_error_from_cloud() {
        let cloud_err = CloudError::NotFound {
            message: "Database not found".to_string(),
        };
        let core_err: CoreError = cloud_err.into();

        assert!(core_err.is_not_found());
        assert!(!core_err.is_unauthorized());
        assert!(!core_err.is_retryable());
    }

    #[test]
    fn test_core_error_from_enterprise() {
        let enterprise_err = RestError::NotFound;
        let core_err: CoreError = enterprise_err.into();

        assert!(core_err.is_not_found());
        assert!(!core_err.is_unauthorized());
    }

    #[test]
    fn test_core_error_cloud_helpers_delegate() {
        // Test that all helper methods properly delegate to Cloud errors
        let unauthorized = CloudError::AuthenticationFailed {
            message: "Bad creds".to_string(),
        };
        let core_err: CoreError = unauthorized.into();
        assert!(core_err.is_unauthorized());

        let rate_limited = CloudError::RateLimited {
            message: "Too many requests".to_string(),
        };
        let core_err: CoreError = rate_limited.into();
        assert!(core_err.is_rate_limited());
        assert!(core_err.is_retryable());

        let bad_request = CloudError::BadRequest {
            message: "Invalid input".to_string(),
        };
        let core_err: CoreError = bad_request.into();
        assert!(core_err.is_bad_request());
    }

    #[test]
    fn test_core_error_enterprise_helpers_delegate() {
        // Test that all helper methods properly delegate to Enterprise errors
        let unauthorized = RestError::AuthenticationFailed;
        let core_err: CoreError = unauthorized.into();
        assert!(core_err.is_unauthorized());

        let server_error = RestError::ServerError("Internal error".to_string());
        let core_err: CoreError = server_error.into();
        assert!(core_err.is_server_error());
        assert!(core_err.is_retryable());
    }

    #[test]
    fn test_core_error_task_timeout() {
        let err = CoreError::TaskTimeout(Duration::from_secs(600));
        assert!(err.is_timeout());
        assert!(err.is_retryable()); // Timeouts are retryable
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_core_error_validation() {
        let err = CoreError::Validation("Invalid module name".to_string());
        assert!(err.is_bad_request()); // Validation errors map to bad request
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_core_error_display() {
        let cloud_err: CoreError = CloudError::NotFound {
            message: "Not found".to_string(),
        }
        .into();
        assert!(cloud_err.to_string().contains("Cloud API error"));

        let timeout_err = CoreError::TaskTimeout(Duration::from_secs(60));
        assert!(timeout_err.to_string().contains("timed out"));
    }
}
