//! Direct Redis database tools

mod keys;
mod server;
mod structures;

#[allow(unused_imports)]
pub use keys::*;
#[allow(unused_imports)]
pub use server::*;
#[allow(unused_imports)]
pub use structures::*;

use std::sync::{Arc, LazyLock};

use tower_mcp::McpRouter;

use crate::state::AppState;

static INSTRUCTIONS: LazyLock<String> = LazyLock::new(|| {
    [
        server::INSTRUCTIONS,
        keys::INSTRUCTIONS,
        structures::INSTRUCTIONS,
    ]
    .concat()
});

/// Instructions text describing all Redis database tools
pub fn instructions() -> &'static str {
    &INSTRUCTIONS
}

/// Helper to format Redis values for display
pub(crate) fn format_value(v: &redis::Value) -> String {
    match v {
        redis::Value::Nil => "(nil)".to_string(),
        redis::Value::Int(i) => i.to_string(),
        redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
        redis::Value::SimpleString(s) => s.clone(),
        redis::Value::Array(arr) => format!(
            "[{}]",
            arr.iter().map(format_value).collect::<Vec<_>>().join(", ")
        ),
        _ => format!("{:?}", v),
    }
}

/// Build an MCP sub-router containing all Redis database tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        .merge(server::router(state.clone()))
        .merge(keys::router(state.clone()))
        .merge(structures::router(state))
}
