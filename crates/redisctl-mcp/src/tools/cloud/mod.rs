//! Redis Cloud API tools

mod account;
mod networking;
mod subscriptions;

#[allow(unused_imports)]
pub use account::*;
#[allow(unused_imports)]
pub use networking::*;
#[allow(unused_imports)]
pub use subscriptions::*;

use std::sync::{Arc, LazyLock};

use tower_mcp::McpRouter;

use crate::state::AppState;

static INSTRUCTIONS: LazyLock<String> = LazyLock::new(|| {
    [
        subscriptions::INSTRUCTIONS,
        account::INSTRUCTIONS,
        networking::INSTRUCTIONS,
    ]
    .concat()
});

/// Instructions text describing all Cloud tools
pub fn instructions() -> &'static str {
    &INSTRUCTIONS
}

/// Build an MCP sub-router containing all Cloud tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(subscriptions::router(state.clone()))
        .merge(account::router(state.clone()))
        .merge(networking::router(state))
}
