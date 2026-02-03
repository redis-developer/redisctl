# Node Commands

Manage Redis Enterprise cluster nodes.

## List Nodes

```bash
redisctl enterprise node list
```

### Filter and Format

```bash
# As JSON
redisctl enterprise node list -o json

# Node status summary
redisctl enterprise node list -o json -q '[].{
  uid: uid,
  addr: addr,
  status: status
}'

# Only active nodes
redisctl enterprise node list -o json -q '[?status==`active`]'
```

## Get Node Details

```bash
redisctl enterprise node get <uid>
```

### Examples

```bash
# Full details
redisctl enterprise node get 1

# Resource usage
redisctl enterprise node get 1 -o json -q '{
  addr: addr,
  cores: cores,
  total_memory: total_memory,
  available_memory: available_memory
}'

# Shard count
redisctl enterprise node get 1 -o json -q '{
  addr: addr,
  shard_count: shard_count
}'
```

## Node Statistics

```bash
# Current stats
redisctl enterprise node stats 1

# Continuous monitoring
redisctl enterprise node stats 1 --follow

# All nodes
redisctl enterprise node stats
```

## Common Queries

### Cluster Capacity Overview

```bash
redisctl enterprise node list -o json -q '[].{
  node: uid,
  addr: addr,
  shards: shard_count,
  memory_gb: to_number(total_memory) / `1073741824`,
  available_gb: to_number(available_memory) / `1073741824`
}'
```

### Find Nodes with Issues

```bash
redisctl enterprise node list -o json -q '[?status!=`active`].{
  node: uid,
  addr: addr,
  status: status
}'
```

### Total Cluster Resources

```bash
redisctl enterprise node list -o json -q '{
  total_nodes: length(@),
  total_cores: sum([].cores),
  total_memory_gb: sum([].total_memory) / `1073741824`,
  total_shards: sum([].shard_count)
}'
```

## Node Operations

### Check Node Health

```bash
redisctl enterprise node get 1 -o json -q '{
  status: status,
  uptime: uptime,
  cores: cores,
  shard_count: shard_count
}'
```

### Monitor Node Load

```bash
# Watch node stats in real-time
redisctl enterprise node stats 1 --follow
```

## Raw API Access

```bash
# All nodes
redisctl api enterprise get /v1/nodes

# Specific node
redisctl api enterprise get /v1/nodes/1

# Node stats
redisctl api enterprise get /v1/nodes/1/stats
```

## Related Commands

- [Cluster](cluster.md) - Cluster configuration
- [Databases](databases.md) - Database management
- [Monitoring](monitoring.md) - Alerts and logs
