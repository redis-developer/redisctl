//! MCP Prompts for Redis management workflows
//!
//! Prompts provide pre-built templates for common Redis operations.

use std::collections::HashMap;

use tower_mcp::prompt::{Prompt, PromptBuilder};
use tower_mcp::protocol::{Content, GetPromptResult, PromptMessage, PromptRole};

/// Build a prompt for troubleshooting database issues
pub fn troubleshoot_database_prompt() -> Prompt {
    PromptBuilder::new("troubleshoot_database")
        .description("Generate a troubleshooting workflow for a Redis database")
        .required_arg("database_name", "Name or ID of the database to troubleshoot")
        .optional_arg("symptoms", "Description of the issue or symptoms observed")
        .handler(|args: HashMap<String, String>| async move {
            let db_name = args.get("database_name").cloned().unwrap_or_default();
            let symptoms = args.get("symptoms").cloned().unwrap_or_default();

            let prompt_text = if symptoms.is_empty() {
                format!(
                    r#"I need to troubleshoot a Redis database named "{}".

Please help me diagnose potential issues by:

1. First, check the database status and basic connectivity using redis_ping
2. Get database information with redis_info to check memory, connections, and replication
3. Check for slow queries using get_slow_log (if Redis Cloud) or redis_info with the slowlog section
4. Look at the key distribution with redis_dbsize and redis_keys with a sample pattern
5. Check client connections with redis_client_list

Based on the results, identify any issues and suggest remediation steps."#,
                    db_name
                )
            } else {
                format!(
                    r#"I need to troubleshoot a Redis database named "{}".

**Reported symptoms**: {}

Please help me diagnose this issue by:

1. First, check the database status and basic connectivity using redis_ping
2. Get database information with redis_info focusing on sections relevant to the symptoms
3. Check for slow queries that might be causing the issue
4. Examine memory usage and eviction policies if memory-related
5. Check replication lag if replication-related

Based on the results and the reported symptoms, identify the root cause and suggest specific remediation steps."#,
                    db_name, symptoms
                )
            };

            Ok(GetPromptResult {
                description: Some(format!("Troubleshoot database: {}", db_name)),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: Content::Text {
                        text: prompt_text,
                        annotations: None,
                    },
                }],
            })
        })
        .build()
}

/// Build a prompt for analyzing performance metrics
pub fn analyze_performance_prompt() -> Prompt {
    PromptBuilder::new("analyze_performance")
        .description("Analyze Redis performance metrics and suggest optimizations")
        .optional_arg(
            "focus",
            "Specific area to focus on (memory, latency, throughput)",
        )
        .handler(|args: HashMap<String, String>| async move {
            let focus = args.get("focus").cloned().unwrap_or_default();

            let prompt_text = if focus.is_empty() {
                r#"I need to analyze the performance of my Redis deployment.

Please help me by:

1. Get overall cluster or database statistics to understand the current state
2. Check memory usage patterns and fragmentation ratio
3. Look at operation throughput and latency metrics
4. Examine connection patterns and client distribution
5. Review any slow queries in the slow log

Based on the analysis, provide:
- A summary of current performance characteristics
- Identification of any bottlenecks or issues
- Specific recommendations for optimization
- Priority order for implementing changes"#
                    .to_string()
            } else {
                format!(
                    r#"I need to analyze the {} performance of my Redis deployment.

Please focus specifically on {} metrics by:

1. Gathering relevant statistics for {}
2. Comparing against best practices and benchmarks
3. Identifying any anomalies or issues
4. Suggesting specific optimizations for {}

Provide actionable recommendations with expected impact."#,
                    focus, focus, focus, focus
                )
            };

            Ok(GetPromptResult {
                description: Some("Analyze Redis performance".to_string()),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: Content::Text {
                        text: prompt_text,
                        annotations: None,
                    },
                }],
            })
        })
        .build()
}

