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
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

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
        .description(
            "Get Redis Enterprise cluster information including name, version, and configuration",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get cluster info: {}", e)))?;

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
        .description(
            "Get Redis Enterprise cluster license information including type, expiration date, \
             cluster name, owner, and enabled features",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetLicenseInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetLicenseInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = LicenseHandler::new(client);
                let license = handler
                    .get()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get license: {}", e)))?;

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
            "Get Redis Enterprise cluster license utilization statistics including shards, \
             nodes, and RAM usage against license limits",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetLicenseUsageInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get license usage: {}", e)))?;

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
        .description(
            "Update the Redis Enterprise cluster license with a new license key. \
             This applies a new license to the cluster. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateLicenseInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to update license: {}", e)))?;

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
        .description(
            "Validate a license key before applying it to the Redis Enterprise cluster. \
             Returns license information if valid, or an error if invalid. \
             This is a dry-run that does not modify the cluster.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ValidateLicenseInput>(
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
                    .map_err(|e| ToolError::new(format!("License validation failed: {}", e)))?;

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
        .description(
            "Update Redis Enterprise cluster configuration settings. \
             Pass a JSON object with the fields to update (e.g., name, email_alerts, rack_aware). \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateClusterInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to update cluster: {}", e)))?;

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
            "Get Redis Enterprise cluster policy settings including default shards placement, \
             rack awareness, default Redis version, and other cluster-wide defaults.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterPolicyInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get cluster policy: {}", e)))?;

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
            "Update Redis Enterprise cluster policy settings. \
             Common settings: default_shards_placement (dense/sparse), rack_aware, \
             default_provisioned_redis_version, persistent_node_removal. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateClusterPolicyInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update cluster policy: {}", e))
                    })?;

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
            "Enable maintenance mode on the Redis Enterprise cluster. \
             When enabled, cluster configuration changes are blocked, allowing safe \
             maintenance operations like upgrades. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, EnableMaintenanceModeInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to enable maintenance mode: {}", e))
                    })?;

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
        .description(
            "Disable maintenance mode on the Redis Enterprise cluster. \
             This re-enables cluster configuration changes after maintenance is complete. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DisableMaintenanceModeInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to disable maintenance mode: {}", e))
                    })?;

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
        .description(
            "Get all certificates configured on the Redis Enterprise cluster including \
             proxy certificates, syncer certificates, and API certificates.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterCertificatesInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get certificates: {}", e)))?;

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
        .description(
            "Rotate all certificates on the Redis Enterprise cluster. \
             This generates new certificates and replaces the existing ones. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, RotateClusterCertificatesInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to rotate certificates: {}", e)))?;

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
            "Update a specific certificate on the Redis Enterprise cluster. \
             Provide the certificate name (proxy, syncer, api), the PEM-encoded certificate, \
             and the PEM-encoded private key. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateClusterCertificatesInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to update certificate: {}", e)))?;

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
        .description("List all nodes in the Redis Enterprise cluster")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListNodesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListNodesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = NodeHandler::new(client);
                let nodes = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list nodes: {}", e)))?;

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
        .description(
            "Get detailed information about a specific node in the Redis Enterprise cluster",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetNodeInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to get node: {}", e)))?;

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
            "Get statistics for a specific node. By default returns the latest stats. \
             Optionally specify interval and time range for historical data.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetNodeStatsInput>(
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
                        .map_err(|e| ToolError::new(format!("Failed to get node stats: {}", e)))?;
                    CallToolResult::from_serialize(&stats)
                } else {
                    let stats = handler
                        .node_last(input.uid)
                        .await
                        .map_err(|e| ToolError::new(format!("Failed to get node stats: {}", e)))?;
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
            "Enable maintenance mode on a specific node in the Redis Enterprise cluster. \
             Shards will be migrated off the node before maintenance begins. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, NodeActionInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to enable node maintenance: {}", e))
                    })?;

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
            "Disable maintenance mode on a specific node in the Redis Enterprise cluster. \
             The node will rejoin the cluster and accept shards again. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, NodeActionInput>(
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
                    .map_err(|e| {
                        ToolError::new(format!("Failed to disable node maintenance: {}", e))
                    })?;

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
        .description(
            "Rebalance shards on a specific node in the Redis Enterprise cluster. \
             Redistributes shards across nodes for optimal performance. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, NodeActionInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to rebalance node: {}", e)))?;

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
        .description(
            "Drain all shards from a specific node in the Redis Enterprise cluster. \
             All shards will be migrated to other available nodes. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, NodeActionInput>(
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
                    .map_err(|e| ToolError::new(format!("Failed to drain node: {}", e)))?;

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
            "Get statistics for the Redis Enterprise cluster. By default returns the latest \
             stats. Optionally specify interval and time range for historical data.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetClusterStatsInput>(
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
                        .map_err(|e| {
                            ToolError::new(format!("Failed to get cluster stats: {}", e))
                        })?;
                    CallToolResult::from_serialize(&stats)
                } else {
                    let stats = handler
                        .cluster_last()
                        .await
                        .map_err(|e| {
                            ToolError::new(format!("Failed to get cluster stats: {}", e))
                        })?;
                    CallToolResult::from_serialize(&stats)
                }
            },
        )
        .build()
}

pub(super) const INSTRUCTIONS: &str = r#"
### Redis Enterprise - Cluster
- get_cluster: Get cluster information
- get_cluster_stats: Get cluster statistics
- update_enterprise_cluster: Update cluster configuration (write)
- get_enterprise_cluster_policy: Get cluster policy settings
- update_enterprise_cluster_policy: Update cluster policy (write)
- enable_enterprise_maintenance_mode: Enable maintenance mode (write)
- disable_enterprise_maintenance_mode: Disable maintenance mode (write)
- get_enterprise_cluster_certificates: Get cluster certificates
- rotate_enterprise_cluster_certificates: Rotate all certificates (write)
- update_enterprise_cluster_certificates: Update a specific certificate (write)

### Redis Enterprise - License
- get_license: Get license information (type, expiration, features)
- get_license_usage: Get license utilization (shards, nodes, RAM vs limits)
- update_enterprise_license: Update cluster license with a new key (write)
- validate_enterprise_license: Validate a license key before applying

### Redis Enterprise - Nodes
- list_nodes: List cluster nodes
- get_node: Get node details
- get_node_stats: Get node statistics
- enable_enterprise_node_maintenance: Enable maintenance on a node (write)
- disable_enterprise_node_maintenance: Disable maintenance on a node (write)
- rebalance_enterprise_node: Rebalance shards on a node (write)
- drain_enterprise_node: Drain all shards from a node (write)
"#;

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
}
