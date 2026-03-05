//! Cluster, license, node, maintenance, and certificate tools

use std::sync::Arc;

use redis_enterprise::cluster::ClusterHandler;
use redis_enterprise::license::{LicenseHandler, LicenseUpdateRequest};
use redis_enterprise::nodes::NodeHandler;
use redis_enterprise::stats::{StatsHandler, StatsQuery};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;

/// Input for getting cluster info (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cluster tool
pub fn get_cluster(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cluster")
        .description("Get cluster information including name, version, and configuration")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetClusterInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let cluster = handler
                    .info()
                    .await
                    .tool_context("Failed to get cluster info")?;

                CallToolResult::from_serialize(&cluster)
            },
        )
        .build()
}

// ============================================================================
// License tools
// ============================================================================

/// Input for getting license info
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLicenseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_license tool
pub fn get_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_license")
        .description("Get license information including type, expiration, and enabled features")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetLicenseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let license = handler.get().await.tool_context("Failed to get license")?;

                CallToolResult::from_serialize(&license)
            },
        )
        .build()
}

/// Input for getting license usage
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLicenseUsageInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_license_usage tool
pub fn get_license_usage(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_license_usage")
        .description(
            "Get license utilization statistics including shards, nodes, and RAM usage against limits",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetLicenseUsageInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let usage = handler
                    .usage()
                    .await
                    .tool_context("Failed to get license usage")?;

                CallToolResult::from_serialize(&usage)
            },
        )
        .build()
}

// ============================================================================
// License Write Operations
// ============================================================================

/// Input for updating license
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateLicenseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// The license key string to install
    pub license_key: String,
}

/// Build the update_license tool
pub fn update_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_license")
        .description("Apply a new license key to the cluster.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateLicenseInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let request = LicenseUpdateRequest {
                    license: input.license_key,
                };
                let license = handler
                    .update(request)
                    .await
                    .tool_context("Failed to update license")?;

                CallToolResult::from_serialize(&license)
            },
        )
        .build()
}

/// Input for validating license
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateLicenseInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// The license key string to validate
    pub license_key: String,
}

/// Build the validate_license tool
pub fn validate_license(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("validate_enterprise_license")
        .description("Validate a license key without applying it (dry-run).")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ValidateLicenseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let license = handler
                    .validate(&input.license_key)
                    .await
                    .tool_context("License validation failed")?;

                CallToolResult::from_serialize(&license)
            },
        )
        .build()
}

// ============================================================================
// Cluster Configuration Operations
// ============================================================================

/// Input for updating cluster configuration
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateClusterInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// JSON object with cluster settings to update (e.g., {"name": "my-cluster", "email_alerts": true})
    pub updates: Value,
}

/// Build the update_cluster tool
pub fn update_cluster(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_cluster")
        .description("Update cluster configuration settings. Pass fields to update as JSON.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateClusterInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let result = handler
                    .update(input.updates)
                    .await
                    .tool_context("Failed to update cluster")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting cluster policy (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterPolicyInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cluster_policy tool
pub fn get_cluster_policy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_cluster_policy")
        .description(
            "Get cluster policy settings including default shards placement, \
             rack awareness, and default Redis version.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetClusterPolicyInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let policy = handler
                    .policy()
                    .await
                    .tool_context("Failed to get cluster policy")?;

                CallToolResult::from_serialize(&policy)
            },
        )
        .build()
}

/// Input for updating cluster policy
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateClusterPolicyInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// JSON object with policy settings to update
    /// (e.g., {"default_shards_placement": "sparse", "rack_aware": true, "default_provisioned_redis_version": "7.2"})
    pub policy: Value,
}

