//! Resilience configuration for API clients
//!
//! This module defines configuration structures for resilience patterns
//! (circuit breaker, retry, rate limiting) that can be stored in profiles.

use serde::{Deserialize, Serialize};

/// Configuration for resilience patterns
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether circuit breaker is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Failure rate threshold (0.0 to 1.0) to open the circuit
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: f32,

    /// Number of calls to track in the sliding window
    #[serde(default = "default_window_size")]
    pub window_size: u32,

    /// Duration in seconds to wait before attempting to close the circuit
    #[serde(default = "default_reset_timeout")]
    pub reset_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 0.5,
            window_size: 20,
            reset_timeout_secs: 60,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Whether retry is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of retry attempts
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Initial backoff in milliseconds
    #[serde(default = "default_backoff_ms")]
    pub backoff_ms: u64,

    /// Maximum backoff in milliseconds
    #[serde(default = "default_max_backoff_ms")]
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            backoff_ms: 100,
            max_backoff_ms: 5000,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Maximum requests per minute
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_minute: 100,
        }
    }
}

// Default value functions for serde
fn default_true() -> bool {
    true
}

fn default_failure_threshold() -> f32 {
    0.5
}

fn default_window_size() -> u32 {
    20
}

fn default_reset_timeout() -> u64 {
    60
}

fn default_max_attempts() -> u32 {
    3
}

fn default_backoff_ms() -> u64 {
    100
}

fn default_max_backoff_ms() -> u64 {
    5000
}

fn default_requests_per_minute() -> u32 {
    100
}
