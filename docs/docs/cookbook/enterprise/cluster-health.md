# Cluster Health Monitoring

Monitor Redis Enterprise cluster status and identify issues.

## Quick Health Check

```bash
# Cluster status
redisctl enterprise cluster get -o json -q '{
  name: name,
  status: status,
  nodes: total_node_count,
  version: version
}'

# Node status
redisctl enterprise node list -o json -q '[].{
  node: uid,
  addr: addr,
  status: status
}'

# Database status
redisctl enterprise database list -o json -q '[].{
  uid: uid,
  name: name,
  status: status
}'
```

## Health Check Script

Save this as `cluster-health.sh`:

```bash
#!/bin/bash

echo "=== Cluster Health Check ==="
echo "Time: $(date)"
echo

echo "--- Cluster ---"
redisctl enterprise cluster get -o json -q '{
  name: name,
  status: status,
  version: version,
  nodes: total_node_count
}'
echo

echo "--- Nodes ---"
redisctl enterprise node list -o json -q '[].{
  id: uid,
  addr: addr,
  status: status,
  shards: shard_count
}'
echo

echo "--- Databases ---"
redisctl enterprise database list -o json -q '[].{
  id: uid,
  name: name,
  status: status,
  memory_mb: to_number(memory_size) / `1048576`
}'
echo

echo "--- Alerts ---"
ALERTS=$(redisctl api enterprise get /v1/cluster/alerts -o json -q 'length(@)')
echo "Active alerts: $ALERTS"
```

## Continuous Monitoring

### Watch Cluster Stats

```bash
redisctl enterprise cluster stats --follow
```

### Watch Node Stats

```bash
redisctl enterprise node stats --follow
```

### Watch Database Stats

```bash
redisctl enterprise database stats 1 --follow
```

## Check for Issues

### Find Unhealthy Nodes

```bash
redisctl enterprise node list -o json -q '[?status!=`active`]'
```

### Find Databases with Issues

```bash
redisctl enterprise database list -o json -q '[?status!=`active`]'
```

### Check Memory Usage

```bash
redisctl enterprise database list -o json -q '[].{
  name: name,
  used_pct: to_number(used_memory) / to_number(memory_size) * `100`
} | sort_by(@, &used_pct) | reverse(@)'
```

## Alerting Integration

### Send to Slack

```bash
#!/bin/bash
SLACK_WEBHOOK="https://hooks.slack.com/services/..."

# Check for unhealthy nodes
UNHEALTHY=$(redisctl enterprise node list -o json -q '[?status!=`active`] | length(@)')

if [ "$UNHEALTHY" -gt 0 ]; then
  curl -X POST "$SLACK_WEBHOOK" \
    -H 'Content-type: application/json' \
    -d "{\"text\": \"Warning: $UNHEALTHY unhealthy nodes detected\"}"
fi
```

### Export Metrics

```bash
# Prometheus-style metrics
echo "# HELP redis_cluster_nodes Total nodes in cluster"
echo "# TYPE redis_cluster_nodes gauge"
echo "redis_cluster_nodes $(redisctl enterprise node list -o json -q 'length(@)')"

echo "# HELP redis_database_memory_bytes Memory per database"
echo "# TYPE redis_database_memory_bytes gauge"
redisctl enterprise database list -o json -q '[].{name: name, memory: memory_size}' | \
  jq -r '.[] | "redis_database_memory_bytes{name=\"\(.name)\"} \(.memory)"'
```

## Daily Report

```bash
#!/bin/bash
REPORT_FILE="cluster-report-$(date +%Y%m%d).json"

redisctl enterprise cluster get -o json > /tmp/cluster.json
redisctl enterprise node list -o json > /tmp/nodes.json
redisctl enterprise database list -o json > /tmp/databases.json

jq -n '{
  timestamp: now | todate,
  cluster: input,
  nodes: input,
  databases: input
}' /tmp/cluster.json /tmp/nodes.json /tmp/databases.json > "$REPORT_FILE"

echo "Report saved to $REPORT_FILE"
```

## Related

- [Support Package](support-package.md) - Generate diagnostics for support
- [Node Management](node-management.md) - Manage cluster nodes
