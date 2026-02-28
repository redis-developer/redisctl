#![allow(dead_code)]

use anyhow::{Context, Result};
use comfy_table::Table;
use jpx_core::Runtime;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::sync::OnceLock;

/// Global JMESPath runtime with extended functions
static JMESPATH_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Get or initialize the JMESPath runtime with extended functions
pub fn get_jmespath_runtime() -> &'static Runtime {
    JMESPATH_RUNTIME.get_or_init(|| Runtime::builder().with_all_extensions().build())
}

/// Normalize backtick literals in JMESPath expressions.
///
/// The JMESPath specification allows "elided quotes" in backtick literals,
/// meaning `` `foo` `` is equivalent to `` `"foo"` ``. However, the Rust
/// jmespath crate requires valid JSON inside backticks.
///
/// This function converts unquoted string literals like `` `foo` `` to
/// properly quoted JSON strings like `` `"foo"` ``.
///
/// Examples:
/// - `` `foo` `` -> `` `"foo"` ``
/// - `` `true` `` -> `` `true` `` (unchanged, valid JSON boolean)
/// - `` `123` `` -> `` `123` `` (unchanged, valid JSON number)
/// - `` `"already quoted"` `` -> `` `"already quoted"` `` (unchanged)
fn normalize_backtick_literals(query: &str) -> String {
    static BACKTICK_RE: OnceLock<Regex> = OnceLock::new();
    let re = BACKTICK_RE.get_or_init(|| {
        // Match backtick-delimited content, handling escaped backticks
        Regex::new(r"`([^`\\]*(?:\\.[^`\\]*)*)`").unwrap()
    });

    re.replace_all(query, |caps: &regex::Captures| {
        let content = &caps[1];
        let trimmed = content.trim();

        // Check if it's already valid JSON
        if serde_json::from_str::<Value>(trimmed).is_ok() {
            // Already valid JSON (number, boolean, null, quoted string, array, object)
            format!("`{}`", content)
        } else {
            // Not valid JSON - treat as unquoted string literal and add quotes
            // Escape any double quotes in the content
            let escaped = trimmed.replace('\\', "\\\\").replace('"', "\\\"");
            format!("`\"{}\"`", escaped)
        }
    })
    .into_owned()
}

/// Compile a JMESPath expression using the extended runtime.
///
/// This function normalizes backtick literals to handle the JMESPath
/// specification's "elided quotes" feature before compilation.
pub fn compile_jmespath(
    query: &str,
) -> Result<jpx_core::Expression<'static>, jpx_core::JmespathError> {
    let normalized = normalize_backtick_literals(query);
    get_jmespath_runtime().compile(&normalized)
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Table,
}

impl OutputFormat {
    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json)
    }

    pub fn is_yaml(&self) -> bool {
        matches!(self, Self::Yaml)
    }
}

pub fn print_output<T: Serialize>(
    data: T,
    format: OutputFormat,
    query: Option<&str>,
) -> Result<()> {
    let mut json_value = serde_json::to_value(data)?;

    // Apply JMESPath query if provided (using extended runtime with 400+ functions)
    if let Some(query_str) = query {
        let expr = compile_jmespath(query_str)
            .with_context(|| format!("Invalid JMESPath expression: {}", query_str))?;
        json_value = expr.search(&json_value).context("JMESPath query failed")?;
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&json_value)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&json_value)?);
        }
        OutputFormat::Table => {
            print_as_table(&json_value)?;
        }
    }

    Ok(())
}

