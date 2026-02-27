//! Fixed/Essentials tier subscription and database tools for Redis Cloud

use std::sync::Arc;

use redis_cloud::fixed::databases::{
    DatabaseTagCreateRequest, DatabaseTagUpdateRequest, DatabaseTagsUpdateRequest,
    FixedDatabaseBackupRequest, FixedDatabaseCreateRequest, FixedDatabaseHandler,
    FixedDatabaseImportRequest, FixedDatabaseUpdateRequest, Tag,
};
use redis_cloud::fixed::subscriptions::{
    FixedSubscriptionCreateRequest, FixedSubscriptionHandler, FixedSubscriptionUpdateRequest,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

// ============================================================================
// Fixed/Essentials Subscription tools
// ============================================================================

/// Input for listing fixed subscriptions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFixedSubscriptionsInput {
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_fixed_subscriptions tool
pub fn list_fixed_subscriptions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_fixed_subscriptions")
        .description(
            "List all Redis Cloud Fixed/Essentials subscriptions in the current account.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListFixedSubscriptionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListFixedSubscriptionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let subscriptions = handler
                    .list()
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to list fixed subscriptions: {}", e)))?;

                CallToolResult::from_serialize(&subscriptions)
            },
        )
        .build()
}

/// Input for getting a specific fixed subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedSubscriptionInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_subscription tool
pub fn get_fixed_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_subscription")
        .description(
            "Get detailed information about a specific Redis Cloud Fixed/Essentials subscription.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedSubscriptionInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let subscription = handler
                    .get_by_id(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed subscription: {}", e))
                    })?;

                CallToolResult::from_serialize(&subscription)
            },
        )
        .build()
}

/// Input for creating a fixed subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFixedSubscriptionInput {
    /// New subscription name
    pub name: String,
    /// Essentials plan ID (use list_fixed_plans to find available plans)
    pub plan_id: i32,
    /// Payment method: "credit-card" or "marketplace"
    #[serde(default)]
    pub payment_method: Option<String>,
    /// Payment method ID (required when payment_method is "credit-card")
    #[serde(default)]
    pub payment_method_id: Option<i32>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_fixed_subscription tool
pub fn create_fixed_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_fixed_subscription")
        .description(
            "Create a new Redis Cloud Fixed/Essentials subscription. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateFixedSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateFixedSubscriptionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = FixedSubscriptionCreateRequest {
                    name: input.name,
                    plan_id: input.plan_id,
                    payment_method: input.payment_method,
                    payment_method_id: input.payment_method_id,
                    command_type: None,
                };

                let handler = FixedSubscriptionHandler::new(client);
                let result = handler.create(&request).await.map_err(|e| {
                    ToolError::new(format!("Failed to create fixed subscription: {}", e))
                })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating a fixed subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateFixedSubscriptionInput {
    /// Fixed subscription ID to update
    pub subscription_id: i32,
    /// Updated subscription name
    #[serde(default)]
    pub name: Option<String>,
    /// New plan ID
    #[serde(default)]
    pub plan_id: Option<i32>,
    /// Payment method: "credit-card" or "marketplace"
    #[serde(default)]
    pub payment_method: Option<String>,
    /// Payment method ID
    #[serde(default)]
    pub payment_method_id: Option<i32>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_fixed_subscription tool
pub fn update_fixed_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_fixed_subscription")
        .description(
            "Update a Redis Cloud Fixed/Essentials subscription. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateFixedSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateFixedSubscriptionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = FixedSubscriptionUpdateRequest {
                    subscription_id: None,
                    name: input.name,
                    plan_id: input.plan_id,
                    payment_method: input.payment_method,
                    payment_method_id: input.payment_method_id,
                    command_type: None,
                };

                let handler = FixedSubscriptionHandler::new(client);
                let result = handler
                    .update(input.subscription_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update fixed subscription: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a fixed subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteFixedSubscriptionInput {
    /// Fixed subscription ID to delete
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_fixed_subscription tool
pub fn delete_fixed_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_fixed_subscription")
        .description(
            "Delete a Redis Cloud Fixed/Essentials subscription. \
             All databases must be deleted first. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteFixedSubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteFixedSubscriptionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let result = handler
                    .delete_by_id(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to delete fixed subscription: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for listing fixed plans
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFixedPlansInput {
    /// Cloud provider filter (e.g., "AWS", "GCP", "Azure")
    #[serde(default)]
    pub provider: Option<String>,
    /// Filter by Redis Flex plans
    #[serde(default)]
    pub redis_flex: Option<bool>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_fixed_plans tool
pub fn list_fixed_plans(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_fixed_plans")
        .description(
            "List available Redis Cloud Fixed/Essentials plans. \
             Plans describe dataset size, cloud provider, region, and pricing.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListFixedPlansInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListFixedPlansInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let plans = handler
                    .list_plans(input.provider, input.redis_flex)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to list fixed plans: {}", e))
                    })?;

                CallToolResult::from_serialize(&plans)
            },
        )
        .build()
}

/// Input for getting fixed plans by subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedPlansBySubscriptionInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_plans_by_subscription tool
pub fn get_fixed_plans_by_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_plans_by_subscription")
        .description(
            "Get compatible Fixed/Essentials plans for a specific subscription. \
             Useful when upgrading or changing a subscription's plan.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedPlansBySubscriptionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedPlansBySubscriptionInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let plans = handler
                    .get_plans_by_subscription_id(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed plans by subscription: {}", e))
                    })?;

                CallToolResult::from_serialize(&plans)
            },
        )
        .build()
}

