# Node Management

Monitor and manage Redis Enterprise cluster nodes.

## View Node Status

```bash
# List all nodes
redisctl enterprise node list

# Detailed view
redisctl enterprise node list -o json -q '[].{
  id: uid,
  addr: addr,
  status: status,
  shards: shard_count,
  memory_gb: to_number(total_memory) / `1073741824`
}'
```

## Check Node Health

```bash
# Single node details
redisctl enterprise node get 1 -o json -q '{
  addr: addr,
  status: status,
  uptime: uptime,
  cores: cores,
  shards: shard_count
}'
```

## Monitor Node Performance

```bash
# Real-time stats
redisctl enterprise node stats 1 --follow

# Stats as JSON
redisctl enterprise node stats 1 -o json
```

## Find Issues

### Unhealthy Nodes

```bash
redisctl enterprise node list -o json -q '[?status!=`active`].{
  id: uid,
  addr: addr,
  status: status
}'
```

### Nodes with High Shard Count

```bash
redisctl enterprise node list -o json -q '[].{
  id: uid,
  addr: addr,
  shards: shard_count
} | sort_by(@, &shards) | reverse(@)'
```

### Memory Availability

```bash
redisctl enterprise node list -o json -q '[].{
  id: uid,
  total_gb: to_number(total_memory) / `1073741824`,
  available_gb: to_number(available_memory) / `1073741824`
}'
```

## Cluster Capacity

### Total Resources

```bash
redisctl enterprise node list -o json -q '{
  nodes: length(@),
  total_cores: sum([].cores),
  total_memory_gb: sum([].total_memory) / `1073741824`,
  available_memory_gb: sum([].available_memory) / `1073741824`,
  total_shards: sum([].shard_count)
}'
```

### Per-Node Summary

```bash
redisctl enterprise node list -o json -q '[].{
  node: uid,
  cores: cores,
  memory_gb: to_number(total_memory) / `1073741824`,
  shards: shard_count
}'
```

## Automation Script

```bash
#!/bin/bash
# node-report.sh - Generate node status report

echo "=== Node Status Report ==="
echo "Generated: $(date)"
echo

# Summary
echo "--- Cluster Summary ---"
redisctl enterprise node list -o json -q '{
  total_nodes: length(@),
  active_nodes: length([?status==`active`]),
  total_shards: sum([].shard_count)
}'

echo -e "\n--- Per-Node Details ---"
redisctl enterprise node list -o json -q '[].{
  node: uid,
  addr: addr,
  status: status,
  shards: shard_count
}'

# Check for issues
UNHEALTHY=$(redisctl enterprise node list -o json -q '[?status!=`active`] | length(@)')
if [ "$UNHEALTHY" -gt 0 ]; then
  echo -e "\n!!! WARNING: $UNHEALTHY unhealthy nodes !!!"
  redisctl enterprise node list -o json -q '[?status!=`active`]'
fi
```

## Related

- [Cluster Health](cluster-health.md) - Overall cluster monitoring
- [Support Package](support-package.md) - Collect diagnostics
