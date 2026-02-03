# Database Commands

Create, configure, and manage Redis Enterprise databases.

## List Databases

```bash
redisctl enterprise database list
```

### Filter and Format

```bash
# As JSON
redisctl enterprise database list -o json

# Just names and IDs
redisctl enterprise database list -o json -q '[].{uid: uid, name: name}'

# Only active databases
redisctl enterprise database list -o json -q '[?status==`active`]'
```

## Get Database Details

```bash
redisctl enterprise database get <uid>
```

### Examples

```bash
# Full details
redisctl enterprise database get 1

# Connection info only
redisctl enterprise database get 1 -o json -q '{
  name: name,
  endpoints: endpoints[0].addr,
  port: endpoints[0].port
}'

# Memory usage
redisctl enterprise database get 1 -o json -q '{
  name: name,
  memory_size: memory_size,
  used_memory: used_memory
}'
```

## Create Database

```bash
redisctl enterprise database create \
  --name mydb \
  --memory-size 1073741824
```

### Options

| Option | Description |
|--------|-------------|
| `--name` | Database name |
| `--memory-size` | Memory limit in bytes |
| `--replication` | Enable replication |
| `--shards-count` | Number of shards |
| `--data` | Full JSON configuration |

### Create with JSON

```bash
redisctl enterprise database create --data '{
  "name": "cache",
  "memory_size": 1073741824,
  "replication": true,
  "shards_count": 2
}'
```

## Update Database

```bash
# Update memory
redisctl enterprise database update 1 --memory-size 2147483648

# Update with JSON
redisctl enterprise database update 1 --data '{
  "memory_size": 2147483648,
  "maxclients": 10000
}'
```

## Delete Database

```bash
redisctl enterprise database delete <uid>
```

!!! warning
    This permanently deletes the database and all its data.

## Database Statistics

```bash
# Current stats
redisctl enterprise database stats 1

# Continuous monitoring
redisctl enterprise database stats 1 --follow

# Multiple databases
redisctl enterprise database stats 1 2 3
```

## Common Queries

### Memory Usage Across All Databases

```bash
redisctl enterprise database list -o json -q '[].{
  name: name,
  memory_mb: to_number(memory_size) / `1048576`,
  used_mb: to_number(used_memory) / `1048576`
}'
```

### Find Large Databases

```bash
redisctl enterprise database list -o json -q '[?memory_size > `1073741824`].{
  name: name,
  memory_gb: to_number(memory_size) / `1073741824`
}'
```

### Get All Endpoints

```bash
redisctl enterprise database list -o json -q '[].{
  name: name,
  endpoint: endpoints[0].addr[0]
}'
```

## Raw API Access

```bash
# All databases
redisctl api enterprise get /v1/bdbs

# Specific database
redisctl api enterprise get /v1/bdbs/1

# Database stats
redisctl api enterprise get /v1/bdbs/1/stats
```

## Related Commands

- [Cluster](cluster.md) - Cluster configuration
- [Nodes](nodes.md) - Node management
- [Monitoring](monitoring.md) - Stats and alerts
