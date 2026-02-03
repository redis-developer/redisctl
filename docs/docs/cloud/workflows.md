# Cloud Workflows

Multi-step operations for Redis Cloud.

## Subscription Setup

Create a subscription with a database in one command:

```bash
redisctl cloud workflow subscription-setup \
  --name production \
  --provider AWS \
  --region us-east-1 \
  --database-name cache \
  --database-memory-gb 2 \
  --wait
```

This creates:
1. A new subscription
2. A database within it
3. Waits for both to be ready

### Options

| Option | Description |
|--------|-------------|
| `--name` | Subscription name |
| `--provider` | AWS, GCP, or Azure |
| `--region` | Cloud region |
| `--database-name` | Database name |
| `--database-memory-gb` | Database memory in GB |
| `--wait` | Wait for completion |

## When to Use Workflows

**Use workflows when:**
- Setting up new environments
- Creating multiple related resources
- Need atomic-like operations

**Use individual commands when:**
- Managing existing resources
- Need fine-grained control
- Debugging issues

## Coming Soon

Additional workflows planned:
- Database migration
- Active-Active setup
- VPC peering setup

## Manual Multi-Step Operations

For complex scenarios not covered by workflows:

```bash
#!/bin/bash
set -e

# Step 1: Create subscription
SUB_ID=$(redisctl cloud subscription create \
  --name production \
  --cloud-provider AWS \
  --region us-east-1 \
  --wait \
  -o json -q 'id')

echo "Subscription created: $SUB_ID"

# Step 2: Create database
redisctl cloud database create \
  --subscription-id "$SUB_ID" \
  --name cache \
  --memory-limit-in-gb 2 \
  --wait

# Step 3: Get connection info
redisctl cloud database list --subscription-id "$SUB_ID" \
  -o json -q '[0].{endpoint: publicEndpoint}'
```

## Related

- [Subscription Commands](commands/subscriptions.md)
- [Database Commands](commands/databases.md)
- [Async Operations](../common/async-operations.md)
