//! MCP tools for Redis Cloud, Enterprise, and direct database operations

#[cfg(any(feature = "cloud", feature = "enterprise"))]
use serde::Serialize;
#[cfg(any(feature = "cloud", feature = "enterprise"))]
use tower_mcp::{CallToolResult, Error as McpError};

#[cfg(feature = "cloud")]
pub mod cloud;
#[cfg(feature = "enterprise")]
pub mod enterprise;
pub mod profile;
#[cfg(feature = "database")]
pub mod redis;

/// Wrap a list of items in a JSON object with a domain-specific key and count field.
///
/// The MCP protocol requires `structuredContent` to be a JSON object, not an array.
/// This helper wraps `Vec<T>` results so they serialize as `{ key: [...], "count": N }`
/// instead of a bare `[...]`.
#[cfg(any(feature = "cloud", feature = "enterprise"))]
pub fn wrap_list<T: Serialize>(key: &str, items: &[T]) -> Result<CallToolResult, McpError> {
    CallToolResult::from_serialize(&serde_json::json!({ key: items, "count": items.len() }))
}
