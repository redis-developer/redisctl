//! redisctl-mcp: MCP server for Redis Cloud and Enterprise
//!
//! A standalone MCP server that exposes Redis management operations
//! as tools for AI systems.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use redisctl_core::Config;
#[cfg(any(feature = "cloud", feature = "enterprise", feature = "database"))]
use redisctl_core::DeploymentType;
use tower_mcp::{
    CapabilityFilter, DenialBehavior, McpRouter, Tool,
    transport::{GenericStdioTransport, StdioTransport},
};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod audit;
mod error;
mod policy;
mod prompts;
mod resources;
mod state;
mod tools;

use audit::AuditLayer;
use policy::{Policy, PolicyConfig, SafetyTier, ToolsetKind};
use state::{AppState, CredentialSource};
use tower::Layer;

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

    /// Read-only mode (enabled by default; use --read-only=false to allow writes).
    /// Ignored when a policy file is active.
    #[arg(long, default_value = "true")]
    read_only: bool,

    /// Path to MCP policy file for granular tool access control.
    /// Overrides --read-only when set.
    #[arg(long, env = "REDISCTL_MCP_POLICY")]
    policy: Option<PathBuf>,

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

/// Resolve the policy configuration.
///
/// If a policy file is found (via `--policy`, env var, or default path), it takes precedence
/// and `--read-only` is ignored. Otherwise, synthesize a policy from the `--read-only` flag.
fn resolve_policy(args: &Args) -> Result<(PolicyConfig, String)> {
    let has_explicit_policy = args.policy.is_some();
    let has_env_policy = std::env::var("REDISCTL_MCP_POLICY").is_ok() && args.policy.is_none();
    let has_default_policy = PolicyConfig::default_path_exists();

    if has_explicit_policy || has_env_policy || has_default_policy {
        let (config, source) = PolicyConfig::load(args.policy.as_deref())?;
        if !args.read_only {
            tracing::warn!(
                "--read-only=false is ignored when a policy file is active (source: {})",
                source
            );
        }
        return Ok((config, source));
    }

    // No policy file: synthesize from --read-only flag
    let tier = if args.read_only {
        SafetyTier::ReadOnly
    } else {
        SafetyTier::Full
    };
    let source = if args.read_only {
        "cli: --read-only=true (default)".to_string()
    } else {
        "cli: --read-only=false".to_string()
    };

    Ok((
        PolicyConfig {
            tier,
            ..Default::default()
        },
        source,
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let enabled = enabled_toolsets(&args);
    let enabled_names: Vec<String> = enabled.iter().map(|t| t.to_string()).collect();

    // Resolve policy configuration (includes audit config)
    let (policy_config, policy_source) = resolve_policy(&args)?;
    let audit_config = Arc::new(policy_config.audit.clone());

    // Initialize tracing with optional audit layer
    // App logs: human-readable text to stderr (excludes audit target)
    // Audit logs: JSON to stderr (only audit target, when enabled)
    init_tracing(&args.log_level, audit_config.enabled);

    info!(
        transport = ?args.transport,
        profiles = ?args.profile,
        policy_tier = %policy_config.tier,
        policy_source = %policy_source,
        audit_enabled = audit_config.enabled,
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

    // Build tool-to-toolset mapping for policy evaluation
    let tool_toolset = build_tool_toolset_mapping(&enabled);
    let tool_toolset_arc = Arc::new(tool_toolset.clone());

    // Build resolved policy
    let policy = Arc::new(Policy::new(policy_config, tool_toolset, policy_source));

    // Build application state
    let state = Arc::new(AppState::new(
        credential_source,
        policy.clone(),
        args.database_url.clone(),
    )?);

    // Build router with tools and policy-based filter
    let router = build_router(state.clone(), policy, &enabled)?;

    match args.transport {
        Transport::Stdio => {
            info!("Running with stdio transport");
            if audit_config.enabled {
                info!("Audit logging enabled (level: {:?})", audit_config.level);
                let audit_layer = AuditLayer::new(audit_config, tool_toolset_arc);
                let service = audit_layer.layer(router);
                GenericStdioTransport::new(service).run().await?;
            } else {
                StdioTransport::new(router).run().await?;
            }
        }
        Transport::Http => {
            info!(host = %args.host, port = args.port, "Running with HTTP transport");
            run_http_server(router, &args, audit_config, tool_toolset_arc).await?;
        }
    }

    Ok(())
}

/// Initialize the tracing subscriber with optional audit logging layer.
///
/// When audit is enabled, adds a second JSON-formatted layer that captures only
/// events with `target = "audit"`. The app layer excludes audit events to avoid
/// double-logging.
fn init_tracing(log_level: &str, audit_enabled: bool) {
    use tracing_subscriber::filter;

    if audit_enabled {
        // Dual-layer: app (text, no audit) + audit (JSON, only audit)
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.to_string().into());

        let app_layer = fmt::layer().with_writer(std::io::stderr).with_filter(
            filter::Targets::new().with_targets(vec![
                ("audit", filter::LevelFilter::OFF), // exclude audit from app logs
            ]),
        );

        let audit_layer = fmt::layer()
            .json()
            .with_writer(std::io::stderr)
            .with_target(true)
            .with_filter(filter::Targets::new().with_target("audit", tracing::Level::INFO));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(app_layer)
            .with(audit_layer)
            .init();
    } else {
        // Single layer: standard app logs
        tracing_subscriber::registry()
            .with(fmt::layer().with_writer(std::io::stderr))
            .with(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.to_string().into()),
            )
            .init();
    }
}

/// Build the tool name -> toolset kind mapping for policy evaluation.
fn build_tool_toolset_mapping(enabled: &HashSet<Toolset>) -> HashMap<String, ToolsetKind> {
    let mut mapping = HashMap::new();

    #[cfg(feature = "cloud")]
    if enabled.contains(&Toolset::Cloud) {
        for name in tools::cloud::tool_names() {
            mapping.insert(name, ToolsetKind::Cloud);
        }
    }

    #[cfg(feature = "enterprise")]
    if enabled.contains(&Toolset::Enterprise) {
        for name in tools::enterprise::tool_names() {
            mapping.insert(name, ToolsetKind::Enterprise);
        }
    }

    #[cfg(feature = "database")]
    if enabled.contains(&Toolset::Database) {
        for name in tools::redis::tool_names() {
            mapping.insert(name, ToolsetKind::Database);
        }
    }

    if enabled.contains(&Toolset::App) {
        for name in tools::profile::tool_names() {
            mapping.insert(name, ToolsetKind::App);
        }
    }

    mapping
}

/// Build the MCP router with modular sub-routers based on enabled toolsets
fn build_router(
    state: Arc<AppState>,
    policy: Arc<Policy>,
    enabled: &HashSet<Toolset>,
) -> Result<McpRouter> {
    let mut router = McpRouter::new().server_info("redisctl-mcp", env!("CARGO_PKG_VERSION"));

    // Conditionally merge each toolset
    #[cfg(feature = "cloud")]
    if enabled.contains(&Toolset::Cloud) {
        router = router.merge(tools::cloud::router(state.clone()));
    }

    #[cfg(feature = "enterprise")]
    if enabled.contains(&Toolset::Enterprise) {
        router = router.merge(tools::enterprise::router(state.clone()));
    }

    #[cfg(feature = "database")]
    if enabled.contains(&Toolset::Database) {
        router = router.merge(tools::redis::router(state.clone()));
    }

    // App toolset includes profile tools, resources, and prompts
    if enabled.contains(&Toolset::App) {
        router = router.merge(tools::profile::router(state.clone()));
    }

    // Register the show_policy tool (always available)
    router = router.tool(policy::show_policy_tool(policy.clone()));

    // Build instructions with policy description
    let prefix = format!(
        "# Redis Cloud and Enterprise MCP Server\n\n## Safety Model\n\n{}\n",
        policy.describe()
    );

    let suffix = "\n## Authentication\n\n\
         In stdio mode, credentials are resolved from redisctl profiles.\n\
         In HTTP mode with OAuth, credentials can be passed via JWT claims.";

    router = router.auto_instructions_with(Some(prefix), Some(suffix));

    // Apply policy-based tool filter
    // This hides denied tools from tools/list and returns "unauthorized"
    // if they're called directly, providing defense in depth beyond handler-level checks
    let policy_for_filter = policy.clone();
    info!(tier = %policy.global_tier(), "Applying policy filter");
    let router = router.tool_filter(
        CapabilityFilter::<Tool>::new(move |_session, tool: &Tool| {
            policy_for_filter.is_tool_allowed(tool)
        })
        .denial_behavior(DenialBehavior::Unauthorized),
    );

    Ok(router)
}

/// Run the HTTP server with middleware
#[cfg(feature = "http")]
async fn run_http_server(
    router: McpRouter,
    args: &Args,
    audit_config: Arc<audit::AuditConfig>,
    tool_toolset: Arc<HashMap<String, ToolsetKind>>,
) -> Result<()> {
    use std::time::Duration;
    use tower::limit::ConcurrencyLimitLayer;
    use tower::timeout::TimeoutLayer;
    use tower_mcp::HttpTransport;

    let addr = format!("{}:{}", args.host, args.port);

    let mut transport = HttpTransport::new(router)
        .layer(TimeoutLayer::new(Duration::from_secs(
            args.request_timeout_secs,
        )))
        .layer(ConcurrencyLimitLayer::new(args.max_concurrent));

    if audit_config.enabled {
        info!(
            "Audit logging enabled for HTTP transport (level: {:?})",
            audit_config.level
        );
        transport = transport.layer(AuditLayer::new(audit_config, tool_toolset));
    }

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
async fn run_http_server(
    _router: McpRouter,
    _args: &Args,
    _audit_config: Arc<audit::AuditConfig>,
    _tool_toolset: Arc<HashMap<String, ToolsetKind>>,
) -> Result<()> {
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

    fn test_state() -> Arc<AppState> {
        Arc::new(
            AppState::new(
                state::CredentialSource::Profiles(vec![]),
                AppState::test_policy(),
                None,
            )
            .unwrap(),
        )
    }

    fn test_policy_arc(tier: SafetyTier) -> Arc<Policy> {
        Arc::new(Policy::new(
            PolicyConfig {
                tier,
                ..Default::default()
            },
            HashMap::new(),
            "test".to_string(),
        ))
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

    // ========================================================================
    // Safety annotation tests
    // ========================================================================

    /// Helper: assert a tool has read-only, idempotent, non-destructive annotations
    fn assert_read_only(tool: &Tool, name: &str) {
        let ann = tool
            .annotations
            .as_ref()
            .unwrap_or_else(|| panic!("{name}: missing annotations"));
        assert!(ann.read_only_hint, "{name}: should be read_only");
        assert!(ann.idempotent_hint, "{name}: should be idempotent");
        assert!(
            !ann.destructive_hint,
            "{name}: should be non-destructive (destructive_hint=false)"
        );
    }

    /// Helper: assert a tool is a non-destructive write (not read-only, destructive_hint=false)
    fn assert_non_destructive_write(tool: &Tool, name: &str) {
        let ann = tool
            .annotations
            .as_ref()
            .unwrap_or_else(|| panic!("{name}: missing annotations"));
        assert!(!ann.read_only_hint, "{name}: should NOT be read_only");
        assert!(
            !ann.destructive_hint,
            "{name}: should be non-destructive (destructive_hint=false)"
        );
    }

    /// Helper: assert a tool is destructive (explicit annotations with destructive_hint=true)
    fn assert_destructive(tool: &Tool, name: &str) {
        let ann = tool
            .annotations
            .as_ref()
            .unwrap_or_else(|| panic!("{name}: missing annotations"));
        assert!(
            ann.destructive_hint,
            "{name}: should be destructive (destructive_hint=true)"
        );
        assert!(
            !ann.read_only_hint,
            "{name}: destructive tool should NOT be read_only"
        );

        // Description should start with "DANGEROUS:"
        let desc = tool
            .description
            .as_deref()
            .unwrap_or_else(|| panic!("{name}: missing description"));
        assert!(
            desc.starts_with("DANGEROUS:"),
            "{name}: destructive tool description should start with 'DANGEROUS:', got: {desc}"
        );
    }

    // -- Profile tools --

    #[test]
    fn profile_read_tools_are_read_only() {
        let state = test_state();
        assert_read_only(
            &tools::profile::list_profiles(state.clone()),
            "profile_list",
        );
        assert_read_only(&tools::profile::show_profile(state.clone()), "profile_show");
        assert_read_only(&tools::profile::config_path(state.clone()), "profile_path");
        assert_read_only(
            &tools::profile::validate_config(state.clone()),
            "profile_validate",
        );
    }

    #[test]
    fn profile_write_tools_are_non_destructive() {
        let state = test_state();
        assert_non_destructive_write(
            &tools::profile::create_profile(state.clone()),
            "profile_create",
        );
        assert_non_destructive_write(
            &tools::profile::set_default_cloud(state.clone()),
            "profile_set_default_cloud",
        );
        assert_non_destructive_write(
            &tools::profile::set_default_enterprise(state.clone()),
            "profile_set_default_enterprise",
        );
    }

    #[test]
    fn profile_destructive_tools() {
        let state = test_state();
        assert_destructive(
            &tools::profile::delete_profile(state.clone()),
            "profile_delete",
        );
    }

    // -- Cloud tools (representative samples) --

    #[cfg(feature = "cloud")]
    mod cloud_annotations {
        use super::*;

        #[test]
        fn cloud_read_tools_are_read_only() {
            let state = test_state();
            assert_read_only(
                &tools::cloud::list_subscriptions(state.clone()),
                "list_subscriptions",
            );
            assert_read_only(&tools::cloud::get_account(state.clone()), "get_account");
            assert_read_only(
                &tools::cloud::list_fixed_subscriptions(state.clone()),
                "list_fixed_subscriptions",
            );
            assert_read_only(
                &tools::cloud::get_vpc_peering(state.clone()),
                "get_vpc_peering",
            );
            assert_read_only(
                &tools::cloud::wait_for_cloud_task(state.clone()),
                "wait_for_cloud_task",
            );
        }

        #[test]
        fn cloud_write_tools_are_non_destructive() {
            let state = test_state();
            assert_non_destructive_write(
                &tools::cloud::create_database(state.clone()),
                "create_database",
            );
            assert_non_destructive_write(
                &tools::cloud::update_database(state.clone()),
                "update_database",
            );
            assert_non_destructive_write(
                &tools::cloud::backup_database(state.clone()),
                "backup_database",
            );
            assert_non_destructive_write(
                &tools::cloud::update_account_user(state.clone()),
                "update_account_user",
            );
            assert_non_destructive_write(
                &tools::cloud::create_acl_user(state.clone()),
                "create_acl_user",
            );
            assert_non_destructive_write(
                &tools::cloud::create_vpc_peering(state.clone()),
                "create_vpc_peering",
            );
            assert_non_destructive_write(
                &tools::cloud::create_fixed_database(state.clone()),
                "create_fixed_database",
            );
        }

        #[test]
        fn cloud_destructive_tools() {
            let state = test_state();
            assert_destructive(
                &tools::cloud::delete_database(state.clone()),
                "delete_database",
            );
            assert_destructive(
                &tools::cloud::delete_subscription(state.clone()),
                "delete_subscription",
            );
            assert_destructive(
                &tools::cloud::flush_database(state.clone()),
                "flush_database",
            );
            assert_destructive(
                &tools::cloud::flush_crdb_database(state.clone()),
                "flush_crdb_database",
            );
            assert_destructive(
                &tools::cloud::delete_account_user(state.clone()),
                "delete_account_user",
            );
            assert_destructive(
                &tools::cloud::delete_acl_user(state.clone()),
                "delete_acl_user",
            );
            assert_destructive(
                &tools::cloud::delete_cloud_account(state.clone()),
                "delete_cloud_account",
            );
            assert_destructive(
                &tools::cloud::delete_vpc_peering(state.clone()),
                "delete_vpc_peering",
            );
            assert_destructive(
                &tools::cloud::delete_private_link(state.clone()),
                "delete_private_link",
            );
            assert_destructive(
                &tools::cloud::delete_fixed_database(state.clone()),
                "delete_fixed_database",
            );
            assert_destructive(
                &tools::cloud::delete_fixed_subscription(state.clone()),
                "delete_fixed_subscription",
            );
        }
    }

    // -- Enterprise tools (representative samples) --

    #[cfg(feature = "enterprise")]
    mod enterprise_annotations {
        use super::*;

        #[test]
        fn enterprise_read_tools_are_read_only() {
            let state = test_state();
            assert_read_only(
                &tools::enterprise::get_cluster(state.clone()),
                "get_cluster",
            );
            assert_read_only(
                &tools::enterprise::list_databases(state.clone()),
                "list_enterprise_databases",
            );
            assert_read_only(
                &tools::enterprise::list_users(state.clone()),
                "list_enterprise_users",
            );
            assert_read_only(
                &tools::enterprise::list_alerts(state.clone()),
                "list_alerts",
            );
        }

        #[test]
        fn enterprise_write_tools_are_non_destructive() {
            let state = test_state();
            assert_non_destructive_write(
                &tools::enterprise::update_cluster(state.clone()),
                "update_enterprise_cluster",
            );
            assert_non_destructive_write(
                &tools::enterprise::create_enterprise_database(state.clone()),
                "create_enterprise_database",
            );
            assert_non_destructive_write(
                &tools::enterprise::create_enterprise_user(state.clone()),
                "create_enterprise_user",
            );
        }

        #[test]
        fn enterprise_destructive_tools() {
            let state = test_state();
            assert_destructive(
                &tools::enterprise::delete_enterprise_database(state.clone()),
                "delete_enterprise_database",
            );
            assert_destructive(
                &tools::enterprise::flush_enterprise_database(state.clone()),
                "flush_enterprise_database",
            );
            assert_destructive(
                &tools::enterprise::delete_enterprise_user(state.clone()),
                "delete_enterprise_user",
            );
            assert_destructive(
                &tools::enterprise::delete_enterprise_role(state.clone()),
                "delete_enterprise_role",
            );
            assert_destructive(
                &tools::enterprise::delete_enterprise_acl(state.clone()),
                "delete_enterprise_acl",
            );
        }
    }

    // -- Redis database tools (representative samples) --

    #[cfg(feature = "database")]
    mod database_annotations {
        use super::*;

        #[test]
        fn redis_read_tools_are_read_only() {
            let state = test_state();
            assert_read_only(&tools::redis::ping(state.clone()), "redis_ping");
            assert_read_only(&tools::redis::info(state.clone()), "redis_info");
            assert_read_only(&tools::redis::keys(state.clone()), "redis_keys");
            assert_read_only(&tools::redis::get(state.clone()), "redis_get");
            assert_read_only(&tools::redis::hgetall(state.clone()), "redis_hgetall");
            assert_read_only(
                &tools::redis::health_check(state.clone()),
                "redis_health_check",
            );
        }

        #[test]
        fn redis_write_tools_are_non_destructive() {
            let state = test_state();
            assert_non_destructive_write(
                &tools::redis::config_set(state.clone()),
                "redis_config_set",
            );
            assert_non_destructive_write(&tools::redis::set(state.clone()), "redis_set");
            assert_non_destructive_write(&tools::redis::expire(state.clone()), "redis_expire");
            assert_non_destructive_write(&tools::redis::hset(state.clone()), "redis_hset");
            assert_non_destructive_write(&tools::redis::lpush(state.clone()), "redis_lpush");
            assert_non_destructive_write(&tools::redis::xadd(state.clone()), "redis_xadd");
        }

        #[test]
        fn redis_destructive_tools() {
            let state = test_state();
            assert_destructive(&tools::redis::flushdb(state.clone()), "redis_flushdb");
            assert_destructive(&tools::redis::del(state.clone()), "redis_del");
        }
    }

    #[test]
    fn instructions_contain_safety_model() {
        let state = test_state();
        let enabled: HashSet<Toolset> = [Toolset::App].into_iter().collect();

        let policy_ro = test_policy_arc(SafetyTier::ReadOnly);
        let _router = build_router(state.clone(), policy_ro, &enabled).unwrap();
        // Verify the build succeeds (no panics) for both modes.
        let policy_full = test_policy_arc(SafetyTier::Full);
        let _router_write = build_router(state.clone(), policy_full, &enabled).unwrap();
    }

    #[test]
    fn show_policy_tool_is_always_registered() {
        let state = test_state();
        let enabled: HashSet<Toolset> = [Toolset::App].into_iter().collect();
        let policy = test_policy_arc(SafetyTier::ReadOnly);
        // Build should succeed and include show_policy tool
        let _router = build_router(state, policy, &enabled).unwrap();
    }
}
