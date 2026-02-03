# Getting Started

This guide walks you through installing and configuring the redisctl MCP server with your AI assistant.

## Installation

### macOS (Homebrew)

```bash
brew install redis-developer/homebrew-tap/redisctl
```

### Linux/Windows

Download from [GitHub Releases](https://github.com/redis-developer/redisctl/releases) or use Docker:

```bash
docker pull ghcr.io/redis-developer/redisctl
```

See the [Installation Guide](../getting-started/installation.md) for all options.

## Setting Up Credentials

Before using the MCP server, configure a profile with your Redis credentials.

### Redis Cloud

```bash
# Interactive setup (prompts for API keys)
redisctl profile add my-cloud-profile --cloud

# Or provide keys directly
redisctl profile add my-cloud-profile --cloud \
  --api-key YOUR_API_KEY \
  --secret-key YOUR_SECRET_KEY
```

Get your API keys from the [Redis Cloud Console](https://app.redislabs.com/) under Account > API Keys.

### Redis Enterprise

```bash
# Interactive setup
redisctl profile add my-enterprise-profile --enterprise

# Or provide credentials directly
redisctl profile add my-enterprise-profile --enterprise \
  --url https://your-cluster:9443 \
  --username admin@redis.com \
  --password YOUR_PASSWORD \
  --insecure  # if using self-signed certs
```

### Verify Your Profile

```bash
# Test Cloud connection
redisctl -p my-cloud-profile cloud account get

# Test Enterprise connection
redisctl -p my-enterprise-profile enterprise cluster get
```

## Starting the MCP Server

```bash
# Start in read-only mode (safe for exploration)
redisctl -p my-profile mcp serve

# Enable write operations (create, update, delete)
redisctl -p my-profile mcp serve --allow-writes

# Connect to a Redis database for direct data operations
redisctl -p my-profile mcp serve --database-url redis://localhost:6379

# Full access: Cloud/Enterprise management + database operations + writes
redisctl -p my-profile mcp serve --allow-writes --database-url redis://localhost:6379

# List available tools
redisctl mcp tools
```

### Database Connection Options

The MCP server provides 125+ database tools for direct Redis operations including all data types, Redis Stack modules (Search, JSON, TimeSeries, Bloom), Streams, and Pub/Sub. You can connect in two ways:

#### Option 1: Direct URL (Recommended for Ad-Hoc Connections)

Use `--database-url` for quick connections to any Redis database:

```bash
# Local Redis
--database-url redis://localhost:6379

# With password
--database-url redis://:mypassword@localhost:6379

# With username and password
--database-url redis://myuser:mypassword@localhost:6379

# Redis Cloud/Enterprise database
--database-url redis://default:password@redis-12345.cloud.redislabs.com:12345

# TLS connection (use rediss:// scheme)
--database-url rediss://default:password@redis-12345.cloud.redislabs.com:12345

# Using environment variable
REDIS_URL=redis://localhost:6379 redisctl mcp serve
```

#### Option 2: Database Profile (Recommended for Regular Use)

Configure a database profile in your redisctl config file (`~/.config/redisctl/config.toml` or `~/Library/Application Support/redisctl/config.toml` on macOS):

```toml
# Default database profile to use when none specified
default_database_profile = "local-redis"

[profiles.local-redis]
deployment_type = "database"

[profiles.local-redis.credentials.database]
host = "localhost"
port = 6379
password = "mypassword"  # optional
tls = false
username = "default"     # optional, defaults to "default"
db = 0                   # optional, defaults to 0
```

Then start the MCP server with that profile:

```bash
# Uses the default database profile from config
redisctl mcp serve

# Or specify a profile explicitly
redisctl -p local-redis mcp serve
```

**Note**: If both `--database-url` and a database profile are available, the `--database-url` takes precedence.

## IDE Configuration

Choose your AI assistant below:

=== "Claude Desktop"

    Add to your Claude Desktop configuration file:

    **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

    **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "/path/to/redisctl",
          "args": ["-p", "my-profile", "mcp", "serve"]
        }
      }
    }
    ```

    For write operations with database access:

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "/path/to/redisctl",
          "args": [
            "-p", "my-profile", "mcp", "serve",
            "--allow-writes",
            "--database-url", "redis://localhost:6379"
          ]
        }
      }
    }
    ```

