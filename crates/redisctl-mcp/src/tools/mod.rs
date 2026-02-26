//! MCP tools for Redis Cloud, Enterprise, and direct database operations

#[cfg(any(feature = "cloud", feature = "enterprise"))]
use serde::Serialize;
#[cfg(any(feature = "cloud", feature = "enterprise"))]
use tower_mcp::{CallToolResult, Error as McpError, ToolError};

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

/// Format a client creation error with structured remediation guidance for LLMs.
///
/// Inspects the error chain to identify common credential issues and provides
/// actionable MCP tool calls the LLM can use to diagnose and fix the problem.
#[cfg(any(feature = "cloud", feature = "enterprise"))]
pub fn credential_error(platform: &str, err: anyhow::Error) -> ToolError {
    let msg = format!("{:#}", err);
    let err_lower = msg.to_lowercase();

    let mut output = String::new();

    if err_lower.contains("no redisctl config available") {
        output.push_str("No profiles configured.\n\n");
        output.push_str("Suggested actions:\n");
        output.push_str(&format!(
            "- Call profile_create with profile_type='{}' to create a profile\n",
            platform
        ));
        output.push_str("- Call profile_path to check the config file location\n");
    } else if err_lower.contains("failed to resolve") && err_lower.contains("profile") {
        output.push_str(&format!("No {} profile available.\n\n", platform));
        output.push_str("Suggested actions:\n");
        output.push_str("- Call profile_list to see available profiles\n");
        output.push_str(&format!(
            "- Call profile_create with profile_type='{}' to create one\n",
            platform
        ));
        output.push_str("- Pass the 'profile' parameter to specify an existing profile by name\n");
    } else if err_lower.contains("not found") {
        output.push_str("The specified profile does not exist.\n\n");
        output.push_str("Suggested actions:\n");
        output.push_str("- Call profile_list to see available profiles\n");
        output.push_str(&format!(
            "- Call profile_create with profile_type='{}' to create a new profile\n",
            platform
        ));
    } else if err_lower.contains("no cloud credentials")
        || err_lower.contains("no enterprise credentials")
    {
        output.push_str(&format!(
            "The resolved profile does not have {} credentials.\n\n",
            platform
        ));
        output.push_str("Suggested actions:\n");
        output.push_str("- Call profile_list to find profiles of the correct type\n");
        output.push_str("- Pass the 'profile' parameter with a profile name of the correct type\n");
        output.push_str(&format!(
            "- Call profile_create with profile_type='{}' to create one\n",
            platform
        ));
    } else if err_lower.contains("credential") {
        output.push_str(
            "Credential resolution failed (keyring or environment variable lookup may have failed).\n\n",
        );
        output.push_str("Suggested actions:\n");
        output.push_str("- Call profile_show to inspect credential sources\n");
        output.push_str("- Call profile_validate with connect=true to diagnose issues\n");
    } else {
        output.push_str(&format!("Failed to initialize {} client.\n\n", platform));
        output.push_str("Suggested actions:\n");
        output.push_str("- Call profile_list to check configured profiles\n");
        output.push_str("- Call profile_validate with connect=true to test connectivity\n");
        output.push_str("- Call profile_show <name> to inspect a specific profile\n");
    }

    output.push_str(&format!("\nError details: {}", msg));

    ToolError::new(output)
}
