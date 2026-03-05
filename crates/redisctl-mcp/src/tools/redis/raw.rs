//! Raw Redis command passthrough tool

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{CallToolResult, Error as McpError, McpRouter, Tool, ToolBuilder};

use crate::state::AppState;

/// Commands that are unconditionally blocked.
const BLOCKED_COMMANDS: &[&str] = &[
    "FLUSHALL",
    "FLUSHDB",
    "SHUTDOWN",
    "DEBUG",
    "SLAVEOF",
    "REPLICAOF",
    "BGSAVE",
    "BGREWRITEAOF",
    "FAILOVER",
];

/// Subcommands blocked for specific command families.
/// Format: (COMMAND, SUBCOMMAND)
const BLOCKED_SUBCOMMANDS: &[(&str, &str)] = &[
    ("CONFIG", "SET"),
    ("CONFIG", "RESETSTAT"),
    ("CONFIG", "REWRITE"),
    ("ACL", "SETUSER"),
    ("ACL", "DELUSER"),
    ("ACL", "SAVE"),
    ("ACL", "LOAD"),
    ("CLUSTER", "RESET"),
    ("CLUSTER", "FAILOVER"),
    ("CLUSTER", "FLUSHSLOTS"),
    ("CLUSTER", "FORGET"),
    ("CLUSTER", "REPLICATE"),
    ("CLUSTER", "SETSLOT"),
    ("CLUSTER", "ADDSLOTS"),
    ("CLUSTER", "DELSLOTS"),
    ("MODULE", "LOAD"),
    ("MODULE", "UNLOAD"),
    ("MODULE", "LOADEX"),
];

/// Check if a command (with optional args) is blocked.
fn is_command_blocked(command: &str, args: &[String]) -> bool {
    let cmd_upper = command.to_uppercase();

    // Check fully blocked commands
    if BLOCKED_COMMANDS.iter().any(|&c| c == cmd_upper) {
        return true;
    }

    // Check blocked subcommands
    if let Some(first_arg) = args.first() {
        let sub_upper = first_arg.to_uppercase();
        if BLOCKED_SUBCOMMANDS
            .iter()
            .any(|&(c, s)| c == cmd_upper && s == sub_upper)
        {
            return true;
        }
    }

    false
}

/// Input for the redis_command tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RedisCommandInput {
    /// Redis command name (e.g., "GET", "HGETALL", "CLIENT")
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional Redis URL (overrides profile)
    #[serde(default)]
    pub url: Option<String>,
    /// Optional profile name to resolve connection from
    #[serde(default)]
    pub profile: Option<String>,
    /// If true, return what would be sent without executing the command
    #[serde(default)]
    pub dry_run: bool,
}

/// Build the redis_command tool.
pub fn redis_command(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("redis_command")
        .description(
            "DANGEROUS: Execute an arbitrary Redis command. \
             Escape hatch for commands not covered by dedicated tools. \
             Certain dangerous commands and subcommands are blocked.",
        )
        .destructive()
        .extractor_handler(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<RedisCommandInput>| async move {
                if !state.is_destructive_allowed() {
                    return Err(McpError::tool("redis_command requires full tier"));
                }

                // Check blocklist
                if is_command_blocked(&input.command, &input.args) {
                    let blocked_desc = if input.args.is_empty() {
                        input.command.to_uppercase()
                    } else {
                        format!(
                            "{} {}",
                            input.command.to_uppercase(),
                            input.args[0].to_uppercase()
                        )
                    };
                    return Err(McpError::tool(format!(
                        "command '{blocked_desc}' is blocked for safety"
                    )));
                }

                // Dry run: return preview
                if input.dry_run {
                    let preview = serde_json::json!({
                        "dry_run": true,
                        "command": input.command,
                        "args": input.args,
                        "url": input.url,
                        "profile": input.profile,
                    });
                    return CallToolResult::from_serialize(&preview);
                }

                let mut conn =
                    super::get_connection(input.url, input.profile.as_deref(), &state).await?;

                let mut cmd = redis::cmd(&input.command);
                for arg in &input.args {
                    cmd.arg(arg);
                }

                let result: redis::Value = cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| McpError::tool(format!("command failed: {e}")))?;

                Ok(CallToolResult::text(super::format_value(&result)))
            },
        )
        .build()
}

/// All tool names registered by this sub-module.
pub(super) const TOOL_NAMES: &[&str] = &["redis_command"];

/// Build a sub-router containing the raw Redis command tool.
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new().tool(redis_command(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_commands_are_blocked() {
        assert!(is_command_blocked("FLUSHALL", &[]));
        assert!(is_command_blocked("flushall", &[]));
        assert!(is_command_blocked("Shutdown", &[]));
        assert!(is_command_blocked("DEBUG", &[]));
        assert!(is_command_blocked("SLAVEOF", &[]));
        assert!(is_command_blocked("REPLICAOF", &[]));
        assert!(is_command_blocked("BGSAVE", &[]));
        assert!(is_command_blocked("BGREWRITEAOF", &[]));
        assert!(is_command_blocked("FAILOVER", &[]));
    }

    #[test]
    fn blocked_subcommands_are_blocked() {
        assert!(is_command_blocked(
            "CONFIG",
            &["SET".to_string(), "maxmemory".to_string()]
        ));
        assert!(is_command_blocked("config", &["set".to_string()]));
        assert!(is_command_blocked("ACL", &["DELUSER".to_string()]));
        assert!(is_command_blocked("CLUSTER", &["RESET".to_string()]));
        assert!(is_command_blocked("MODULE", &["LOAD".to_string()]));
        assert!(is_command_blocked("module", &["loadex".to_string()]));
    }

    #[test]
    fn allowed_commands_pass() {
        assert!(!is_command_blocked("GET", &["mykey".to_string()]));
        assert!(!is_command_blocked(
            "SET",
            &["key".to_string(), "val".to_string()]
        ));
        assert!(!is_command_blocked("INFO", &[]));
        assert!(!is_command_blocked("PING", &[]));
    }

    #[test]
    fn allowed_subcommands_pass() {
        // CONFIG GET is allowed (only SET/RESETSTAT/REWRITE blocked)
        assert!(!is_command_blocked("CONFIG", &["GET".to_string()]));
        // ACL LIST, ACL WHOAMI are allowed
        assert!(!is_command_blocked("ACL", &["LIST".to_string()]));
        assert!(!is_command_blocked("ACL", &["WHOAMI".to_string()]));
        // CLUSTER INFO, CLUSTER NODES are allowed
        assert!(!is_command_blocked("CLUSTER", &["INFO".to_string()]));
        assert!(!is_command_blocked("CLUSTER", &["NODES".to_string()]));
        // MODULE LIST is allowed
        assert!(!is_command_blocked("MODULE", &["LIST".to_string()]));
    }
}
