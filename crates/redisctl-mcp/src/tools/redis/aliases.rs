//! Command alias tools — save and replay named Redis command sequences

use tower_mcp::{CallToolResult, ResultExt};

use crate::tools::macros::{database_tool, mcp_module};

/// A command entry for defining an alias.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AliasCommand {
    /// Redis command arguments (e.g. ["SET", "key", "value"] or ["JSON.SET", "doc:1", "$", "{}"])
    pub args: Vec<String>,
}

mcp_module! {
    alias_set => "redis_alias_set",
    alias_run => "redis_alias_run",
    alias_list => "redis_alias_list",
    alias_delete => "redis_alias_delete"
}

database_tool!(write_stateful, alias_set, "redis_alias_set",
    "Save a named command alias for this session. The alias stores a sequence of Redis commands \
     that can be replayed with redis_alias_run.\n\n\
     Use this to capture a repeatable workflow (e.g. seed + query, write + verify round-trip) \
     and replay it without reconstructing the commands each time.\n\n\
     Aliases are session-scoped (in-memory only) and are lost when the MCP server restarts.",
    {
        /// Alias name (e.g. "seed-users", "health-check")
        pub name: String,
        /// Commands to store. Each command is an args array (e.g. [\"SET\", \"k\", \"v\"]).
        pub commands: Vec<AliasCommand>,
    } => |state, _conn, input| {
        if input.commands.is_empty() {
            return Err(tower_mcp::Error::tool("commands must not be empty"));
        }
        for (i, cmd) in input.commands.iter().enumerate() {
            if cmd.args.is_empty() {
                return Err(tower_mcp::Error::tool(format!(
                    "command at index {} has empty args", i
                )));
            }
        }

        let command_count = input.commands.len();
        let commands: Vec<Vec<String>> = input.commands.into_iter().map(|c| c.args).collect();
        state.set_alias(input.name.clone(), commands).await;

        Ok(CallToolResult::text(format!(
            "Alias '{}' saved with {} command(s)", input.name, command_count
        )))
    }
);

database_tool!(read_only_stateful, alias_run, "redis_alias_run",
    "Run a previously saved command alias. Executes the stored commands in order via a \
     Redis pipeline and returns per-command results.\n\n\
     Use redis_alias_list to see available aliases.",
    {
        /// Alias name to run
        pub name: String,
    } => |state, conn, input| {
        let commands = state.get_alias(&input.name).await
            .ok_or_else(|| tower_mcp::Error::tool(format!(
                "Alias '{}' not found. Use redis_alias_list to see available aliases.", input.name
            )))?;

        let mut pipe = redis::pipe();
        for cmd_args in &commands {
            let mut cmd = redis::cmd(&cmd_args[0]);
            for arg in &cmd_args[1..] {
                cmd.arg(arg);
            }
            pipe.add_command(cmd);
        }

        let results: Vec<redis::Value> = pipe
            .query_async(&mut conn)
            .await
            .tool_context(format!("Alias '{}' pipeline failed", input.name))?;

        let mut lines = Vec::with_capacity(results.len() + 2);
        for (i, (cmd_args, result)) in commands.iter().zip(results.iter()).enumerate() {
            let label = cmd_args.first().map(|s| s.as_str()).unwrap_or("?");
            let key = cmd_args.get(1).map(|s| s.as_str()).unwrap_or("");
            let result_str = super::format_value(result);
            lines.push(format!("[{:>4}] {:<12} {}  →  {}", i, label, key, result_str));
        }
        lines.push(String::new());
        lines.push(format!("Alias '{}': {} command(s) executed", input.name, results.len()));

        Ok(CallToolResult::text(lines.join("\n")))
    }
);

database_tool!(read_only_stateful, alias_list, "redis_alias_list",
    "List all saved command aliases for this session.",
    {
    } => |state, _conn, _input| {
        let entries = state.list_aliases().await;

        if entries.is_empty() {
            return Ok(CallToolResult::text(
                "No aliases saved. Use redis_alias_set to create one."
            ));
        }

        let lines: Vec<String> = entries.iter()
            .map(|(name, count)| format!("  {:<24} {} command(s)", name, count))
            .collect();

        Ok(CallToolResult::text(format!(
            "Aliases ({}):\n{}",
            entries.len(),
            lines.join("\n")
        )))
    }
);

database_tool!(write_stateful, alias_delete, "redis_alias_delete",
    "Delete a saved command alias.",
    {
        /// Alias name to delete
        pub name: String,
    } => |state, _conn, input| {
        if state.delete_alias(&input.name).await {
            Ok(CallToolResult::text(format!("Deleted alias '{}'", input.name)))
        } else {
            Ok(CallToolResult::text(format!(
                "Alias '{}' not found", input.name
            )))
        }
    }
);
