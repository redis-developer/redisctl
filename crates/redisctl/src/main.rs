use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use redisctl_config::Config;
use tracing::{debug, error, info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod commands;
mod connection;
mod error;
mod output;
mod workflows;

use cli::{Cli, Commands};
use connection::ConnectionManager;
use error::RedisCtlError;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on verbosity level
    init_tracing(cli.verbose);

    // Load configuration from specified path or default location
    let (config, config_path) = if let Some(config_file) = &cli.config_file {
        let path = std::path::PathBuf::from(config_file);
        debug!("Loading config from explicit path: {:?}", path);
        let config = Config::load_from_path(&path)?;
        (config, Some(path))
    } else {
        debug!("Loading config from default location");
        (Config::load()?, None)
    };
    debug!(
        "Creating ConnectionManager with config_path: {:?}",
        config_path
    );
    let conn_mgr = ConnectionManager::with_config_path(config, config_path);

    // Execute command
    if let Err(e) = execute_command(&cli, &conn_mgr).await {
        eprintln!("{}", e.display_with_suggestions());
        std::process::exit(1);
    }

    Ok(())
}

fn init_tracing(verbose: u8) {
    // Check for RUST_LOG env var first, then fall back to verbosity flag
    let filter = if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        let level = match verbose {
            0 => "redisctl=warn,redis_cloud=warn,redis_enterprise=warn",
            1 => "redisctl=info,redis_cloud=info,redis_enterprise=info",
            2 => "redisctl=debug,redis_cloud=debug,redis_enterprise=debug",
            _ => "redisctl=trace,redis_cloud=trace,redis_enterprise=trace",
        };
        tracing_subscriber::EnvFilter::new(level)
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .compact(),
        )
        .init();

    debug!("Tracing initialized with verbosity level: {}", verbose);
}

