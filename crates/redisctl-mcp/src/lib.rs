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
//! # Multiple profiles for multi-cluster support
//! redisctl-mcp --profile cluster-west --profile cluster-east --profile cluster-central
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
//!     CredentialSource::Profiles(vec!["default".to_string()]),
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
    fn test_credential_source_profiles() {
        let source = CredentialSource::Profiles(vec!["test".to_string()]);
        match source {
            CredentialSource::Profiles(profiles) => assert_eq!(profiles, vec!["test".to_string()]),
            _ => panic!("Expected Profiles variant"),
        }
    }

    #[test]
    fn test_credential_source_multiple_profiles() {
        let source = CredentialSource::Profiles(vec![
            "cluster-west".to_string(),
            "cluster-east".to_string(),
        ]);
        match source {
            CredentialSource::Profiles(profiles) => {
                assert_eq!(profiles.len(), 2);
                assert_eq!(profiles[0], "cluster-west");
                assert_eq!(profiles[1], "cluster-east");
            }
            _ => panic!("Expected Profiles variant"),
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
            CredentialSource::Profiles(vec![]),
            true, // read-only
            None,
        )
        .unwrap();

        assert!(!state.is_write_allowed());
    }

    #[test]
    fn test_app_state_write_allowed() {
        let state = AppState::new(
            CredentialSource::Profiles(vec![]),
            false, // not read-only
            None,
        )
        .unwrap();

        assert!(state.is_write_allowed());
    }

    #[test]
    fn test_app_state_database_url() {
        let state = AppState::new(
            CredentialSource::Profiles(vec![]),
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
    fn test_app_state_available_profiles() {
        let state = AppState::new(
            CredentialSource::Profiles(vec![
                "cluster-west".to_string(),
                "cluster-east".to_string(),
            ]),
            true,
            None,
        )
        .unwrap();

        let profiles = state.available_profiles();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0], "cluster-west");
        assert_eq!(profiles[1], "cluster-east");
    }

    #[test]
    fn test_cloud_tools_build() {
        let state =
            Arc::new(AppState::new(CredentialSource::Profiles(vec![]), true, None).unwrap());

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
        let _ = tools::cloud::get_account_user(state.clone());
        let _ = tools::cloud::list_acl_users(state.clone());
        let _ = tools::cloud::get_acl_user(state.clone());
        let _ = tools::cloud::list_acl_roles(state.clone());
        let _ = tools::cloud::list_redis_rules(state.clone());
        // Logs
        let _ = tools::cloud::get_system_logs(state.clone());
        let _ = tools::cloud::get_session_logs(state.clone());
        // Tasks
        let _ = tools::cloud::list_tasks(state.clone());
        let _ = tools::cloud::get_task(state.clone());
        // Write operations
        let _ = tools::cloud::create_database(state.clone());
        let _ = tools::cloud::update_database(state.clone());
        let _ = tools::cloud::delete_database(state.clone());
        let _ = tools::cloud::backup_database(state.clone());
        let _ = tools::cloud::import_database(state.clone());
        let _ = tools::cloud::delete_subscription(state.clone());
        let _ = tools::cloud::flush_database(state.clone());
        let _ = tools::cloud::create_subscription(state.clone());
    }

    #[test]
    fn test_enterprise_tools_build() {
        let state =
            Arc::new(AppState::new(CredentialSource::Profiles(vec![]), true, None).unwrap());

        // Verify all enterprise tools build successfully
        // Cluster
        let _ = tools::enterprise::get_cluster(state.clone());
        // License
        let _ = tools::enterprise::get_license(state.clone());
        let _ = tools::enterprise::get_license_usage(state.clone());
        // Logs
        let _ = tools::enterprise::list_logs(state.clone());
        // Databases
        let _ = tools::enterprise::list_databases(state.clone());
        let _ = tools::enterprise::get_database(state.clone());
        // Nodes
        let _ = tools::enterprise::list_nodes(state.clone());
        let _ = tools::enterprise::get_node(state.clone());
        // Users
        let _ = tools::enterprise::list_users(state.clone());
        let _ = tools::enterprise::get_user(state.clone());
        // Alerts
        let _ = tools::enterprise::list_alerts(state.clone());
        let _ = tools::enterprise::list_database_alerts(state.clone());
        // Stats
        let _ = tools::enterprise::get_cluster_stats(state.clone());
        let _ = tools::enterprise::get_database_stats(state.clone());
        let _ = tools::enterprise::get_node_stats(state.clone());
        let _ = tools::enterprise::get_all_nodes_stats(state.clone());
        let _ = tools::enterprise::get_all_databases_stats(state.clone());
        // Shards
        let _ = tools::enterprise::list_shards(state.clone());
        let _ = tools::enterprise::get_shard(state.clone());
        let _ = tools::enterprise::get_shard_stats(state.clone());
        let _ = tools::enterprise::get_all_shards_stats(state.clone());
        // Endpoints
        let _ = tools::enterprise::get_database_endpoints(state.clone());
        // Debug info
        let _ = tools::enterprise::list_debug_info_tasks(state.clone());
        let _ = tools::enterprise::get_debug_info_status(state.clone());
        // Modules
        let _ = tools::enterprise::list_modules(state.clone());
        let _ = tools::enterprise::get_module(state.clone());
        // Write operations
        let _ = tools::enterprise::backup_enterprise_database(state.clone());
        let _ = tools::enterprise::import_enterprise_database(state.clone());
        let _ = tools::enterprise::create_enterprise_database(state.clone());
        let _ = tools::enterprise::update_enterprise_database(state.clone());
        let _ = tools::enterprise::delete_enterprise_database(state.clone());
        let _ = tools::enterprise::flush_enterprise_database(state.clone());
    }

    #[test]
    fn test_profile_tools_build() {
        let state =
            Arc::new(AppState::new(CredentialSource::Profiles(vec![]), true, None).unwrap());

        // Verify profile tools build successfully
        let _ = tools::profile::list_profiles(state.clone());
        let _ = tools::profile::show_profile(state.clone());
        let _ = tools::profile::config_path(state.clone());
        let _ = tools::profile::validate_config(state.clone());
        let _ = tools::profile::set_default_cloud(state.clone());
        let _ = tools::profile::set_default_enterprise(state.clone());
        let _ = tools::profile::delete_profile(state.clone());
    }
}
