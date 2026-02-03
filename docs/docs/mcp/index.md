# MCP Server

redisctl includes a built-in [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that enables AI assistants to manage your Redis deployments through natural language.

## What is MCP?

The Model Context Protocol is an open standard that allows AI systems to securely interact with external tools and data sources. The redisctl MCP server exposes Redis management operations as tools that AI assistants like Claude, Cursor, and others can discover and invoke.

## What Can You Do?

With the MCP server, you can:

- **Query infrastructure** - "List all my Redis databases" or "Show cluster health"
- **Create resources** - "Create a new 256MB database called cache-db"
- **Monitor status** - "What's the license expiration date?" or "Any active alerts?"
- **Work with data directly** - "Add this user to my hash" or "Show me the top 10 leaderboard scores"
- **Use Redis Stack modules** - Full-text search, JSON documents, time series, Bloom filters, Streams, and Pub/Sub
- **Analyze data** - Combine with JMESPath for advanced querying and reporting

## Key Features

<div class="grid cards" markdown>

-   :material-shield-check:{ .lg .middle } **Secure by Default**

    ---

    Read-only mode by default. Credentials stay in your profiles, never exposed to AI.

-   :material-cloud:{ .lg .middle } **Full Coverage**

    ---

    237 tools covering Redis Cloud, Redis Enterprise, and direct database operations including Redis Stack modules.

-   :material-cog:{ .lg .middle } **IDE Integration**

    ---

    Works with Claude Desktop, Claude Code, Cursor, Windsurf, VS Code, and Zed.

-   :material-chart-line:{ .lg .middle } **Advanced Analytics**

    ---

    Combine with JMESPath MCP server for complex queries and reporting.

</div>

## Quick Example

Once configured, interact naturally with your Redis infrastructure:

> **You**: What databases do I have in my enterprise cluster?
>
> **Claude**: You have 2 databases:
>
> - `default-db` (uid: 1) - 1GB, active, with modules: RediSearch, RedisJSON
> - `cache-db` (uid: 2) - 256MB, active

> **You**: Show me the license status
>
> **Claude**: Your cluster is running a Trial license:
>
> - **Expires**: February 11, 2026 (30 days remaining)
> - **Shards**: 1 of 4 used (25% utilization)
> - **Features**: bigstore enabled

## Getting Started

Ready to set up MCP? Head to the [Getting Started](getting-started.md) guide.

Want to see what's possible? Check out [Advanced Usage](advanced-usage.md) for JMESPath integration and complex analytics pipelines.
