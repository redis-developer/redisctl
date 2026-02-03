# Configuration File

Profile and settings file format reference.

## Location

| Platform | Path |
|----------|------|
| Linux | `~/.config/redisctl/config.toml` |
| macOS | `~/.config/redisctl/config.toml` |
| Windows | `%APPDATA%\redis\redisctl\config.toml` |

## File Format

```toml
# Default profile to use when --profile is not specified
default_profile = "prod"

# Redis Cloud profile
[profiles.cloud-prod]
cloud_api_key = "your-api-key"
cloud_secret_key = "your-secret-key"

# Redis Enterprise profile
[profiles.enterprise-prod]
enterprise_url = "https://cluster.example.com:9443"
enterprise_user = "admin@cluster.local"
enterprise_password = "your-password"
enterprise_insecure = true

# Combined profile (both Cloud and Enterprise)
[profiles.all-prod]
cloud_api_key = "your-api-key"
cloud_secret_key = "your-secret-key"
enterprise_url = "https://cluster.example.com:9443"
enterprise_user = "admin@cluster.local"
enterprise_password = "your-password"

# Profile with environment variable references
[profiles.ci]
cloud_api_key = "${REDIS_CLOUD_API_KEY}"
cloud_secret_key = "${REDIS_CLOUD_SECRET_KEY}"

# Profile with keyring storage (requires secure-storage feature)
[profiles.secure]
cloud_api_key = "keyring:prod-api-key"
cloud_secret_key = "keyring:prod-secret-key"

# Files.com API key for support package uploads
[global]
files_api_key = "your-files-api-key"
```

## Profile Fields

### Redis Cloud

| Field | Description |
|-------|-------------|
| `cloud_api_key` | API account key |
| `cloud_secret_key` | API secret key |
| `cloud_api_url` | Custom API URL (optional) |

### Redis Enterprise

| Field | Description |
|-------|-------------|
| `enterprise_url` | Cluster API URL (e.g., `https://cluster:9443`) |
| `enterprise_user` | Username |
| `enterprise_password` | Password |
| `enterprise_insecure` | Skip TLS verification (`true`/`false`) |

### Global Settings

| Field | Description |
|-------|-------------|
| `files_api_key` | Files.com API key for support uploads |

## Credential Storage Options

### Plaintext (Default)

```toml
[profiles.dev]
cloud_api_key = "actual-key-value"
```

!!! warning
    Credentials stored in plaintext. Use for development only.

### Environment Variables

```toml
[profiles.ci]
cloud_api_key = "${REDIS_CLOUD_API_KEY}"
cloud_secret_key = "${REDIS_CLOUD_SECRET_KEY}"
```

Variables are resolved at runtime.

### OS Keyring

```toml
[profiles.prod]
cloud_api_key = "keyring:prod-api-key"
cloud_secret_key = "keyring:prod-secret-key"
```

Requires `secure-storage` feature. Credentials stored in:
- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service

## Managing Profiles

### Create Profile

```bash
redisctl profile set myprofile \
  --cloud-api-key "$KEY" \
  --cloud-secret-key "$SECRET"
```

### List Profiles

```bash
redisctl profile list
```

### Set Default

```bash
redisctl profile set-default myprofile
```

### Delete Profile

```bash
redisctl profile delete myprofile
```

## Example Configurations

### Multi-Environment Setup

```toml
default_profile = "dev"

[profiles.dev]
enterprise_url = "https://dev-cluster:9443"
enterprise_user = "admin@dev.local"
enterprise_password = "${DEV_PASSWORD}"
enterprise_insecure = true

[profiles.staging]
enterprise_url = "https://staging-cluster:9443"
enterprise_user = "admin@staging.local"
enterprise_password = "${STAGING_PASSWORD}"

[profiles.prod]
enterprise_url = "https://prod-cluster:9443"
enterprise_user = "admin@prod.local"
enterprise_password = "keyring:prod-password"
```

### Cloud + Enterprise Combined

```toml
[profiles.full]
# Cloud credentials
cloud_api_key = "${REDIS_CLOUD_API_KEY}"
cloud_secret_key = "${REDIS_CLOUD_SECRET_KEY}"

# Enterprise credentials
enterprise_url = "https://cluster:9443"
enterprise_user = "admin@cluster.local"
enterprise_password = "${ENTERPRISE_PASSWORD}"
enterprise_insecure = true
```
