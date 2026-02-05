//! Shared utilities for handling asynchronous Cloud operations with --wait flag support
//!
//! This module provides CLI-level async handling that wraps Layer 2's poll_task
//! with progress bar output and CLI-specific formatting.

use std::time::Duration;

use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

use crate::cli::OutputFormat;
use crate::connection::ConnectionManager;
use crate::error::{RedisCtlError, Result as CliResult};
use crate::output::print_output;

/// Helper to print non-table output
fn print_json_or_yaml(data: Value, output_format: OutputFormat) -> CliResult<()> {
    match output_format {
        OutputFormat::Json => print_output(data, crate::output::OutputFormat::Json, None)?,
        OutputFormat::Yaml => print_output(data, crate::output::OutputFormat::Yaml, None)?,
        OutputFormat::Auto | OutputFormat::Table => {
            print_output(data, crate::output::OutputFormat::Json, None)?
        }
    }
    Ok(())
}

/// Common CLI arguments for async operations
#[derive(Args, Debug, Clone)]
pub struct AsyncOperationArgs {
    /// Wait for operation to complete
    #[arg(long)]
    pub wait: bool,

    /// Maximum time to wait in seconds
    #[arg(long, default_value = "300", requires = "wait")]
    pub wait_timeout: u64,

    /// Polling interval in seconds
    #[arg(long, default_value = "5", requires = "wait")]
    pub wait_interval: u64,
}

/// Handle an async operation response, optionally waiting for completion
pub async fn handle_async_response(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    response: Value,
    async_ops: &AsyncOperationArgs,
    output_format: OutputFormat,
    query: Option<&str>,
    success_message: &str,
) -> CliResult<()> {
    // Extract task ID from various possible locations
    let task_id = response
        .get("taskId")
        .or_else(|| response.get("task_id"))
        .or_else(|| response.get("response").and_then(|r| r.get("id")))
        .and_then(|v| v.as_str());

    // Apply JMESPath query if provided
    let result = if let Some(q) = query {
        crate::commands::cloud::utils::apply_jmespath(&response, q)?
    } else {
        response.clone()
    };

    // If we have a task ID and should wait
    if let Some(task_id) = task_id
        && async_ops.wait
    {
        // Wait for the task to complete
        wait_for_task(
            conn_mgr,
            profile_name,
            task_id,
            async_ops.wait_timeout,
            async_ops.wait_interval,
            output_format,
        )
        .await?;

        // Print success message for table format
        if matches!(output_format, OutputFormat::Table) {
            println!("{}", success_message);
        }
        return Ok(());
    }

    // Normal output without waiting
    match output_format {
        OutputFormat::Auto | OutputFormat::Table => {
            println!("{}", success_message);
            if let Some(task_id) = task_id {
                println!("Task ID: {}", task_id);
                println!(
                    "To wait for completion, run: redisctl cloud task wait {}",
                    task_id
                );
            }
        }
        OutputFormat::Json | OutputFormat::Yaml => print_json_or_yaml(result, output_format)?,
    }

    Ok(())
}

/// Wait for a task to complete using Layer 2's poll_task
///
/// This wraps redisctl_core::poll_task with CLI-specific progress bar output
/// and task detail formatting.
pub async fn wait_for_task(
    conn_mgr: &ConnectionManager,
    profile_name: Option<&str>,
    task_id: &str,
    timeout_secs: u64,
    interval_secs: u64,
    output_format: OutputFormat,
) -> CliResult<()> {
    let client = conn_mgr.create_cloud_client(profile_name).await?;
    let timeout = Duration::from_secs(timeout_secs);
    let interval = Duration::from_secs(interval_secs);

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg} [{elapsed_precise}]")
            .unwrap(),
    );
    pb.set_message(format!("Waiting for task {}", task_id));

    // Create progress callback that updates the spinner
    let pb_clone = pb.clone();
    let progress_callback = Some(
        Box::new(move |event: redisctl_core::ProgressEvent| match &event {
            redisctl_core::ProgressEvent::Started { task_id } => {
                pb_clone.set_message(format!("Task {} started", task_id));
            }
            redisctl_core::ProgressEvent::Polling {
                task_id, status, ..
            } => {
                pb_clone.set_message(format!("Task {}: {}", task_id, format_task_state(status)));
            }
            redisctl_core::ProgressEvent::Completed { task_id, .. } => {
                pb_clone.finish_with_message(format!(
                    "Task {}: {}",
                    task_id,
                    format_task_state("completed")
                ));
            }
            redisctl_core::ProgressEvent::Failed { task_id, error } => {
                pb_clone.finish_with_message(format!("Task {} failed: {}", task_id, error));
            }
        }) as redisctl_core::ProgressCallback,
    );

    // Use Layer 2's poll_task
    let result =
        redisctl_core::poll_task(&client, task_id, timeout, interval, progress_callback).await;

    match result {
        Ok(task) => {
            // Convert typed task to JSON for output formatting
            let task_json = serde_json::to_value(&task).unwrap_or_else(|_| serde_json::json!({}));

            match output_format {
                OutputFormat::Auto | OutputFormat::Table => {
                    print_task_details(&task_json)?;
                }
                OutputFormat::Json => {
                    print_output(task_json, crate::output::OutputFormat::Json, None)?;
                }
                OutputFormat::Yaml => {
                    print_output(task_json, crate::output::OutputFormat::Yaml, None)?;
                }
            }
            Ok(())
        }
        Err(e) => {
            pb.finish_with_message(format!("Task {} failed", task_id));
            Err(RedisCtlError::from(e))
        }
    }
}

