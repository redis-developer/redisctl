# MCP Server (AI Integration)

redisctl includes a built-in [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that enables AI assistants like Claude to manage your Redis deployments through natural language.

## Overview

The MCP server exposes redisctl functionality as tools that AI systems can discover and invoke. This allows you to:

- Ask an AI to "list all my Redis databases"
- Request "create a new 256MB database called cache-db"
- Query "what's the status of my cluster nodes"

All operations use your existing redisctl profiles for authentication.

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

Before using the MCP server, configure a profile with your Redis credentials:

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

## Quick Start

The MCP server is a separate binary called `redisctl-mcp`:

```bash
# Start the MCP server (read-only mode by default, safe for exploration)
redisctl-mcp --profile my-profile

# Enable write operations (create, update, delete)
redisctl-mcp --profile my-profile --read-only=false
```

## Configuring Claude Desktop

Add the following to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-profile"]
    }
  }
}
```

For write operations:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-profile", "--read-only=false"]
    }
  }
}
```

## Configuring Claude Code

Add to your project's `.mcp.json` or global MCP settings:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-profile", "--read-only=false"]
    }
  }
}
```

To load only specific toolsets, add `--tools`:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": [
        "--profile", "my-profile",
        "--tools", "cloud:subscriptions,cloud:account"
      ]
    }
  }
}
```

## Configuring Cursor

Add to your Cursor MCP configuration file:

**macOS**: `~/.cursor/mcp.json`

**Windows**: `%USERPROFILE%\.cursor\mcp.json`

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-profile", "--read-only=false"]
    }
  }
}
```

After saving, restart Cursor or use the command palette to reload MCP servers.

## Configuring Windsurf

Add to your Windsurf MCP configuration:

**macOS**: `~/.codeium/windsurf/mcp_config.json`

**Windows**: `%USERPROFILE%\.codeium\windsurf\mcp_config.json`

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-profile", "--read-only=false"]
    }
  }
}
```

Restart Windsurf after updating the configuration.

## Configuring VS Code with Continue

If you're using [Continue](https://continue.dev/) in VS Code, add to your Continue configuration:

**Config location**: `~/.continue/config.json`

```json
{
  "experimental": {
    "modelContextProtocolServers": [
      {
        "transport": {
          "type": "stdio",
          "command": "redisctl-mcp",
          "args": ["--profile", "my-profile", "--read-only=false"]
        }
      }
    ]
  }
}
```

## Configuring Zed

Add to your Zed settings (`~/.config/zed/settings.json` on Linux/macOS):

```json
{
  "context_servers": {
    "redisctl": {
      "command": {
        "path": "redisctl-mcp",
        "args": ["--profile", "my-profile", "--read-only=false"]
      }
    }
  }
}
```

## Tool Selection

By default the MCP server loads all compiled-in toolsets (300+ tools). Use `--tools` to load only what you need:

```bash
# Cloud tools only
redisctl-mcp --profile my-cloud --tools cloud

# Specific sub-modules
redisctl-mcp --profile my-cloud --tools cloud:subscriptions,cloud:account

# Enterprise monitoring subset
redisctl-mcp --profile my-re --tools enterprise:cluster,enterprise:observability
```

See [Configuration](../mcp/configuration.md) for the full `--tools` syntax, safety tiers, and presets.

## Available Tools

The MCP server provides **340 tools** across 4 toolsets:

| Toolset | Tools | Description |
|---------|-------|-------------|
| **Cloud** | 148 | Subscriptions, databases, networking, Essentials, account management |
| **Enterprise** | 92 | Cluster, databases, RBAC, observability, proxies, services |
| **Database** | 90 | Direct Redis operations -- keys, data structures, diagnostics |
| **App** | 8 | Profile and configuration management |
| **System** | 2 | `list_available_tools`, `show_policy` (always available) |

For the complete tool catalog with sub-module breakdowns, see the [Tools Reference](../mcp/tools-reference.md).

## Example Conversations

Once configured, you can interact naturally with your Redis infrastructure:

> **You**: What databases do I have in my enterprise cluster?
>
> **Claude**: *uses list_enterprise_databases*
> You have 2 databases:
>
> - `default-db` (uid: 1) - 1GB, active
> - `cache-db` (uid: 2) - 256MB, active

> **You**: Create a new database called session-store with 512MB
>
> **Claude**: *uses create_enterprise_database*
> Created database `session-store` (uid: 3) with 512MB memory. Status: active.

> **You**: Show me any active alerts
>
> **Claude**: *uses list_alerts*
> No active alerts in your cluster.

> **You**: What Essentials plans are available on AWS?
>
> **Claude**: *uses list_fixed_plans with provider: AWS*
> Here are the available Essentials plans on AWS:
>
> - 250MB Cache ($5/month) - us-east-1
> - 1GB Cache ($18/month) - us-east-1
> - 250MB Persistence ($6/month) - us-east-1
> - ...

> **You**: Create a new Essentials subscription with the 250MB plan
>
> **Claude**: *uses create_fixed_subscription*
> Created Essentials subscription `my-cache` with plan 250MB. Task ID: abc123. Use `get_task` to monitor progress.

## Security Considerations

### Read-Only Mode (Default)

By default, the MCP server runs in read-only mode. This prevents any destructive operations and is recommended for:

- Exploring your infrastructure
- Monitoring and reporting
- Learning about your deployments

### Write Mode

Use `--read-only=false` when you need to create or modify resources. Consider:

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
redisctl --profile my-profile enterprise cluster get

# Test the MCP server directly
redisctl-mcp --profile my-profile
```

### Claude can't find the server

1. Ensure `redisctl-mcp` is in your PATH or use an absolute path
2. Restart your IDE after config changes
3. Check MCP logs for connection errors

### Operations timing out

The MCP server inherits redisctl's timeout settings from the profile configuration.

## Protocol Details

The MCP server uses:

- **Transport**: stdio (standard input/output)
- **Protocol Version**: 2024-11-05
- **Capabilities**: Tools only (no resources or prompts currently)

For MCP protocol details, see the [MCP Specification](https://spec.modelcontextprotocol.io/).
