---
name: redisctl-cloud-management
description: Manage Redis Cloud subscriptions, databases, and resources via the redisctl CLI. Use when provisioning, updating, or monitoring Redis Cloud infrastructure.
---

## Overview

Manage the full lifecycle of Redis Cloud resources: subscriptions, databases, users, ACLs, and tasks.

## Subscriptions

```bash
# List all subscriptions
redisctl cloud subscription list

# Get subscription details
redisctl cloud subscription get --id 12345

# Create a subscription (JSON input)
redisctl cloud subscription create --data '{...}'

# Update a subscription
redisctl cloud subscription update --id 12345 --data '{...}'

# Delete a subscription
redisctl cloud subscription delete --id 12345
```

## Databases

```bash
# List databases in a subscription
redisctl cloud database list --subscription-id 12345

# Get database details
redisctl cloud database get --subscription-id 12345 --id 67890

# Create a database
redisctl cloud database create --subscription-id 12345 --data '{...}'

# Update a database
redisctl cloud database update --subscription-id 12345 --id 67890 --data '{...}'

# Delete a database
redisctl cloud database delete --subscription-id 12345 --id 67890

# Pause/recover a database
redisctl cloud database pause --subscription-id 12345 --id 67890
redisctl cloud database recover --subscription-id 12345 --id 67890

# Backup a database
redisctl cloud database backup --subscription-id 12345 --id 67890
```

## Essentials / Fixed Tier

For smaller, fixed-price databases:

```bash
redisctl cloud fixed-subscription list
redisctl cloud fixed-database list --subscription-id 12345
redisctl cloud fixed-database create --subscription-id 12345 --data '{...}'
```

## Task Tracking

Cloud operations are async. Track tasks:

```bash
# List recent tasks
redisctl cloud task list

# Get task status
redisctl cloud task get --id <task-id>

# Wait for a task to complete
redisctl cloud task wait --id <task-id>
```

## Workflows

Multi-step operations:

```bash
# Complete subscription setup with optional database
redisctl cloud workflow subscription-setup
```

## Cost Reports

```bash
# Generate a cost report (FOCUS format)
redisctl cloud cost-report generate --start 2026-01-01 --end 2026-01-31

# Download a generated report
redisctl cloud cost-report download --id <report-id>

# Generate and download in one step
redisctl cloud cost-report export --start 2026-01-01 --end 2026-01-31
```

## Account Management

```bash
# Get account details
redisctl cloud account get

# List/manage users
redisctl cloud user list
redisctl cloud user create --data '{...}'

# ACL management
redisctl cloud acl list
redisctl cloud acl create --data '{...}'
```

## Tips

- Use `--profile <name>` to target a specific Cloud profile
- Use `--output json` for machine-readable output
- Most create/update commands accept `--data` with a JSON payload or `--file` for a JSON file
- Task IDs are returned from async operations -- use `cloud task wait` to block until completion