fn print_as_table(value: &Value) -> Result<()> {
    match value {
        Value::Array(arr) if !arr.is_empty() => {
            let mut table = Table::new();

            // Get headers from first object
            if let Value::Object(first) = &arr[0] {
                let headers: Vec<String> = first.keys().cloned().collect();
                table.set_header(&headers);

                // Add rows
                for item in arr {
                    if let Value::Object(obj) = item {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| format_value(obj.get(h).unwrap_or(&Value::Null)))
                            .collect();
                        table.add_row(row);
                    }
                }
            } else {
                // Simple array of values
                table.set_header(vec!["Value"]);
                for item in arr {
                    table.add_row(vec![format_value(item)]);
                }
            }

            println!("{}", table);
        }
        Value::Object(obj) => {
            let mut table = Table::new();
            table.set_header(vec!["Key", "Value"]);

            for (key, val) in obj {
                table.add_row(vec![key.clone(), format_value(val)]);
            }

            println!("{}", table);
        }
        _ => {
            println!("{}", format_value(value));
        }
    }

    Ok(())
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{} fields}}", obj.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_backtick_unquoted_string() {
        // Standard JMESPath backtick literal without quotes
        assert_eq!(
            normalize_backtick_literals(r#"[?name==`foo`]"#),
            r#"[?name==`"foo"`]"#
        );
    }

    #[test]
    fn test_normalize_backtick_already_quoted() {
        // Already properly quoted - should not double-quote
        assert_eq!(
            normalize_backtick_literals(r#"[?name==`"foo"`]"#),
            r#"[?name==`"foo"`]"#
        );
    }

    #[test]
    fn test_normalize_backtick_number() {
        // Numbers are valid JSON - should not be quoted
        assert_eq!(
            normalize_backtick_literals(r#"[?count==`123`]"#),
            r#"[?count==`123`]"#
        );
    }

    #[test]
    fn test_normalize_backtick_boolean() {
        // Booleans are valid JSON - should not be quoted
        assert_eq!(
            normalize_backtick_literals(r#"[?enabled==`true`]"#),
            r#"[?enabled==`true`]"#
        );
        assert_eq!(
            normalize_backtick_literals(r#"[?enabled==`false`]"#),
            r#"[?enabled==`false`]"#
        );
    }

    #[test]
    fn test_normalize_backtick_null() {
        // null is valid JSON - should not be quoted
        assert_eq!(
            normalize_backtick_literals(r#"[?value==`null`]"#),
            r#"[?value==`null`]"#
        );
    }

    #[test]
    fn test_normalize_backtick_array() {
        // Arrays are valid JSON - should not be modified
        assert_eq!(
            normalize_backtick_literals(r#"`[1, 2, 3]`"#),
            r#"`[1, 2, 3]`"#
        );
    }

    #[test]
    fn test_normalize_backtick_object() {
        // Objects are valid JSON - should not be modified
        assert_eq!(
            normalize_backtick_literals(r#"`{"key": "value"}`"#),
            r#"`{"key": "value"}`"#
        );
    }

    #[test]
    fn test_normalize_multiple_backticks() {
        // Multiple backtick literals in one expression
        assert_eq!(
            normalize_backtick_literals(r#"[?name==`foo` && type==`bar`]"#),
            r#"[?name==`"foo"` && type==`"bar"`]"#
        );
    }

    #[test]
    fn test_jmespath_backtick_literal_compiles() {
        // The original failing case should now work
        let query = r#"[?module_name==`jmespath`]"#;
        let result = compile_jmespath(query);
        assert!(
            result.is_ok(),
            "Backtick literals should be supported: {:?}",
            result
        );
    }

    #[test]
    fn test_jmespath_complex_filter() {
        // Complex filter expression from the bug report
        let query = r#"[?module_name==`jmespath`].uid | [0]"#;
        let result = compile_jmespath(query);
        assert!(
            result.is_ok(),
            "Complex filter with backtick should work: {:?}",
            result
        );
    }

    #[test]
    fn test_jmespath_double_quote_literal() {
        // Double quotes work as field references, not literals
        let query = r#"[?module_name=="jmespath"]"#;
        let result = compile_jmespath(query);
        // This compiles but semantically compares field to field, not field to literal
        assert!(result.is_ok());
    }

    #[test]
    fn test_jmespath_single_quote_literal() {
        // Single quotes are raw string literals in JMESPath
        let query = "[?module_name=='jmespath']";
        let result = compile_jmespath(query);
        assert!(result.is_ok());
    }
}
