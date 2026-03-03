//! Utility functions for Enterprise commands
use crate::error::Result as CliResult;
use anyhow::Context;
use dialoguer::Confirm;
use serde_json::Value;

pub use crate::output::{apply_jmespath, handle_output, print_formatted_output};

/// Confirm an action with the user
pub fn confirm_action(message: &str) -> CliResult<bool> {
    #[cfg(unix)]
    {
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            Ok(Confirm::new()
                .with_prompt(message)
                .default(false)
                .interact()
                .context("Failed to get user confirmation")?)
        } else {
            // In non-interactive mode, print warning and return false
            eprintln!("Warning: {} Use --force to skip confirmation.", message);
            Ok(false)
        }
    }

    #[cfg(not(unix))]
    {
        Ok(Confirm::new()
            .with_prompt(message)
            .default(false)
            .interact()
            .context("Failed to get user confirmation")?)
    }
}

/// Read JSON data from string, file, or stdin
pub fn read_json_data(data: &str) -> CliResult<Value> {
    let json_str = if data == "-" {
        // Read from stdin
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {}", e))?;
        buffer
    } else if let Some(file_path) = data.strip_prefix('@') {
        // Read from file
        std::fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?
    } else {
        // Direct JSON string
        data.to_string()
    };

    serde_json::from_str(&json_str).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e).into())
}
