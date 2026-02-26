//! redisctl-mcp: MCP server for Redis Cloud and Enterprise
//!
//! A standalone MCP server that exposes Redis management operations
//! as tools for AI systems.

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use redisctl_core::Config;
#[cfg(any(feature = "cloud", feature = "enterprise", feature = "database"))]
use redisctl_core::DeploymentType;
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

/// Toolsets that can be enabled or disabled at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
enum Toolset {
    /// Redis Cloud management tools
    #[cfg(feature = "cloud")]
    Cloud,
    /// Redis Enterprise management tools
    #[cfg(feature = "enterprise")]
    Enterprise,
    /// Direct Redis database tools
    #[cfg(feature = "database")]
    Database,
    /// App-level tools: profile management, resources, and prompts
    App,
}

impl std::fmt::Display for Toolset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "cloud")]
            Toolset::Cloud => write!(f, "cloud"),
            #[cfg(feature = "enterprise")]
            Toolset::Enterprise => write!(f, "enterprise"),
            #[cfg(feature = "database")]
            Toolset::Database => write!(f, "database"),
            Toolset::App => write!(f, "app"),
        }
    }
}

/// MCP server for Redis Cloud and Enterprise management
#[derive(Parser, Debug)]
#[command(name = "redisctl-mcp")]
#[command(version, about, long_about = None)]
struct Args {
    /// Transport mode
    #[arg(short, long, value_enum, default_value = "stdio")]
    transport: Transport,

    /// Profile name(s) for local credential resolution. Can be specified multiple times.
    #[arg(short, long, env = "REDISCTL_PROFILE")]
    profile: Vec<String>,

    /// Read-only mode (enabled by default; use --read-only=false to allow writes)
    #[arg(long, default_value = "true")]
    read_only: bool,

    /// Redis database URL for direct connections
    #[arg(long, env = "REDIS_URL")]
    database_url: Option<String>,

    /// Toolsets to enable (default: all compiled-in). Options: cloud, enterprise, database, app.
    #[arg(long, value_delimiter = ',', value_enum)]
    tools: Option<Vec<Toolset>>,

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

/// Derive which toolsets to enable based on profile types in the config.
/// Returns `None` if config has no profiles (caller should fall back to all compiled-in).
fn toolsets_from_config(config: &Config) -> Option<HashSet<Toolset>> {
    if config.profiles.is_empty() {
        return None;
    }

    let mut set = HashSet::new();
    set.insert(Toolset::App); // always include profile management

    #[cfg(feature = "cloud")]
    if !config
        .get_profiles_of_type(DeploymentType::Cloud)
        .is_empty()
    {
        set.insert(Toolset::Cloud);
    }
    #[cfg(feature = "enterprise")]
    if !config
        .get_profiles_of_type(DeploymentType::Enterprise)
        .is_empty()
    {
        set.insert(Toolset::Enterprise);
    }
    #[cfg(feature = "database")]
    if !config
        .get_profiles_of_type(DeploymentType::Database)
        .is_empty()
    {
        set.insert(Toolset::Database);
    }

    Some(set)
}

/// Try to auto-detect toolsets from the config file on disk.
/// Returns `None` if config cannot be loaded or has no profiles.
fn detect_toolsets_from_config() -> Option<HashSet<Toolset>> {
    let config = Config::load().ok()?;
    let result = toolsets_from_config(&config);
    if let Some(ref toolsets) = result {
        let names: Vec<String> = toolsets.iter().map(|t| t.to_string()).collect();
        info!(toolsets = ?names, "Auto-detected toolsets from config profiles");
    }
    result
}

/// Resolve which toolsets are enabled based on CLI args, config profiles, and compiled features.
///
/// Priority: explicit `--tools` flag > config-based auto-detection > all compiled-in features.
fn enabled_toolsets(args: &Args) -> HashSet<Toolset> {
    // 1. Explicit --tools flag always wins
    if let Some(ref tools) = args.tools {
        return tools.iter().copied().collect();
    }

    // 2. Auto-detect from config profiles
    if let Some(toolsets) = detect_toolsets_from_config() {
        return toolsets;
    }

    // 3. Fallback: all compiled-in features
    let mut set = HashSet::new();
    #[cfg(feature = "cloud")]
    set.insert(Toolset::Cloud);
    #[cfg(feature = "enterprise")]
    set.insert(Toolset::Enterprise);
    #[cfg(feature = "database")]
    set.insert(Toolset::Database);
    set.insert(Toolset::App);
    set
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| args.log_level.clone().into()))
        .init();

    let enabled = enabled_toolsets(&args);
    let enabled_names: Vec<String> = enabled.iter().map(|t| t.to_string()).collect();

    info!(
        transport = ?args.transport,
        profiles = ?args.profile,
        read_only = args.read_only,
        toolsets = ?enabled_names,
        "Starting redisctl-mcp server"
    );

    // Determine credential source
    let credential_source = if args.oauth {
        CredentialSource::OAuth {
            issuer: args.oauth_issuer.clone(),
            audience: args.oauth_audience.clone(),
        }
    } else {
        CredentialSource::Profiles(args.profile.clone())
    };

    // Build application state
    let state = Arc::new(AppState::new(
        credential_source,
        args.read_only,
        args.database_url.clone(),
    )?);

    // Build router with tools and optional read-only filter
    let router = build_router(state.clone(), args.read_only, &enabled)?;

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

