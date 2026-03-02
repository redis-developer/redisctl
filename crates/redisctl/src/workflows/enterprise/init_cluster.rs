//! Initialize Redis Enterprise cluster workflow
//!
//! This workflow automates the process of setting up a new Redis Enterprise cluster,
//! including bootstrap, waiting for initialization, creating admin user, and
//! optionally creating a default database.

use crate::workflows::{Workflow, WorkflowArgs, WorkflowContext, WorkflowResult};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use redis_enterprise::EnterpriseClient;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;

pub struct InitClusterWorkflow;

impl InitClusterWorkflow {
    pub fn new() -> Self {
        Self
    }
}

impl Workflow for InitClusterWorkflow {
    fn name(&self) -> &str {
        "init-cluster"
    }

    fn description(&self) -> &str {
        "Initialize a Redis Enterprise cluster with bootstrap and optional database creation"
    }

    fn execute(
        &self,
        context: WorkflowContext,
        args: WorkflowArgs,
    ) -> Pin<Box<dyn Future<Output = Result<WorkflowResult>> + Send>> {
        Box::pin(async move {
            use crate::output::OutputFormat;

            // Only print human-readable output for Table/Auto format
            let is_human_output = matches!(
                context.output_format,
                OutputFormat::Table | OutputFormat::Auto
            );

            if is_human_output {
                println!("Initializing Redis Enterprise cluster...");
            }

            // Get parameters
            let cluster_name = args
                .get_string("name")
                .unwrap_or_else(|| "redis-cluster".to_string());
            let username = args
                .get_string("username")
                .unwrap_or_else(|| "admin@redis.local".to_string());
            let password = args
                .get_string("password")
                .context("Password is required for cluster initialization")?;
            let create_db = args.get_bool("create_database").unwrap_or(true);
            let db_name = args
                .get_string("database_name")
                .unwrap_or_else(|| "default-db".to_string());
            let db_memory_gb = args.get_i64("database_memory_gb").unwrap_or(1);

            // Create unauthenticated client for bootstrap operations
            // Bootstrap doesn't require auth, but we need the URL from the environment/profile
            let base_url = std::env::var("REDIS_ENTERPRISE_URL")
                .unwrap_or_else(|_| "https://localhost:9443".to_string());
            let insecure = std::env::var("REDIS_ENTERPRISE_INSECURE")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false);

            let client = redis_enterprise::EnterpriseClient::builder()
                .base_url(base_url)
                .username("") // Bootstrap doesn't require auth
                .password("") // Bootstrap doesn't require auth
                .insecure(insecure)
                .build()
                .context("Failed to create Enterprise client for bootstrap")?;

            // Step 1: Check if cluster is already initialized
            let needs_bootstrap = check_if_needs_bootstrap(&client).await?;

            if !needs_bootstrap {
                if is_human_output {
                    println!("Cluster is already initialized");
                }
                return Ok(WorkflowResult::success("Cluster already initialized")
                    .with_output("cluster_name", &cluster_name)
                    .with_output("already_initialized", true));
            }

            // Step 2: Bootstrap the cluster
            let bootstrap_data = json!({
                "action": "create_cluster",
                "cluster": {
                    "name": cluster_name
                },
                "credentials": {
                    "username": username,
                    "password": password
                },
                "flash_enabled": false
            });

            let bootstrap_result = client
                .post_bootstrap("/v1/bootstrap/create_cluster", &bootstrap_data)
                .await
                .context("Failed to bootstrap cluster")?;

            // Check if bootstrap returned an action ID (async operation)
            if let Some(action_id) = bootstrap_result.get("action_uid").and_then(|v| v.as_str()) {
                // Wait for bootstrap to complete
                wait_for_action(&client, action_id, "cluster bootstrap").await?;
            } else {
                // Bootstrap was synchronous, just wait a bit for cluster to stabilize
                sleep(Duration::from_secs(5)).await;
            }

            if is_human_output {
                println!("Bootstrap completed successfully");
            }

            // Step 3: Cluster should be ready after bootstrap
            // Wait longer for cluster to fully stabilize
            sleep(Duration::from_secs(10)).await;
            if is_human_output {
                println!("Cluster is ready");
            }

            // After bootstrap, we need to create a new client with the credentials we just set
            // Get the base URL from environment or use default
            let base_url = std::env::var("REDIS_ENTERPRISE_URL")
                .unwrap_or_else(|_| "https://localhost:9443".to_string());
            let insecure = std::env::var("REDIS_ENTERPRISE_INSECURE")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true);

            let authenticated_client = redis_enterprise::EnterpriseClient::builder()
                .base_url(base_url)
                .username(username.clone())
                .password(password.clone())
                .insecure(insecure)
                .build()
                .context("Failed to create authenticated client after bootstrap")?;

            // Step 4: Optionally create a default database
            if create_db {
                if is_human_output {
                    println!("Creating default database '{}'...", db_name);
                }

                let db_data = json!({
                    "name": db_name,
                    "memory_size": db_memory_gb * 1024 * 1024 * 1024,  // Convert GB to bytes
                    "type": "redis",
                    "replication": false
                });

                match authenticated_client.post_raw("/v1/bdbs", db_data).await {
                    Ok(db_result) => {
                        // Check for async operation
                        if let Some(action_id) =
                            db_result.get("action_uid").and_then(|v| v.as_str())
                        {
                            wait_for_action(&authenticated_client, action_id, "database creation")
                                .await?;
                        }

                        let db_uid = db_result
                            .get("uid")
                            .or_else(|| db_result.get("resource_id"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as u32;

                        if is_human_output {
                            println!("Database created successfully (ID: {})", db_uid);
                        }

                        // Verify database connectivity with PING command
                        if db_uid > 0 {
                            // Wait a moment for database to be fully ready
                            sleep(Duration::from_secs(2)).await;

                            match authenticated_client.execute_command(db_uid, "PING").await {
                                Ok(response) => {
                                    if let Some(result) = response.get("response") {
                                        // The command endpoint returns {"response": true} for successful PING
                                        if (result.as_bool() == Some(true)
                                            || result.as_str() == Some("PONG"))
                                            && is_human_output
                                        {
                                            println!(
                                                "Database connectivity verified (PING successful)"
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    if is_human_output {
                                        eprintln!(
                                            "Note: Could not verify database connectivity: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Database creation failed, but cluster is initialized
                        if is_human_output {
                            eprintln!("Warning: Failed to create database: {}", e);
                            eprintln!("Cluster is initialized but database creation failed.");
                            eprintln!("You can create a database manually later.");
                        }
                    }
                }
            } else if is_human_output {
                println!("Skipping database creation (--skip-database flag set)");
            }

            // Final summary (only for human output)
            if is_human_output {
                println!();
                println!("Cluster initialization completed successfully");
                println!();
                println!("Cluster name: {}", cluster_name);
                println!("Admin user: {}", username);
                if create_db {
                    println!("Database: {} ({}GB)", db_name, db_memory_gb);
                }
                println!();
                println!("Access endpoints:");
                println!("  Web UI: https://localhost:8443");
                println!("  API: https://localhost:9443");
            }

            Ok(WorkflowResult::success("Cluster initialized successfully")
                .with_output("cluster_name", &cluster_name)
                .with_output("username", &username)
                .with_output("database_created", create_db)
                .with_output("database_name", &db_name))
        })
    }
}

/// Check if the cluster needs bootstrap
async fn check_if_needs_bootstrap(client: &EnterpriseClient) -> Result<bool> {
    match client.get_raw("/v1/bootstrap").await {
        Ok(status) => {
            // Check if cluster is already bootstrapped
            if let Some(state) = status.get("state").and_then(|v| v.as_str()) {
                Ok(state == "unconfigured" || state == "new")
            } else {
                // If we can't determine state, assume it needs bootstrap
                Ok(true)
            }
        }
        Err(_) => {
            // If we can't get status, cluster might not be initialized
            Ok(true)
        }
    }
}

/// Wait for an async action to complete
async fn wait_for_action(
    client: &EnterpriseClient,
    action_id: &str,
    operation_name: &str,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Waiting for {} to complete...", operation_name));

    let max_attempts = 120; // 10 minutes with 5 second intervals
    for attempt in 1..=max_attempts {
        pb.set_message(format!(
            "Waiting for {} to complete... (attempt {}/{})",
            operation_name, attempt, max_attempts
        ));

        match client.get_raw(&format!("/v1/actions/{}", action_id)).await {
            Ok(action) => {
                if let Some(status) = action.get("status").and_then(|v| v.as_str()) {
                    match status {
                        "completed" | "done" => {
                            pb.finish_and_clear();
                            return Ok(());
                        }
                        "failed" | "error" => {
                            pb.finish_and_clear();
                            let error_msg = action
                                .get("error")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error");
                            anyhow::bail!("{} failed: {}", operation_name, error_msg);
                        }
                        _ => {
                            // Still in progress
                        }
                    }
                }
            }
            Err(_) => {
                // Action might not be available yet
            }
        }

        sleep(Duration::from_secs(5)).await;
    }

    pb.finish_and_clear();
    anyhow::bail!("{} did not complete within 10 minutes", operation_name)
}