async fn execute_command(cli: &Cli, conn_mgr: &ConnectionManager) -> Result<(), RedisCtlError> {
    // Log command execution with sanitized parameters
    trace!("Executing command: {:?}", cli.command);
    info!("Command: {}", format_command(&cli.command));

    let start = std::time::Instant::now();
    let result = match &cli.command {
        Commands::Version => {
            debug!("Showing version information");
            match cli.output {
                cli::OutputFormat::Json | cli::OutputFormat::Yaml => {
                    let output_data = serde_json::json!({
                        "version": env!("CARGO_PKG_VERSION"),
                        "name": env!("CARGO_PKG_NAME"),
                    });

                    let fmt = match cli.output {
                        cli::OutputFormat::Json => output::OutputFormat::Json,
                        cli::OutputFormat::Yaml => output::OutputFormat::Yaml,
                        _ => output::OutputFormat::Json,
                    };

                    crate::output::print_output(&output_data, fmt, None)?;
                }
                _ => {
                    println!("redisctl {}", env!("CARGO_PKG_VERSION"));
                }
            }
            Ok(())
        }
        Commands::Completions { shell } => {
            debug!("Generating completions for {:?}", shell);
            generate_completions(*shell);
            Ok(())
        }

        Commands::Profile(profile_cmd) => {
            debug!("Executing profile command");
            commands::profile::handle_profile_command(profile_cmd, conn_mgr, cli.output).await
        }

        Commands::FilesKey(files_key_cmd) => {
            debug!("Executing files-key command");
            execute_files_key_command(files_key_cmd).await
        }

        Commands::Api {
            deployment,
            method,
            path,
            data,
        } => {
            info!(
                "API call: {} {} {} (deployment: {:?})",
                method,
                path,
                if data.is_some() {
                    "with data"
                } else {
                    "no data"
                },
                deployment
            );
            execute_api_command(cli, conn_mgr, deployment, method, path, data.as_deref()).await
        }

        Commands::Cloud(cloud_cmd) => execute_cloud_command(cli, conn_mgr, cloud_cmd).await,

        Commands::Enterprise(enterprise_cmd) => {
            execute_enterprise_command(
                enterprise_cmd,
                conn_mgr,
                cli.profile.as_deref(),
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
    };

    let duration = start.elapsed();
    match &result {
        Ok(_) => info!("Command completed successfully in {:?}", duration),
        Err(e) => error!("Command failed after {:?}: {}", duration, e),
    }

    result
}

/// Generate shell completions
fn generate_completions(shell: cli::Shell) {
    let mut cmd = cli::Cli::command();
    let name = cmd.get_name().to_string();

    match shell {
        cli::Shell::Bash => generate(shells::Bash, &mut cmd, name, &mut std::io::stdout()),
        cli::Shell::Zsh => generate(shells::Zsh, &mut cmd, name, &mut std::io::stdout()),
        cli::Shell::Fish => generate(shells::Fish, &mut cmd, name, &mut std::io::stdout()),
        cli::Shell::PowerShell => {
            generate(shells::PowerShell, &mut cmd, name, &mut std::io::stdout())
        }
        cli::Shell::Elvish => generate(shells::Elvish, &mut cmd, name, &mut std::io::stdout()),
    }
}

/// Format command for human-readable logging (without sensitive data)
fn format_command(command: &Commands) -> String {
    match command {
        Commands::Version => "version".to_string(),
        Commands::Completions { shell } => format!("completions {:?}", shell),
        Commands::Profile(cmd) => {
            use cli::ProfileCommands::*;
            match cmd {
                List => "profile list".to_string(),
                Path => "profile path".to_string(),
                Show { name } => format!("profile show {}", name),
                Set { name, .. } => format!("profile set {} [credentials redacted]", name),
                Remove { name } => format!("profile remove {}", name),
                DefaultEnterprise { name } => format!("profile default-enterprise {}", name),
                DefaultCloud { name } => format!("profile default-cloud {}", name),
                DefaultDatabase { name } => format!("profile default-database {}", name),
                Validate => "profile validate".to_string(),
            }
        }
        Commands::Api {
            deployment,
            method,
            path,
            ..
        } => {
            format!("api {:?} {} {}", deployment, method, path)
        }
        Commands::Cloud(cmd) => format!("cloud {:?}", cmd),
        Commands::Enterprise(cmd) => format!("enterprise {:?}", cmd),
        Commands::FilesKey(cmd) => {
            use cli::FilesKeyCommands::*;
            match cmd {
                Set { .. } => "files-key set [key redacted]".to_string(),
                Get { profile } => format!("files-key get {:?}", profile),
                Remove { .. } => "files-key remove".to_string(),
            }
        }
    }
}

async fn execute_enterprise_command(
    enterprise_cmd: &cli::EnterpriseCommands,
    conn_mgr: &ConnectionManager,
    profile: Option<&str>,
    output: cli::OutputFormat,
    query: Option<&str>,
) -> Result<(), RedisCtlError> {
    use cli::EnterpriseCommands::*;

    match enterprise_cmd {
        Action(action_cmd) => {
            commands::enterprise::actions::handle_action_command(
                conn_mgr,
                profile,
                action_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Alerts(alerts_cmd) => alerts_cmd
            .execute(&conn_mgr.config, profile, output, query)
            .await
            .map_err(|e| RedisCtlError::Configuration(e.to_string())),
        BdbGroup(bdb_group_cmd) => {
            commands::enterprise::bdb_group::handle_bdb_group_command(
                conn_mgr,
                profile,
                bdb_group_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Cluster(cluster_cmd) => {
            commands::enterprise::cluster::handle_cluster_command(
                conn_mgr,
                profile,
                cluster_cmd,
                output,
                query,
            )
            .await
        }
        CmSettings(cm_settings_cmd) => {
            commands::enterprise::cm_settings::handle_cm_settings_command(
                conn_mgr,
                profile,
                cm_settings_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Database(db_cmd) => {
            commands::enterprise::database::handle_database_command(
                conn_mgr, profile, db_cmd, output, query,
            )
            .await
        }
        DebugInfo(debuginfo_cmd) => {
            commands::enterprise::debuginfo::handle_debuginfo_command(
                conn_mgr,
                profile,
                debuginfo_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Diagnostics(diagnostics_cmd) => {
            commands::enterprise::diagnostics::handle_diagnostics_command(
                conn_mgr,
                profile,
                diagnostics_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Endpoint(endpoint_cmd) => {
            commands::enterprise::endpoint::handle_endpoint_command(
                conn_mgr,
                profile,
                endpoint_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Node(node_cmd) => {
            commands::enterprise::node::handle_node_command(
                conn_mgr, profile, node_cmd, output, query,
            )
            .await
        }
        Proxy(proxy_cmd) => {
            commands::enterprise::proxy::handle_proxy_command(
                conn_mgr,
                profile,
                proxy_cmd.clone(),
                output,
                query,
            )
            .await
        }
        User(user_cmd) => {
            commands::enterprise::rbac::handle_user_command(
                conn_mgr, profile, user_cmd, output, query,
            )
            .await
        }
        Role(role_cmd) => {
            commands::enterprise::rbac::handle_role_command(
                conn_mgr, profile, role_cmd, output, query,
            )
            .await
        }
        Acl(acl_cmd) => {
            commands::enterprise::rbac::handle_acl_command(
                conn_mgr, profile, acl_cmd, output, query,
            )
            .await
        }
        Ldap(ldap_cmd) => {
            commands::enterprise::ldap::handle_ldap_command(
                conn_mgr,
                profile,
                ldap_cmd.clone(),
                output,
                query,
            )
            .await
        }
        LdapMappings(ldap_mappings_cmd) => {
            commands::enterprise::ldap::handle_ldap_mappings_command(
                conn_mgr,
                profile,
                ldap_mappings_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Auth(auth_cmd) => {
            commands::enterprise::rbac::handle_auth_command(
                conn_mgr, profile, auth_cmd, output, query,
            )
            .await
        }
        Bootstrap(bootstrap_cmd) => {
            commands::enterprise::bootstrap::handle_bootstrap_command(
                conn_mgr,
                profile,
                bootstrap_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Crdb(crdb_cmd) => {
            commands::enterprise::crdb::handle_crdb_command(
                conn_mgr, profile, crdb_cmd, output, query,
            )
            .await
        }
        CrdbTask(crdb_task_cmd) => {
            commands::enterprise::crdb_task::handle_crdb_task_command(
                conn_mgr,
                profile,
                crdb_task_cmd.clone(),
                output,
                query,
            )
            .await
        }
        JobScheduler(job_scheduler_cmd) => {
            commands::enterprise::job_scheduler::handle_job_scheduler_command(
                conn_mgr,
                profile,
                job_scheduler_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Jsonschema(jsonschema_cmd) => {
            commands::enterprise::jsonschema::handle_jsonschema_command(
                conn_mgr,
                profile,
                jsonschema_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Logs(logs_cmd) => {
            commands::enterprise::logs_impl::handle_logs_commands(
                conn_mgr, profile, logs_cmd, output, query,
            )
            .await
        }
        License(license_cmd) => license_cmd
            .execute(&conn_mgr.config, profile, output, query)
            .await
            .map_err(|e| RedisCtlError::Configuration(e.to_string())),
        Migration(migration_cmd) => {
            commands::enterprise::migration::handle_migration_command(
                conn_mgr,
                profile,
                migration_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Module(module_cmd) => {
            commands::enterprise::module_impl::handle_module_commands(
                conn_mgr, profile, module_cmd, output, query,
            )
            .await
        }
        Ocsp(ocsp_cmd) => {
            commands::enterprise::ocsp::handle_ocsp_command(
                conn_mgr,
                profile,
                ocsp_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Services(services_cmd) => {
            commands::enterprise::services::handle_services_command(
                conn_mgr,
                profile,
                services_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Workflow(workflow_cmd) => {
            handle_enterprise_workflow_command(conn_mgr, profile, workflow_cmd, output).await
        }
        Local(local_cmd) => {
            commands::enterprise::local::handle_local_command(
                conn_mgr,
                profile,
                local_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Shard(shard_cmd) => {
            commands::enterprise::shard::handle_shard_command(
                conn_mgr,
                profile,
                shard_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Stats(stats_cmd) => {
            commands::enterprise::stats::handle_stats_command(
                conn_mgr, profile, stats_cmd, output, query,
            )
            .await
        }
        Status {
            cluster,
            nodes,
            databases,
            shards,
        } => {
            let sections = commands::enterprise::status::StatusSections {
                cluster: *cluster,
                nodes: *nodes,
                databases: *databases,
                shards: *shards,
            };
            commands::enterprise::status::get_status(conn_mgr, profile, sections, output, query)
                .await
        }
        SupportPackage(support_cmd) => {
            commands::enterprise::support_package::handle_support_package_command(
                conn_mgr,
                profile,
                support_cmd.clone(),
                output,
                query,
            )
            .await
        }
        Suffix(suffix_cmd) => {
            commands::enterprise::suffix::handle_suffix_command(
                conn_mgr,
                profile,
                suffix_cmd.clone(),
                output,
                query,
            )
            .await
        }
        UsageReport(usage_report_cmd) => {
            commands::enterprise::usage_report::handle_usage_report_command(
                conn_mgr,
                profile,
                usage_report_cmd.clone(),
                output,
                query,
            )
            .await
        }
    }
}

async fn handle_cloud_workflow_command(
    conn_mgr: &ConnectionManager,
    cli: &cli::Cli,
    workflow_cmd: &cli::CloudWorkflowCommands,
) -> Result<(), RedisCtlError> {
    use cli::CloudWorkflowCommands::*;
    use workflows::{WorkflowArgs, WorkflowContext, WorkflowRegistry};

    let output = cli.output;
    let profile = cli.profile.as_deref();

    match workflow_cmd {
        List => {
            let registry = WorkflowRegistry::new();
            let workflows = registry.list();

            // Filter to show only cloud workflows
            let cloud_workflows: Vec<_> = workflows
                .into_iter()
                .filter(|(name, _)| name.contains("subscription") || name.contains("cloud"))
                .collect();

            match output {
                cli::OutputFormat::Json | cli::OutputFormat::Yaml => {
                    let workflow_list: Vec<serde_json::Value> = cloud_workflows
                        .into_iter()
                        .map(|(name, description)| {
                            serde_json::json!({
                                "name": name,
                                "description": description
                            })
                        })
                        .collect();
                    let output_format = match output {
                        cli::OutputFormat::Json => output::OutputFormat::Json,
                        cli::OutputFormat::Yaml => output::OutputFormat::Yaml,
                        _ => output::OutputFormat::Table,
                    };
                    crate::output::print_output(
                        serde_json::json!(workflow_list),
                        output_format,
                        None,
                    )?;
                }
                _ => {
                    println!("Available Cloud Workflows:");
                    println!();
                    for (name, description) in cloud_workflows {
                        println!("  {} - {}", name, description);
                    }
                }
            }
            Ok(())
        }
        SubscriptionSetup(args) => {
            let mut workflow_args = WorkflowArgs::new();
            workflow_args.insert("args", args);

            let output_format = match output {
                cli::OutputFormat::Json => output::OutputFormat::Json,
                cli::OutputFormat::Yaml => output::OutputFormat::Yaml,
                cli::OutputFormat::Table | cli::OutputFormat::Auto => output::OutputFormat::Table,
            };

            let context = WorkflowContext {
                conn_mgr: conn_mgr.clone(),
                profile_name: profile.map(String::from),
                output_format,
                wait_timeout: args.wait_timeout as u64,
            };

            let registry = WorkflowRegistry::new();
            let workflow =
                registry
                    .get("subscription-setup")
                    .ok_or_else(|| RedisCtlError::ApiError {
                        message: "Workflow not found".to_string(),
                    })?;

            let result = workflow
                .execute(context, workflow_args)
                .await
                .map_err(|e| RedisCtlError::ApiError {
                    message: e.to_string(),
                })?;

            if !result.success {
                return Err(RedisCtlError::ApiError {
                    message: result.message,
                });
            }

            // Print result as JSON/YAML if requested
            match output {
                cli::OutputFormat::Json | cli::OutputFormat::Yaml => {
                    let result_json = serde_json::json!({
                        "success": result.success,
                        "message": result.message,
                        "outputs": result.outputs,
                    });
                    crate::output::print_output(&result_json, output_format, None)?;
                }
                _ => {
                    // Human output
                    println!("{}", result.message);
                }
            }

            Ok(())
        }
    }
}

async fn handle_enterprise_workflow_command(
    conn_mgr: &ConnectionManager,
    profile: Option<&str>,
    workflow_cmd: &cli::EnterpriseWorkflowCommands,
    output: cli::OutputFormat,
) -> Result<(), RedisCtlError> {
    use cli::EnterpriseWorkflowCommands::*;
    use workflows::{WorkflowArgs, WorkflowContext, WorkflowRegistry};

    match workflow_cmd {
        List => {
            let registry = WorkflowRegistry::new();
            let workflows = registry.list();

            match output {
                cli::OutputFormat::Json | cli::OutputFormat::Yaml => {
                    let workflow_list: Vec<serde_json::Value> = workflows
                        .into_iter()
                        .map(|(name, description)| {
                            serde_json::json!({
                                "name": name,
                                "description": description
                            })
                        })
                        .collect();
                    let output_format = match output {
                        cli::OutputFormat::Json => output::OutputFormat::Json,
                        cli::OutputFormat::Yaml => output::OutputFormat::Yaml,
                        _ => output::OutputFormat::Table,
                    };
                    crate::output::print_output(
                        serde_json::json!(workflow_list),
                        output_format,
                        None,
                    )?;
                }
                _ => {
                    println!("Available Enterprise Workflows:");
                    println!();
                    for (name, description) in workflows {
                        println!("  {} - {}", name, description);
                    }
                }
            }
            Ok(())
        }
        License(license_workflow_cmd) => license_workflow_cmd
            .execute(&conn_mgr.config, output, None)
            .await
            .map_err(|e| RedisCtlError::Configuration(e.to_string())),
        InitCluster {
            name,
            username,
            password,
            skip_database,
            database_name,
            database_memory_gb,
            async_ops,
        } => {
            let mut args = WorkflowArgs::new();
            args.insert("name", name);
            args.insert("username", username);
            args.insert("password", password);
            args.insert("create_database", !skip_database);
            args.insert("database_name", database_name);
            args.insert("database_memory_gb", database_memory_gb);

            let output_format = match output {
                cli::OutputFormat::Json => output::OutputFormat::Json,
                cli::OutputFormat::Yaml => output::OutputFormat::Yaml,
                cli::OutputFormat::Table | cli::OutputFormat::Auto => output::OutputFormat::Table,
            };

            let context = WorkflowContext {
                conn_mgr: conn_mgr.clone(),
                profile_name: profile.map(String::from),
                output_format,
                wait_timeout: if async_ops.wait {
                    async_ops.wait_timeout
                } else {
                    0
                },
            };

            let registry = WorkflowRegistry::new();
            let workflow = registry
                .get("init-cluster")
                .ok_or_else(|| RedisCtlError::ApiError {
                    message: "Workflow not found".to_string(),
                })?;

            let result =
                workflow
                    .execute(context, args)
                    .await
                    .map_err(|e| RedisCtlError::ApiError {
                        message: e.to_string(),
                    })?;

            if !result.success {
                return Err(RedisCtlError::ApiError {
                    message: result.message,
                });
            }

            // Print result as JSON/YAML if requested
            match output {
                cli::OutputFormat::Json | cli::OutputFormat::Yaml => {
                    let result_json = serde_json::json!({
                        "success": result.success,
                        "message": result.message,
                        "outputs": result.outputs,
                    });
                    crate::output::print_output(&result_json, output_format, None)?;
                }
                _ => {
                    // Human output was already printed by the workflow
                }
            }

            Ok(())
        }
    }
}

async fn execute_files_key_command(
    files_key_cmd: &cli::FilesKeyCommands,
) -> Result<(), RedisCtlError> {
    use cli::FilesKeyCommands::*;

    match files_key_cmd {
        Set {
            api_key,
            #[cfg(feature = "secure-storage")]
            use_keyring,
            global,
            profile,
        } => commands::files_key::handle_set(
            api_key.clone(),
            #[cfg(feature = "secure-storage")]
            *use_keyring,
            *global,
            profile.clone(),
        )
        .await
        .map_err(RedisCtlError::from),
        Get { profile } => commands::files_key::handle_get(profile.clone())
            .await
            .map_err(RedisCtlError::from),
        Remove {
            #[cfg(feature = "secure-storage")]
            keyring,
            global,
            profile,
        } => commands::files_key::handle_remove(
            #[cfg(feature = "secure-storage")]
            *keyring,
            *global,
            profile.clone(),
        )
        .await
        .map_err(RedisCtlError::from),
    }
}

async fn execute_api_command(
    cli: &Cli,
    conn_mgr: &ConnectionManager,
    deployment: &redisctl_config::DeploymentType,
    method: &cli::HttpMethod,
    path: &str,
    data: Option<&str>,
) -> Result<(), RedisCtlError> {
    commands::api::handle_api_command(commands::api::ApiCommandParams {
        config: conn_mgr.config.clone(),
        config_path: conn_mgr.config_path.clone(),
        profile_name: cli.profile.clone(),
        deployment: *deployment,
        method: method.clone(),
        path: path.to_string(),
        data: data.map(|s| s.to_string()),
        query: cli.query.clone(),
        output_format: cli.output,
    })
    .await
}

async fn execute_cloud_command(
    cli: &Cli,
    conn_mgr: &ConnectionManager,
    cloud_cmd: &cli::CloudCommands,
) -> Result<(), RedisCtlError> {
    use cli::CloudCommands::*;

    match cloud_cmd {
        Account(account_cmd) => {
            commands::cloud::handle_account_command(
                conn_mgr,
                cli.profile.as_deref(),
                account_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }

        PaymentMethod(payment_method_cmd) => {
            commands::cloud::handle_payment_method_command(
                conn_mgr,
                cli.profile.as_deref(),
                payment_method_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }

        Subscription(sub_cmd) => {
            commands::cloud::handle_subscription_command(
                conn_mgr,
                cli.profile.as_deref(),
                sub_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }

        Database(db_cmd) => {
            commands::cloud::handle_database_command(
                conn_mgr,
                cli.profile.as_deref(),
                db_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }

        User(user_cmd) => {
            commands::cloud::handle_user_command(
                conn_mgr,
                cli.profile.as_deref(),
                user_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        Acl(acl_cmd) => {
            commands::cloud::acl::handle_acl_command(
                conn_mgr,
                cli.profile.as_deref(),
                acl_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        ProviderAccount(provider_account_cmd) => {
            commands::cloud::cloud_account::handle_cloud_account_command(
                conn_mgr,
                cli.profile.as_deref(),
                provider_account_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        Task(task_cmd) => {
            commands::cloud::task::handle_task_command(
                conn_mgr,
                cli.profile.as_deref(),
                task_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        Connectivity(connectivity_cmd) => {
            commands::cloud::connectivity::handle_connectivity_command(
                conn_mgr,
                cli.profile.as_deref(),
                connectivity_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        FixedDatabase(fixed_db_cmd) => {
            commands::cloud::fixed_database::handle_fixed_database_command(
                conn_mgr,
                cli.profile.as_deref(),
                fixed_db_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        FixedSubscription(fixed_sub_cmd) => {
            commands::cloud::fixed_subscription::handle_fixed_subscription_command(
                conn_mgr,
                cli.profile.as_deref(),
                fixed_sub_cmd,
                cli.output,
                cli.query.as_deref(),
            )
            .await
        }
        Workflow(workflow_cmd) => handle_cloud_workflow_command(conn_mgr, cli, workflow_cmd).await,
        CostReport(cost_report_cmd) => {
            commands::cloud::cost_report::handle_cost_report_command(
                conn_mgr,
                cli.profile.as_deref(),
                cost_report_cmd.clone(),
                cli.output,
            )
            .await
        }
    }
}
