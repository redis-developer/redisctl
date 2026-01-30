# redisctl-mcp

MCP (Model Context Protocol) server for Redis Cloud and Enterprise management.

This standalone binary exposes Redis management operations as tools that AI assistants
like Claude can use to help manage your Redis infrastructure.

## Installation

```bash
# From source
cargo install --path crates/redisctl-mcp

# Or build directly
cargo build --release -p redisctl-mcp
```

## Quick Start

### With Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "redis": {
      "command": "/path/to/redisctl-mcp",
      "args": ["--profile", "my-profile"]
    }
  }
}
```

### With Claude Code

Add to your project's `.mcp.json`:

```json
{
  "mcpServers": {
    "redis": {
      "command": "redisctl-mcp",
      "args": ["--profile", "default"]
    }
  }
}
```

## Usage

### Stdio Transport (Default)

For local integrations with Claude Desktop, Claude Code, or other MCP clients:

```bash
# Use the default profile
redisctl-mcp

# Use a specific profile
redisctl-mcp --profile production

# Read-only mode (disables write operations)
redisctl-mcp --profile production --read-only

# With a direct Redis database connection
redisctl-mcp --database-url redis://localhost:6379
```

### HTTP Transport

For shared deployments accessible over the network:

```bash
# Basic HTTP server
redisctl-mcp --transport http --port 8080

# With OAuth authentication
redisctl-mcp --transport http --port 8080 \
  --oauth \
  --oauth-issuer https://accounts.google.com \
  --oauth-audience my-app-id

# With custom rate limiting
redisctl-mcp --transport http --port 8080 \
  --max-concurrent 20 \
  --request-timeout-secs 60
```

## Available Tools

### Redis Cloud

| Tool | Description |
|------|-------------|
| `list_subscriptions` | List all Redis Cloud subscriptions |
| `get_subscription` | Get details of a specific subscription |
| `list_databases` | List databases in a subscription |
| `get_database` | Get database configuration details |

### Redis Enterprise

| Tool | Description |
|------|-------------|
| `get_cluster` | Get cluster information (name, version, config) |
| `list_enterprise_databases` | List all databases on the cluster |
| `get_enterprise_database` | Get database details by UID |
| `list_nodes` | List all cluster nodes |

### Direct Redis Operations

| Tool | Description |
|------|-------------|
| `redis_ping` | Test connectivity to a Redis database |
| `redis_info` | Get Redis INFO output (optionally by section) |
| `redis_keys` | List keys matching a pattern (uses SCAN) |

## Configuration

### Profile-Based Authentication

The server uses `redisctl` profiles for credential management. Configure profiles in `~/.config/redisctl/config.toml`:

```toml
default_cloud_profile = "cloud-prod"
default_enterprise_profile = "enterprise-dev"

[profiles.cloud-prod]
type = "cloud"
api_key = "${REDIS_CLOUD_API_KEY}"
api_secret = "${REDIS_CLOUD_API_SECRET}"

[profiles.enterprise-dev]
type = "enterprise"
url = "https://cluster.example.com:9443"
username = "admin"
password = "${RE_PASSWORD}"
insecure = true
```

### Environment Variables

For OAuth/HTTP mode or when not using profiles:

```bash
# Redis Cloud
export REDIS_CLOUD_API_KEY=your-key
export REDIS_CLOUD_API_SECRET=your-secret

# Redis Enterprise
export REDIS_ENTERPRISE_URL=https://cluster:9443
export REDIS_ENTERPRISE_USER=admin
export REDIS_ENTERPRISE_PASSWORD=secret
export REDIS_ENTERPRISE_INSECURE=true  # optional, for self-signed certs

# Direct Redis connection
export REDIS_URL=redis://localhost:6379
```

## Command Line Options

```
Options:
  -t, --transport <TRANSPORT>      Transport mode [default: stdio]
                                   - stdio: For CLI integrations
                                   - http: For web deployments
  -p, --profile <PROFILE>          Profile name for credentials
      --read-only                  Disable write operations
      --database-url <URL>         Redis URL for direct connections

  HTTP Options:
      --host <HOST>                Bind host [default: 127.0.0.1]
      --port <PORT>                Bind port [default: 8080]
      --oauth                      Enable OAuth authentication
      --oauth-issuer <URL>         OAuth issuer URL
      --oauth-audience <AUD>       OAuth audience
      --jwks-uri <URI>             JWKS URI (auto-discovered if not set)
      --max-concurrent <N>         Max concurrent requests [default: 10]
      --rate-limit-ms <MS>         Rate limit interval [default: 100]
      --request-timeout-secs <S>   Request timeout [default: 30]

  Logging:
      --log-level <LEVEL>          Log level [default: info]
```

## Library Usage

You can embed these tools in your own MCP server:

```rust
use std::sync::Arc;
use redisctl_mcp::{AppState, CredentialSource, tools};
use tower_mcp::McpRouter;

let state = Arc::new(AppState::new(
    CredentialSource::Profile(Some("default".to_string())),
    true, // read-only
    None, // no database URL
)?);

let router = McpRouter::new()
    .tool(tools::cloud::list_subscriptions(state.clone()))
    .tool(tools::cloud::get_subscription(state.clone()))
    .tool(tools::enterprise::get_cluster(state.clone()))
    .tool(tools::redis::ping(state.clone()));
```

## Security Considerations

- Use `--read-only` mode in production to prevent accidental modifications
- For HTTP transport, always enable OAuth in production environments
- Store credentials using environment variables or secure credential storage
- The server respects profile-based credential isolation
