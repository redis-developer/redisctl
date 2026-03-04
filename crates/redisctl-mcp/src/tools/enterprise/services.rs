//! Cluster service management tools for Redis Enterprise

use std::sync::Arc;

use redis_enterprise::services::ServicesHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder};

use crate::state::AppState;
use crate::tools::wrap_list;

/// Input for listing services
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListServicesInput {
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the list_enterprise_services tool
pub fn list_services(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_enterprise_services")
        .description("List all services running on the Redis Enterprise cluster.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, ListServicesInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ListServicesInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let services = handler
                    .list()
                    .await
                    .tool_context("Failed to list services")?;

                wrap_list("services", &services)
            },
        )
        .build()
}

/// Input for getting a specific service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetServiceInput {
    /// Service ID (e.g., "cm_server", "mdns_server", "stats_archiver")
    pub service_id: String,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_enterprise_service tool
pub fn get_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_service")
        .description("Get detailed information about a specific cluster service by ID.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetServiceInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetServiceInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let service = handler
                    .get(&input.service_id)
                    .await
                    .tool_context("Failed to get service")?;

                CallToolResult::from_serialize(&service)
            },
        )
        .build()
}

/// Input for getting service status
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetServiceStatusInput {
    /// Service ID
    pub service_id: String,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_enterprise_service_status tool
pub fn get_service_status(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_enterprise_service_status")
        .description("Get the current status of a specific cluster service, including per-node status.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetServiceStatusInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetServiceStatusInput>| async move {
                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let status = handler
                    .status(&input.service_id)
                    .await
                    .tool_context("Failed to get service status")?;

                CallToolResult::from_serialize(&status)
            },
        )
        .build()
}

/// Input for updating a service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateServiceInput {
    /// Service ID to update
    pub service_id: String,
    /// Updated service configuration as JSON (e.g., enabled, config, node_uids)
    pub config: Value,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_enterprise_service tool
pub fn update_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_enterprise_service")
        .description(
            "Update a cluster service's configuration. Pass the configuration fields as JSON. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateServiceInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<UpdateServiceInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let request = serde_json::from_value(input.config)
                    .map_err(|e| McpError::tool(format!("Invalid service config: {}", e)))?;
                let result = handler
                    .update(&input.service_id, request)
                    .await
                    .tool_context("Failed to update service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for service action (start/stop/restart)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ServiceActionInput {
    /// Service ID
    pub service_id: String,
    /// Profile name for multi-cluster support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the start_enterprise_service tool
pub fn start_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("start_enterprise_service")
        .description(
            "Start a cluster service that is currently stopped. Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ServiceActionInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ServiceActionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let result = handler
                    .start(&input.service_id)
                    .await
                    .tool_context("Failed to start service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Build the stop_enterprise_service tool
pub fn stop_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("stop_enterprise_service")
        .description("Stop a running cluster service. Requires write permission.")
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ServiceActionInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ServiceActionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let result = handler
                    .stop(&input.service_id)
                    .await
                    .tool_context("Failed to stop service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Build the restart_enterprise_service tool
pub fn restart_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("restart_enterprise_service")
        .description(
            "Restart a cluster service. The service will be stopped and started. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, ServiceActionInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<ServiceActionInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations require policy tier 'read-write' or 'full'",
                    ));
                }

                let client = state
                    .enterprise_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("enterprise", e))?;

                let handler = ServicesHandler::new(client);
                let result = handler
                    .restart(&input.service_id)
                    .await
                    .tool_context("Failed to restart service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &[
    "list_enterprise_services",
    "get_enterprise_service",
    "get_enterprise_service_status",
    "update_enterprise_service",
    "start_enterprise_service",
    "stop_enterprise_service",
    "restart_enterprise_service",
];

/// Build the services sub-router
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .tool(list_services(state.clone()))
        .tool(get_service(state.clone()))
        .tool(get_service_status(state.clone()))
        .tool(update_service(state.clone()))
        .tool(start_service(state.clone()))
        .tool(stop_service(state.clone()))
        .tool(restart_service(state.clone()))
}
