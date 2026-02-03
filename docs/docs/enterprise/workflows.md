# Enterprise Workflows

Multi-step operations for Redis Enterprise.

## Init Cluster

Initialize a new cluster:

```bash
redisctl enterprise workflow init-cluster \
  --license-file ./license.txt \
  --cluster-name production
```

This:
1. Uploads the license
2. Configures the cluster name
3. Waits for initialization

## When to Use Workflows

**Use workflows when:**
- Initial cluster setup
- Complex multi-step operations
- Need consistent orchestration

**Use individual commands when:**
- Day-to-day management
- Need fine-grained control
- Troubleshooting

## Manual Multi-Step Operations

For scenarios not covered by workflows:

### Database with Replication

```bash
#!/bin/bash
set -e

DB_NAME="production-cache"

# Create database
redisctl enterprise database create --data "{
  \"name\": \"$DB_NAME\",
  \"memory_size\": 2147483648,
  \"replication\": true,
  \"shards_count\": 2
}"

# Verify creation
redisctl enterprise database list -o json -q "[?name=='$DB_NAME'] | [0]"
```

### Cluster Health Check and Report

```bash
#!/bin/bash

echo "=== Cluster Health Workflow ==="

# Check cluster
CLUSTER=$(redisctl enterprise cluster get -o json)
echo "Cluster: $(echo $CLUSTER | jq -r '.name')"
echo "Status: $(echo $CLUSTER | jq -r '.status')"

# Check nodes
UNHEALTHY=$(redisctl enterprise node list -o json -q '[?status!=`active`] | length(@)')
if [ "$UNHEALTHY" -gt 0 ]; then
  echo "WARNING: $UNHEALTHY unhealthy nodes"
  redisctl enterprise support-package cluster --optimize
else
  echo "All nodes healthy"
fi
```

## Coming Soon

Additional workflows planned:
- Database migration
- Active-Active setup
- Cluster expansion

## Related

- [Cluster Commands](commands/cluster.md)
- [Database Commands](commands/databases.md)
- [Support Package](operations/support-package.md)
