# Authentication

Configure credentials for Redis Cloud and Redis Enterprise.

## Overview

redisctl supports three credential sources, in order of precedence:

1. **Command-line flags** - Highest priority
2. **Environment variables** - Good for CI/CD
3. **Profiles** - Best for daily use

## Environment Variables

### Redis Cloud

| Variable | Description |
|----------|-------------|
| `REDIS_CLOUD_API_KEY` | API account key |
| `REDIS_CLOUD_SECRET_KEY` | API secret key |

```bash
export REDIS_CLOUD_API_KEY="your-api-key"
export REDIS_CLOUD_SECRET_KEY="your-secret-key"
redisctl cloud subscription list
```

### Redis Enterprise

| Variable | Description |
|----------|-------------|
| `REDIS_ENTERPRISE_URL` | Cluster API URL (e.g., `https://cluster:9443`) |
| `REDIS_ENTERPRISE_USER` | Admin username |
| `REDIS_ENTERPRISE_PASSWORD` | Admin password |
| `REDIS_ENTERPRISE_INSECURE` | Skip TLS verification (`true`/`false`) |

```bash
export REDIS_ENTERPRISE_URL="https://cluster.example.com:9443"
export REDIS_ENTERPRISE_USER="admin@cluster.local"
export REDIS_ENTERPRISE_PASSWORD="your-password"
redisctl enterprise cluster get
```

## Profiles

Profiles store credentials for multiple environments. Much better than juggling environment variables.

### Create a Profile

```bash
# Redis Cloud profile
redisctl profile set prod-cloud \
  --cloud-api-key "$API_KEY" \
  --cloud-secret-key "$SECRET_KEY"

# Redis Enterprise profile
redisctl profile set prod-enterprise \
  --enterprise-url "https://cluster.example.com:9443" \
  --enterprise-user "admin@cluster.local" \
  --enterprise-password "$PASSWORD"
```

### Use a Profile

```bash
# Specify profile per command
redisctl --profile prod-cloud cloud subscription list

# Or set a default
redisctl profile set-default prod-cloud
redisctl cloud subscription list  # Uses prod-cloud
```

### List Profiles

```bash
redisctl profile list
```

### Secure Storage

!!! warning "Default Storage"
    By default, credentials are stored in plaintext in `~/.config/redisctl/config.toml`. Use secure storage for production credentials.

#### OS Keyring

Store credentials in your operating system's keychain:

```bash
# Requires secure-storage feature
cargo install redisctl --features secure-storage

# Store in keyring
redisctl profile set prod \
  --cloud-api-key "$KEY" \
  --cloud-secret-key "$SECRET" \
  --use-keyring
```

#### Environment Variable References

Reference environment variables instead of storing values:

```bash
redisctl profile set prod \
  --cloud-api-key '${REDIS_CLOUD_API_KEY}' \
  --cloud-secret-key '${REDIS_CLOUD_SECRET_KEY}'
```

The variables are resolved at runtime.

## Configuration File

Profiles are stored in:

| Platform | Location |
|----------|----------|
| Linux/macOS | `~/.config/redisctl/config.toml` |
| Windows | `%APPDATA%\redis\redisctl\config.toml` |

Example:

```toml
default_profile = "prod"

[profiles.prod]
cloud_api_key = "your-api-key"
cloud_secret_key = "your-secret-key"

[profiles.dev]
enterprise_url = "https://dev-cluster:9443"
enterprise_user = "admin@cluster.local"
enterprise_password = "dev-password"
```

## Next Steps

- [Output Formats](../common/output-formats.md) - JSON, YAML, and table output
- [JMESPath Queries](../common/jmespath.md) - Filter and transform output
