//! Redis Cloud API tools

mod account;
mod fixed;
mod networking;
mod raw;
mod subscriptions;

#[allow(unused_imports)]
pub use account::*;
#[allow(unused_imports)]
pub use fixed::*;
#[allow(unused_imports)]
pub use networking::*;
#[allow(unused_imports)]
pub use raw::*;
#[allow(unused_imports)]
pub use subscriptions::*;

use std::sync::Arc;

use tower_mcp::McpRouter;

use crate::state::AppState;

/// All tool names registered by the Cloud toolset.
pub const TOOL_NAMES: &[&str] = &[
    // subscriptions
    "list_subscriptions",
    "get_subscription",
    "list_databases",
    "get_database",
    "get_backup_status",
    "get_slow_log",
    "get_database_tags",
    "get_database_certificate",
    "create_database",
    "update_database",
    "delete_database",
    "backup_database",
    "import_database",
    "delete_subscription",
    "flush_database",
    "create_subscription",
    "update_subscription",
    "get_subscription_pricing",
    "get_redis_versions",
    "get_subscription_cidr_allowlist",
    "update_subscription_cidr_allowlist",
    "get_subscription_maintenance_windows",
    "update_subscription_maintenance_windows",
    "get_active_active_regions",
    "add_active_active_region",
    "delete_active_active_regions",
    "get_available_database_versions",
    "upgrade_database_redis_version",
    "get_database_upgrade_status",
    "get_database_import_status",
    "create_database_tag",
    "update_database_tag",
    "delete_database_tag",
    "update_database_tags",
    "update_crdb_local_properties",
    // account
    "get_account",
    "get_system_logs",
    "get_session_logs",
    "get_regions",
    "get_modules",
    "list_account_users",
    "get_account_user",
    "list_acl_users",
    "get_acl_user",
    "list_acl_roles",
    "list_redis_rules",
    "create_acl_user",
    "update_acl_user",
    "delete_acl_user",
    "create_acl_role",
    "update_acl_role",
    "delete_acl_role",
    "create_redis_rule",
    "update_redis_rule",
    "delete_redis_rule",
    "generate_cost_report",
    "download_cost_report",
    "list_payment_methods",
    "list_tasks",
    "get_task",
    "list_cloud_accounts",
    "get_cloud_account",
    "create_cloud_account",
    "update_cloud_account",
    "delete_cloud_account",
    // networking
    "get_vpc_peering",
    "create_vpc_peering",
    "update_vpc_peering",
    "delete_vpc_peering",
    "get_aa_vpc_peering",
    "create_aa_vpc_peering",
    "update_aa_vpc_peering",
    "delete_aa_vpc_peering",
    "get_tgw_attachments",
    "get_tgw_invitations",
    "accept_tgw_invitation",
    "reject_tgw_invitation",
    "create_tgw_attachment",
    "update_tgw_attachment_cidrs",
    "delete_tgw_attachment",
    "get_aa_tgw_attachments",
    "get_aa_tgw_invitations",
    "accept_aa_tgw_invitation",
    "reject_aa_tgw_invitation",
    "create_aa_tgw_attachment",
    "update_aa_tgw_attachment_cidrs",
    "delete_aa_tgw_attachment",
    "get_psc_service",
    "create_psc_service",
    "delete_psc_service",
    "get_psc_endpoints",
    "create_psc_endpoint",
    "update_psc_endpoint",
    "delete_psc_endpoint",
    "get_psc_creation_script",
    "get_psc_deletion_script",
    "get_aa_psc_service",
    "create_aa_psc_service",
    "delete_aa_psc_service",
    "get_aa_psc_endpoints",
    "create_aa_psc_endpoint",
    "update_aa_psc_endpoint",
    "delete_aa_psc_endpoint",
    "get_aa_psc_creation_script",
    "get_aa_psc_deletion_script",
    "get_private_link",
    "create_private_link",
    "delete_private_link",
    "add_private_link_principals",
    "remove_private_link_principals",
    "get_private_link_endpoint_script",
    "get_aa_private_link",
    "create_aa_private_link",
    "add_aa_private_link_principals",
    "remove_aa_private_link_principals",
    "get_aa_private_link_endpoint_script",
    // fixed
    "list_fixed_subscriptions",
    "get_fixed_subscription",
    "create_fixed_subscription",
    "update_fixed_subscription",
    "delete_fixed_subscription",
    "list_fixed_plans",
    "get_fixed_plans_by_subscription",
    "get_fixed_plan",
    "get_fixed_redis_versions",
    "list_fixed_databases",
    "get_fixed_database",
    "create_fixed_database",
    "update_fixed_database",
    "delete_fixed_database",
    "get_fixed_database_backup_status",
    "backup_fixed_database",
    "get_fixed_database_import_status",
    "import_fixed_database",
    "get_fixed_database_slow_log",
    "get_fixed_database_tags",
    "create_fixed_database_tag",
    "update_fixed_database_tag",
    "delete_fixed_database_tag",
    "update_fixed_database_tags",
    "get_fixed_database_upgrade_versions",
    "get_fixed_database_upgrade_status",
    "upgrade_fixed_database_redis_version",
    // raw
    "cloud_raw_api",
];

/// Get all Cloud tool names as owned strings.
pub fn tool_names() -> Vec<String> {
    TOOL_NAMES.iter().map(|s| (*s).to_string()).collect()
}

/// Build an MCP sub-router containing all Cloud tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(subscriptions::router(state.clone()))
        .merge(account::router(state.clone()))
        .merge(networking::router(state.clone()))
        .merge(fixed::router(state.clone()))
        .merge(raw::router(state))
}