/// Build a prompt for capacity planning
pub fn capacity_planning_prompt() -> Prompt {
    PromptBuilder::new("capacity_planning")
        .description("Help with Redis capacity planning and scaling decisions")
        .required_arg("current_usage", "Description of current usage patterns")
        .optional_arg("growth_rate", "Expected growth rate (e.g., '20% monthly')")
        .handler(|args: HashMap<String, String>| async move {
            let current_usage = args.get("current_usage").cloned().unwrap_or_default();
            let growth_rate = args.get("growth_rate").cloned().unwrap_or_default();

            let growth_section = if growth_rate.is_empty() {
                String::new()
            } else {
                format!("\n**Expected growth rate**: {}\n", growth_rate)
            };

            let prompt_text = format!(
                r#"I need help with capacity planning for my Redis deployment.

**Current usage**: {}{}

Please help me by:

1. First, gather current metrics:
   - Memory usage and limits
   - Key count and data size distribution
   - Operation throughput (ops/sec)
   - Connection count and patterns

2. Analyze the data to determine:
   - Current utilization percentage
   - Memory efficiency (fragmentation, overhead)
   - Headroom for growth

3. Based on the analysis, provide:
   - Projected resource needs over 3, 6, and 12 months
   - Recommended scaling strategy (vertical vs horizontal)
   - Cost optimization opportunities
   - Warning thresholds to monitor"#,
                current_usage, growth_section
            );

            Ok(GetPromptResult {
                description: Some("Redis capacity planning".to_string()),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: Content::Text {
                        text: prompt_text,
                        annotations: None,
                    },
                }],
            })
        })
        .build()
}

/// Build a prompt for migration planning
pub fn migration_planning_prompt() -> Prompt {
    PromptBuilder::new("migration_planning")
        .description("Plan a Redis migration between environments or providers")
        .required_arg("source", "Source environment description")
        .required_arg("target", "Target environment description")
        .handler(|args: HashMap<String, String>| async move {
            let source = args.get("source").cloned().unwrap_or_default();
            let target = args.get("target").cloned().unwrap_or_default();

            let prompt_text = format!(
                r#"I need to plan a Redis migration.

**Source**: {}
**Target**: {}

Please help me create a migration plan by:

1. First, analyze the source environment:
   - Get database configuration and size
   - Check data types and key patterns used
   - Identify any Redis modules in use
   - Note persistence and replication settings

2. Assess compatibility with the target:
   - Version compatibility
   - Feature parity
   - Module availability
   - Network and security requirements

3. Create a migration plan including:
   - Pre-migration checklist
   - Data migration approach (snapshot vs live sync)
   - Cutover strategy with minimal downtime
   - Rollback plan
   - Validation steps post-migration

4. Identify risks and mitigation strategies"#,
                source, target
            );

            Ok(GetPromptResult {
                description: Some("Redis migration planning".to_string()),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: Content::Text {
                        text: prompt_text,
                        annotations: None,
                    },
                }],
            })
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_troubleshoot_prompt() {
        let prompt = troubleshoot_database_prompt();
        assert_eq!(prompt.name, "troubleshoot_database");
        assert_eq!(prompt.arguments.len(), 2);
        assert!(prompt.arguments[0].required);
        assert!(!prompt.arguments[1].required);

        let mut args = HashMap::new();
        args.insert("database_name".to_string(), "my-cache".to_string());
        args.insert("symptoms".to_string(), "high latency".to_string());

        let result = prompt.get(args).await.unwrap();
        assert_eq!(result.messages.len(), 1);
        match &result.messages[0].content {
            Content::Text { text, .. } => {
                assert!(text.contains("my-cache"));
                assert!(text.contains("high latency"));
            }
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_analyze_performance_prompt() {
        let prompt = analyze_performance_prompt();
        assert_eq!(prompt.name, "analyze_performance");

        let result = prompt.get(HashMap::new()).await.unwrap();
        match &result.messages[0].content {
            Content::Text { text, .. } => {
                assert!(text.contains("performance"));
            }
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_capacity_planning_prompt() {
        let prompt = capacity_planning_prompt();
        assert_eq!(prompt.name, "capacity_planning");
        assert!(prompt.arguments[0].required);

        let mut args = HashMap::new();
        args.insert(
            "current_usage".to_string(),
            "2GB memory, 100k keys".to_string(),
        );

        let result = prompt.get(args).await.unwrap();
        match &result.messages[0].content {
            Content::Text { text, .. } => {
                assert!(text.contains("2GB memory"));
            }
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_migration_planning_prompt() {
        let prompt = migration_planning_prompt();
        assert_eq!(prompt.name, "migration_planning");
        assert_eq!(prompt.arguments.len(), 2);
        assert!(prompt.arguments.iter().all(|a| a.required));

        let mut args = HashMap::new();
        args.insert("source".to_string(), "AWS ElastiCache".to_string());
        args.insert("target".to_string(), "Redis Cloud".to_string());

        let result = prompt.get(args).await.unwrap();
        match &result.messages[0].content {
            Content::Text { text, .. } => {
                assert!(text.contains("AWS ElastiCache"));
                assert!(text.contains("Redis Cloud"));
            }
            _ => panic!("Expected text content"),
        }
    }
}
