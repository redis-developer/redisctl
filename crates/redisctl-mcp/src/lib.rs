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
pub mod prompts;
pub mod resources;
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
    fn test_app_state_default_profile_normalized() {
        // Passing "default" as profile name should be treated as None (use configured default)
        let state = AppState::new(
            CredentialSource::Profile(Some("default".to_string())),
            true,
            None,
        )
        .unwrap();

        // Verify the credential source was normalized to None
        match &state.credential_source {
            CredentialSource::Profile(None) => {} // expected
            other => panic!("Expected Profile(None), got {:?}", other),
        }
    }

    #[test]
    fn test_app_state_default_profile_case_insensitive() {
        // "DEFAULT" should also be normalized to None
        let state = AppState::new(
            CredentialSource::Profile(Some("DEFAULT".to_string())),
            true,
            None,
        )
        .unwrap();

        match &state.credential_source {
            CredentialSource::Profile(None) => {} // expected
            other => panic!("Expected Profile(None), got {:?}", other),
        }
    }

    #[test]
    fn test_app_state_explicit_profile_preserved() {
        // Non-"default" profile names should be preserved
        let state = AppState::new(
            CredentialSource::Profile(Some("my-profile".to_string())),
            true,
            None,
        )
        .unwrap();

        match &state.credential_source {
            CredentialSource::Profile(Some(name)) if name == "my-profile" => {} // expected
            other => panic!("Expected Profile(Some(\"my-profile\")), got {:?}", other),
        }
    }

    #[test]
    fn test_cloud_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all cloud tools build successfully
        // Subscriptions & Databases
        let _ = tools::cloud::list_subscriptions(state.clone());
        let _ = tools::cloud::get_subscription(state.clone());
        let _ = tools::cloud::list_databases(state.clone());
        let _ = tools::cloud::get_database(state.clone());
        let _ = tools::cloud::get_backup_status(state.clone());
        let _ = tools::cloud::get_slow_log(state.clone());
        let _ = tools::cloud::get_tags(state.clone());
        // Account & Configuration
        let _ = tools::cloud::get_account(state.clone());
        let _ = tools::cloud::get_regions(state.clone());
        let _ = tools::cloud::get_modules(state.clone());
        let _ = tools::cloud::list_account_users(state.clone());
        let _ = tools::cloud::list_acl_users(state.clone());
        let _ = tools::cloud::list_acl_roles(state.clone());
        let _ = tools::cloud::list_redis_rules(state.clone());
        // Logs
        let _ = tools::cloud::get_system_logs(state.clone());
        let _ = tools::cloud::get_session_logs(state.clone());
        // Tasks
        let _ = tools::cloud::list_tasks(state.clone());
        let _ = tools::cloud::get_task(state.clone());
    }

    #[test]
    fn test_enterprise_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all enterprise tools build successfully
        // Cluster
        let _ = tools::enterprise::get_cluster(state.clone());
        let _ = tools::enterprise::get_cluster_stats(state.clone());
        // License
        let _ = tools::enterprise::get_license(state.clone());
        let _ = tools::enterprise::get_license_usage(state.clone());
        // Logs
        let _ = tools::enterprise::list_logs(state.clone());
        // Databases
        let _ = tools::enterprise::list_databases(state.clone());
        let _ = tools::enterprise::get_database(state.clone());
        let _ = tools::enterprise::get_database_stats(state.clone());
        let _ = tools::enterprise::get_database_endpoints(state.clone());
        let _ = tools::enterprise::list_database_alerts(state.clone());
        // Nodes
        let _ = tools::enterprise::list_nodes(state.clone());
        let _ = tools::enterprise::get_node(state.clone());
        let _ = tools::enterprise::get_node_stats(state.clone());
        // Users & Alerts
        let _ = tools::enterprise::list_users(state.clone());
        let _ = tools::enterprise::get_user(state.clone());
        let _ = tools::enterprise::list_alerts(state.clone());
        let _ = tools::enterprise::list_shards(state.clone());
        // Aggregate Stats
        let _ = tools::enterprise::get_all_nodes_stats(state.clone());
        let _ = tools::enterprise::get_all_databases_stats(state.clone());
        let _ = tools::enterprise::get_shard_stats(state.clone());
        let _ = tools::enterprise::get_all_shards_stats(state.clone());
        // Debug Info
        let _ = tools::enterprise::list_debug_info_tasks(state.clone());
        let _ = tools::enterprise::get_debug_info_status(state.clone());
        // Modules
        let _ = tools::enterprise::list_modules(state.clone());
        let _ = tools::enterprise::get_module(state.clone());
    }

    #[test]
    fn test_redis_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all redis tools build successfully
        // Connection
        let _ = tools::redis::ping(state.clone());
        let _ = tools::redis::info(state.clone());
        let _ = tools::redis::dbsize(state.clone());
        let _ = tools::redis::client_list(state.clone());
        let _ = tools::redis::cluster_info(state.clone());
        // Keys
        let _ = tools::redis::keys(state.clone());
        let _ = tools::redis::get(state.clone());
        let _ = tools::redis::key_type(state.clone());
        let _ = tools::redis::ttl(state.clone());
        let _ = tools::redis::exists(state.clone());
        let _ = tools::redis::memory_usage(state.clone());
        // Data Structures
        let _ = tools::redis::hgetall(state.clone());
        let _ = tools::redis::lrange(state.clone());
        let _ = tools::redis::smembers(state.clone());
        let _ = tools::redis::zrange(state.clone());
    }

    #[test]
    fn test_profile_tools_build() {
        let state = Arc::new(AppState::new(CredentialSource::Profile(None), true, None).unwrap());

        // Verify all profile tools build successfully
        // Read operations
        let _ = tools::profile::list_profiles(state.clone());
        let _ = tools::profile::show_profile(state.clone());
        let _ = tools::profile::config_path(state.clone());
        let _ = tools::profile::validate_config(state.clone());
        // Write operations
        let _ = tools::profile::set_default_cloud(state.clone());
        let _ = tools::profile::set_default_enterprise(state.clone());
        let _ = tools::profile::delete_profile(state.clone());
    }

    #[test]
    fn test_profile_input_deserialization() {
        // ListProfilesInput
        let input: tools::profile::ListProfilesInput = serde_json::from_str("{}").unwrap();
        let _ = input;

        // ShowProfileInput
        let input: tools::profile::ShowProfileInput =
            serde_json::from_str(r#"{"name": "my-profile"}"#).unwrap();
        assert_eq!(input.name, "my-profile");

        // SetDefaultCloudInput
        let input: tools::profile::SetDefaultCloudInput =
            serde_json::from_str(r#"{"name": "cloud-profile"}"#).unwrap();
        assert_eq!(input.name, "cloud-profile");

        // DeleteProfileInput
        let input: tools::profile::DeleteProfileInput =
            serde_json::from_str(r#"{"name": "old-profile"}"#).unwrap();
        assert_eq!(input.name, "old-profile");
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

    #[test]
    fn test_read_only_tool_annotations() {
        use tower_mcp::{CallToolResult, ToolBuilder};

        // Build a read-only tool
        let read_tool = ToolBuilder::new("get_data")
            .description("Read data")
            .read_only()
            .idempotent()
            .handler(|_: serde_json::Value| async { Ok(CallToolResult::text("data")) })
            .build()
            .expect("valid tool");

        // Build a write tool (no read_only annotation)
        let write_tool = ToolBuilder::new("set_data")
            .description("Write data")
            .handler(|_: serde_json::Value| async { Ok(CallToolResult::text("ok")) })
            .build()
            .expect("valid tool");

        // Verify annotations are set correctly
        assert!(
            read_tool
                .annotations
                .as_ref()
                .map(|a| a.read_only_hint)
                .unwrap_or(false)
        );

        assert!(
            !write_tool
                .annotations
                .as_ref()
                .map(|a| a.read_only_hint)
                .unwrap_or(false)
        );
    }

    #[test]
    fn test_capability_filter_read_only() {
        use tower_mcp::{CallToolResult, CapabilityFilter, Tool, ToolBuilder};

        // Build test tools
        let read_tool = ToolBuilder::new("get_data")
            .description("Read data")
            .read_only()
            .handler(|_: serde_json::Value| async { Ok(CallToolResult::text("data")) })
            .build()
            .expect("valid tool");

        let write_tool = ToolBuilder::new("set_data")
            .description("Write data")
            .handler(|_: serde_json::Value| async { Ok(CallToolResult::text("ok")) })
            .build()
            .expect("valid tool");

        // Create a filter that only shows read-only tools
        let filter = CapabilityFilter::new(|_session, tool: &Tool| {
            tool.annotations
                .as_ref()
                .map(|a| a.read_only_hint)
                .unwrap_or(false)
        });

        // Create a mock session
        let session = tower_mcp::SessionState::new();

        // Verify filter behavior
        assert!(filter.is_visible(&session, &read_tool));
        assert!(!filter.is_visible(&session, &write_tool));
    }
}
