//! MCP tools for Redis Cloud, Enterprise, and direct database operations

use serde::Serialize;
use tower_mcp::{CallToolResult, Error as McpError};

pub mod cloud;
pub mod enterprise;
pub mod profile;
pub mod redis;

/// Wrap a list of items in a JSON object with a domain-specific key and count field.
///
/// The MCP protocol requires `structuredContent` to be a JSON object, not an array.
/// This helper wraps `Vec<T>` results so they serialize as `{ key: [...], "count": N }`
/// instead of a bare `[...]`.
pub fn wrap_list<T: Serialize>(key: &str, items: &[T]) -> Result<CallToolResult, McpError> {
    CallToolResult::from_serialize(&serde_json::json!({ key: items, "count": items.len() }))
}
