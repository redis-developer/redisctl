//! Convenience parameter structs for Cloud database operations
//!
//! These structs provide a simpler interface for common operations,
//! while still allowing fallback to Layer 1 builders for edge cases.

use redis_cloud::databases::{DatabaseCreateRequest, DatabaseImportRequest, DatabaseUpdateRequest};

/// Parameters for creating a database
///
/// This is a convenience wrapper around `DatabaseCreateRequest` that
/// provides a simpler API for common use cases. For advanced options,
/// use `DatabaseCreateRequest::builder()` directly.
///
/// # Example
///
/// ```rust,ignore
/// use redisctl_core::cloud::CreateDatabaseParams;
///
/// let params = CreateDatabaseParams::new("my-database", 1.0)
///     .with_replication(true)
///     .with_protocol("stack")
///     .with_data_persistence("aof-every-1-second");
///
/// let request = params.into_request();
/// ```
#[derive(Debug, Clone)]
pub struct CreateDatabaseParams {
    /// Database name (required)
    pub name: String,
    /// Memory limit in GB (required)
    pub memory_limit_in_gb: f64,
    /// Enable replication (default: true)
    pub replication: Option<bool>,
    /// Protocol: "redis", "stack", or "memcached" (default: "redis")
    pub protocol: Option<String>,
    /// Data persistence: "none", "aof-every-1-second", "aof-every-write",
    /// "snapshot-every-1-hour", "snapshot-every-6-hours", "snapshot-every-12-hours"
    pub data_persistence: Option<String>,
    /// Data eviction policy (default: "volatile-lru")
    pub data_eviction_policy: Option<String>,
    /// Redis version
    pub redis_version: Option<String>,
    /// Support OSS Cluster API
    pub support_oss_cluster_api: Option<bool>,
    /// TCP port (10000-19999)
    pub port: Option<i32>,
}

impl CreateDatabaseParams {
    /// Create new params with required fields
    #[must_use]
    pub fn new(name: impl Into<String>, memory_limit_in_gb: f64) -> Self {
        Self {
            name: name.into(),
            memory_limit_in_gb,
            replication: None,
            protocol: None,
            data_persistence: None,
            data_eviction_policy: None,
            redis_version: None,
            support_oss_cluster_api: None,
            port: None,
        }
    }

    /// Set replication
    #[must_use]
    pub fn with_replication(mut self, replication: bool) -> Self {
        self.replication = Some(replication);
        self
    }

    /// Set protocol
    #[must_use]
    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Set data persistence
    #[must_use]
    pub fn with_data_persistence(mut self, persistence: impl Into<String>) -> Self {
        self.data_persistence = Some(persistence.into());
        self
    }

    /// Set eviction policy
    #[must_use]
    pub fn with_eviction_policy(mut self, policy: impl Into<String>) -> Self {
        self.data_eviction_policy = Some(policy.into());
        self
    }

    /// Set Redis version
    #[must_use]
    pub fn with_redis_version(mut self, version: impl Into<String>) -> Self {
        self.redis_version = Some(version.into());
        self
    }

    /// Enable OSS Cluster API support
    #[must_use]
    pub fn with_oss_cluster_api(mut self, enabled: bool) -> Self {
        self.support_oss_cluster_api = Some(enabled);
        self
    }

    /// Set TCP port
    #[must_use]
    pub fn with_port(mut self, port: i32) -> Self {
        self.port = Some(port);
        self
    }

    /// Convert to Layer 1 `DatabaseCreateRequest`
    ///
    /// Uses the TypedBuilder pattern from redis-cloud, setting only
    /// the fields that have values.
    #[must_use]
    pub fn into_request(self) -> DatabaseCreateRequest {
        // Build with just the required name field, then set optionals
        // We need to use the full builder chain due to TypedBuilder's type system
        DatabaseCreateRequest::builder()
            .name(&self.name)
            .memory_limit_in_gb(self.memory_limit_in_gb)
            .replication(self.replication.unwrap_or(true))
            .protocol(self.protocol.unwrap_or_else(|| "redis".to_string()))
            .data_persistence(self.data_persistence.unwrap_or_else(|| "none".to_string()))
            .data_eviction_policy(
                self.data_eviction_policy
                    .unwrap_or_else(|| "volatile-lru".to_string()),
            )
            .build()
    }
}

/// Parameters for updating a database
///
/// All fields are optional - only set fields you want to change.
#[derive(Debug, Clone, Default)]
pub struct UpdateDatabaseParams {
    /// New database name
    pub name: Option<String>,
    /// New memory limit in GB
    pub memory_limit_in_gb: Option<f64>,
    /// Change replication setting
    pub replication: Option<bool>,
    /// Change data persistence
    pub data_persistence: Option<String>,
    /// Change eviction policy
    pub data_eviction_policy: Option<String>,
    /// Change OSS Cluster API support
    pub support_oss_cluster_api: Option<bool>,
}