=== "Claude Code"

    Add to your project's `.mcp.json` or global MCP settings:

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "redisctl",
          "args": [
            "-p", "my-profile", "mcp", "serve",
            "--allow-writes",
            "--database-url", "redis://localhost:6379"
          ]
        }
      }
    }
    ```

=== "Cursor"

    Add to your Cursor MCP configuration file:

    **macOS**: `~/.cursor/mcp.json`

    **Windows**: `%USERPROFILE%\.cursor\mcp.json`

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "/path/to/redisctl",
          "args": [
            "-p", "my-profile", "mcp", "serve",
            "--allow-writes",
            "--database-url", "redis://localhost:6379"
          ]
        }
      }
    }
    ```

    Restart Cursor or use the command palette to reload MCP servers.

=== "Windsurf"

    Add to your Windsurf MCP configuration:

    **macOS**: `~/.codeium/windsurf/mcp_config.json`

    **Windows**: `%USERPROFILE%\.codeium\windsurf\mcp_config.json`

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "/path/to/redisctl",
          "args": [
            "-p", "my-profile", "mcp", "serve",
            "--allow-writes",
            "--database-url", "redis://localhost:6379"
          ]
        }
      }
    }
    ```

    Restart Windsurf after updating the configuration.

=== "VS Code (Continue)"

    If you're using [Continue](https://continue.dev/) in VS Code:

    **Config location**: `~/.continue/config.json`

    ```json
    {
      "experimental": {
        "modelContextProtocolServers": [
          {
            "transport": {
              "type": "stdio",
              "command": "/path/to/redisctl",
              "args": [
                "-p", "my-profile", "mcp", "serve",
                "--allow-writes",
                "--database-url", "redis://localhost:6379"
              ]
            }
          }
        ]
      }
    }
    ```

=== "Zed"

    Add to your Zed settings (`~/.config/zed/settings.json` on Linux/macOS):

    ```json
    {
      "context_servers": {
        "redisctl": {
          "command": {
            "path": "/path/to/redisctl",
            "args": [
              "-p", "my-profile", "mcp", "serve",
              "--allow-writes",
              "--database-url", "redis://localhost:6379"
            ]
          }
        }
      }
    }
    ```

## Security Considerations

### Read-Only Mode (Default)

By default, the MCP server runs in read-only mode. This prevents any destructive operations and is recommended for:

- Exploring your infrastructure
- Monitoring and reporting
- Learning about your deployments

### Write Mode

Use `--allow-writes` only when you need to create or modify resources. Consider:

- Using separate profiles for read-only vs write access
- Running write-enabled servers only in development environments
- Reviewing AI-suggested changes before confirming

### Profile-Based Authentication

The MCP server uses your existing redisctl profiles, which means:

- Credentials are never exposed to the AI
- You control which environments are accessible
- Standard profile security applies (keyring support, etc.)

## Troubleshooting

### Server won't start

```bash
# Check your profile works
redisctl -p my-profile enterprise cluster get

# Verify MCP feature is enabled
redisctl mcp tools
```

### AI can't find the server

1. Ensure the path to redisctl is absolute in your config
2. Restart your IDE after config changes
3. Check logs for connection errors

### Operations timing out

The MCP server inherits redisctl's timeout settings. For slow operations, ensure your profile has appropriate timeout configuration.

## Protocol Details

The MCP server uses:

- **Transport**: stdio (standard input/output)
- **Protocol Version**: 2024-11-05
- **Capabilities**: Tools only (no resources or prompts currently)

For MCP protocol details, see the [MCP Specification](https://spec.modelcontextprotocol.io/).

## Next Steps

- [Tools Reference](tools-reference.md) - Complete list of available tools
- [Advanced Usage](advanced-usage.md) - JMESPath integration and analytics
- [Workflows](workflows.md) - Real-world use cases and examples