/// Footer instructions for authentication
const AUTH_INSTRUCTIONS: &str = r#"
## Authentication

In stdio mode, credentials are resolved from redisctl profiles.
In HTTP mode with OAuth, credentials can be passed via JWT claims.
"#;

/// Build the MCP router with modular sub-routers based on enabled toolsets
fn build_router(
    state: Arc<AppState>,
    read_only: bool,
    enabled: &HashSet<Toolset>,
) -> Result<McpRouter> {
    let mut instructions = String::from(
        r#"Redis Cloud and Enterprise MCP Server

This server provides comprehensive tools for managing Redis Cloud subscriptions and databases,
Redis Enterprise clusters and databases, and direct Redis database operations.

## Available Tool Categories
"#,
    );

    let mut router = McpRouter::new().server_info("redisctl-mcp", env!("CARGO_PKG_VERSION"));

    // Conditionally merge each toolset
    #[cfg(feature = "cloud")]
    if enabled.contains(&Toolset::Cloud) {
        router = router.merge(tools::cloud::router(state.clone()));
        instructions.push_str(tools::cloud::instructions());
    }

    #[cfg(feature = "enterprise")]
    if enabled.contains(&Toolset::Enterprise) {
        router = router.merge(tools::enterprise::router(state.clone()));
        instructions.push_str(tools::enterprise::instructions());
    }

    #[cfg(feature = "database")]
    if enabled.contains(&Toolset::Database) {
        router = router.merge(tools::redis::router(state.clone()));
        instructions.push_str(tools::redis::instructions());
    }

    // App toolset includes profile tools, resources, and prompts
    if enabled.contains(&Toolset::App) {
        router = router.merge(tools::profile::router(state.clone()));
        instructions.push_str(tools::profile::instructions());
    }

    instructions.push_str(AUTH_INSTRUCTIONS);
    router = router.instructions(&instructions);

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

#[cfg(test)]
mod tests {
    use super::*;
    use redisctl_core::{Profile, ProfileCredentials};

    fn cloud_profile() -> Profile {
        Profile {
            deployment_type: DeploymentType::Cloud,
            credentials: ProfileCredentials::Cloud {
                api_key: "key".to_string(),
                api_secret: "secret".to_string(),
                api_url: "https://api.redislabs.com/v1".to_string(),
            },
            files_api_key: None,
            resilience: None,
            tags: vec![],
        }
    }

    fn enterprise_profile() -> Profile {
        Profile {
            deployment_type: DeploymentType::Enterprise,
            credentials: ProfileCredentials::Enterprise {
                url: "https://localhost:9443".to_string(),
                username: "admin".to_string(),
                password: Some("password".to_string()),
                insecure: false,
                ca_cert: None,
            },
            files_api_key: None,
            resilience: None,
            tags: vec![],
        }
    }

    fn database_profile() -> Profile {
        Profile {
            deployment_type: DeploymentType::Database,
            credentials: ProfileCredentials::Database {
                host: "localhost".to_string(),
                port: 6379,
                password: None,
                tls: false,
                username: "default".to_string(),
                database: 0,
            },
            files_api_key: None,
            resilience: None,
            tags: vec![],
        }
    }

    #[test]
    fn empty_config_returns_none() {
        let config = Config::default();
        assert!(toolsets_from_config(&config).is_none());
    }

    #[test]
    fn cloud_only_profiles() {
        let mut config = Config::default();
        config.set_profile("mycloud".to_string(), cloud_profile());

        let toolsets = toolsets_from_config(&config).unwrap();
        assert!(toolsets.contains(&Toolset::App));
        #[cfg(feature = "cloud")]
        assert!(toolsets.contains(&Toolset::Cloud));
        #[cfg(feature = "enterprise")]
        assert!(!toolsets.contains(&Toolset::Enterprise));
        #[cfg(feature = "database")]
        assert!(!toolsets.contains(&Toolset::Database));
    }

    #[test]
    fn enterprise_only_profiles() {
        let mut config = Config::default();
        config.set_profile("myent".to_string(), enterprise_profile());

        let toolsets = toolsets_from_config(&config).unwrap();
        assert!(toolsets.contains(&Toolset::App));
        #[cfg(feature = "cloud")]
        assert!(!toolsets.contains(&Toolset::Cloud));
        #[cfg(feature = "enterprise")]
        assert!(toolsets.contains(&Toolset::Enterprise));
        #[cfg(feature = "database")]
        assert!(!toolsets.contains(&Toolset::Database));
    }

    #[test]
    fn cloud_and_enterprise_profiles() {
        let mut config = Config::default();
        config.set_profile("mycloud".to_string(), cloud_profile());
        config.set_profile("myent".to_string(), enterprise_profile());

        let toolsets = toolsets_from_config(&config).unwrap();
        assert!(toolsets.contains(&Toolset::App));
        #[cfg(feature = "cloud")]
        assert!(toolsets.contains(&Toolset::Cloud));
        #[cfg(feature = "enterprise")]
        assert!(toolsets.contains(&Toolset::Enterprise));
        #[cfg(feature = "database")]
        assert!(!toolsets.contains(&Toolset::Database));
    }

    #[test]
    fn database_only_profiles() {
        let mut config = Config::default();
        config.set_profile("mydb".to_string(), database_profile());

        let toolsets = toolsets_from_config(&config).unwrap();
        assert!(toolsets.contains(&Toolset::App));
        #[cfg(feature = "cloud")]
        assert!(!toolsets.contains(&Toolset::Cloud));
        #[cfg(feature = "enterprise")]
        assert!(!toolsets.contains(&Toolset::Enterprise));
        #[cfg(feature = "database")]
        assert!(toolsets.contains(&Toolset::Database));
    }

    #[test]
    fn all_three_profile_types() {
        let mut config = Config::default();
        config.set_profile("mycloud".to_string(), cloud_profile());
        config.set_profile("myent".to_string(), enterprise_profile());
        config.set_profile("mydb".to_string(), database_profile());

        let toolsets = toolsets_from_config(&config).unwrap();
        assert!(toolsets.contains(&Toolset::App));
        #[cfg(feature = "cloud")]
        assert!(toolsets.contains(&Toolset::Cloud));
        #[cfg(feature = "enterprise")]
        assert!(toolsets.contains(&Toolset::Enterprise));
        #[cfg(feature = "database")]
        assert!(toolsets.contains(&Toolset::Database));
    }
}
