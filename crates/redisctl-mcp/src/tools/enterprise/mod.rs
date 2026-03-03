//! Redis Enterprise API tools

mod cluster;
mod databases;
mod observability;
mod rbac;

#[allow(unused_imports)]
pub use cluster::*;
#[allow(unused_imports)]
pub use databases::*;
#[allow(unused_imports)]
pub use observability::*;
#[allow(unused_imports)]
pub use rbac::*;

use std::sync::Arc;

use tower_mcp::McpRouter;

use crate::state::AppState;

/// All tool names registered by the Enterprise toolset.
pub const TOOL_NAMES: &[&str] = &[
    // cluster
    "get_cluster",
    "get_license",
    "get_license_usage",
    "update_enterprise_license",
    "validate_enterprise_license",
    "update_enterprise_cluster",
    "get_enterprise_cluster_policy",
    "update_enterprise_cluster_policy",
    "enable_enterprise_maintenance_mode",
    "disable_enterprise_maintenance_mode",
    "get_enterprise_cluster_certificates",
    "rotate_enterprise_cluster_certificates",
    "update_enterprise_cluster_certificates",
    "list_nodes",
    "get_node",
    "get_node_stats",
    "enable_enterprise_node_maintenance",
    "disable_enterprise_node_maintenance",
    "rebalance_enterprise_node",
    "drain_enterprise_node",
    "get_cluster_stats",
    // databases
    "list_enterprise_databases",
    "get_enterprise_database",
    "get_database_stats",
    "get_database_endpoints",
    "list_database_alerts",
    "backup_enterprise_database",
    "import_enterprise_database",
    "create_enterprise_database",
    "update_enterprise_database",
    "delete_enterprise_database",
    "flush_enterprise_database",
    "list_enterprise_crdbs",
    "get_enterprise_crdb",
    "get_enterprise_crdb_tasks",
    // rbac
    "list_enterprise_users",
    "get_enterprise_user",
    "create_enterprise_user",
    "update_enterprise_user",
    "delete_enterprise_user",
    "list_enterprise_roles",
    "get_enterprise_role",
    "create_enterprise_role",
    "update_enterprise_role",
    "delete_enterprise_role",
    "list_enterprise_acls",
    "get_enterprise_acl",
    "create_enterprise_acl",
    "update_enterprise_acl",
    "delete_enterprise_acl",
    "get_enterprise_ldap_config",
    "update_enterprise_ldap_config",
    // observability
    "list_alerts",
    "list_logs",
    "get_all_nodes_stats",
    "get_all_databases_stats",
    "get_shard_stats",
    "get_all_shards_stats",
    "list_shards",
    "get_shard",
    "list_debug_info_tasks",
    "get_debug_info_status",
    "list_modules",
    "get_module",
];

/// Get all Enterprise tool names as owned strings.
pub fn tool_names() -> Vec<String> {
    TOOL_NAMES.iter().map(|s| (*s).to_string()).collect()
}

/// Build an MCP sub-router containing all Enterprise tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(cluster::router(state.clone()))
        .merge(databases::router(state.clone()))
        .merge(rbac::router(state.clone()))
        .merge(observability::router(state))
}
