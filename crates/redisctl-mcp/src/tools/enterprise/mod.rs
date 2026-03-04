//! Redis Enterprise API tools

mod cluster;
mod databases;
mod observability;
mod proxy;
mod raw;
mod rbac;
mod services;

#[allow(unused_imports)]
pub use cluster::*;
#[allow(unused_imports)]
pub use databases::*;
#[allow(unused_imports)]
pub use observability::*;
#[allow(unused_imports)]
pub use proxy::*;
#[allow(unused_imports)]
pub use raw::*;
#[allow(unused_imports)]
pub use rbac::*;
#[allow(unused_imports)]
pub use services::*;

use std::sync::Arc;

use tower_mcp::McpRouter;

use super::SubModule;
use crate::state::AppState;

/// Sub-modules within the Enterprise toolset, each with its own tool names and router.
pub const SUB_MODULES: &[SubModule] = &[
    SubModule {
        name: "cluster",
        tool_names: cluster::TOOL_NAMES,
    },
    SubModule {
        name: "databases",
        tool_names: databases::TOOL_NAMES,
    },
    SubModule {
        name: "rbac",
        tool_names: rbac::TOOL_NAMES,
    },
    SubModule {
        name: "observability",
        tool_names: observability::TOOL_NAMES,
    },
    SubModule {
        name: "proxy",
        tool_names: proxy::TOOL_NAMES,
    },
    SubModule {
        name: "services",
        tool_names: services::TOOL_NAMES,
    },
    SubModule {
        name: "raw",
        tool_names: raw::TOOL_NAMES,
    },
];

/// Get all Enterprise tool names as owned strings.
pub fn tool_names() -> Vec<String> {
    SUB_MODULES
        .iter()
        .flat_map(|sm| sm.tool_names.iter().map(|s| (*s).to_string()))
        .collect()
}

/// Get tool names for a specific sub-module by name.
pub fn sub_tool_names(name: &str) -> Option<&'static [&'static str]> {
    SUB_MODULES
        .iter()
        .find(|sm| sm.name == name)
        .map(|sm| sm.tool_names)
}

/// Build an MCP sub-router for a specific sub-module by name.
pub fn sub_router(name: &str, state: Arc<AppState>) -> Option<McpRouter> {
    match name {
        "cluster" => Some(cluster::router(state)),
        "databases" => Some(databases::router(state)),
        "rbac" => Some(rbac::router(state)),
        "observability" => Some(observability::router(state)),
        "proxy" => Some(proxy::router(state)),
        "services" => Some(services::router(state)),
        "raw" => Some(raw::router(state)),
        _ => None,
    }
}

/// Build an MCP sub-router containing all Enterprise tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(cluster::router(state.clone()))
        .merge(databases::router(state.clone()))
        .merge(rbac::router(state.clone()))
        .merge(observability::router(state.clone()))
        .merge(proxy::router(state.clone()))
        .merge(services::router(state.clone()))
        .merge(raw::router(state))
}
