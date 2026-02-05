# redisctl

> **A modern CLI for Redis Cloud and Redis Enterprise** — Automate deployments, manage resources, and troubleshoot issues from one unified interface.

[![Crates.io](https://img.shields.io/crates/v/redisctl.svg)](https://crates.io/crates/redisctl)
[![Documentation](https://docs.rs/redisctl/badge.svg)](https://docs.rs/redisctl)
[![CI](https://github.com/redis-developer/redisctl/actions/workflows/ci.yml/badge.svg)](https://github.com/redis-developer/redisctl/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/redis-developer/redisctl#license)

```bash
# Create a Redis Cloud subscription with one command
redisctl cloud subscription create @subscription.json --wait

# Stream logs in real-time
redisctl enterprise logs list --follow

# Generate and upload support packages
redisctl enterprise support-package cluster --optimize --upload
```

---

## Why redisctl?

Managing Redis Cloud and Redis Enterprise through REST APIs means juggling curl commands, parsing JSON, and manually polling for operation completion. **redisctl eliminates that friction.**

### What You Get

- **One CLI for Everything** — Manage both Redis Cloud and Enterprise from a single tool  
- **Intelligent Async Handling** — `--wait` flags automatically poll long-running operations  
- **Real-Time Streaming** — Tail logs and metrics with `--follow`  
- **Automated Workflows** — High-level commands like `subscription-setup` handle complex tasks  
- **Smart Output** — Tables for humans, JSON for scripts, with JMESPath filtering built-in  
- **Production Ready** — Secure credential storage, profile management, and comprehensive error handling

---

## Quick Start

### 1. Install

```bash
# Homebrew (macOS/Linux)
brew install redis-developer/homebrew-tap/redisctl

# Cargo
cargo install redisctl

# Or download from releases
# https://github.com/redis-developer/redisctl/releases
```

### 2. Configure

```bash
# Redis Cloud
redisctl profile set prod \
  --deployment cloud \
  --api-key "$REDIS_CLOUD_API_KEY" \
  --api-secret "$REDIS_CLOUD_SECRET_KEY"

# Redis Enterprise  
redisctl profile set dev \
  --deployment enterprise \
  --url "https://cluster.local:9443" \
  --username "admin@redis.local" \
  --password "$REDIS_ENTERPRISE_PASSWORD"
```

### 3. Run Your First Commands

```bash
# List all databases
redisctl cloud database list

# Get cluster info in table format
redisctl enterprise cluster get -o table

# Create a database and wait for it to be ready
redisctl cloud database create @db-config.json --wait

# Stream cluster logs
redisctl enterprise logs list --follow
```

**That's it!** You're ready to manage your Redis deployments.

[**Full Documentation →**](https://redis-field-engineering.github.io/redisctl-docs/)

---

## Feature Showcase

### Async Operations Made Easy

No more manual polling. The `--wait` flag handles it automatically:

```bash
# Old way: Create and manually check status
curl -X POST .../databases -d @config.json
# Wait...check status...wait...check again...

# New way: Create and wait automatically
redisctl cloud database create @config.json --wait
# ✓ Database created and ready in 45s
```

### Flexible Output Formats

```bash
# Human-friendly tables
redisctl cloud subscription list -o table

# Machine-readable JSON
redisctl cloud database list -o json

# Filter with JMESPath
redisctl cloud database list -q 'databases[?status==`active`].name'
```

### High-Level Workflows

Complex multi-step operations in one command:

```bash
# Set up a complete subscription with databases, ACLs, and networking
redisctl cloud workflow subscription-setup @workflow.yaml

# Results in:
# ✓ Subscription created
# ✓ VPC peering configured
# ✓ Databases provisioned
# ✓ ACL rules applied
# ✓ Ready for production
```

### Real-Time Streaming

Monitor your infrastructure live:

```bash
# Tail cluster logs
redisctl enterprise logs list --follow

# Watch with custom poll interval
redisctl enterprise logs list --follow --poll-interval 1
```

### Support Package Automation

Generate diagnostic packages and upload to Redis Support in one step:

```bash
# Generate, optimize, and upload cluster diagnostics
redisctl enterprise support-package cluster \
  --optimize \
  --upload \
  --no-save

# Saves 20-30% space and uploads directly to Files.com
# ✓ Package generated (542 MB)
# ✓ Optimized to 389 MB
# ✓ Uploaded to Redis Support
```

---

## Real-World Examples

### Scenario: Deploy a New Database

```bash
# 1. Check available subscriptions
redisctl cloud subscription list -o table

# 2. Create database config
cat > database.json <<EOF
{
  "name": "production-cache",
  "protocol": "redis",
  "memoryLimitInGb": 5.0,
  "replication": true,
  "dataEvictionPolicy": "allkeys-lru",
  "throughputMeasurement": {
    "by": "operations-per-second",
    "value": 25000
  }
}
EOF

# 3. Create and wait for provisioning
redisctl cloud database create \
  --subscription 12345 \
  database.json \
  --wait \
  -o json | jq '{id: .databaseId, endpoint: .publicEndpoint}'

# Output:
# {
#   "id": 67890,
#   "endpoint": "redis-12345.c1.us-east-1-1.ec2.redislabs.com:12000"
# }
```

### Scenario: Troubleshoot Cluster Issues

```bash
# 1. Check cluster health
redisctl enterprise cluster get -q 'name, state'

# 2. Stream logs for errors
redisctl enterprise logs list --follow | grep ERROR

# 3. Generate support package if needed
redisctl enterprise support-package cluster --optimize --upload

# 4. Check specific node
redisctl enterprise node get 1 -o table
```

### Scenario: Automate Database Backups

```bash
#!/bin/bash
# backup-all-databases.sh

# Get all active databases
databases=$(redisctl cloud database list \
  -q 'databases[?status==`active`].[subscriptionId,databaseId]' \
  -o json)

# Backup each one
echo "$databases" | jq -r '.[] | "\(.[0]) \(.[1])"' | while read sub_id db_id; do
  echo "Backing up database $db_id..."
  redisctl cloud database backup \
    --subscription "$sub_id" \
    --database "$db_id" \
    --wait
done

echo "All backups complete!"
```

### Scenario: Manage Active-Active (CRDB) Databases

```bash
# Create Active-Active database across multiple regions
redisctl cloud database create @crdb-config.json --wait

# Add a new region
redisctl cloud subscription add-aa-region \
  --subscription 12345 \
  @region-config.json \
  --wait

# Monitor replication status
redisctl cloud database get 67890 \
  -q 'replication.{status: status, regions: regions[].{name: region, status: status}}'
```

---

## Installation

### Homebrew (macOS/Linux)
```bash
brew install redis-developer/homebrew-tap/redisctl
```

### Cargo (Rust)
```bash
# Basic installation
cargo install redisctl

# With secure keyring support (recommended)
cargo install redisctl --features secure-storage
```

### Binary Releases
Download the latest release for your platform from [GitHub Releases](https://github.com/redis-developer/redisctl/releases).

Binaries are available for:
- macOS (Intel and Apple Silicon)
- Linux (x86_64 and ARM64)
- Windows (x86_64)

### Docker
```bash
# Run directly
docker run --rm \
  -e REDIS_CLOUD_API_KEY \
  -e REDIS_CLOUD_SECRET_KEY \
  ghcr.io/redis-developer/redisctl:latest \
  cloud subscription list

# Mount config for persistent profiles
docker run --rm \
  -v ~/.config/redisctl:/root/.config/redisctl:ro \
  ghcr.io/redis-developer/redisctl:latest \
  cloud database list

# Development environment
docker compose up -d  # Start test cluster
```

---

## Configuration

### Environment Variables

The fastest way to get started:

```bash
# Redis Cloud
export REDIS_CLOUD_API_KEY="your-api-key"
export REDIS_CLOUD_SECRET_KEY="your-secret-key"

# Redis Enterprise
export REDIS_ENTERPRISE_URL="https://cluster.local:9443"
export REDIS_ENTERPRISE_USER="admin@redis.local"
export REDIS_ENTERPRISE_PASSWORD="your-password"
export REDIS_ENTERPRISE_INSECURE="true"  # For self-signed certs
```

### Profiles

For managing multiple environments:

```bash
# Create profiles
redisctl profile set prod --deployment cloud --api-key xxx --api-secret yyy
redisctl profile set staging --deployment cloud --api-key aaa --api-secret bbb
redisctl profile set dev --deployment enterprise --url https://localhost:9443 ...

# List profiles
redisctl profile list

# Use a specific profile
redisctl --profile prod cloud database list

# Set default
redisctl profile default-cloud prod
```

### Secure Storage

Store credentials in your OS keyring instead of plain text:

```bash
# Requires: cargo install redisctl --features secure-storage

redisctl profile set prod \
  --deployment cloud \
  --api-key "$REDIS_CLOUD_API_KEY" \
  --api-secret "$REDIS_CLOUD_SECRET_KEY" \
  --use-keyring  # Stores in macOS Keychain, Windows Credential Store, or Linux Secret Service
```

Config file location:
- **Linux/macOS**: `~/.config/redisctl/config.toml`
- **Windows**: `%APPDATA%\redis\redisctl\config.toml`

---

## Key Features

### Complete API Coverage

**Redis Cloud** — 100% coverage of Cloud API v1:
- Subscriptions (Pro and Essentials)
- Databases (flexible and fixed)
- VPC Peering, Transit Gateway, PrivateLink, Private Service Connect
- ACLs, Users, Cloud Accounts
- Tasks and Async Operations

**Redis Enterprise** — 100% coverage of Enterprise API v1/v2:
- Clusters, Nodes, Shards
- Databases (BDBs), Active-Active (CRDBs)
- Users, Roles, LDAP
- Logs, Metrics, Alerts
- Support Packages, Diagnostics

### Raw API Access

For any endpoint not yet wrapped in a high-level command:

```bash
# Redis Cloud
redisctl api cloud get /subscriptions/12345/databases
redisctl api cloud post /databases -d @config.json

# Redis Enterprise
redisctl api enterprise get /v1/cluster
redisctl api enterprise put /v1/bdbs/1 -d @update.json
```

### Advanced Output Control

```bash
# JMESPath filtering
redisctl cloud database list -q 'databases[?memoryLimitInGb > `10`]'

# Multiple output formats
redisctl enterprise cluster get -o table
redisctl enterprise cluster get -o json | jq
redisctl enterprise cluster get -o yaml
```

### Python Bindings

Use the API client libraries from Python:

```bash
pip install redis-cloud redis-enterprise
```

```python
from redis_cloud import CloudClient
from redis_enterprise import EnterpriseClient

# Redis Cloud
cloud = CloudClient.from_env()
subs = cloud.subscriptions_sync()

# Redis Enterprise
enterprise = EnterpriseClient.from_env()
dbs = enterprise.databases_sync()

# Async support
async def main():
    subs = await cloud.subscriptions()
```

- [redis-cloud on PyPI](https://pypi.org/project/redis-cloud/)
- [redis-enterprise on PyPI](https://pypi.org/project/redis-enterprise/)

### MCP Server (AI Integration)

redisctl includes an MCP server (`redisctl-mcp`) that enables AI assistants to manage Redis deployments:

```bash
# Start the MCP server
redisctl-mcp --profile my-profile

# Enable write operations
redisctl-mcp --profile my-profile --read-only=false
```

Configure your AI assistant (Claude Desktop, Cursor, etc.) to use it:

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

**For development:** Copy `.mcp.json.example` to `.mcp.json`, update the profile name, and restart your IDE.

See the [MCP Documentation](https://redis-field-engineering.github.io/redisctl-docs/mcp/) for full setup instructions.

---

## Documentation

**[Complete Documentation](https://redis-field-engineering.github.io/redisctl-docs/)**

- [Getting Started Guide](https://redis-field-engineering.github.io/redisctl-docs/getting-started/)
- [Command Reference](https://redis-field-engineering.github.io/redisctl-docs/reference/)
- [Configuration Guide](https://redis-field-engineering.github.io/redisctl-docs/configuration/)
- [Workflow Examples](https://redis-field-engineering.github.io/redisctl-docs/workflows/)
- [Troubleshooting](https://redis-field-engineering.github.io/redisctl-docs/troubleshooting/)

---

## Changelogs

Individual crate changelogs:
- [redisctl CLI](crates/redisctl/CHANGELOG.md) - Command-line interface
- [redisctl-config](crates/redisctl-config/CHANGELOG.md) - Configuration management library
- [redisctl-mcp](crates/redisctl-mcp/CHANGELOG.md) - MCP server

The API client libraries are maintained in separate repositories:
- [redis-cloud](https://github.com/redis-developer/redis-cloud-rs) - Redis Cloud API client
- [redis-enterprise](https://github.com/redis-developer/redis-enterprise-rs) - Redis Enterprise API client

---

## Contributing

Contributions welcome! See our [Contributing Guide](https://redis-field-engineering.github.io/redisctl-docs/developer/contributing.html).

```bash
# Clone and build
git clone https://github.com/redis-developer/redisctl.git
cd redisctl
cargo build --release

# Run tests
cargo test --workspace

# Check code
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## Support

- [Documentation](https://redis-field-engineering.github.io/redisctl-docs/)
- [Issue Tracker](https://github.com/redis-developer/redisctl/issues)
- [Discussions](https://github.com/redis-developer/redisctl/discussions)
