//! Redis Cloud API tools

mod account;
mod fixed;
mod networking;
mod subscriptions;

#[allow(unused_imports)]
pub use account::*;
#[allow(unused_imports)]
pub use fixed::*;
#[allow(unused_imports)]
pub use networking::*;
#[allow(unused_imports)]
pub use subscriptions::*;

use std::sync::Arc;

use tower_mcp::McpRouter;

use crate::state::AppState;

/// Build an MCP sub-router containing all Cloud tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(subscriptions::router(state.clone()))
        .merge(account::router(state.clone()))
        .merge(networking::router(state.clone()))
        .merge(fixed::router(state))
}
