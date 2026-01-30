//! MCP (Model Context Protocol) server for Redis Cloud and Enterprise
//!
//! This crate provides an MCP server that exposes Redis Cloud and Enterprise
//! management operations as tools for AI systems.
//!
//! ## Binary Usage
//!
//! The primary way to use this crate is as a standalone binary:
//!
//! ```bash
//! # Stdio transport (for Claude Desktop, etc.)
//! redisctl-mcp --profile my-profile
//!
//! # HTTP transport with OAuth (for shared deployments)
//! redisctl-mcp --transport http --port 8080 --oauth --oauth-issuer https://accounts.google.com
//! ```
//!
//! ## Library Usage
//!
//! You can also embed the tools in your own MCP server:
//!
//! ```no_run
//! use std::sync::Arc;
//! use redisctl_mcp::{AppState, CredentialSource, tools};
//! use tower_mcp::McpRouter;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let state = Arc::new(AppState::new(
//!     CredentialSource::Profile(Some("default".to_string())),
//!     true, // read-only
//!     None, // no database URL
//! )?);
//!
//! let router = McpRouter::new()
//!     .tool(tools::cloud::list_subscriptions(state.clone()))
//!     .tool(tools::enterprise::get_cluster(state.clone()));
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod state;
pub mod tools;

pub use error::McpError;
pub use state::{AppState, CredentialSource};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_credential_source_profile() {
        let source = CredentialSource::Profile(Some("test".to_string()));
        match source {
            CredentialSource::Profile(Some(name)) => assert_eq!(name, "test"),
            _ => panic!("Expected Profile variant"),
        }
    }

    #[test]
    fn test_credential_source_oauth() {
        let source = CredentialSource::OAuth {
            issuer: Some("https://example.com".to_string()),
            audience: Some("my-api".to_string()),
        };
        match source {
            CredentialSource::OAuth { issuer, audience } => {
                assert_eq!(issuer, Some("https://example.com".to_string()));
                assert_eq!(audience, Some("my-api".to_string()));
            }
            _ => panic!("Expected OAuth variant"),
        }
    }

    #[test]
    fn test_app_state_read_only() {
        let state = AppState::new(
            CredentialSource::Profile(None),
            true, // read-only
            None,
        )
        .unwrap();

        assert!(!state.is_write_allowed());
    }

    #[test]
    fn test_app_state_write_allowed() {
        let state = AppState::new(
            CredentialSource::Profile(None),
            false, // not read-only
            None,
        )
        .unwrap();

        assert!(state.is_write_allowed());
    }

    #[test]
    fn test_app_state_database_url() {
        let state = AppState::new(
            CredentialSource::Profile(None),
            true,
            Some("redis://localhost:6379".to_string()),
        )
        .unwrap();

        assert_eq!(
            state.database_url,
            Some("redis://localhost:6379".to_string())
        );
    }

    #[test]
    fn test_cloud_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all cloud tools build successfully
        let _ = tools::cloud::list_subscriptions(state.clone());
        let _ = tools::cloud::get_subscription(state.clone());
        let _ = tools::cloud::list_databases(state.clone());
        let _ = tools::cloud::get_database(state.clone());
    }

    #[test]
    fn test_enterprise_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all enterprise tools build successfully
        let _ = tools::enterprise::get_cluster(state.clone());
        let _ = tools::enterprise::list_databases(state.clone());
        let _ = tools::enterprise::get_database(state.clone());
        let _ = tools::enterprise::list_nodes(state.clone());
    }

    #[test]
    fn test_redis_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all redis tools build successfully
        let _ = tools::redis::ping(state.clone());
        let _ = tools::redis::info(state.clone());
        let _ = tools::redis::keys(state.clone());
    }

    #[test]
    fn test_cloud_input_deserialization() {
        // ListSubscriptionsInput
        let input: tools::cloud::ListSubscriptionsInput = serde_json::from_str("{}").unwrap();
        let _ = input; // Just verify it deserializes

        // GetSubscriptionInput
        let input: tools::cloud::GetSubscriptionInput =
            serde_json::from_str(r#"{"subscription_id": 123}"#).unwrap();
        assert_eq!(input.subscription_id, 123);

        // ListDatabasesInput
        let input: tools::cloud::ListDatabasesInput =
            serde_json::from_str(r#"{"subscription_id": 456}"#).unwrap();
        assert_eq!(input.subscription_id, 456);

        // GetDatabaseInput
        let input: tools::cloud::GetDatabaseInput =
            serde_json::from_str(r#"{"subscription_id": 789, "database_id": 101}"#).unwrap();
        assert_eq!(input.subscription_id, 789);
        assert_eq!(input.database_id, 101);
    }

    #[test]
    fn test_enterprise_input_deserialization() {
        // GetClusterInput
        let input: tools::enterprise::GetClusterInput = serde_json::from_str("{}").unwrap();
        let _ = input;

        // ListDatabasesInput with filter
        let input: tools::enterprise::ListDatabasesInput =
            serde_json::from_str(r#"{"name_filter": "test"}"#).unwrap();
        assert_eq!(input.name_filter, Some("test".to_string()));

        // ListDatabasesInput without filter
        let input: tools::enterprise::ListDatabasesInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.name_filter, None);

        // GetDatabaseInput
        let input: tools::enterprise::GetDatabaseInput =
            serde_json::from_str(r#"{"uid": 42}"#).unwrap();
        assert_eq!(input.uid, 42);

        // ListNodesInput
        let input: tools::enterprise::ListNodesInput = serde_json::from_str("{}").unwrap();
        let _ = input;
    }

    #[test]
    fn test_redis_input_deserialization() {
        // PingInput with URL
        let input: tools::redis::PingInput =
            serde_json::from_str(r#"{"url": "redis://localhost:6379"}"#).unwrap();
        assert_eq!(input.url, Some("redis://localhost:6379".to_string()));

        // PingInput without URL
        let input: tools::redis::PingInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.url, None);

        // InfoInput with section
        let input: tools::redis::InfoInput =
            serde_json::from_str(r#"{"section": "memory"}"#).unwrap();
        assert_eq!(input.section, Some("memory".to_string()));

        // KeysInput with all fields
        let input: tools::redis::KeysInput =
            serde_json::from_str(r#"{"pattern": "user:*", "limit": 50}"#).unwrap();
        assert_eq!(input.pattern, "user:*");
        assert_eq!(input.limit, 50);

        // KeysInput with defaults
        let input: tools::redis::KeysInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.pattern, "*");
        assert_eq!(input.limit, 100);
    }

    #[test]
    fn test_mcp_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("test error");
        let mcp_err: McpError = anyhow_err.into();
        assert!(matches!(mcp_err, McpError::ToolExecution(_)));
        assert!(mcp_err.to_string().contains("test error"));
    }

    #[test]
    fn test_mcp_error_variants() {
        let err = McpError::Configuration("config issue".to_string());
        assert!(err.to_string().contains("config issue"));

        let err = McpError::CloudApi("cloud issue".to_string());
        assert!(err.to_string().contains("cloud issue"));

        let err = McpError::EnterpriseApi("enterprise issue".to_string());
        assert!(err.to_string().contains("enterprise issue"));

        let err = McpError::ReadOnlyMode;
        assert!(err.to_string().contains("read-only"));
    }
}
