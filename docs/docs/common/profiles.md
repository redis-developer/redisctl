# Profiles

Manage credentials for multiple Redis deployments.

## Why Profiles?

Instead of juggling environment variables or passing credentials on every command, profiles let you:

- Store credentials for multiple environments (dev, staging, prod)
- Switch between Redis Cloud, Enterprise, and direct database connections
- Keep credentials secure with OS keyring integration
- Share configuration (without secrets) across teams

## Profile Types

redisctl supports three profile types:

| Type | Use Case |
|------|----------|
| **Cloud** | Redis Cloud API access (subscriptions, databases, etc.) |
| **Enterprise** | Redis Enterprise REST API access (cluster management) |
| **Database** | Direct Redis database connections |

## Creating Profiles

### Redis Cloud

```bash
redisctl profile set my-cloud --type cloud \
  --api-key "your-api-key" \
  --api-secret "your-secret-key"
```

### Redis Enterprise

```bash
redisctl profile set my-enterprise --type enterprise \
  --url "https://cluster.example.com:9443" \
  --username "admin@cluster.local" \
  --password "your-password"
```

### Database (Direct Redis Connection)

For direct connections to Redis databases:

```bash
redisctl profile set my-cache --type database \
  --host "redis-12345.cloud.redislabs.com" \
  --port 12345 \
  --password "your-password"
```

Database profiles support these options:

| Option | Description | Default |
|--------|-------------|---------|
| `--host` | Redis server hostname | (required) |
| `--port` | Redis server port | (required) |
| `--password` | Redis password | (none) |
| `--username` | Redis ACL username (Redis 6+) | `default` |
| `--no-tls` | Disable TLS | TLS enabled |
| `--db` | Redis database number | `0` |

Example for local development without TLS:

```bash
redisctl profile set local-redis --type database \
  --host localhost \
  --port 6379 \
  --no-tls
```

## Using Profiles

### Per-Command

```bash
redisctl --profile prod cloud subscription list
redisctl --profile dev enterprise cluster get
```

### Default Profiles

Set defaults for each profile type:

```bash
# Set default Cloud profile
redisctl profile default-cloud prod-cloud

# Set default Enterprise profile
redisctl profile default-enterprise prod-cluster

# Set default Database profile
redisctl profile default-database my-cache
```

Now commands use the appropriate default automatically:

```bash
redisctl cloud subscription list      # Uses prod-cloud
redisctl enterprise cluster get       # Uses prod-cluster
```

### Override with Environment

Environment variables override profile settings:

```bash
# Profile says one thing, env var wins
export REDIS_CLOUD_API_KEY="override-key"
redisctl --profile prod cloud subscription list  # Uses override-key
```

## Managing Profiles

### List All Profiles

```bash
redisctl profile list
```

```
Profiles:
  Cloud:
    * prod-cloud (default)
      dev-cloud
  Enterprise:
    * prod-cluster (default)
      staging-cluster
  Database:
    * my-cache (default)
      local-redis
```

### Show Profile Details

```bash
redisctl profile show prod-cloud
```

### Delete a Profile

```bash
redisctl profile remove dev-cloud
```

### Validate Configuration

```bash
redisctl profile validate
```

## Secure Storage

!!! danger "Default is Plaintext"
    By default, credentials are stored in plaintext. Use one of the secure options below for sensitive environments.

### Option 1: OS Keyring

Store credentials in macOS Keychain, Windows Credential Manager, or Linux Secret Service:

```bash
# Create profile with keyring storage
redisctl profile set prod --type cloud \
  --api-key "$KEY" \
  --api-secret "$SECRET" \
  --use-keyring
```

The config file only stores a reference:

```toml
[profiles.prod]
deployment_type = "cloud"
api_key = "keyring:prod-api-key"
api_secret = "keyring:prod-api-secret"
```

### Option 2: Environment References

Store references to environment variables:

```bash
redisctl profile set prod --type cloud \
  --api-key '${REDIS_CLOUD_API_KEY}' \
  --api-secret '${REDIS_CLOUD_SECRET_KEY}'
```

Variables are resolved at runtime. Great for CI/CD where secrets are injected.

## Configuration File Location

| Platform | Path |
|----------|------|
| Linux | `~/.config/redisctl/config.toml` |
| macOS | `~/.config/redisctl/config.toml` |
| Windows | `%APPDATA%\redis\redisctl\config.toml` |

## Example Configuration

```toml
default_cloud = "prod-cloud"
default_enterprise = "prod-cluster"
default_database = "my-cache"

[profiles.prod-cloud]
deployment_type = "cloud"
api_key = "keyring:prod-api-key"
api_secret = "keyring:prod-api-secret"

[profiles.prod-cluster]
deployment_type = "enterprise"
url = "https://prod-cluster:9443"
username = "admin@cluster.local"
password = "${PROD_PASSWORD}"

[profiles.dev-cluster]
deployment_type = "enterprise"
url = "https://dev-cluster:9443"
username = "admin@cluster.local"
password = "${DEV_PASSWORD}"
insecure = true

[profiles.my-cache]
deployment_type = "database"
host = "redis-12345.cloud.redislabs.com"
port = 12345
password = "keyring:my-cache-password"
tls = true
username = "default"

[profiles.local-redis]
deployment_type = "database"
host = "localhost"
port = 6379
tls = false
```
