//! Application state and credential resolution

#[cfg(any(feature = "cloud", feature = "enterprise"))]
use std::collections::HashMap;

#[cfg(any(feature = "cloud", feature = "enterprise", feature = "database"))]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "cloud")]
use redis_cloud::CloudClient;
#[cfg(feature = "enterprise")]
use redis_enterprise::EnterpriseClient;
use redisctl_core::Config;
use tokio::sync::RwLock;

/// How credentials are resolved
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CredentialSource {
    /// Resolve from redisctl profiles (local mode)
    /// Empty vec means use default profiles from config
    Profiles(Vec<String>),
    /// Resolve from OAuth token claims (HTTP mode)
    OAuth {
        issuer: Option<String>,
        audience: Option<String>,
    },
}

/// Cached API clients (per-profile for multi-cluster support)
pub struct CachedClients {
    #[cfg(feature = "cloud")]
    pub cloud: HashMap<String, CloudClient>,
    #[cfg(feature = "enterprise")]
    pub enterprise: HashMap<String, EnterpriseClient>,
}

/// Shared application state
pub struct AppState {
    /// Credential source configuration
    pub credential_source: CredentialSource,
    /// Read-only mode flag
    pub read_only: bool,
    /// Optional Redis database URL for direct connections
    pub database_url: Option<String>,
    /// redisctl config (for profile-based auth)
    config: Option<Config>,
    /// Configured profiles (for multi-cluster support)
    profiles: Vec<String>,
    /// Cached API clients (keyed by profile name, "_default" for default)
    #[allow(dead_code)]
    clients: RwLock<CachedClients>,
}

impl AppState {
    /// Create new application state
    pub fn new(
        credential_source: CredentialSource,
        read_only: bool,
        database_url: Option<String>,
    ) -> Result<Self> {
        // Extract profiles list
        let profiles = match &credential_source {
            CredentialSource::Profiles(p) => p.clone(),
            CredentialSource::OAuth { .. } => vec![],
        };

        // Load config if using profile-based auth
        let config = match &credential_source {
            CredentialSource::Profiles(_) => Config::load().ok(),
            CredentialSource::OAuth { .. } => None,
        };

        Ok(Self {
            credential_source,
            read_only,
            database_url,
            config,
            profiles,
            clients: RwLock::new(CachedClients {
                #[cfg(feature = "cloud")]
                cloud: HashMap::new(),
                #[cfg(feature = "enterprise")]
                enterprise: HashMap::new(),
            }),
        })
    }

    /// Get the list of configured profiles
    #[allow(dead_code)]
    pub fn available_profiles(&self) -> &[String] {
        &self.profiles
    }