/// Build the update_cluster_policy tool
pub fn update_cluster_policy(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_cluster_policy")
        .description(
            "Update cluster policy settings. Pass fields to update as JSON.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateClusterPolicyInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let result = handler
                    .policy_update(input.policy)
                    .await
                    .tool_context("Failed to update cluster policy")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Maintenance Mode Operations
// ============================================================================

/// Input for enabling maintenance mode (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnableMaintenanceModeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the enable_maintenance_mode tool
pub fn enable_maintenance_mode(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("enable_enterprise_maintenance_mode")
        .description(
            "Enable maintenance mode. Configuration changes will be blocked \
             until maintenance mode is disabled.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<EnableMaintenanceModeInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                // Enable maintenance mode by setting block_cluster_changes to true
                let result = handler
                    .update(serde_json::json!({"block_cluster_changes": true}))
                    .await
                    .tool_context("Failed to enable maintenance mode")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Maintenance mode enabled",
                    "result": result
                }))
            },
        )
        .build()
}

/// Input for disabling maintenance mode (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DisableMaintenanceModeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the disable_maintenance_mode tool
pub fn disable_maintenance_mode(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("disable_enterprise_maintenance_mode")
        .description("Disable maintenance mode and re-enable configuration changes.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DisableMaintenanceModeInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                // Disable maintenance mode by setting block_cluster_changes to false
                let result = handler
                    .update(serde_json::json!({"block_cluster_changes": false}))
                    .await
                    .tool_context("Failed to disable maintenance mode")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Maintenance mode disabled",
                    "result": result
                }))
            },
        )
        .build()
}

// ============================================================================
// Certificate Operations
// ============================================================================

/// Input for getting cluster certificates (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterCertificatesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_cluster_certificates tool
pub fn get_cluster_certificates(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_cluster_certificates")
        .description("Get all configured certificates (proxy, syncer, API).")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetClusterCertificatesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let certificates = handler
                    .certificates()
                    .await
                    .tool_context("Failed to get certificates")?;

                CallToolResult::from_serialize(&certificates)
            },
        )
        .build()
}

/// Input for rotating cluster certificates (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RotateClusterCertificatesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the rotate_cluster_certificates tool
pub fn rotate_cluster_certificates(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("rotate_enterprise_cluster_certificates")
        .description("Rotate all certificates, generating new ones to replace existing.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RotateClusterCertificatesInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let result = handler
                    .certificates_rotate()
                    .await
                    .tool_context("Failed to rotate certificates")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Certificate rotation initiated",
                    "result": result
                }))
            },
        )
        .build()
}

/// Input for updating cluster certificates
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateClusterCertificatesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Certificate name (e.g., "proxy", "syncer", "api")
    pub name: String,
    /// PEM-encoded certificate content
    pub certificate: String,
    /// PEM-encoded private key content
    pub key: String,
}

/// Build the update_cluster_certificates tool
pub fn update_cluster_certificates(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_cluster_certificates")
        .description(
            "Update a specific certificate. Provide the certificate name (proxy, syncer, api), \
             PEM-encoded certificate, and PEM-encoded private key.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateClusterCertificatesInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let body = serde_json::json!({
                    "name": input.name,
                    "certificate": input.certificate,
                    "key": input.key
                });
                let result = handler
                    .update_cert(body)
                    .await
                    .tool_context("Failed to update certificate")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Certificate updated successfully",
                    "name": input.name,
                    "result": result
                }))
            },
        )
        .build()
}

// ============================================================================
// Node tools
// ============================================================================

/// Input for listing nodes
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNodesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_nodes tool
pub fn list_nodes(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_nodes")
        .description("List all nodes.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListNodesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let nodes = handler.list().await.tool_context("Failed to list nodes")?;

                crate::tools::wrap_list("nodes", &nodes)
            },
        )
        .build()
}

/// Input for getting a specific node
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Node UID
    pub uid: u32,
}

/// Build the get_node tool
pub fn get_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_node")
        .description("Get detailed information about a specific node by UID.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetNodeInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let node = handler
                    .get(input.uid)
                    .await
                    .tool_context("Failed to get node")?;

                CallToolResult::from_serialize(&node)
            },
        )
        .build()
}

/// Input for getting node stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNodeStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Node UID
    pub uid: u32,
    /// Time interval for aggregation: "1sec", "10sec", "5min", "15min", "1hour", "12hour", "1week"
    #[serde(default)]
    pub interval: Option<String>,
    /// Start time for historical query (ISO 8601 format, e.g., "2024-01-15T10:00:00Z")
    #[serde(default)]
    pub start_time: Option<String>,
    /// End time for historical query (ISO 8601 format)
    #[serde(default)]
    pub end_time: Option<String>,
}

