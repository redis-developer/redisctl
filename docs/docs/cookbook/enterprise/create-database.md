# Create a Database

Create a Redis Enterprise database from the command line.

## Quick Create

```bash
redisctl enterprise database create \
  --name mydb \
  --memory-size 1073741824
```

This creates a 1GB database with default settings.

## Create with Options

```bash
redisctl enterprise database create \
  --name production-cache \
  --memory-size 2147483648 \
  --replication \
  --shards-count 2
```

## Create with Full JSON Config

For complex configurations, use the `--data` flag:

```bash
redisctl enterprise database create --data '{
  "name": "session-store",
  "memory_size": 4294967296,
  "replication": true,
  "shards_count": 4,
  "data_eviction_policy": "volatile-lru",
  "maxclients": 10000
}'
```

## Memory Size Reference

| Size | Bytes |
|------|-------|
| 256 MB | 268435456 |
| 512 MB | 536870912 |
| 1 GB | 1073741824 |
| 2 GB | 2147483648 |
| 4 GB | 4294967296 |
| 8 GB | 8589934592 |

## Verify Creation

```bash
# List all databases
redisctl enterprise database list

# Get details for new database
redisctl enterprise database get 1 -o json -q '{
  name: name,
  memory_size: memory_size,
  status: status,
  endpoint: endpoints[0].addr[0]
}'
```

## Get Connection Info

```bash
# Find the endpoint
redisctl enterprise database list -o json -q '[?name==`mydb`].{
  uid: uid,
  endpoint: endpoints[0].addr[0],
  port: endpoints[0].port
}'
```

## Create Database Script

```bash
#!/bin/bash
set -e

DB_NAME="${1:?Usage: $0 <name> <memory_gb>}"
MEMORY_GB="${2:-1}"
MEMORY_BYTES=$((MEMORY_GB * 1073741824))

echo "Creating database '$DB_NAME' with ${MEMORY_GB}GB..."

redisctl enterprise database create --data "{
  \"name\": \"$DB_NAME\",
  \"memory_size\": $MEMORY_BYTES,
  \"replication\": true
}"

echo "Database created!"
redisctl enterprise database list -o json -q "[?name=='$DB_NAME'] | [0]"
```

## Next Steps

- [Cluster Health](cluster-health.md) - Monitor your cluster
- [Support Package](support-package.md) - Generate diagnostics
