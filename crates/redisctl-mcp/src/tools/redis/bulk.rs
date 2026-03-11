//! Bulk data loading and seeding tools (bulk_load, seed)

use std::time::Instant;

use tower_mcp::{CallToolResult, ResultExt};

use crate::serde_helpers;
use crate::tools::macros::{database_tool, mcp_module};

/// A single Redis command represented as a list of arguments.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Command {
    /// Redis command arguments (e.g. ["SET", "key", "value"] or ["ZADD", "myset", "1.0", "member"])
    pub args: Vec<String>,
}

/// A field-value pair for hash seeding.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FieldValue {
    /// Field name (supports {i} substitution)
    pub name: String,
    /// Value pattern (supports {i} substitution)
    pub value: String,
}

fn default_batch_size() -> usize {
    1000
}

/// Substitute `{i}` and `{i:0N}` patterns in a string with the given index.
fn substitute_pattern(pattern: &str, i: u64) -> String {
    let mut result = pattern.to_string();
    // Handle {i:0N} zero-padded patterns
    while let Some(start) = result.find("{i:0") {
        if let Some(end) = result[start..].find('}') {
            let width_str = &result[start + 4..start + end];
            if let Ok(width) = width_str.parse::<usize>() {
                let replacement = format!("{:0>width$}", i, width = width);
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    replacement,
                    &result[start + end + 1..]
                );
                continue;
            }
        }
        break;
    }
    result.replace("{i}", &i.to_string())
}

mcp_module! {
    bulk_load => "redis_bulk_load",
    seed => "redis_seed",
}

database_tool!(write, bulk_load, "redis_bulk_load",
    "Pipelined command execution. Accept a batch of Redis commands and execute them \
     using Redis pipelining for high throughput. Returns count of commands executed, \
     elapsed time, and throughput.",
    {
        /// List of commands to execute. Each command is an args array (e.g. [\"SET\", \"k\", \"v\"]).
        pub commands: Vec<Command>,
        /// Pipeline batch size (default: 1000). Commands are sent in batches of this size.
        #[serde(default = "default_batch_size", deserialize_with = "serde_helpers::string_or_usize::deserialize")]
        pub batch_size: usize,
    } => |conn, input| {
        if input.commands.is_empty() {
            return Ok(CallToolResult::text("No commands to execute"));
        }

        let batch_size = input.batch_size.max(1);
        let start = Instant::now();
        let mut total_ok = 0usize;

        for (batch_idx, chunk) in input.commands.chunks(batch_size).enumerate() {
            let mut pipe = redis::pipe();
            for cmd_input in chunk {
                if cmd_input.args.is_empty() {
                    continue;
                }
                let mut cmd = redis::cmd(&cmd_input.args[0]);
                for arg in &cmd_input.args[1..] {
                    cmd.arg(arg);
                }
                pipe.add_command(cmd).ignore();
            }
            pipe.query_async::<()>(&mut conn)
                .await
                .tool_context(format!("Pipeline batch {} failed", batch_idx))?;
            total_ok += chunk.len();
        }

        let elapsed = start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 {
            total_ok as f64 / elapsed.as_secs_f64()
        } else {
            total_ok as f64
        };

        Ok(CallToolResult::text(format!(
            "Bulk load complete: {} commands executed in {:.2}s ({:.0} cmd/s)",
            total_ok,
            elapsed.as_secs_f64(),
            rate
        )))
    }
);