    /// Get or create Cloud API client for a specific profile
    ///
    /// If profile is None, uses the first configured profile or default from config
    #[cfg(feature = "cloud")]
    pub async fn cloud_client_for_profile(&self, profile: Option<&str>) -> Result<CloudClient> {
        let cache_key = profile.unwrap_or("_default").to_string();

        // Check cache first
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.cloud.get(&cache_key) {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client = self.create_cloud_client(profile).await?;

        // Cache it
        {
            let mut clients = self.clients.write().await;
            clients.cloud.insert(cache_key, client.clone());
        }

        Ok(client)
    }

    /// Get or create Cloud API client (uses default profile)
    #[cfg(feature = "cloud")]
    #[allow(dead_code)]
    pub async fn cloud_client(&self) -> Result<CloudClient> {
        self.cloud_client_for_profile(None).await
    }

    /// Get or create Enterprise API client for a specific profile
    ///
    /// If profile is None, uses the first configured profile or default from config
    #[cfg(feature = "enterprise")]
    pub async fn enterprise_client_for_profile(
        &self,
        profile: Option<&str>,
    ) -> Result<EnterpriseClient> {
        let cache_key = profile.unwrap_or("_default").to_string();

        // Check cache first
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.enterprise.get(&cache_key) {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client = self.create_enterprise_client(profile).await?;

        // Cache it
        {
            let mut clients = self.clients.write().await;
            clients.enterprise.insert(cache_key, client.clone());
        }

        Ok(client)
    }

    /// Get or create Enterprise API client (uses default profile)
    #[cfg(feature = "enterprise")]
    #[allow(dead_code)]
    pub async fn enterprise_client(&self) -> Result<EnterpriseClient> {
        self.enterprise_client_for_profile(None).await
    }

    /// Create a new Cloud client from credentials
    #[cfg(feature = "cloud")]
    async fn create_cloud_client(&self, profile: Option<&str>) -> Result<CloudClient> {
        match &self.credential_source {
            CredentialSource::Profiles(profiles) => {
                let config = self
                    .config
                    .as_ref()
                    .context("No redisctl config available")?;

                // Use specified profile, first configured profile, or let config resolve default
                let profile_to_use = profile
                    .map(|s| s.to_string())
                    .or_else(|| profiles.first().cloned());

                // Resolve the profile name
                let resolved_profile_name = config
                    .resolve_cloud_profile(profile_to_use.as_deref())
                    .context("Failed to resolve cloud profile")?;

                // Get the profile
                let profile = config
                    .profiles
                    .get(&resolved_profile_name)
                    .with_context(|| format!("Profile '{}' not found", resolved_profile_name))?;

                // Get credentials
                let (api_key, api_secret, _base_url) = profile
                    .resolve_cloud_credentials()
                    .context("Failed to resolve cloud credentials")?
                    .context("No cloud credentials in profile")?;

                CloudClient::builder()
                    .api_key(api_key)
                    .api_secret(api_secret)
                    .build()
                    .context("Failed to build Cloud client")
            }
            CredentialSource::OAuth { .. } => {
                // In OAuth mode, credentials come from environment variables
                let api_key =
                    std::env::var("REDIS_CLOUD_API_KEY").context("REDIS_CLOUD_API_KEY not set")?;
                let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")
                    .context("REDIS_CLOUD_API_SECRET not set")?;

                CloudClient::builder()
                    .api_key(api_key)
                    .api_secret(api_secret)
                    .build()
                    .context("Failed to build Cloud client")
            }
        }
    }

    /// Create a new Enterprise client from credentials
    #[cfg(feature = "enterprise")]
    async fn create_enterprise_client(&self, profile: Option<&str>) -> Result<EnterpriseClient> {
        match &self.credential_source {
            CredentialSource::Profiles(profiles) => {
                let config = self
                    .config
                    .as_ref()
                    .context("No redisctl config available")?;

                // Use specified profile, first configured profile, or let config resolve default
                let profile_to_use = profile
                    .map(|s| s.to_string())
                    .or_else(|| profiles.first().cloned());

                // Resolve the profile name
                let resolved_profile_name = config
                    .resolve_enterprise_profile(profile_to_use.as_deref())
                    .context("Failed to resolve enterprise profile")?;

                // Get the profile
                let profile_config = config
                    .profiles
                    .get(&resolved_profile_name)
                    .with_context(|| format!("Profile '{}' not found", resolved_profile_name))?;

                // Get credentials
                let (url, username, password, insecure, ca_cert) = profile_config
                    .resolve_enterprise_credentials()
                    .context("Failed to resolve enterprise credentials")?
                    .context("No enterprise credentials in profile")?;

                let mut builder = EnterpriseClient::builder()
                    .base_url(&url)
                    .username(&username)
                    .insecure(insecure);

                if let Some(pwd) = password {
                    builder = builder.password(&pwd);
                }

                if let Some(cert_path) = ca_cert {
                    builder = builder.ca_cert(&cert_path);
                }

                builder.build().context("Failed to build Enterprise client")
            }
            CredentialSource::OAuth { .. } => {
                // In OAuth mode, credentials come from environment variables
                let url = std::env::var("REDIS_ENTERPRISE_URL")
                    .context("REDIS_ENTERPRISE_URL not set")?;
                let username = std::env::var("REDIS_ENTERPRISE_USER")
                    .context("REDIS_ENTERPRISE_USER not set")?;
                let password = std::env::var("REDIS_ENTERPRISE_PASSWORD").ok();
                let insecure = std::env::var("REDIS_ENTERPRISE_INSECURE")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false);

                let mut builder = EnterpriseClient::builder()
                    .base_url(&url)
                    .username(&username)
                    .insecure(insecure);

                if let Some(pwd) = password {
                    builder = builder.password(&pwd);
                }

                builder.build().context("Failed to build Enterprise client")
            }
        }
    }

    /// Get Redis connection for direct database operations
    #[cfg(feature = "database")]
    #[allow(dead_code)]
    pub async fn redis_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        let url = self
            .database_url
            .as_ref()
            .cloned()
            .or_else(|| std::env::var("REDIS_URL").ok())
            .context("No Redis URL configured")?;

        let client = redis::Client::open(url.as_str()).context("Failed to create Redis client")?;

        client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to connect to Redis")
    }

