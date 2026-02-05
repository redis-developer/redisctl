//! redisctl-mcp: MCP server for Redis Cloud and Enterprise
//!
//! A standalone MCP server that exposes Redis management operations
//! as tools for AI systems.

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use tower_mcp::{CapabilityFilter, DenialBehavior, McpRouter, Tool, transport::StdioTransport};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod error;
mod prompts;
mod resources;
mod state;
mod tools;

use state::{AppState, CredentialSource};

/// Transport mode for the MCP server
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum Transport {
    /// Standard input/output (for CLI integrations)
    #[default]
    Stdio,
    /// HTTP with Server-Sent Events (for shared deployments)
    Http,
}

/// MCP server for Redis Cloud and Enterprise management
#[derive(Parser, Debug)]
#[command(name = "redisctl-mcp")]
#[command(version, about, long_about = None)]
struct Args {
    /// Transport mode
    #[arg(short, long, value_enum, default_value = "stdio")]
    transport: Transport,

    /// Profile name (for local credential resolution)
    #[arg(short, long, env = "REDISCTL_PROFILE")]
    profile: Option<String>,

    /// Read-only mode (disables write operations)
    #[arg(long, default_value = "false")]
    read_only: bool,

    /// Redis database URL for direct connections
    #[arg(long, env = "REDIS_URL")]
    database_url: Option<String>,

    // --- HTTP transport options ---
    /// Host to bind HTTP server
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind HTTP server
    #[arg(long, default_value = "8080")]
    port: u16,

    // --- OAuth options (HTTP mode) ---
    /// Enable OAuth authentication for HTTP transport
    #[arg(long)]
    oauth: bool,

    /// OAuth issuer URL (e.g., https://accounts.google.com)
    #[arg(long, env = "OAUTH_ISSUER")]
    oauth_issuer: Option<String>,

    /// OAuth audience (client ID or API identifier)
    #[arg(long, env = "OAUTH_AUDIENCE")]
    oauth_audience: Option<String>,

    /// JWKS URI for token validation (auto-discovered from issuer if not set)
    #[arg(long, env = "OAUTH_JWKS_URI")]
    jwks_uri: Option<String>,

    // --- Rate limiting ---
    /// Maximum concurrent requests
    #[arg(long, default_value = "10")]
    max_concurrent: usize,

    /// Rate limit interval in milliseconds
    #[arg(long, default_value = "100")]
    rate_limit_ms: u64,

    /// Request timeout in seconds (HTTP mode)
    #[arg(long, default_value = "30")]
    request_timeout_secs: u64,

    // --- Logging ---
    /// Log level
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| args.log_level.clone().into()))
        .init();

    info!(
        transport = ?args.transport,
        profile = ?args.profile,
        read_only = args.read_only,
        "Starting redisctl-mcp server"
    );

    // Determine credential source
    let credential_source = if args.oauth {
        CredentialSource::OAuth {
            issuer: args.oauth_issuer.clone(),
            audience: args.oauth_audience.clone(),
        }
    } else {
        CredentialSource::Profile(args.profile.clone())
    };

    // Build application state
    let state = Arc::new(AppState::new(
        credential_source,
        args.read_only,
        args.database_url.clone(),
    )?);

    // Build router with tools and optional read-only filter
    let router = build_router(state.clone(), args.read_only)?;

    match args.transport {
        Transport::Stdio => {
            info!("Running with stdio transport");
            StdioTransport::new(router).run().await?;
        }
        Transport::Http => {
            info!(host = %args.host, port = args.port, "Running with HTTP transport");
            run_http_server(router, &args).await?;
        }
    }

    Ok(())
}