/// Format task state for display with status icons
fn format_task_state(state: &str) -> String {
    match state.to_lowercase().as_str() {
        "completed" | "complete" | "succeeded" | "success" | "processing-completed" => {
            format!("\u{2713} {}", state) // checkmark
        }
        "failed" | "error" | "processing-error" => format!("\u{2717} {}", state), // x mark
        "cancelled" => format!("\u{2298} {}", state),                             // circle slash
        "processing" | "running" | "in_progress" => format!("\u{21bb} {}", state), // arrow circle
        _ => state.to_string(),
    }
}

/// Print detailed task information
fn print_task_details(task: &Value) -> CliResult<()> {
    println!("\nTask Details:");
    println!("-------------");

    if let Some(id) = task.get("taskId").or_else(|| task.get("id")) {
        println!("ID: {}", id);
    }

    if let Some(status) = task.get("status").or_else(|| task.get("state")) {
        println!("Status: {}", status);
    }

    if let Some(description) = task.get("description") {
        println!("Description: {}", description);
    }

    if let Some(progress) = task.get("progress") {
        println!("Progress: {}", progress);
    }

    if let Some(created) = task.get("createdAt").or_else(|| task.get("created_at")) {
        println!("Created: {}", created);
    }

    if let Some(updated) = task.get("updatedAt").or_else(|| task.get("updated_at")) {
        println!("Updated: {}", updated);
    }

    // Handle error details - check both top-level and nested in response
    if let Some(error) = task.get("error").or_else(|| task.get("errorMessage")) {
        println!("Error: {}", error);
    } else if let Some(response) = task.get("response")
        && let Some(error) = response.get("error")
    {
        // Handle nested error object
        if let Some(error_type) = error.get("type") {
            println!("Error Type: {}", error_type);
        }
        if let Some(error_status) = error.get("status") {
            println!("Error Status: {}", error_status);
        }
        if let Some(error_description) = error.get("description") {
            println!("Error Description: {}", error_description);
        }
        // If error is a simple string
        if error.is_string() {
            println!("Error: {}", error);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_task_state_success_variants() {
        assert!(format_task_state("completed").contains("completed"));
        assert!(format_task_state("complete").contains("complete"));
        assert!(format_task_state("succeeded").contains("succeeded"));
        assert!(format_task_state("success").contains("success"));
        assert!(format_task_state("processing-completed").contains("processing-completed"));
    }

    #[test]
    fn test_format_task_state_failure_variants() {
        assert!(format_task_state("failed").contains("failed"));
        assert!(format_task_state("error").contains("error"));
        assert!(format_task_state("processing-error").contains("processing-error"));
    }

    #[test]
    fn test_format_task_state_cancelled() {
        assert!(format_task_state("cancelled").contains("cancelled"));
    }

    #[test]
    fn test_format_task_state_in_progress_variants() {
        assert!(format_task_state("processing").contains("processing"));
        assert!(format_task_state("running").contains("running"));
        assert!(format_task_state("in_progress").contains("in_progress"));
    }

    #[test]
    fn test_format_task_state_unknown() {
        assert_eq!(format_task_state("pending"), "pending");
        assert_eq!(format_task_state("unknown"), "unknown");
        assert_eq!(format_task_state("custom_state"), "custom_state");
    }

    #[test]
    fn test_print_task_details_full() {
        let task = json!({
            "taskId": "task-123",
            "status": "completed",
            "description": "Create database",
            "progress": 100,
            "createdAt": "2025-01-01T00:00:00Z",
            "updatedAt": "2025-01-01T00:05:00Z"
        });

        let result = print_task_details(&task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_task_details_with_error() {
        let task = json!({
            "taskId": "task-456",
            "status": "failed",
            "error": "Database creation failed",
            "description": "Create database"
        });

        let result = print_task_details(&task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_task_details_minimal() {
        let task = json!({"id": "task-minimal"});
        let result = print_task_details(&task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_task_details_empty() {
        let task = json!({});
        let result = print_task_details(&task);
        assert!(result.is_ok());
    }
}
