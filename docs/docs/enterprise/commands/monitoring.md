# Monitoring Commands

Stats, alerts, and logs for Redis Enterprise clusters.

## Statistics

### Cluster Stats

```bash
# Current stats
redisctl enterprise cluster stats

# Continuous monitoring
redisctl enterprise cluster stats --follow

# As JSON
redisctl enterprise cluster stats -o json
```

### Node Stats

```bash
# All nodes
redisctl enterprise node stats

# Specific node
redisctl enterprise node stats 1

# Continuous
redisctl enterprise node stats 1 --follow
```

### Database Stats

```bash
# Specific database
redisctl enterprise database stats 1

# Multiple databases
redisctl enterprise database stats 1 2 3

# Continuous monitoring
redisctl enterprise database stats 1 --follow
```

## Alerts

### List Alerts

```bash
redisctl enterprise alert list
```

### Filter Alerts

```bash
# As JSON
redisctl enterprise alert list -o json

# Count alerts
redisctl enterprise alert list -o json -q 'length(@)'

# Critical only
redisctl enterprise alert list -o json -q '[?severity==`critical`]'
```

## Logs

### View Logs

```bash
# Cluster logs
redisctl api enterprise get /v1/logs

# Recent entries
redisctl api enterprise get /v1/logs -q '[-10:]'
```

## Common Monitoring Tasks

### Quick Health Dashboard

```bash
#!/bin/bash
echo "=== Cluster Health ==="
redisctl enterprise cluster get -o json -q '{status: status, nodes: total_node_count}'

echo -e "\n=== Node Status ==="
redisctl enterprise node list -o json -q '[].{id: uid, status: status}'

echo -e "\n=== Active Alerts ==="
redisctl enterprise alert list -o json -q 'length(@)'
```

### Memory Usage Report

```bash
redisctl enterprise database list -o json -q '[].{
  name: name,
  allocated_mb: to_number(memory_size) / `1048576`,
  used_mb: to_number(used_memory) / `1048576`,
  pct: to_number(used_memory) / to_number(memory_size) * `100`
} | sort_by(@, &pct) | reverse(@)'
```

### Watch for Issues

```bash
# Monitor and alert on unhealthy nodes
watch -n 30 'redisctl enterprise node list -o json -q "[?status!=\`active\`]"'
```

## Raw API Access

```bash
# Cluster alerts
redisctl api enterprise get /v1/cluster/alerts

# Cluster stats
redisctl api enterprise get /v1/cluster/stats

# Node stats
redisctl api enterprise get /v1/nodes/stats

# Database stats
redisctl api enterprise get /v1/bdbs/1/stats
```

## Related

- [Cluster](cluster.md) - Cluster configuration
- [Nodes](nodes.md) - Node management
- [Databases](databases.md) - Database management
