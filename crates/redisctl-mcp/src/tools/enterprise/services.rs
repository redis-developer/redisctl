//! Cluster service management tools for Redis Enterprise

use serde_json::Value;
use tower_mcp::{CallToolResult, ResultExt};

use crate::tools::macros::{enterprise_tool, mcp_module};
use crate::tools::wrap_list;

mcp_module! {
    list_services => "list_enterprise_services",
    get_service => "get_enterprise_service",
    get_service_status => "get_enterprise_service_status",
    update_service => "update_enterprise_service",
    start_service => "start_enterprise_service",
    stop_service => "stop_enterprise_service",
    restart_service => "restart_enterprise_service",
}

enterprise_tool!(read_only, list_services, "list_enterprise_services",
    "List all cluster services.",
    {} => |client, _input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let services = handler
            .list()
            .await
            .tool_context("Failed to list services")?;

        wrap_list("services", &services)
    }
);

enterprise_tool!(read_only, get_service, "get_enterprise_service",
    "Get service details by ID.",
    {
        /// Service ID (e.g., "cm_server", "mdns_server", "stats_archiver")
        pub service_id: String,
    } => |client, input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let service = handler
            .get(&input.service_id)
            .await
            .tool_context("Failed to get service")?;

        CallToolResult::from_serialize(&service)
    }
);

enterprise_tool!(read_only, get_service_status, "get_enterprise_service_status",
    "Get service status including per-node status.",
    {
        /// Service ID
        pub service_id: String,
    } => |client, input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let status = handler
            .status(&input.service_id)
            .await
            .tool_context("Failed to get service status")?;

        CallToolResult::from_serialize(&status)
    }
);

enterprise_tool!(write, update_service, "update_enterprise_service",
    "Update a service's configuration. Pass fields as JSON.",
    {
        /// Service ID to update
        pub service_id: String,
        /// Updated service configuration as JSON (e.g., enabled, config, node_uids)
        pub config: Value,
    } => |client, input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let request = serde_json::from_value(input.config)
            .map_err(|e| tower_mcp::Error::tool(format!("Invalid service config: {}", e)))?;
        let result = handler
            .update(&input.service_id, request)
            .await
            .tool_context("Failed to update service")?;

        CallToolResult::from_serialize(&result)
    }
);

enterprise_tool!(write, start_service, "start_enterprise_service",
    "Start a stopped service.",
    {
        /// Service ID
        pub service_id: String,
    } => |client, input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let result = handler
            .start(&input.service_id)
            .await
            .tool_context("Failed to start service")?;

        CallToolResult::from_serialize(&result)
    }
);

enterprise_tool!(write, stop_service, "stop_enterprise_service",
    "Stop a running service.",
    {
        /// Service ID
        pub service_id: String,
    } => |client, input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let result = handler
            .stop(&input.service_id)
            .await
            .tool_context("Failed to stop service")?;

        CallToolResult::from_serialize(&result)
    }
);

enterprise_tool!(write, restart_service, "restart_enterprise_service",
    "Restart a service (stop then start).",
    {
        /// Service ID
        pub service_id: String,
    } => |client, input| {
        let handler = redis_enterprise::services::ServicesHandler::new(client);
        let result = handler
            .restart(&input.service_id)
            .await
            .tool_context("Failed to restart service")?;

        CallToolResult::from_serialize(&result)
    }
);
