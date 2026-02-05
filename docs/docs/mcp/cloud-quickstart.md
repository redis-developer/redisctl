# Cloud Quickstart

Get up and running with MCP for Redis Cloud in 5 minutes.

## Prerequisites

- Redis Cloud account with API access enabled
- API key and secret key from [Redis Cloud Console](https://cloud.redis.io/) → Account → API Keys
- redisctl installed ([Installation Guide](../getting-started/installation.md))

## Step 1: Create a Profile

```bash
redisctl profile set my-cloud \
  --type cloud \
  --api-key YOUR_API_KEY \
  --api-secret YOUR_SECRET_KEY
```

Verify it works:

```bash
redisctl -p my-cloud cloud subscription list
```

## Step 2: Configure Your AI Assistant

Add to your MCP configuration (read-only by default for safe exploration):

=== "Claude Desktop"

    **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "redisctl-mcp",
          "args": ["--profile", "my-cloud"]
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
          "args": ["--profile", "my-cloud"]
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
          "args": ["--profile", "my-cloud"]
        }
      }
    }
    ```

Restart your AI assistant to load the new configuration.

## Step 3: Try These Prompts

Start a conversation with your AI assistant and try these examples:

### Explore Your Infrastructure

| Prompt | What it does |
|--------|--------------|
| "List my Redis Cloud subscriptions" | Shows all your subscriptions with IDs and names |
| "Show databases in subscription 12345" | Lists databases in a specific subscription |
| "What's my current account info?" | Displays account details and limits |

### Database Information

| Prompt | What it does |
|--------|--------------|
| "Show me database 67890 details" | Full database configuration |
| "What modules are enabled on my databases?" | Lists databases with their modules |
| "Which databases have persistence enabled?" | Filters by configuration |

### Monitoring

| Prompt | What it does |
|--------|--------------|
| "Are there any active tasks?" | Shows pending async operations |
| "What's the status of task abc-123?" | Checks a specific task |
| "Show me my cost report for this month" | Displays billing information |

## Optional: Enable Write Operations

By default, the MCP server runs in **read-only mode** - you can explore freely without risk of modifying anything. When you're ready for write operations, update your config:

```json
{
  "mcpServers": {
    "redisctl": {
      "command": "redisctl-mcp",
      "args": ["--profile", "my-cloud", "--read-only=false"]
    }
  }
}
```

Write prompts to try:

| Prompt | What it does |
|--------|--------------|
| "Create a 1GB database called cache-prod in subscription 12345" | Creates a new database |
| "Backup database 67890" | Triggers an on-demand backup |

## Example Conversation

> **You**: What subscriptions do I have?
>
> **AI**: You have 2 subscriptions:
>
> | ID | Name | Cloud | Region | Databases |
> |----|------|-------|--------|-----------|
> | 12345 | Production | AWS | us-east-1 | 3 |
> | 12346 | Staging | GCP | us-central1 | 1 |

> **You**: Show me the databases in Production
>
> **AI**: Production (subscription 12345) has 3 databases:
>
> | ID | Name | Memory | Status |
> |----|------|--------|--------|
> | 101 | users-cache | 2 GB | active |
> | 102 | sessions | 512 MB | active |
> | 103 | analytics | 4 GB | active |

> **You**: What's the total memory I'm using?
>
> **AI**: Across your 2 subscriptions, you're using **6.5 GB** of memory:
> - Production: 6.5 GB (3 databases)
> - Staging: 256 MB (1 database)

## Troubleshooting

### "No subscriptions found"

- Verify your API key has the correct permissions
- Check that the account has subscriptions created

### "Authentication failed"

- Ensure API key and secret are correct
- Verify API access is enabled in Cloud Console

### Tools not appearing

- Restart your AI assistant after config changes
- Check that `redisctl-mcp` is in your PATH

## Next Steps

- [Tools Reference](tools-reference.md) - Complete list of Cloud tools
- [Advanced Usage](advanced-usage.md) - Complex queries with JMESPath
- [Enterprise Quickstart](enterprise-quickstart.md) - If you also use Redis Enterprise
