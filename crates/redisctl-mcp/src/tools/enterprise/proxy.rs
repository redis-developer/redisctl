//! Proxy management tools for Redis Enterprise

use serde_json::Value;
use tower_mcp::{CallToolResult, ResultExt};

use crate::tools::macros::{enterprise_tool, mcp_module};
use crate::tools::wrap_list;

mcp_module! {
    list_proxies => "list_enterprise_proxies",
    get_proxy => "get_enterprise_proxy",
    get_proxy_stats => "get_enterprise_proxy_stats",
    update_proxy => "update_enterprise_proxy",
}

enterprise_tool!(read_only, list_proxies, "list_enterprise_proxies",
    "List all proxy instances.",
    {} => |client, _input| {
        let handler = redis_enterprise::proxies::ProxyHandler::new(client);
        let proxies = handler
            .list()
            .await
            .tool_context("Failed to list proxies")?;

        wrap_list("proxies", &proxies)
    }
);

enterprise_tool!(read_only, get_proxy, "get_enterprise_proxy",
    "Get proxy details by UID.",
    {
        /// Proxy UID
        pub uid: u32,
    } => |client, input| {
        let handler = redis_enterprise::proxies::ProxyHandler::new(client);
        let proxy = handler
            .get(input.uid)
            .await
            .tool_context("Failed to get proxy")?;

        CallToolResult::from_serialize(&proxy)
    }
);

enterprise_tool!(read_only, get_proxy_stats, "get_enterprise_proxy_stats",
    "Get statistics for a specific proxy including connection counts and throughput.",
    {
        /// Proxy UID
        pub uid: u32,
    } => |client, input| {
        let handler = redis_enterprise::proxies::ProxyHandler::new(client);
        let stats = handler
            .stats(input.uid)
            .await
            .tool_context("Failed to get proxy stats")?;

        CallToolResult::from_serialize(&stats)
    }
);

enterprise_tool!(write, update_proxy, "update_enterprise_proxy",
    "Update a proxy's configuration. Pass fields to update as JSON.",
    {
        /// Proxy UID to update
        pub uid: u32,
        /// Updated proxy configuration as JSON (e.g., max_connections, threads)
        pub updates: Value,
    } => |client, input| {
        let handler = redis_enterprise::proxies::ProxyHandler::new(client);
        let update = serde_json::from_value(input.updates)
            .map_err(|e| tower_mcp::Error::tool(format!("Invalid proxy update: {}", e)))?;
        let result = handler
            .update(input.uid, update)
            .await
            .tool_context("Failed to update proxy")?;

        CallToolResult::from_serialize(&result)
    }
);