    /// Check if write operations are allowed
    #[allow(dead_code)]
    pub fn is_write_allowed(&self) -> bool {
        !self.read_only
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        // Note: We don't clone the clients cache, each clone gets fresh cache
        Self {
            credential_source: self.credential_source.clone(),
            read_only: self.read_only,
            database_url: self.database_url.clone(),
            config: self.config.clone(),
            profiles: self.profiles.clone(),
            clients: RwLock::new(CachedClients {
                #[cfg(feature = "cloud")]
                cloud: HashMap::new(),
                #[cfg(feature = "enterprise")]
                enterprise: HashMap::new(),
            }),
        }
    }
}

/// Test helpers for creating AppState with pre-configured clients
#[allow(dead_code)]
impl AppState {
    /// Create test state with a pre-configured Cloud client
    #[cfg(feature = "cloud")]
    pub fn with_cloud_client(client: CloudClient) -> Self {
        let mut cloud = HashMap::new();
        cloud.insert("_default".to_string(), client);
        Self {
            credential_source: CredentialSource::Profiles(vec![]),
            read_only: true,
            database_url: None,
            config: None,
            profiles: vec![],
            clients: RwLock::new(CachedClients {
                cloud,
                #[cfg(feature = "enterprise")]
                enterprise: HashMap::new(),
            }),
        }
    }

    /// Create test state with a pre-configured Enterprise client
    #[cfg(feature = "enterprise")]
    pub fn with_enterprise_client(client: EnterpriseClient) -> Self {
        let mut enterprise = HashMap::new();
        enterprise.insert("_default".to_string(), client);
        Self {
            credential_source: CredentialSource::Profiles(vec![]),
            read_only: true,
            database_url: None,
            config: None,
            profiles: vec![],
            clients: RwLock::new(CachedClients {
                #[cfg(feature = "cloud")]
                cloud: HashMap::new(),
                enterprise,
            }),
        }
    }

    /// Create test state with both Cloud and Enterprise clients
    #[cfg(all(feature = "cloud", feature = "enterprise"))]
    pub fn with_clients(cloud: CloudClient, enterprise: EnterpriseClient) -> Self {
        let mut cloud_map = HashMap::new();
        cloud_map.insert("_default".to_string(), cloud);
        let mut enterprise_map = HashMap::new();
        enterprise_map.insert("_default".to_string(), enterprise);
        Self {
            credential_source: CredentialSource::Profiles(vec![]),
            read_only: true,
            database_url: None,
            config: None,
            profiles: vec![],
            clients: RwLock::new(CachedClients {
                cloud: cloud_map,
                enterprise: enterprise_map,
            }),
        }
    }
}