/// Build the get_node_stats tool
pub fn get_node_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_node_stats")
        .description(
            "Get statistics for a specific node. Optionally specify interval and time range \
             for historical data.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetNodeStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);

                if input.interval.is_some()
                    || input.start_time.is_some()
                    || input.end_time.is_some()
                {
                    let query = StatsQuery {
                        interval: input.interval,
                        stime: input.start_time,
                        etime: input.end_time,
                        metrics: None,
                    };
                    let stats = handler
                        .node(input.uid, Some(query))
                        .await
                        .tool_context("Failed to get node stats")?;
                    CallToolResult::from_serialize(&stats)
                } else {
                    let stats = handler
                        .node_last(input.uid)
                        .await
                        .tool_context("Failed to get node stats")?;
                    CallToolResult::from_serialize(&stats)
                }
            },
        )
        .build()
}

// ============================================================================
// Node Action Operations
// ============================================================================

/// Input for a node action (maintenance, rebalance, drain)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NodeActionInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Node UID
    pub uid: u32,
}

/// Build the enable_node_maintenance tool
pub fn enable_node_maintenance(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("enable_enterprise_node_maintenance")
        .description(
            "Enable maintenance mode on a specific node. Shards will be migrated off first.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<NodeActionInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let result = handler
                    .execute_action(input.uid, "maintenance_on")
                    .await
                    .tool_context("Failed to enable node maintenance")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Node maintenance mode enabled",
                    "node_uid": input.uid,
                    "action_uid": result.action_uid,
                    "description": result.description
                }))
            },
        )
        .build()
}

/// Build the disable_node_maintenance tool
pub fn disable_node_maintenance(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("disable_enterprise_node_maintenance")
        .description(
            "Disable maintenance mode on a specific node. The node will accept shards again.",
        )
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<NodeActionInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let result = handler
                    .execute_action(input.uid, "maintenance_off")
                    .await
                    .tool_context("Failed to disable node maintenance")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Node maintenance mode disabled",
                    "node_uid": input.uid,
                    "action_uid": result.action_uid,
                    "description": result.description
                }))
            },
        )
        .build()
}

/// Build the rebalance_node tool
pub fn rebalance_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("rebalance_enterprise_node")
        .description("Rebalance shards on a specific node for optimal distribution.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<NodeActionInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let result = handler
                    .execute_action(input.uid, "rebalance")
                    .await
                    .tool_context("Failed to rebalance node")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Node rebalance initiated",
                    "node_uid": input.uid,
                    "action_uid": result.action_uid,
                    "description": result.description
                }))
            },
        )
        .build()
}

/// Build the drain_node tool
pub fn drain_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("drain_enterprise_node")
        .description("Drain all shards from a specific node, migrating them to other nodes.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<NodeActionInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let result = handler
                    .execute_action(input.uid, "drain")
                    .await
                    .tool_context("Failed to drain node")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Node drain initiated",
                    "node_uid": input.uid,
                    "action_uid": result.action_uid,
                    "description": result.description
                }))
            },
        )
        .build()
}

// ============================================================================
// Node Update/Remove Operations
// ============================================================================

/// Input for updating a node
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateNodeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Node UID
    pub uid: u32,
    /// JSON object with node settings to update
    pub updates: Value,
}

/// Build the update_enterprise_node tool
pub fn update_enterprise_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_node")
        .description("Update a node's configuration. Pass fields to update as JSON.")
        .non_destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateNodeInput>| async move {
                // Check write permission
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let node = handler
                    .update(input.uid, input.updates)
                    .await
                    .tool_context("Failed to update node")?;

                CallToolResult::from_serialize(&node)
            },
        )
        .build()
}

/// Input for removing a node
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveNodeInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Node UID
    pub uid: u32,
}