impl UpdateDatabaseParams {
    /// Create empty update params
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set new name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set new memory limit
    #[must_use]
    pub fn with_memory_limit(mut self, memory_gb: f64) -> Self {
        self.memory_limit_in_gb = Some(memory_gb);
        self
    }

    /// Set replication
    #[must_use]
    pub fn with_replication(mut self, replication: bool) -> Self {
        self.replication = Some(replication);
        self
    }

    /// Set data persistence
    #[must_use]
    pub fn with_data_persistence(mut self, persistence: impl Into<String>) -> Self {
        self.data_persistence = Some(persistence.into());
        self
    }

    /// Set eviction policy
    #[must_use]
    pub fn with_eviction_policy(mut self, policy: impl Into<String>) -> Self {
        self.data_eviction_policy = Some(policy.into());
        self
    }

    /// Enable/disable OSS Cluster API
    #[must_use]
    pub fn with_oss_cluster_api(mut self, enabled: bool) -> Self {
        self.support_oss_cluster_api = Some(enabled);
        self
    }

    /// Check if any fields are set
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.memory_limit_in_gb.is_none()
            && self.replication.is_none()
            && self.data_persistence.is_none()
            && self.data_eviction_policy.is_none()
            && self.support_oss_cluster_api.is_none()
    }

    /// Convert to Layer 1 `DatabaseUpdateRequest`
    #[must_use]
    pub fn into_request(self) -> DatabaseUpdateRequest {
        // DatabaseUpdateRequest has all optional fields, so we can build with defaults
        // and only set what we have
        let mut req = DatabaseUpdateRequest::builder().build();

        req.name = self.name;
        req.memory_limit_in_gb = self.memory_limit_in_gb;
        req.replication = self.replication;
        req.data_persistence = self.data_persistence;
        req.data_eviction_policy = self.data_eviction_policy;
        req.support_oss_cluster_api = self.support_oss_cluster_api;

        req
    }
}

/// Parameters for importing data into a database
#[derive(Debug, Clone)]
pub struct ImportDatabaseParams {
    /// Source type: "http", "redis", "ftp", "aws-s3", "azure-blob-storage", "google-blob-storage"
    pub source_type: String,
    /// URIs to import from
    pub import_from_uri: Vec<String>,
}

impl ImportDatabaseParams {
    /// Create new import params
    #[must_use]
    pub fn new(source_type: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            source_type: source_type.into(),
            import_from_uri: vec![uri.into()],
        }
    }

    /// Add additional URI to import from
    #[must_use]
    pub fn with_additional_uri(mut self, uri: impl Into<String>) -> Self {
        self.import_from_uri.push(uri.into());
        self
    }

    /// Convert to Layer 1 `DatabaseImportRequest`
    #[must_use]
    pub fn into_request(self) -> DatabaseImportRequest {
        DatabaseImportRequest::builder()
            .source_type(&self.source_type)
            .import_from_uri(self.import_from_uri)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_database_params_basic() {
        let params = CreateDatabaseParams::new("test-db", 1.0);
        let request = params.into_request();
        assert_eq!(request.name, "test-db");
        assert_eq!(request.memory_limit_in_gb, Some(1.0));
    }

    #[test]
    fn test_create_database_params_with_options() {
        let params = CreateDatabaseParams::new("test-db", 2.0)
            .with_replication(true)
            .with_protocol("stack")
            .with_data_persistence("aof-every-1-second");

        let request = params.into_request();
        assert_eq!(request.name, "test-db");
        assert_eq!(request.memory_limit_in_gb, Some(2.0));
        assert_eq!(request.replication, Some(true));
        assert_eq!(request.protocol, Some("stack".to_string()));
        assert_eq!(
            request.data_persistence,
            Some("aof-every-1-second".to_string())
        );
    }

    #[test]
    fn test_update_database_params_empty() {
        let params = UpdateDatabaseParams::new();
        assert!(params.is_empty());
    }

    #[test]
    fn test_update_database_params_with_changes() {
        let params = UpdateDatabaseParams::new()
            .with_name("new-name")
            .with_memory_limit(4.0);

        assert!(!params.is_empty());
        let request = params.into_request();
        assert_eq!(request.name, Some("new-name".to_string()));
        assert_eq!(request.memory_limit_in_gb, Some(4.0));
    }

    #[test]
    fn test_import_database_params() {
        let params = ImportDatabaseParams::new("aws-s3", "s3://bucket/file.rdb")
            .with_additional_uri("s3://bucket/file2.rdb");

        let request = params.into_request();
        // source_type and import_from_uri are NOT Option-wrapped in DatabaseImportRequest
        assert_eq!(request.source_type, "aws-s3");
        assert_eq!(
            request.import_from_uri,
            vec![
                "s3://bucket/file.rdb".to_string(),
                "s3://bucket/file2.rdb".to_string()
            ]
        );
    }
}