/// Input for getting a specific fixed plan
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedPlanInput {
    /// Plan ID
    pub plan_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_plan tool
pub fn get_fixed_plan(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_plan")
        .description(
            "Get detailed information about a specific Fixed/Essentials plan \
             including pricing, capacity, and feature support.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedPlanInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetFixedPlanInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let plan = handler
                    .get_plan_by_id(input.plan_id)
                    .await
                    .map_err(|e| ToolError::new(format!("Failed to get fixed plan: {}", e)))?;

                CallToolResult::from_serialize(&plan)
            },
        )
        .build()
}

/// Input for getting fixed Redis versions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedRedisVersionsInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_redis_versions tool
pub fn get_fixed_redis_versions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_redis_versions")
        .description(
            "Get available Redis database versions for a specific Fixed/Essentials subscription.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedRedisVersionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedRedisVersionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedSubscriptionHandler::new(client);
                let versions = handler
                    .get_redis_versions(input.subscription_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed Redis versions: {}", e))
                    })?;

                CallToolResult::from_serialize(&versions)
            },
        )
        .build()
}

// ============================================================================
// Fixed/Essentials Database tools
// ============================================================================

/// Input for listing fixed databases
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFixedDatabasesInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Number of entries to skip (for pagination)
    #[serde(default)]
    pub offset: Option<i32>,
    /// Maximum number of entries to return
    #[serde(default)]
    pub limit: Option<i32>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_fixed_databases tool
pub fn list_fixed_databases(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_fixed_databases")
        .description(
            "List all databases in a Redis Cloud Fixed/Essentials subscription.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, ListFixedDatabasesInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ListFixedDatabasesInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let databases = handler
                    .list(input.subscription_id, input.offset, input.limit)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to list fixed databases: {}", e))
                    })?;

                CallToolResult::from_serialize(&databases)
            },
        )
        .build()
}

/// Input for getting a specific fixed database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database tool
pub fn get_fixed_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database")
        .description(
            "Get detailed information about a specific database in a \
             Redis Cloud Fixed/Essentials subscription.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let database = handler
                    .get_by_id(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed database: {}", e))
                    })?;

                CallToolResult::from_serialize(&database)
            },
        )
        .build()
}

