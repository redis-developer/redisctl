# Enterprise Quickstart

Get up and running with MCP for Redis Enterprise in 5 minutes.

## Prerequisites

- Redis Enterprise cluster with REST API access (port 9443)
- Admin credentials for your cluster(s)
- redisctl installed ([Installation Guide](../getting-started/installation.md))

## Step 1: Create a Profile

```bash
redisctl profile set my-cluster \
  --type enterprise \
  --url https://your-cluster:9443 \
  --username admin@redis.local \
  --password YOUR_PASSWORD \
  --insecure  # if using self-signed certificates
```

Verify it works:

```bash
redisctl -p my-cluster enterprise cluster get
```

### Multiple Clusters

A key advantage of redisctl is **multi-cluster management**. Add profiles for each cluster:

```bash
# Production cluster
redisctl profile set prod-cluster \
  --type enterprise \
  --url https://prod.example.com:9443 \
  --username admin \
  --password PROD_PASSWORD

# Staging cluster
redisctl profile set staging-cluster \
  --type enterprise \
  --url https://staging.example.com:9443 \
  --username admin \
  --password STAGING_PASSWORD

# Development cluster
redisctl profile set dev-cluster \
  --type enterprise \
  --url https://dev.example.com:9443 \
  --username admin \
  --password DEV_PASSWORD \
  --insecure
```

## Step 2: Configure Your AI Assistant

Add to your MCP configuration (read-only by default - explore safely without modifying anything):

=== "Claude Desktop"

    **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "redisctl-mcp",
          "args": ["--profile", "my-cluster"]
        }
      }
    }
    ```

=== "Claude Code"

    Add to `.mcp.json` in your project:

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "redisctl-mcp",
          "args": ["--profile", "my-cluster"]
        }
      }
    }
    ```

=== "Cursor"

    **macOS**: `~/.cursor/mcp.json`

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "redisctl-mcp",
          "args": ["--profile", "my-cluster"]
        }
      }
    }
    ```

Restart your AI assistant to load the new configuration.

## Step 3: Try These Prompts

Start a conversation with your AI assistant and try these examples:

### Cluster Overview

| Prompt | What it does |
|--------|--------------|
| "Show me the cluster info" | Cluster name, version, node count |
| "List all nodes in the cluster" | Node IDs, addresses, status |
| "What's the cluster health?" | Overall status and any issues |

### Database Management

| Prompt | What it does |
|--------|--------------|
| "List all databases" | All databases with memory and status |
| "Show database 1 details" | Full configuration for a database |
| "Which databases have persistence enabled?" | Filter by configuration |
| "What modules are loaded?" | List available modules |

### License Management

| Prompt | What it does |
|--------|--------------|
| "Check the license status" | Expiration, type, features |
| "Show license usage" | Shards and RAM used vs limits |
| "Is the license expiring soon?" | Quick expiration check |

### Monitoring

| Prompt | What it does |
|--------|--------------|
| "Are there any alerts?" | Active cluster alerts |
| "Show cluster statistics" | CPU, memory, throughput |
| "Get stats for database 1" | Per-database metrics |

### Multi-Cluster Operations

| Prompt | What it does |
|--------|--------------|
| "List my profiles" | Shows all configured clusters |
| "Switch to prod-cluster" | Changes default Enterprise profile |
| "Show cluster info for staging-cluster" | Query a specific cluster |

## Optional: Enable Write Operations

By default, the MCP server runs in **read-only mode** - you can explore your clusters freely without risk of modifying anything. When you're ready for write operations, update your config:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-cluster", "--read-only=false"]
    }
  }
}
```

Write prompts to try:

| Prompt | What it does |
|--------|--------------|
| "Create a 2GB database called cache-db" | Creates a new database |
| "Enable maintenance mode" | Prepares cluster for upgrades |
| "Update the cluster license with: LICENSE_STRING" | Applies a new license |

## Example Conversation

> **You**: What profiles do I have configured?
>
> **AI**: You have 3 Enterprise profiles:
>
> | Name | URL | Default |
> |------|-----|---------|
> | prod-cluster | https://prod:9443 | Yes |
> | staging-cluster | https://staging:9443 | No |
> | dev-cluster | https://dev:9443 | No |

> **You**: Check the license on all my clusters
>
> **AI**: I'll check each cluster:
>
> **prod-cluster**: Enterprise license, expires March 15, 2027 (407 days remaining)
> - Shards: 24/100 used
> - Status: OK
>
> **staging-cluster**: Trial license, expires February 28, 2026 (23 days remaining)
> - Shards: 4/4 used
> - Status: WARNING - Expiring soon, at shard limit
>
> **dev-cluster**: Trial license, expires February 20, 2026 (15 days remaining)
> - Shards: 2/4 used
> - Status: WARNING - Expiring soon

> **You**: What's my total memory allocation across all clusters?
>
> **AI**: Total memory across 3 clusters: **52 GB**
>
> | Cluster | Databases | Memory |
> |---------|-----------|--------|
> | prod-cluster | 8 | 40 GB |
> | staging-cluster | 3 | 8 GB |
> | dev-cluster | 2 | 4 GB |

## Multi-Cluster Workflow

The MCP server can query across multiple clusters by switching profiles:

1. **List profiles** - AI discovers your configured clusters
2. **Set default** - AI switches to a cluster using `profile_set_default_enterprise`
3. **Query** - AI runs tools against the current default
4. **Repeat** - AI switches and queries other clusters
5. **Aggregate** - AI combines results into a unified view

This enables powerful cross-cluster operations that aren't possible with the native Redis Enterprise UI.

## Troubleshooting

### "Connection refused"

- Verify the cluster URL and port (default: 9443)
- Check firewall rules allow access
- Ensure the REST API is enabled

### "Authentication failed"

- Verify username and password
- Check the user has appropriate permissions

### "Certificate error"

- Add `--insecure` for self-signed certificates
- Or configure a CA certificate with `--ca-cert`

### Tools not appearing

- Restart your AI assistant after config changes
- Check that `redisctl-mcp` is in your PATH

## Next Steps

- [Tools Reference](tools-reference.md) - Complete list of Enterprise tools
- [Advanced Usage](advanced-usage.md) - Complex queries with JMESPath
- [Cloud Quickstart](cloud-quickstart.md) - If you also use Redis Cloud
