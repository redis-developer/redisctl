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
# West region cluster
redisctl profile set cluster-west \
  --type enterprise \
  --url https://west.example.com:9443 \
  --username admin@redis.local \
  --password YOUR_PASSWORD \
  --insecure

# East region cluster
redisctl profile set cluster-east \
  --type enterprise \
  --url https://east.example.com:9443 \
  --username admin@redis.local \
  --password YOUR_PASSWORD \
  --insecure

# Central region cluster
redisctl profile set cluster-central \
  --type enterprise \
  --url https://central.example.com:9443 \
  --username admin@redis.local \
  --password YOUR_PASSWORD \
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

### Multi-Profile Configuration

For multi-cluster management, specify multiple profiles when starting the MCP server:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": [
        "--profile", "cluster-west",
        "--profile", "cluster-east",
        "--profile", "cluster-central"
      ]
    }
  }
}
```

With multiple profiles configured, the AI can target specific clusters directly using the `profile` parameter on any tool call - no need to switch defaults.

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
| "Show cluster info for cluster-central" | Query a specific cluster |
| "Check the license on cluster-west" | Query with profile targeting |
| "List databases across all my clusters" | Aggregate cross-cluster query |

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
> | cluster-west | https://west:9443 | Yes |
> | cluster-east | https://east:9443 | No |
> | cluster-central | https://central:9443 | No |

> **You**: Check the license on all my clusters
>
> **AI**: I'll check each cluster:
>
> **cluster-west**: Trial license, expires March 15, 2026 (38 days remaining)
> - Shards: 2/4 used
> - Status: OK
>
> **cluster-east**: Trial license, expires February 28, 2026 (23 days remaining)
> - Shards: 3/4 used
> - Status: WARNING - Expiring soon
>
> **cluster-central**: Trial license, expires February 20, 2026 (15 days remaining)
> - Shards: 3/4 used
> - Status: WARNING - Expiring soon

> **You**: What's my total memory allocation across all clusters?
>
> **AI**: Total memory across 3 clusters: **1.1 GB**
>
> | Cluster | Databases | Memory |
> |---------|-----------|--------|
> | cluster-west | 2 | 200 MB |
> | cluster-east | 3 | 350 MB |
> | cluster-central | 3 | 550 MB |

## Multi-Cluster Workflow

When configured with multiple profiles, the AI can query across clusters in two ways:

### Direct Profile Targeting (Recommended)

Every Enterprise tool accepts an optional `profile` parameter. The AI can target any cluster directly without changing defaults:

```
"Get the license status from cluster-west"
→ AI calls get_license with profile="cluster-west"

"List databases on cluster-east"
→ AI calls list_enterprise_databases with profile="cluster-east"

"Check alerts across all my clusters"
→ AI calls list_alerts for each profile, aggregates results
```

### Profile Switching (Legacy)

Alternatively, the AI can switch the default profile:

1. **List profiles** - AI discovers your configured clusters
2. **Set default** - AI switches to a cluster using `profile_set_default_enterprise`
3. **Query** - AI runs tools against the current default
4. **Repeat** - AI switches and queries other clusters
5. **Aggregate** - AI combines results into a unified view

Direct profile targeting is more efficient for cross-cluster queries since it doesn't require switching defaults between each call.

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
