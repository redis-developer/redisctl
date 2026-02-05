//! Resilience patterns for API clients using Tower middleware.
//!
//! This module provides circuit breaker, retry, and rate limiting capabilities
//! for both Cloud and Enterprise API clients using tower-resilience.
//!
//! TODO: Complete implementation after tower-resilience API stabilizes

use redisctl_core::config::resilience::ResilienceConfig;

/// Wrap a Redis Cloud client with resilience patterns
///
/// TODO: Implement tower-resilience middleware once API is stable
#[allow(dead_code)]
pub fn wrap_cloud_client<S>(service: S, _config: &ResilienceConfig) -> S
where
    S: tower::Service<
            redis_cloud::tower_support::ApiRequest,
            Response = redis_cloud::tower_support::ApiResponse,
            Error = redis_cloud::CloudError,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    // TODO: Add circuit breaker, retry, and rate limiting middleware
    // For now, return the service unchanged
    service
}

/// Wrap a Redis Enterprise client with resilience patterns
///
/// TODO: Implement tower-resilience middleware once API is stable
#[allow(dead_code)]
pub fn wrap_enterprise_client<S>(service: S, _config: &ResilienceConfig) -> S
where
    S: tower::Service<
            redis_enterprise::tower_support::ApiRequest,
            Response = redis_enterprise::tower_support::ApiResponse,
            Error = redis_enterprise::RestError,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    // TODO: Add circuit breaker, retry, and rate limiting middleware
    // For now, return the service unchanged
    service
}

/// Check if resilience is disabled via CLI flags
#[allow(dead_code)]
pub fn is_resilience_disabled(
    no_resilience: bool,
    _no_circuit_breaker: bool,
    _no_retry: bool,
) -> bool {
    no_resilience
}

/// Apply CLI overrides to resilience configuration
#[allow(dead_code)]
pub fn apply_cli_overrides(
    config: &mut ResilienceConfig,
    no_resilience: bool,
    no_circuit_breaker: bool,
    no_retry: bool,
    retry_attempts: Option<u32>,
    rate_limit: Option<u32>,
) {
    // Disable everything if --no-resilience is set
    if no_resilience {
        config.circuit_breaker.enabled = false;
        config.retry.enabled = false;
        config.rate_limit.enabled = false;
        return;
    }

    // Individual disables
    if no_circuit_breaker {
        config.circuit_breaker.enabled = false;
    }
    if no_retry {
        config.retry.enabled = false;
    }

    // Overrides
    if let Some(attempts) = retry_attempts {
        config.retry.max_attempts = attempts;
        config.retry.enabled = true; // Enable if explicitly set
    }
    if let Some(rpm) = rate_limit {
        config.rate_limit.requests_per_minute = rpm;
        config.rate_limit.enabled = true; // Enable if explicitly set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ResilienceConfig::default();
        assert!(config.circuit_breaker.enabled);
        assert!(config.retry.enabled);
        assert!(!config.rate_limit.enabled);
        assert_eq!(config.retry.max_attempts, 3);
    }

    #[test]
    fn test_cli_overrides() {
        let mut config = ResilienceConfig::default();

        // Test --no-resilience
        apply_cli_overrides(&mut config, true, false, false, None, None);
        assert!(!config.circuit_breaker.enabled);
        assert!(!config.retry.enabled);
        assert!(!config.rate_limit.enabled);

        // Test individual disables
        let mut config = ResilienceConfig::default();
        apply_cli_overrides(&mut config, false, true, true, None, None);
        assert!(!config.circuit_breaker.enabled);
        assert!(!config.retry.enabled);

        // Test retry attempts override
        let mut config = ResilienceConfig::default();
        apply_cli_overrides(&mut config, false, false, false, Some(5), None);
        assert_eq!(config.retry.max_attempts, 5);
        assert!(config.retry.enabled);

        // Test rate limit override
        let mut config = ResilienceConfig::default();
        apply_cli_overrides(&mut config, false, false, false, None, Some(200));
        assert_eq!(config.rate_limit.requests_per_minute, 200);
        assert!(config.rate_limit.enabled);
    }
}
