# Operations

Operational tools for Redis Enterprise clusters.

## Support Package

Generate diagnostic packages for Redis Support:

```bash
# Basic support package
redisctl enterprise support-package cluster

# Optimized (smaller) package
redisctl enterprise support-package cluster --optimize

# Generate and upload to Redis Support
redisctl enterprise support-package cluster --optimize --upload
```

[:octicons-arrow-right-24: Support Package Guide](support-package.md)

## Debug Info

Collect debugging information:

```bash
# All debug info
redisctl enterprise debuginfo all

# Node-specific
redisctl enterprise debuginfo node 1
```

[:octicons-arrow-right-24: Debug Info Guide](debuginfo.md)

## Common Operations

### Health Check Script

```bash
#!/bin/bash
# Quick cluster health check

echo "=== Cluster Status ==="
redisctl enterprise cluster get -o json -q '{name: name, status: status}'

echo -e "\n=== Node Status ==="
redisctl enterprise node list -o json -q '[].{id: uid, addr: addr, status: status}'

echo -e "\n=== Database Status ==="
redisctl enterprise database list -o json -q '[].{id: uid, name: name, status: status}'

echo -e "\n=== Active Alerts ==="
redisctl enterprise alert list -o json -q 'length(@)'
```

### Backup Configuration

```bash
# Export cluster config
redisctl enterprise cluster get -o json > cluster-config.json

# Export all database configs
redisctl enterprise database list -o json > databases.json
```
