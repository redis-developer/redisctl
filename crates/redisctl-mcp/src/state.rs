//! Application state and credential resolution

use anyhow::{Context, Result};
use redis_cloud::CloudClient;
use redis_enterprise::EnterpriseClient;
use redisctl_config::Config;
use tokio::sync::RwLock;

/// How credentials are resolved
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CredentialSource {
    /// Resolve from redisctl profile (local mode)
    Profile(Option<String>),
    /// Resolve from OAuth token claims (HTTP mode)
    OAuth {
        issuer: Option<String>,
        audience: Option<String>,
    },
}

/// Cached API clients
pub struct CachedClients {
    pub cloud: Option<CloudClient>,
    pub enterprise: Option<EnterpriseClient>,
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
    /// Cached API clients
    clients: RwLock<CachedClients>,
}

impl AppState {
    /// Create new application state
    pub fn new(
        credential_source: CredentialSource,
        read_only: bool,
        database_url: Option<String>,
    ) -> Result<Self> {
        // Normalize credential source: treat "default" as None (use configured default)
        let credential_source = match credential_source {
            CredentialSource::Profile(Some(ref name)) if name.eq_ignore_ascii_case("default") => {
                CredentialSource::Profile(None)
            }
            other => other,
        };

        // Load config if using profile-based auth
        let config = match &credential_source {
            CredentialSource::Profile(_) => Config::load().ok(),
            CredentialSource::OAuth { .. } => None,
        };

        Ok(Self {
            credential_source,
            read_only,
            database_url,
            config,
            clients: RwLock::new(CachedClients {
                cloud: None,
                enterprise: None,
            }),
        })
    }

    /// Get or create Cloud API client
    pub async fn cloud_client(&self) -> Result<CloudClient> {
        // Check cache first
        {
            let clients = self.clients.read().await;
            if let Some(client) = &clients.cloud {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client = self.create_cloud_client().await?;

        // Cache it
        {
            let mut clients = self.clients.write().await;
            clients.cloud = Some(client.clone());
        }

        Ok(client)
    }

    /// Get or create Enterprise API client
    pub async fn enterprise_client(&self) -> Result<EnterpriseClient> {
        // Check cache first
        {
            let clients = self.clients.read().await;
            if let Some(client) = &clients.enterprise {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client = self.create_enterprise_client().await?;

        // Cache it
        {
            let mut clients = self.clients.write().await;
            clients.enterprise = Some(client.clone());
        }

        Ok(client)
    }

    /// Create a new Cloud client from credentials
    async fn create_cloud_client(&self) -> Result<CloudClient> {
        match &self.credential_source {
            CredentialSource::Profile(profile_name) => {
                let config = self
                    .config
                    .as_ref()
                    .context("No redisctl config available")?;

                // Resolve the profile name
                let resolved_profile_name = config
                    .resolve_cloud_profile(profile_name.as_deref())
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
    async fn create_enterprise_client(&self) -> Result<EnterpriseClient> {
        match &self.credential_source {
            CredentialSource::Profile(profile_name) => {
                let config = self
                    .config
                    .as_ref()
                    .context("No redisctl config available")?;

                // Resolve the profile name
                let resolved_profile_name = config
                    .resolve_enterprise_profile(profile_name.as_deref())
                    .context("Failed to resolve enterprise profile")?;

                // Get the profile
                let profile = config
                    .profiles
                    .get(&resolved_profile_name)
                    .with_context(|| format!("Profile '{}' not found", resolved_profile_name))?;

                // Get credentials
                let (url, username, password, insecure) = profile
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
            clients: RwLock::new(CachedClients {
                cloud: None,
                enterprise: None,
            }),
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
#[allow(dead_code)]
impl AppState {
    /// Create test state with a pre-configured Cloud client
    pub fn with_cloud_client(client: CloudClient) -> Self {
        Self {
            credential_source: CredentialSource::Profile(None),
            read_only: true,
            database_url: None,
            config: None,
            clients: RwLock::new(CachedClients {
                cloud: Some(client),
                enterprise: None,
            }),
        }
    }

    /// Create test state with a pre-configured Enterprise client
    pub fn with_enterprise_client(client: EnterpriseClient) -> Self {
        Self {
            credential_source: CredentialSource::Profile(None),
            read_only: true,
            database_url: None,
            config: None,
            clients: RwLock::new(CachedClients {
                cloud: None,
                enterprise: Some(client),
            }),
        }
    }

    /// Create test state with both Cloud and Enterprise clients
    pub fn with_clients(cloud: CloudClient, enterprise: EnterpriseClient) -> Self {
        Self {
            credential_source: CredentialSource::Profile(None),
            read_only: true,
            database_url: None,
            config: None,
            clients: RwLock::new(CachedClients {
                cloud: Some(cloud),
                enterprise: Some(enterprise),
            }),
        }
    }
}