/// Input for creating a fixed database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFixedDatabaseInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database name (max 40 chars, letters, digits, and hyphens only)
    pub name: String,
    /// Database protocol: "stack" (default) or "redis" (for Redis Flex)
    #[serde(default)]
    pub protocol: Option<String>,
    /// Total memory in GB including replication overhead (Pay-as-you-go only)
    #[serde(default)]
    pub memory_limit_in_gb: Option<f64>,
    /// Maximum dataset size in GB (Pay-as-you-go only)
    #[serde(default)]
    pub dataset_size_in_gb: Option<f64>,
    /// Support Redis OSS Cluster API (Pay-as-you-go only)
    #[serde(default)]
    pub support_oss_cluster_api: Option<bool>,
    /// Redis database version
    #[serde(default)]
    pub redis_version: Option<String>,
    /// Data persistence mode (e.g., "none", "aof-every-1-second", "snapshot-every-1-hour")
    #[serde(default)]
    pub data_persistence: Option<String>,
    /// Data eviction policy
    #[serde(default)]
    pub data_eviction_policy: Option<String>,
    /// Enable replication for high availability
    #[serde(default)]
    pub replication: Option<bool>,
    /// Enable TLS for connections
    #[serde(default)]
    pub enable_tls: Option<bool>,
    /// Database password (random generated if not set)
    #[serde(default)]
    pub password: Option<String>,
    /// List of source IP addresses or subnet masks to allow
    #[serde(default)]
    pub source_ips: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_fixed_database tool
