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

```bash
# Start the MCP server (read-only mode, safe for exploration)
redisctl -p my-profile mcp serve

# Enable write operations (create, update, delete)
redisctl -p my-profile mcp serve --allow-writes

# List available tools
redisctl mcp tools
```

## Configuring Claude Desktop

Add the following to your Claude Desktop configuration file:

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

For write operations:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "/path/to/redisctl",
      "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
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
      "command": "redisctl",
      "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
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
      "command": "/path/to/redisctl",
      "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
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
      "command": "/path/to/redisctl",
      "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
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
          "command": "/path/to/redisctl",
          "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
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
        "path": "/path/to/redisctl",
        "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
      }
    }
  }
}
```

## Available Tools

### Redis Cloud Tools (17 tools)

#### Account & Infrastructure

| Tool | Description |
|------|-------------|
| `cloud_account_get` | Get account information |
| `cloud_payment_methods_get` | List all payment methods configured for your account |
| `cloud_database_modules_get` | List all available database modules (capabilities) |
| `cloud_regions_get` | Get available regions across cloud providers (AWS, GCP, Azure) |

#### Pro Subscriptions

| Tool | Description |
|------|-------------|
| `cloud_subscriptions_list` | List all Pro subscriptions |
| `cloud_subscription_get` | Get Pro subscription details |
| `cloud_pro_subscription_create` | Create a new Pro subscription *(write)* |
| `cloud_pro_subscription_delete` | Delete a Pro subscription *(write)* |

#### Essentials Subscriptions

| Tool | Description |
|------|-------------|
| `cloud_essentials_subscriptions_list` | List all Essentials subscriptions |
| `cloud_essentials_subscription_get` | Get Essentials subscription details |
| `cloud_essentials_subscription_create` | Create a new Essentials subscription *(write)* |
| `cloud_essentials_subscription_delete` | Delete an Essentials subscription *(write)* |
| `cloud_essentials_plans_list` | List available Essentials plans with pricing |

#### Database & Task Operations

| Tool | Description |
|------|-------------|
| `cloud_databases_list` | List databases in a subscription |
| `cloud_database_get` | Get database details |
| `cloud_tasks_list` | List recent async tasks |
| `cloud_task_get` | Get task status |

### Redis Enterprise Tools (48 tools)

#### Cluster Operations

| Tool | Description |
|------|-------------|
| `enterprise_cluster_get` | Get cluster information |
| `enterprise_cluster_stats` | Get cluster statistics |
| `enterprise_cluster_settings` | Get cluster settings |
| `enterprise_cluster_topology` | Get cluster topology |
| `enterprise_cluster_update` | Update cluster configuration *(write)* |

#### Database Operations

| Tool | Description |
|------|-------------|
| `enterprise_databases_list` | List all databases |
| `enterprise_database_get` | Get database details |
| `enterprise_database_stats` | Get database statistics |
| `enterprise_database_metrics` | Get database performance metrics |
| `enterprise_database_create` | Create a new database *(write)* |
| `enterprise_database_update` | Update database configuration *(write)* |
| `enterprise_database_delete` | Delete a database *(write)* |
| `enterprise_database_flush` | Flush all data from database *(write)* |
| `enterprise_database_export` | Export database to external location *(write)* |
| `enterprise_database_import` | Import data into database *(write)* |
| `enterprise_database_backup` | Trigger database backup *(write)* |
| `enterprise_database_restore` | Restore database from backup *(write)* |

#### Node Operations

| Tool | Description |
|------|-------------|
| `enterprise_nodes_list` | List all cluster nodes |
| `enterprise_node_get` | Get node details |
| `enterprise_node_stats` | Get node statistics |
| `enterprise_node_update` | Update node configuration *(write)* |
| `enterprise_node_remove` | Remove node from cluster *(write)* |

#### Shard & Alert Operations

| Tool | Description |
|------|-------------|
| `enterprise_shards_list` | List all shards |
| `enterprise_shard_get` | Get shard details |
| `enterprise_alerts_list` | List active alerts |
| `enterprise_alert_get` | Get alert details |

#### User & Access Management

| Tool | Description |
|------|-------------|
| `enterprise_users_list` | List all users |
| `enterprise_user_get` | Get user details |
| `enterprise_user_create` | Create a new user *(write)* |
| `enterprise_user_delete` | Delete a user *(write)* |
| `enterprise_roles_list` | List all roles |
| `enterprise_role_get` | Get role details |
| `enterprise_role_create` | Create a new role *(write)* |
| `enterprise_role_delete` | Delete a role *(write)* |
| `enterprise_acls_list` | List all Redis ACLs |
| `enterprise_acl_get` | Get ACL details |
| `enterprise_acl_create` | Create a new Redis ACL *(write)* |
| `enterprise_acl_delete` | Delete a Redis ACL *(write)* |

#### Other Operations

| Tool | Description |
|------|-------------|
| `enterprise_logs_get` | Get cluster event logs |
| `enterprise_license_get` | Get license information |
| `enterprise_modules_list` | List available modules |
| `enterprise_module_get` | Get module details |
| `enterprise_crdbs_list` | List Active-Active databases |
| `enterprise_crdb_get` | Get Active-Active database details |
| `enterprise_crdb_update` | Update Active-Active database *(write)* |
| `enterprise_crdb_delete` | Delete Active-Active database *(write)* |
| `enterprise_debuginfo_list` | List debug info tasks |
| `enterprise_debuginfo_status` | Get debug info task status |

**Total: 65 tools** (17 Cloud + 48 Enterprise)

## Example Conversations

Once configured, you can interact naturally with your Redis infrastructure:

> **You**: What databases do I have in my enterprise cluster?
>
> **Claude**: *uses enterprise_databases_list*
> You have 2 databases:
>
> - `default-db` (uid: 1) - 1GB, active
> - `cache-db` (uid: 2) - 256MB, active

> **You**: Create a new database called session-store with 512MB
>
> **Claude**: *uses enterprise_database_create*
> Created database `session-store` (uid: 3) with 512MB memory. Status: active.

> **You**: Show me any active alerts
>
> **Claude**: *uses enterprise_alerts_list*
> No active alerts in your cluster.

> **You**: What Essentials plans are available on AWS?
>
> **Claude**: *uses cloud_essentials_plans_list with provider: AWS*
> Here are the available Essentials plans on AWS:
>
> - 250MB Cache ($5/month) - us-east-1
> - 1GB Cache ($18/month) - us-east-1
> - 250MB Persistence ($6/month) - us-east-1
> - ...

> **You**: Create a new Essentials subscription with the 250MB plan
>
> **Claude**: *uses cloud_essentials_subscription_create*
> Created Essentials subscription `my-cache` with plan 250MB. Task ID: abc123. Use cloud_task_get to monitor progress.

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

### Claude can't find the server

1. Ensure the path to redisctl is absolute in your config
2. Restart Claude Desktop after config changes
3. Check Claude's MCP logs for connection errors

### Operations timing out

The MCP server inherits redisctl's timeout settings. For slow operations:

```bash
# Enterprise operations may need longer timeouts
redisctl -p my-profile mcp serve --allow-writes
```

## Protocol Details

The MCP server uses:

- **Transport**: stdio (standard input/output)
- **Protocol Version**: 2024-11-05
- **Capabilities**: Tools only (no resources or prompts currently)

For MCP protocol details, see the [MCP Specification](https://spec.modelcontextprotocol.io/).