/// Build the MCP router with all tools
fn build_router(state: Arc<AppState>, read_only: bool) -> Result<McpRouter> {
    let instructions = r#"
Redis Cloud and Enterprise MCP Server

This server provides comprehensive tools for managing Redis Cloud subscriptions and databases,
Redis Enterprise clusters and databases, and direct Redis database operations.

## Available Tool Categories

### Redis Cloud - Subscriptions & Databases
- list_subscriptions: List all Cloud subscriptions
- get_subscription: Get subscription details
- list_databases: List databases in a subscription
- get_database: Get database details
- get_backup_status: Get database backup status
- get_slow_log: Get slow query log
- get_database_tags: Get tags for a database

### Redis Cloud - Account & Configuration
- get_account: Get current account information
- get_regions: Get supported cloud regions
- get_modules: Get supported Redis modules
- list_account_users: List team members
- get_account_user: Get team member details by ID
- list_acl_users: List database ACL users
- get_acl_user: Get ACL user details by ID
- list_acl_roles: List ACL roles
- list_redis_rules: List Redis ACL rules

### Redis Cloud - Logs
- get_system_logs: Get system audit logs (subscription/database changes)
- get_session_logs: Get session activity logs (login/logout events)

### Redis Cloud - Tasks
- list_tasks: List async operations
- get_task: Get task status

### Redis Cloud - Write Operations (require --read-only=false)
- create_database: Create a new database and wait for it to be ready
- update_database: Update a database configuration
- delete_database: Delete a database
- backup_database: Trigger a manual backup
- import_database: Import data into a database
- delete_subscription: Delete a subscription (all databases must be deleted first)

### Redis Enterprise - Cluster
- get_cluster: Get cluster information
- get_cluster_stats: Get cluster statistics

### Redis Enterprise - License
- get_license: Get license information (type, expiration, features)
- get_license_usage: Get license utilization (shards, nodes, RAM vs limits)

### Redis Enterprise - Logs
- list_logs: List cluster event logs (with time range and pagination)

### Redis Enterprise - Databases
- list_enterprise_databases: List all databases
- get_enterprise_database: Get database details
- get_database_stats: Get database statistics
- get_database_endpoints: Get connection endpoints
- list_database_alerts: Get alerts for a database

### Redis Enterprise - Nodes
- list_nodes: List cluster nodes
- get_node: Get node details
- get_node_stats: Get node statistics

### Redis Enterprise - Users & Alerts
- list_enterprise_users: List cluster users
- get_enterprise_user: Get user details
- list_alerts: List all active alerts

### Redis Enterprise - Shards
- list_shards: List database shards (with optional database filter)
- get_shard: Get shard details by UID

### Redis Enterprise - Aggregate Stats
- get_all_nodes_stats: Get stats for all nodes in one call
- get_all_databases_stats: Get stats for all databases in one call
- get_shard_stats: Get stats for a specific shard
- get_all_shards_stats: Get stats for all shards in one call

### Redis Enterprise - Debug Info
- list_debug_info_tasks: List debug info collection tasks
- get_debug_info_status: Get status of a debug info collection task

### Redis Enterprise - Modules
- list_modules: List installed Redis modules (RedisJSON, RediSearch, etc.)
- get_module: Get details about a specific module

### Redis Enterprise - Write Operations (require --read-only=false)
- backup_enterprise_database: Trigger a database backup and wait for completion
- import_enterprise_database: Import data into a database and wait for completion
- create_enterprise_database: Create a new database
- update_enterprise_database: Update database configuration
- delete_enterprise_database: Delete a database
- flush_enterprise_database: Flush all data from a database

### Redis Database - Connection
- redis_ping: Test connectivity
- redis_info: Get server information
- redis_dbsize: Get key count
- redis_client_list: Get connected clients
- redis_cluster_info: Get cluster info (if clustered)
- redis_slowlog: Get slow query log entries

### Redis Database - Keys
- redis_keys: List keys matching pattern (SCAN)
- redis_scan: Scan keys with type filter (string, list, set, zset, hash, stream)
- redis_get: Get string value
- redis_type: Get key type
- redis_ttl: Get key TTL
- redis_exists: Check key existence
- redis_memory_usage: Get key memory usage
- redis_object_encoding: Get key internal encoding

### Redis Database - Data Structures
- redis_hgetall: Get all hash fields
- redis_lrange: Get list range
- redis_smembers: Get set members
- redis_zrange: Get sorted set range

### Profile Management - Read
- profile_list: List all configured profiles
- profile_show: Show profile details (credentials masked)
- profile_path: Show configuration file path
- profile_validate: Validate configuration file

### Profile Management - Write (requires --read-only=false)
- profile_set_default_cloud: Set default Cloud profile
- profile_set_default_enterprise: Set default Enterprise profile
- profile_delete: Delete a profile

## Resources

Read-only data accessible via URI:
- redis://config/path - Configuration file path
- redis://profiles - List of configured profiles
- redis://help - Usage instructions and help

## Prompts

Pre-built templates for common workflows:
- troubleshoot_database - Diagnose database issues
- analyze_performance - Analyze performance metrics
- capacity_planning - Help with capacity planning
- migration_planning - Plan Redis migrations

## Authentication

In stdio mode, credentials are resolved from redisctl profiles.
In HTTP mode with OAuth, credentials can be passed via JWT claims.
"#;

    let router = McpRouter::new()
        .server_info("redisctl-mcp", env!("CARGO_PKG_VERSION"))
        .instructions(instructions)
        // Cloud - Subscriptions & Databases
        .tool(tools::cloud::list_subscriptions(state.clone()))
        .tool(tools::cloud::get_subscription(state.clone()))
        .tool(tools::cloud::list_databases(state.clone()))
        .tool(tools::cloud::get_database(state.clone()))
        .tool(tools::cloud::get_backup_status(state.clone()))
        .tool(tools::cloud::get_slow_log(state.clone()))
        .tool(tools::cloud::get_tags(state.clone()))
        // Cloud - Account & Configuration
        .tool(tools::cloud::get_account(state.clone()))
        .tool(tools::cloud::get_regions(state.clone()))
        .tool(tools::cloud::get_modules(state.clone()))
        .tool(tools::cloud::list_account_users(state.clone()))
        .tool(tools::cloud::get_account_user(state.clone()))
        .tool(tools::cloud::list_acl_users(state.clone()))
        .tool(tools::cloud::get_acl_user(state.clone()))
        .tool(tools::cloud::list_acl_roles(state.clone()))
        .tool(tools::cloud::list_redis_rules(state.clone()))
        // Cloud - Logs
        .tool(tools::cloud::get_system_logs(state.clone()))
        .tool(tools::cloud::get_session_logs(state.clone()))
        // Cloud - Tasks
        .tool(tools::cloud::list_tasks(state.clone()))
        .tool(tools::cloud::get_task(state.clone()))
        // Cloud - Write Operations (require --read-only=false)
        .tool(tools::cloud::create_database(state.clone()))
        .tool(tools::cloud::update_database(state.clone()))
        .tool(tools::cloud::delete_database(state.clone()))
        .tool(tools::cloud::backup_database(state.clone()))
        .tool(tools::cloud::import_database(state.clone()))
        .tool(tools::cloud::delete_subscription(state.clone()))
        // Enterprise - Cluster
        .tool(tools::enterprise::get_cluster(state.clone()))
        .tool(tools::enterprise::get_cluster_stats(state.clone()))
        // Enterprise - License
        .tool(tools::enterprise::get_license(state.clone()))
        .tool(tools::enterprise::get_license_usage(state.clone()))
        // Enterprise - Logs
        .tool(tools::enterprise::list_logs(state.clone()))
        // Enterprise - Databases
        .tool(tools::enterprise::list_databases(state.clone()))
        .tool(tools::enterprise::get_database(state.clone()))
        .tool(tools::enterprise::get_database_stats(state.clone()))
        .tool(tools::enterprise::get_database_endpoints(state.clone()))
        .tool(tools::enterprise::list_database_alerts(state.clone()))
        // Enterprise - Nodes
        .tool(tools::enterprise::list_nodes(state.clone()))
        .tool(tools::enterprise::get_node(state.clone()))
        .tool(tools::enterprise::get_node_stats(state.clone()))
        // Enterprise - Users & Alerts
        .tool(tools::enterprise::list_users(state.clone()))
        .tool(tools::enterprise::get_user(state.clone()))
        .tool(tools::enterprise::list_alerts(state.clone()))
        .tool(tools::enterprise::list_shards(state.clone()))
        .tool(tools::enterprise::get_shard(state.clone()))
        // Enterprise - Aggregate Stats
        .tool(tools::enterprise::get_all_nodes_stats(state.clone()))
        .tool(tools::enterprise::get_all_databases_stats(state.clone()))
        .tool(tools::enterprise::get_shard_stats(state.clone()))
        .tool(tools::enterprise::get_all_shards_stats(state.clone()))
        // Enterprise - Debug Info
        .tool(tools::enterprise::list_debug_info_tasks(state.clone()))
        .tool(tools::enterprise::get_debug_info_status(state.clone()))
        // Enterprise - Modules
        .tool(tools::enterprise::list_modules(state.clone()))
        .tool(tools::enterprise::get_module(state.clone()))
        // Enterprise - Write Operations (require --read-only=false)
        .tool(tools::enterprise::backup_enterprise_database(state.clone()))
        .tool(tools::enterprise::import_enterprise_database(state.clone()))
        .tool(tools::enterprise::create_enterprise_database(state.clone()))
        .tool(tools::enterprise::update_enterprise_database(state.clone()))
        .tool(tools::enterprise::delete_enterprise_database(state.clone()))
        .tool(tools::enterprise::flush_enterprise_database(state.clone()))
        // Redis - Connection
        .tool(tools::redis::ping(state.clone()))
        .tool(tools::redis::info(state.clone()))
        .tool(tools::redis::dbsize(state.clone()))
        .tool(tools::redis::client_list(state.clone()))
        .tool(tools::redis::cluster_info(state.clone()))
        .tool(tools::redis::slowlog(state.clone()))
        // Redis - Keys
        .tool(tools::redis::keys(state.clone()))
        .tool(tools::redis::scan(state.clone()))
        .tool(tools::redis::get(state.clone()))
        .tool(tools::redis::key_type(state.clone()))
        .tool(tools::redis::ttl(state.clone()))
        .tool(tools::redis::exists(state.clone()))
        .tool(tools::redis::memory_usage(state.clone()))
        .tool(tools::redis::object_encoding(state.clone()))
        // Redis - Data Structures
        .tool(tools::redis::hgetall(state.clone()))
        .tool(tools::redis::lrange(state.clone()))
        .tool(tools::redis::smembers(state.clone()))
        .tool(tools::redis::zrange(state.clone()))
        // Profile Management - Read
        .tool(tools::profile::list_profiles(state.clone()))
        .tool(tools::profile::show_profile(state.clone()))
        .tool(tools::profile::config_path(state.clone()))
        .tool(tools::profile::validate_config(state.clone()))
        // Profile Management - Write
        .tool(tools::profile::set_default_cloud(state.clone()))
        .tool(tools::profile::set_default_enterprise(state.clone()))
        .tool(tools::profile::delete_profile(state.clone()))
        // Resources
        .resource(resources::config_path_resource())
        .resource(resources::profiles_resource())
        .resource(resources::help_resource())
        // Prompts
        .prompt(prompts::troubleshoot_database_prompt())
        .prompt(prompts::analyze_performance_prompt())
        .prompt(prompts::capacity_planning_prompt())
        .prompt(prompts::migration_planning_prompt());

    // Apply read-only filter if enabled
    // This hides write tools entirely from tools/list and returns "method not found"
    // if they're called directly, providing defense in depth beyond handler-level checks
    let router = if read_only {
        info!("Applying read-only filter - write tools will be hidden");
        router.tool_filter(
            CapabilityFilter::new(|_session, tool: &Tool| {
                // Only show tools that are marked as read-only
                tool.annotations
                    .as_ref()
                    .map(|a| a.read_only_hint)
                    .unwrap_or(false)
            })
            .denial_behavior(DenialBehavior::Unauthorized),
        )
    } else {
        router
    };

    Ok(router)
}

/// Run the HTTP server with middleware
#[cfg(feature = "http")]
async fn run_http_server(router: McpRouter, args: &Args) -> Result<()> {
    use std::time::Duration;
    use tower::limit::ConcurrencyLimitLayer;
    use tower::timeout::TimeoutLayer;
    use tower_mcp::HttpTransport;

    let addr = format!("{}:{}", args.host, args.port);

    let transport = HttpTransport::new(router)
        .layer(TimeoutLayer::new(Duration::from_secs(
            args.request_timeout_secs,
        )))
        .layer(ConcurrencyLimitLayer::new(args.max_concurrent));

    if args.oauth {
        // OAuth-enabled HTTP transport
        let _issuer = args
            .oauth_issuer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--oauth-issuer required when OAuth is enabled"))?;

        info!(issuer = %_issuer, "OAuth authentication enabled");

        // TODO: Configure OAuth with ProtectedResourceMetadata
        // transport = transport.oauth(metadata);
    }

    transport.serve(&addr).await?;

    Ok(())
}

#[cfg(not(feature = "http"))]
async fn run_http_server(_router: McpRouter, _args: &Args) -> Result<()> {
    anyhow::bail!("HTTP transport requires the 'http' feature")
}
