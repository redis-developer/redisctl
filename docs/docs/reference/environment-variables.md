# Environment Variables

Complete reference of environment variables supported by redisctl.

## Redis Cloud

| Variable | Description | Example |
|----------|-------------|---------|
| `REDIS_CLOUD_API_KEY` | API account key | `A3qcymrvqpn9rr...` |
| `REDIS_CLOUD_SECRET_KEY` | API secret key | `S3s8ecrrnaguqk...` |
| `REDIS_CLOUD_API_URL` | API endpoint (optional) | `https://api.redislabs.com/v1` |

## Redis Enterprise

| Variable | Description | Example |
|----------|-------------|---------|
| `REDIS_ENTERPRISE_URL` | Cluster API URL | `https://cluster:9443` |
| `REDIS_ENTERPRISE_USER` | Username | `admin@cluster.local` |
| `REDIS_ENTERPRISE_PASSWORD` | Password | `your-password` |
| `REDIS_ENTERPRISE_INSECURE` | Allow self-signed certs | `true` or `false` |

## Files.com (Support Package Upload)

| Variable | Description | Example |
|----------|-------------|---------|
| `REDIS_ENTERPRISE_FILES_API_KEY` | Files.com API key | `your-files-api-key` |

## General

| Variable | Description | Example |
|----------|-------------|---------|
| `REDISCTL_PROFILE` | Default profile name | `production` |
| `REDISCTL_OUTPUT` | Default output format | `json`, `yaml`, `table` |
| `RUST_LOG` | Logging level | `error`, `warn`, `info`, `debug` |
| `NO_COLOR` | Disable colored output | `1` or any value |

## Usage Examples

### Basic Setup

=== "Redis Cloud"

    ```bash
    export REDIS_CLOUD_API_KEY="your-key"
    export REDIS_CLOUD_SECRET_KEY="your-secret"

    redisctl cloud subscription list
    ```

=== "Redis Enterprise"

    ```bash
    export REDIS_ENTERPRISE_URL="https://localhost:9443"
    export REDIS_ENTERPRISE_USER="admin@cluster.local"
    export REDIS_ENTERPRISE_PASSWORD="password"
    export REDIS_ENTERPRISE_INSECURE="true"

    redisctl enterprise cluster get
    ```

### Debugging

```bash
# Enable debug logging
export RUST_LOG=debug
redisctl api cloud get /subscriptions

# Trace specific modules
export RUST_LOG=redisctl=debug,redis_cloud=trace
```

### CI/CD

=== "GitHub Actions"

    ```yaml
    env:
      REDIS_CLOUD_API_KEY: ${{ secrets.REDIS_CLOUD_API_KEY }}
      REDIS_CLOUD_SECRET_KEY: ${{ secrets.REDIS_CLOUD_SECRET_KEY }}
    ```

=== "GitLab CI"

    ```yaml
    variables:
      REDIS_CLOUD_API_KEY: $REDIS_CLOUD_API_KEY
      REDIS_CLOUD_SECRET_KEY: $REDIS_CLOUD_SECRET_KEY
    ```

## Precedence

From highest to lowest priority:

1. **Command-line flags** - Always win
2. **Environment variables** - Override profiles
3. **Profile settings** - From config file
4. **Default values** - Built-in defaults
