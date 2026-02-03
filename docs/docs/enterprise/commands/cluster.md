# Cluster Commands

Manage Redis Enterprise cluster configuration and status.

## Get Cluster Info

```bash
redisctl enterprise cluster get
```

Returns cluster configuration including name, version, nodes, and settings.

### Example Output

```bash
$ redisctl enterprise cluster get -o json -q '{name: name, version: version, nodes: total_node_count}'
```

```json
{
  "name": "prod-cluster",
  "version": "7.2.4-92",
  "nodes": 3
}
```

## Cluster Statistics

```bash
# Current stats
redisctl enterprise cluster stats

# Continuous monitoring
redisctl enterprise cluster stats --follow

# Stats as JSON
redisctl enterprise cluster stats -o json
```

## Update Cluster Settings

```bash
redisctl enterprise cluster update --data '{"email": "alerts@company.com"}'
```

## Common Queries

### Cluster Health Check

```bash
# Quick health summary
redisctl enterprise cluster get -o json -q '{
  name: name,
  status: status,
  nodes: total_node_count,
  version: version
}'
```

### Check License

```bash
redisctl enterprise cluster get -o json -q '{
  license_expired: license_expired,
  shards_limit: shards_limit
}'
```

### Get Cluster FQDN

```bash
redisctl enterprise cluster get -o json -q 'name'
```

## Raw API Access

```bash
# Full cluster object
redisctl api enterprise get /v1/cluster

# Cluster certificates
redisctl api enterprise get /v1/cluster/certificates

# Cluster alerts
redisctl api enterprise get /v1/cluster/alerts
```

## Related Commands

- [Nodes](nodes.md) - Manage cluster nodes
- [Databases](databases.md) - Manage databases
- [Monitoring](monitoring.md) - Stats and alerts
