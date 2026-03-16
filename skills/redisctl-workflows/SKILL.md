---
name: redisctl-workflows
description: Multi-step operational workflows combining redisctl commands. Use for end-to-end provisioning, cluster initialization, migrations, and backup and recovery procedures.
---

## Overview

Workflows combine multiple redisctl commands into common operational patterns. Use these as templates for provisioning, maintenance, and recovery tasks.

## Cloud: Provision a Subscription and Database

```bash
# 1. Create a subscription
redisctl cloud subscription create --data '{
  "name": "my-app-prod",
  "cloudProviders": [{
    "provider": "AWS",
    "regions": [{"region": "us-east-1"}]
  }]
}'

# 2. Wait for provisioning
redisctl cloud task wait --id <task-id>

# 3. Create a database
redisctl cloud database create \
  --subscription-id <sub-id> \
  --data '{
    "name": "cache",
    "memoryLimitInGb": 1,
    "modules": [{"name": "RedisJSON"}]
  }'

# 4. Wait for database creation
redisctl cloud task wait --id <task-id>

# 5. Get connection details
redisctl cloud database get --subscription-id <sub-id> --id <db-id>
```

Or use the built-in workflow:

```bash
redisctl cloud workflow subscription-setup
```

## Enterprise: Initialize a Cluster

```bash
# Built-in workflow for cluster initialization
redisctl enterprise workflow init-cluster
```

Or step by step:

```bash
# 1. Check cluster status
redisctl enterprise status --brief

# 2. Upload license
redisctl enterprise license update --file license.key

# 3. Configure cluster policy
redisctl enterprise cluster update-policy --data '{...}'

# 4. Create a database
redisctl enterprise database create --data '{
  "name": "my-app",
  "memory_size": 1073741824,
  "type": "redis",
  "replication": true
}'

# 5. Verify
redisctl enterprise status --databases
```

## Enterprise: License Management

```bash
redisctl enterprise workflow license
```

## Backup and Recovery

### Cloud

```bash
# Trigger a backup
redisctl cloud database backup \
  --subscription-id <sub-id> \
  --id <db-id>

# Check backup status
redisctl cloud task wait --id <task-id>

# Import from backup
redisctl cloud database import \
  --subscription-id <sub-id> \
  --id <db-id> \
  --data '{"sourceType": "aws-s3", "importFromUri": "s3://..."}'
```

### Enterprise

```bash
# Export a database
redisctl enterprise database export --id 1

# Restore from backup
redisctl enterprise database restore --id 1 --data '{...}'
```

## Health Check Pattern

Quick health check across environments:

```bash
# Cloud
redisctl cloud subscription list
redisctl cloud database list --subscription-id <sub-id>

# Enterprise
redisctl enterprise status --brief

# Direct Redis
redisctl db open --profile prod-redis
# then: INFO, DBSIZE, SLOWLOG GET 10
```

## Migration Pattern

Move data between Redis instances:

```bash
# 1. Get source database info
redisctl enterprise database get --id 1

# 2. Create target database with matching config
redisctl cloud database create --subscription-id <sub-id> --data '{...}'

# 3. Wait for target to be ready
redisctl cloud task wait --id <task-id>

# 4. Get target connection details
redisctl cloud database get --subscription-id <sub-id> --id <db-id>

# 5. Use redis-cli or migration tool to replicate data
```

## Tips

- Always check task status after async operations with `cloud task wait`
- Use `--profile` to target specific environments in each step
- Enterprise workflows may require maintenance mode for some operations
- Back up before any destructive operation
- For complex multi-step operations, consider scripting with the raw API: `redisctl api`