/// Build the remove_enterprise_node tool
pub fn remove_enterprise_node(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("remove_enterprise_node")
        .description("DANGEROUS: Remove a node. All shards must be drained first.")
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RemoveNodeInput>| async move {
                // Check destructive permission
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool(
                        "Destructive operations require policy tier 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                handler
                    .remove(input.uid)
                    .await
                    .tool_context("Failed to remove node")?;

                CallToolResult::from_serialize(&serde_json::json!({
                    "message": "Node removed successfully",
                    "uid": input.uid
                }))
            },
        )
        .build()
}

// ============================================================================
// Cluster Services
// ============================================================================

/// Input for getting cluster services (no required parameters)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterServicesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_enterprise_cluster_services tool
pub fn get_enterprise_cluster_services(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_cluster_services")
        .description("Get the list of cluster services.")
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetClusterServicesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ClusterHandler::new(client);
                let services = handler
                    .services_configuration()
                    .await
                    .tool_context("Failed to get cluster services")?;

                CallToolResult::from_serialize(&services)
            },
        )
        .build()
}

/// Input for getting cluster stats
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetClusterStatsInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
    /// Time interval for aggregation: "1sec", "10sec", "5min", "15min", "1hour", "12hour", "1week"
    #[serde(default)]
    pub interval: Option<String>,
    /// Start time for historical query (ISO 8601 format, e.g., "2024-01-15T10:00:00Z")
    #[serde(default)]
    pub start_time: Option<String>,
    /// End time for historical query (ISO 8601 format)
    #[serde(default)]
    pub end_time: Option<String>,
}

/// Build the get_cluster_stats tool
pub fn get_cluster_stats(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_cluster_stats")
        .description(
            "Get cluster-level statistics. Optionally specify interval and time range \
             for historical data.",
        )
        .read_only_safe()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetClusterStatsInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = StatsHandler::new(client);

                // If any query params provided, get historical stats
                if input.interval.is_some()
                    || input.start_time.is_some()
                    || input.end_time.is_some()
                {
                    let query = StatsQuery {
                        interval: input.interval,
                        stime: input.start_time,
                        etime: input.end_time,
                        metrics: None,
                    };
                    let stats = handler
                        .cluster(Some(query))
                        .await
                        .tool_context("Failed to get cluster stats")?;
                    CallToolResult::from_serialize(&stats)
                } else {
                    let stats = handler
                        .cluster_last()
                        .await
                        .tool_context("Failed to get cluster stats")?;
                    CallToolResult::from_serialize(&stats)
                }
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
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
    "get_enterprise_cluster_services",
    "list_nodes",
    "get_node",
    "get_node_stats",
    "enable_enterprise_node_maintenance",
    "disable_enterprise_node_maintenance",
    "rebalance_enterprise_node",
    "drain_enterprise_node",
    "update_enterprise_node",
    "remove_enterprise_node",
    "get_cluster_stats",
];

/// Build an MCP sub-router containing cluster, license, and node tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Cluster
        .tool(get_cluster(state.clone()))
        .tool(get_cluster_stats(state.clone()))
        .tool(update_cluster(state.clone()))
        .tool(get_cluster_policy(state.clone()))
        .tool(update_cluster_policy(state.clone()))
        .tool(enable_maintenance_mode(state.clone()))
        .tool(disable_maintenance_mode(state.clone()))
        .tool(get_cluster_certificates(state.clone()))
        .tool(rotate_cluster_certificates(state.clone()))
        .tool(update_cluster_certificates(state.clone()))
        // License
        .tool(get_license(state.clone()))
        .tool(get_license_usage(state.clone()))
        .tool(update_license(state.clone()))
        .tool(validate_license(state.clone()))
        // Nodes
        .tool(list_nodes(state.clone()))
        .tool(get_node(state.clone()))
        .tool(get_node_stats(state.clone()))
        .tool(enable_node_maintenance(state.clone()))
        .tool(disable_node_maintenance(state.clone()))
        .tool(rebalance_node(state.clone()))
        .tool(drain_node(state.clone()))
        .tool(update_enterprise_node(state.clone()))
        .tool(remove_enterprise_node(state.clone()))
        // Cluster Services
        .tool(get_enterprise_cluster_services(state.clone()))
}