database_tool!(write, seed, "redis_seed",
    "Declarative data generation for test/prototype data. Generates keys matching a pattern \
     using Redis pipelining for high throughput.\n\n\
     Supported data_type values: \"string\", \"hash\", \"sorted_set\", \"set\", \"list\", \"json\".\n\n\
     Pattern substitution: use {i} for the index, {i:0N} for zero-padded (e.g. {i:06} for 6 digits).\n\n\
     Examples:\n\
     - String: key_pattern=\"user:{i}\", value_pattern=\"value-{i}\", count=1000\n\
     - Hash: key_pattern=\"user:{i}\", field_values=[{name:\"name\",value:\"user-{i}\"},{name:\"score\",value:\"{i}\"}], count=1000\n\
     - Sorted set: key_pattern=\"leaderboard\", member_pattern=\"player-{i:06}\", count=10000, score_min=0, score_max=10000\n\
     - JSON: key_pattern=\"doc:{i}\", value_pattern='{\"id\":{i},\"name\":\"item-{i}\"}', count=1000",
    {
        /// Data type to generate: "string", "hash", "sorted_set", "set", "list", "json"
        pub data_type: String,
        /// Key pattern with {i} placeholder for index (e.g. "user:{i}", "shard:{i:04}:data")
        pub key_pattern: String,
        /// Number of items to generate
        #[serde(deserialize_with = "serde_helpers::string_or_u64::deserialize")]
        pub count: u64,
        /// For hash type: field-value pairs to set on each key. Supports {i} in both name and value.
        #[serde(default)]
        pub field_values: Option<Vec<FieldValue>>,
        /// For sorted_set/set/list: member pattern with {i} (e.g. "member-{i:08}")
        #[serde(default)]
        pub member_pattern: Option<String>,
        /// For string/json: value pattern with {i}
        #[serde(default)]
        pub value_pattern: Option<String>,
        /// For sorted_set: minimum score (default: 0.0)
        #[serde(default)]
        pub score_min: Option<f64>,
        /// For sorted_set: maximum score (default: count)
        #[serde(default)]
        pub score_max: Option<f64>,
        /// Optional TTL in seconds (applied to string, hash, and json types)
        #[serde(default, deserialize_with = "serde_helpers::string_or_opt_u64::deserialize")]
        pub ttl: Option<u64>,
        /// Pipeline batch size (default: 1000)
        #[serde(default = "default_batch_size", deserialize_with = "serde_helpers::string_or_usize::deserialize")]
        pub batch_size: usize,
    } => |conn, input| {
        let batch_size = input.batch_size.max(1);
        let count = input.count;
        let data_type = input.data_type.to_lowercase();
        let start = Instant::now();
        let mut total_commands = 0usize;

        // Validate data type
        match data_type.as_str() {
            "string" | "hash" | "sorted_set" | "set" | "list" | "json" => {}
            _ => {
                return Err(tower_mcp::Error::tool(format!(
                    "Invalid data_type '{}'. Valid types: string, hash, sorted_set, set, list, json",
                    input.data_type
                )));
            }
        }

        // Validate required fields per type
        match data_type.as_str() {
            "string" => {
                if input.value_pattern.is_none() {
                    return Err(tower_mcp::Error::tool(
                        "value_pattern is required for string type"
                    ));
                }
            }
            "hash" => {
                if input.field_values.as_ref().is_none_or(|f| f.is_empty()) {
                    return Err(tower_mcp::Error::tool(
                        "field_values with at least one entry is required for hash type"
                    ));
                }
            }
            "sorted_set" | "set" | "list" => {
                if input.member_pattern.is_none() {
                    return Err(tower_mcp::Error::tool(format!(
                        "member_pattern is required for {} type",
                        data_type
                    )));
                }
            }
            "json" => {
                if input.value_pattern.is_none() {
                    return Err(tower_mcp::Error::tool(
                        "value_pattern is required for json type"
                    ));
                }
            }
            _ => unreachable!(),
        }

        let score_min = input.score_min.unwrap_or(0.0);
        let score_max = input.score_max.unwrap_or(count as f64);

        // Generate commands in batches
        let indices: Vec<u64> = (0..count).collect();
        for chunk in indices.chunks(batch_size) {
            let mut pipe = redis::pipe();

            for &i in chunk {
                let key = substitute_pattern(&input.key_pattern, i);

                match data_type.as_str() {
                    "string" => {
                        let value = substitute_pattern(input.value_pattern.as_ref().unwrap(), i);
                        let mut cmd = redis::cmd("SET");
                        cmd.arg(&key).arg(&value);
                        pipe.add_command(cmd).ignore();
                        total_commands += 1;

                        if let Some(ttl) = input.ttl {
                            let mut cmd = redis::cmd("EXPIRE");
                            cmd.arg(&key).arg(ttl);
                            pipe.add_command(cmd).ignore();
                            total_commands += 1;
                        }
                    }
                    "hash" => {
                        let fields = input.field_values.as_ref().unwrap();
                        let mut cmd = redis::cmd("HSET");
                        cmd.arg(&key);
                        for fv in fields {
                            let name = substitute_pattern(&fv.name, i);
                            let value = substitute_pattern(&fv.value, i);
                            cmd.arg(&name).arg(&value);
                        }
                        pipe.add_command(cmd).ignore();
                        total_commands += 1;

                        if let Some(ttl) = input.ttl {
                            let mut cmd = redis::cmd("EXPIRE");
                            cmd.arg(&key).arg(ttl);
                            pipe.add_command(cmd).ignore();
                            total_commands += 1;
                        }
                    }
                    "sorted_set" => {
                        let member = substitute_pattern(input.member_pattern.as_ref().unwrap(), i);
                        let score = if count > 1 {
                            score_min + (score_max - score_min) * (i as f64 / (count - 1) as f64)
                        } else {
                            score_min
                        };
                        let mut cmd = redis::cmd("ZADD");
                        cmd.arg(&key).arg(score).arg(&member);
                        pipe.add_command(cmd).ignore();
                        total_commands += 1;
                    }
                    "set" => {
                        let member = substitute_pattern(input.member_pattern.as_ref().unwrap(), i);
                        let mut cmd = redis::cmd("SADD");
                        cmd.arg(&key).arg(&member);
                        pipe.add_command(cmd).ignore();
                        total_commands += 1;
                    }
                    "list" => {
                        let member = substitute_pattern(input.member_pattern.as_ref().unwrap(), i);
                        let mut cmd = redis::cmd("RPUSH");
                        cmd.arg(&key).arg(&member);
                        pipe.add_command(cmd).ignore();
                        total_commands += 1;
                    }
                    "json" => {
                        let value = substitute_pattern(input.value_pattern.as_ref().unwrap(), i);
                        let mut cmd = redis::cmd("JSON.SET");
                        cmd.arg(&key).arg("$").arg(&value);
                        pipe.add_command(cmd).ignore();
                        total_commands += 1;

                        if let Some(ttl) = input.ttl {
                            let mut cmd = redis::cmd("EXPIRE");
                            cmd.arg(&key).arg(ttl);
                            pipe.add_command(cmd).ignore();
                            total_commands += 1;
                        }
                    }
                    _ => unreachable!(),
                }
            }

            pipe.query_async::<()>(&mut conn)
                .await
                .tool_context("Seed pipeline failed")?;
        }

        let elapsed = start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 {
            total_commands as f64 / elapsed.as_secs_f64()
        } else {
            total_commands as f64
        };

        Ok(CallToolResult::text(format!(
            "Seed complete: {} {} items seeded ({} commands) in {:.2}s ({:.0} cmd/s)\n\n\
             Tip: use redis_info with section=\"memory\" to check memory impact, \
             or redis_dbsize to verify key count.",
            count,
            data_type,
            total_commands,
            elapsed.as_secs_f64(),
            rate
        )))
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_simple() {
        assert_eq!(substitute_pattern("user:{i}", 42), "user:42");
        assert_eq!(substitute_pattern("no-placeholder", 5), "no-placeholder");
    }

    #[test]
    fn test_substitute_padded() {
        assert_eq!(substitute_pattern("user-{i:06}", 42), "user-000042");
        assert_eq!(substitute_pattern("key-{i:08}", 1), "key-00000001");
        assert_eq!(
            substitute_pattern("shard-{i:02}:member-{i}", 7),
            "shard-07:member-7"
        );
    }

    #[test]
    fn test_substitute_multiple() {
        assert_eq!(substitute_pattern("{i}-{i}-{i}", 3), "3-3-3");
    }
}