pub fn create_fixed_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_fixed_database")
        .description(
            "Create a new database in a Redis Cloud Fixed/Essentials subscription. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateFixedDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateFixedDatabaseInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = FixedDatabaseCreateRequest {
                    subscription_id: None,
                    name: input.name,
                    protocol: input.protocol,
                    memory_limit_in_gb: input.memory_limit_in_gb,
                    dataset_size_in_gb: input.dataset_size_in_gb,
                    support_oss_cluster_api: input.support_oss_cluster_api,
                    redis_version: input.redis_version,
                    resp_version: None,
                    use_external_endpoint_for_oss_cluster_api: None,
                    enable_database_clustering: None,
                    number_of_shards: None,
                    data_persistence: input.data_persistence,
                    data_eviction_policy: input.data_eviction_policy,
                    replication: input.replication,
                    periodic_backup_path: None,
                    source_ips: input.source_ips,
                    regex_rules: None,
                    replica_of: None,
                    replica: None,
                    client_ssl_certificate: None,
                    client_tls_certificates: None,
                    enable_tls: input.enable_tls,
                    password: input.password,
                    alerts: None,
                    modules: None,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .create(input.subscription_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to create fixed database: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating a fixed database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateFixedDatabaseInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID to update
    pub database_id: i32,
    /// Updated database name
    #[serde(default)]
    pub name: Option<String>,
    /// Total memory in GB including replication overhead (Pay-as-you-go only)
    #[serde(default)]
    pub memory_limit_in_gb: Option<f64>,
    /// Data persistence mode
    #[serde(default)]
    pub data_persistence: Option<String>,
    /// Data eviction policy
    #[serde(default)]
    pub data_eviction_policy: Option<String>,
    /// Enable or disable replication
    #[serde(default)]
    pub replication: Option<bool>,
    /// Enable or disable TLS
    #[serde(default)]
    pub enable_tls: Option<bool>,
    /// Updated database password
    #[serde(default)]
    pub password: Option<String>,
    /// List of source IP addresses or subnet masks to allow
    #[serde(default)]
    pub source_ips: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_fixed_database tool
pub fn update_fixed_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_fixed_database")
        .description(
            "Update a database in a Redis Cloud Fixed/Essentials subscription. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateFixedDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateFixedDatabaseInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = FixedDatabaseUpdateRequest {
                    subscription_id: None,
                    database_id: None,
                    name: input.name,
                    memory_limit_in_gb: input.memory_limit_in_gb,
                    dataset_size_in_gb: None,
                    support_oss_cluster_api: None,
                    resp_version: None,
                    use_external_endpoint_for_oss_cluster_api: None,
                    enable_database_clustering: None,
                    number_of_shards: None,
                    data_persistence: input.data_persistence,
                    data_eviction_policy: input.data_eviction_policy,
                    replication: input.replication,
                    periodic_backup_path: None,
                    source_ips: input.source_ips,
                    replica_of: None,
                    replica: None,
                    regex_rules: None,
                    client_ssl_certificate: None,
                    client_tls_certificates: None,
                    enable_tls: input.enable_tls,
                    password: input.password,
                    enable_default_user: None,
                    alerts: None,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .update(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update fixed database: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a fixed database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteFixedDatabaseInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID to delete
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_fixed_database tool
pub fn delete_fixed_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_fixed_database")
        .description(
            "Delete a database from a Redis Cloud Fixed/Essentials subscription. \
             This is a destructive operation. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteFixedDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteFixedDatabaseInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .delete_by_id(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to delete fixed database: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Fixed/Essentials Database operations tools
// ============================================================================

/// Input for getting fixed database backup status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseBackupStatusInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database_backup_status tool
pub fn get_fixed_database_backup_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database_backup_status")
        .description("Get the latest backup status for a Fixed/Essentials database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseBackupStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseBackupStatusInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let status = handler
                    .get_backup_status(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed database backup status: {}", e))
                    })?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for backing up a fixed database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BackupFixedDatabaseInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Custom backup path (overrides the configured periodicBackupPath)
    #[serde(default)]
    pub adhoc_backup_path: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the backup_fixed_database tool
pub fn backup_fixed_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("backup_fixed_database")
        .description(
            "Trigger a manual backup of a Fixed/Essentials database. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, BackupFixedDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<BackupFixedDatabaseInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = FixedDatabaseBackupRequest {
                    subscription_id: None,
                    database_id: None,
                    adhoc_backup_path: input.adhoc_backup_path,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .backup(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to backup fixed database: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting fixed database import status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseImportStatusInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database_import_status tool
pub fn get_fixed_database_import_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database_import_status")
        .description("Get the latest import status for a Fixed/Essentials database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseImportStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseImportStatusInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let status = handler
                    .get_import_status(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed database import status: {}", e))
                    })?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for importing data into a fixed database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportFixedDatabaseInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Source type: "http", "redis", "ftp", "aws-s3", "azure-blob-storage", "google-blob-storage"
    pub source_type: String,
    /// One or more URIs to import from
    pub import_from_uri: Vec<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the import_fixed_database tool
pub fn import_fixed_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("import_fixed_database")
        .description(
            "Import data into a Fixed/Essentials database from an external source. \
             WARNING: This will overwrite existing data. Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, ImportFixedDatabaseInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<ImportFixedDatabaseInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = FixedDatabaseImportRequest {
                    subscription_id: None,
                    database_id: None,
                    source_type: input.source_type,
                    import_from_uri: input.import_from_uri,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .import(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to import fixed database: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting fixed database slow log
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseSlowLogInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database_slow_log tool
pub fn get_fixed_database_slow_log(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database_slow_log")
        .description(
            "Get slow log entries for a Fixed/Essentials database. \
             Shows slow queries for debugging performance issues.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseSlowLogInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseSlowLogInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let log = handler
                    .get_slow_log(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed database slow log: {}", e))
                    })?;

                CallToolResult::from_serialize(&log)
            },
        )
        .build()
}

// ============================================================================
// Fixed/Essentials Database tag tools
// ============================================================================

/// Input for getting fixed database tags
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseTagsInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database_tags tool
pub fn get_fixed_database_tags(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database_tags")
        .description("Get tags attached to a Fixed/Essentials database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseTagsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseTagsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let tags = handler
                    .get_tags(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to get fixed database tags: {}", e))
                    })?;

                CallToolResult::from_serialize(&tags)
            },
        )
        .build()
}

/// Input for creating a fixed database tag
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFixedDatabaseTagInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tag key
    pub key: String,
    /// Tag value
    pub value: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_fixed_database_tag tool
pub fn create_fixed_database_tag(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_fixed_database_tag")
        .description(
            "Create a tag on a Fixed/Essentials database. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, CreateFixedDatabaseTagInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateFixedDatabaseTagInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = DatabaseTagCreateRequest {
                    key: input.key,
                    value: input.value,
                    subscription_id: None,
                    database_id: None,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let tag = handler
                    .create_tag(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to create fixed database tag: {}", e))
                    })?;

                CallToolResult::from_serialize(&tag)
            },
        )
        .build()
}

/// Input for updating a fixed database tag
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateFixedDatabaseTagInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tag key to update
    pub tag_key: String,
    /// New tag value
    pub value: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_fixed_database_tag tool
pub fn update_fixed_database_tag(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_fixed_database_tag")
        .description(
            "Update a tag value on a Fixed/Essentials database. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateFixedDatabaseTagInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateFixedDatabaseTagInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = DatabaseTagUpdateRequest {
                    subscription_id: None,
                    database_id: None,
                    key: None,
                    value: input.value,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let tag = handler
                    .update_tag(
                        input.subscription_id,
                        input.database_id,
                        input.tag_key,
                        &request,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update fixed database tag: {}", e))
                    })?;

                CallToolResult::from_serialize(&tag)
            },
        )
        .build()
}

/// Input for deleting a fixed database tag
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteFixedDatabaseTagInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tag key to delete
    pub tag_key: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_fixed_database_tag tool
pub fn delete_fixed_database_tag(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_fixed_database_tag")
        .description(
            "Delete a tag from a Fixed/Essentials database. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, DeleteFixedDatabaseTagInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteFixedDatabaseTagInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .delete_tag(input.subscription_id, input.database_id, input.tag_key)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to delete fixed database tag: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for a tag key-value pair
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FixedTagInput {
    /// Tag key
    pub key: String,
    /// Tag value
    pub value: String,
}

/// Input for updating all fixed database tags
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateFixedDatabaseTagsInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Tags to set on the database (replaces all existing tags)
    pub tags: Vec<FixedTagInput>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_fixed_database_tags tool
pub fn update_fixed_database_tags(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_fixed_database_tags")
        .description(
            "Update all tags on a Fixed/Essentials database (replaces existing tags). \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpdateFixedDatabaseTagsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateFixedDatabaseTagsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let tags = input
                    .tags
                    .into_iter()
                    .map(|t| Tag {
                        key: t.key,
                        value: t.value,
                        command_type: None,
                    })
                    .collect();

                let request = DatabaseTagsUpdateRequest {
                    subscription_id: None,
                    database_id: None,
                    tags,
                    command_type: None,
                };

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .update_tags(input.subscription_id, input.database_id, &request)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!("Failed to update fixed database tags: {}", e))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Fixed/Essentials Database upgrade tools
// ============================================================================

/// Input for getting fixed database available upgrade versions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseUpgradeVersionsInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database_upgrade_versions tool
pub fn get_fixed_database_upgrade_versions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database_upgrade_versions")
        .description(
            "Get available target Redis versions that a Fixed/Essentials database can be upgraded to.",
        )
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseUpgradeVersionsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseUpgradeVersionsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let versions = handler
                    .get_available_target_versions(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to get fixed database upgrade versions: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&versions)
            },
        )
        .build()
}

/// Input for getting fixed database upgrade status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFixedDatabaseUpgradeStatusInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_fixed_database_upgrade_status tool
pub fn get_fixed_database_upgrade_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_fixed_database_upgrade_status")
        .description("Get the latest Redis version upgrade status for a Fixed/Essentials database.")
        .read_only()
        .idempotent()
        .extractor_handler_typed::<_, _, _, GetFixedDatabaseUpgradeStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetFixedDatabaseUpgradeStatusInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let status = handler
                    .get_upgrade_status(input.subscription_id, input.database_id)
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to get fixed database upgrade status: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for upgrading a fixed database Redis version
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpgradeFixedDatabaseRedisVersionInput {
    /// Fixed subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
    /// Target Redis version to upgrade to (use get_fixed_database_upgrade_versions to see available versions)
    pub target_version: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the upgrade_fixed_database_redis_version tool
pub fn upgrade_fixed_database_redis_version(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("upgrade_fixed_database_redis_version")
        .description(
            "Upgrade the Redis version of a Fixed/Essentials database. \
             Requires write permission.",
        )
        .extractor_handler_typed::<_, _, _, UpgradeFixedDatabaseRedisVersionInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpgradeFixedDatabaseRedisVersionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = FixedDatabaseHandler::new(client);
                let result = handler
                    .upgrade_redis_version(
                        input.subscription_id,
                        input.database_id,
                        &input.target_version,
                    )
                    .await
                    .map_err(|e| {
                        ToolError::new(format!(
                            "Failed to upgrade fixed database Redis version: {}",
                            e
                        ))
                    })?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Instructions & Router
// ============================================================================

pub(super) const INSTRUCTIONS: &str = r#"
### Redis Cloud - Fixed/Essentials Subscription Tools
- list_fixed_subscriptions: List all Fixed/Essentials subscriptions
- get_fixed_subscription: Get Fixed/Essentials subscription details
- list_fixed_plans: List available Fixed/Essentials plans (filter by provider, Redis Flex)
- get_fixed_plans_by_subscription: Get compatible plans for a subscription
- get_fixed_plan: Get details of a specific plan
- get_fixed_redis_versions: Get available Redis versions for a Fixed/Essentials subscription

### Redis Cloud - Fixed/Essentials Database Tools
- list_fixed_databases: List databases in a Fixed/Essentials subscription
- get_fixed_database: Get Fixed/Essentials database details
- get_fixed_database_backup_status: Get backup status for a Fixed/Essentials database
- get_fixed_database_import_status: Get import status for a Fixed/Essentials database
- get_fixed_database_slow_log: Get slow log entries for a Fixed/Essentials database
- get_fixed_database_tags: Get tags for a Fixed/Essentials database
- get_fixed_database_upgrade_versions: Get available Redis upgrade versions
- get_fixed_database_upgrade_status: Get Redis version upgrade status

### Redis Cloud - Fixed/Essentials Write Operations (require --read-only=false)
- create_fixed_subscription: Create a new Fixed/Essentials subscription
- update_fixed_subscription: Update a Fixed/Essentials subscription
- delete_fixed_subscription: Delete a Fixed/Essentials subscription
- create_fixed_database: Create a new Fixed/Essentials database
- update_fixed_database: Update a Fixed/Essentials database
- delete_fixed_database: Delete a Fixed/Essentials database
- backup_fixed_database: Trigger a manual backup
- import_fixed_database: Import data into a Fixed/Essentials database
- create_fixed_database_tag: Create a tag on a Fixed/Essentials database
- update_fixed_database_tag: Update a tag on a Fixed/Essentials database
- delete_fixed_database_tag: Delete a tag from a Fixed/Essentials database
- update_fixed_database_tags: Update all tags on a Fixed/Essentials database
- upgrade_fixed_database_redis_version: Upgrade the Redis version of a Fixed/Essentials database
"#;

/// Build an MCP sub-router containing all Fixed/Essentials tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // Fixed Subscription read tools
        .tool(list_fixed_subscriptions(state.clone()))
        .tool(get_fixed_subscription(state.clone()))
        .tool(list_fixed_plans(state.clone()))
        .tool(get_fixed_plans_by_subscription(state.clone()))
        .tool(get_fixed_plan(state.clone()))
        .tool(get_fixed_redis_versions(state.clone()))
        // Fixed Database read tools
        .tool(list_fixed_databases(state.clone()))
        .tool(get_fixed_database(state.clone()))
        .tool(get_fixed_database_backup_status(state.clone()))
        .tool(get_fixed_database_import_status(state.clone()))
        .tool(get_fixed_database_slow_log(state.clone()))
        .tool(get_fixed_database_tags(state.clone()))
        .tool(get_fixed_database_upgrade_versions(state.clone()))
        .tool(get_fixed_database_upgrade_status(state.clone()))
        // Fixed Subscription write tools
        .tool(create_fixed_subscription(state.clone()))
        .tool(update_fixed_subscription(state.clone()))
        .tool(delete_fixed_subscription(state.clone()))
        // Fixed Database write tools
        .tool(create_fixed_database(state.clone()))
        .tool(update_fixed_database(state.clone()))
        .tool(delete_fixed_database(state.clone()))
        .tool(backup_fixed_database(state.clone()))
        .tool(import_fixed_database(state.clone()))
        // Fixed Database tag write tools
        .tool(create_fixed_database_tag(state.clone()))
        .tool(update_fixed_database_tag(state.clone()))
        .tool(delete_fixed_database_tag(state.clone()))
        .tool(update_fixed_database_tags(state.clone()))
        // Fixed Database upgrade write tool
        .tool(upgrade_fixed_database_redis_version(state))
}
