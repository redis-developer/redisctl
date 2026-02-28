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

/// Build an MCP sub-router containing all Enterprise tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(cluster::router(state.clone()))
        .merge(databases::router(state.clone()))
        .merge(rbac::router(state.clone()))
        .merge(observability::router(state))
}
