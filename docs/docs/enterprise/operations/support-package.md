# Support Package

Generate diagnostic packages for Redis Support in seconds instead of minutes.

## Overview

The `support-package` command is the fastest way to collect diagnostic information for Redis Support tickets. What used to take 10+ minutes of manual work now takes 30 seconds.

```bash
# Generate, optimize, and upload in one command
redisctl enterprise support-package cluster --optimize --upload
```

## Quick Start

### Generate a Package

```bash
redisctl enterprise support-package cluster
```

Output:
```
Redis Enterprise Support Package
================================
Cluster: prod-cluster-01
Version: 7.2.4
Nodes: 3
Databases: 5

Generating support package...

Support package created successfully
  File: support-package-cluster-20240115T143000.tar.gz
  Size: 487.3 MB
  Time: 154s
```

### Optimize Package Size

Reduce package size by 20-30%:

```bash
redisctl enterprise support-package cluster --optimize
```

```
Optimization: 487.3 MB -> 358.2 MB (26.5% reduction)
```

### Upload Directly to Redis Support

Skip the manual upload step:

```bash
export REDIS_ENTERPRISE_FILES_API_KEY="your-api-key"
redisctl enterprise support-package cluster --upload
```

### All-in-One

```bash
redisctl enterprise support-package cluster --optimize --upload --no-save
```

This generates, optimizes, uploads, and doesn't leave a local copy.

## Package Types

### Cluster Package

Full cluster diagnostics - use this for most issues:

```bash
redisctl enterprise support-package cluster
```

### Database Package

For database-specific issues:

```bash
redisctl enterprise support-package database 1
```

### Node Package

For node-specific issues:

```bash
# Specific node
redisctl enterprise support-package node 2

# All nodes
redisctl enterprise support-package node
```

## Options

| Option | Description |
|--------|-------------|
| `-o, --output` | Output file path |
| `--optimize` | Reduce package size (20-30% smaller) |
| `--optimize-verbose` | Show optimization details |
| `--log-lines` | Lines to keep per log file (default: 1000) |
| `--upload` | Upload to Redis Support (Files.com) |
| `--no-save` | Don't save locally after upload |
| `--skip-checks` | Skip pre-flight checks |
| `--wait` | Wait for completion (default) |
| `--wait-timeout` | Max wait time in seconds |

## Package Optimization

Large clusters can generate 500MB-2GB packages. Optimization helps:

```bash
# Basic optimization
redisctl enterprise support-package cluster --optimize

# See what was optimized
redisctl enterprise support-package cluster --optimize --optimize-verbose

# Keep more log history
redisctl enterprise support-package cluster --optimize --log-lines 5000
```

### What Gets Optimized

- **Log truncation**: Keeps most recent lines per log file
- **Redundant data removal**: Removes duplicate files
- **Nested archive cleanup**: Removes nested .gz files

!!! note "When to Skip Optimization"
    Skip optimization if Redis Support specifically requests full logs or you're investigating historical issues.

## Direct Upload

### Setup

Get your Files.com API key from Redis Support, then configure it:

=== "Environment Variable"

    ```bash
    export REDIS_ENTERPRISE_FILES_API_KEY="your-api-key"
    ```

=== "Secure Keyring"

    ```bash
    # Requires secure-storage feature
    redisctl files-key set "$KEY" --use-keyring
    ```

=== "Config File"

    ```bash
    redisctl files-key set "$KEY" --global
    ```

### Upload

```bash
# Generate and upload
redisctl enterprise support-package cluster --upload

# Upload without keeping local copy
redisctl enterprise support-package cluster --upload --no-save

# Optimize then upload
redisctl enterprise support-package cluster --optimize --upload
```

## CI/CD Integration

### JSON Output

```bash
redisctl enterprise support-package cluster -o json
```

```json
{
  "success": true,
  "package_type": "cluster",
  "file_path": "support-package-cluster-20240115T143000.tar.gz",
  "file_size": 510234567,
  "file_size_display": "487.3 MB",
  "elapsed_seconds": 154,
  "cluster_name": "prod-cluster-01",
  "cluster_version": "7.2.4-92"
}
```

### GitHub Actions Example

```yaml
- name: Collect support package on failure
  if: failure()
  run: |
    result=$(redisctl enterprise support-package cluster --optimize -o json)

    if [ $(echo "$result" | jq -r '.success') = "true" ]; then
      file=$(echo "$result" | jq -r '.file_path')
      echo "Package created: $file"
    fi
  env:
    REDIS_ENTERPRISE_URL: ${{ secrets.REDIS_ENTERPRISE_URL }}
    REDIS_ENTERPRISE_USER: ${{ secrets.REDIS_ENTERPRISE_USER }}
    REDIS_ENTERPRISE_PASSWORD: ${{ secrets.REDIS_ENTERPRISE_PASSWORD }}
```

### Automated Daily Collection

```bash
#!/bin/bash
OUTPUT_DIR="/backup/support-packages"
RETENTION_DAYS=7

# Generate with date-based naming
redisctl enterprise support-package cluster \
  -o "$OUTPUT_DIR/daily-$(date +%Y%m%d).tar.gz"

# Clean up old packages
find "$OUTPUT_DIR" -name "daily-*.tar.gz" -mtime +$RETENTION_DAYS -delete
```

## Best Practices

### Before Maintenance

```bash
# Collect baseline before changes
redisctl enterprise support-package cluster \
  -o "baseline-pre-upgrade-$(date +%Y%m%d).tar.gz"

# Perform maintenance...

# Collect after changes
redisctl enterprise support-package cluster \
  -o "post-upgrade-$(date +%Y%m%d).tar.gz"
```

### Organized Collection for Support Cases

```bash
#!/bin/bash
CASE_ID="CASE-12345"
mkdir -p "./support-$CASE_ID"

# Collect relevant packages
redisctl enterprise support-package cluster \
  -o "./support-$CASE_ID/cluster.tar.gz"

redisctl enterprise support-package database 1 \
  -o "./support-$CASE_ID/database-1.tar.gz"

# Document the issue
echo "Case: $CASE_ID" > "./support-$CASE_ID/README.txt"
echo "Issue: Database 1 high latency" >> "./support-$CASE_ID/README.txt"
```

## Troubleshooting

### Package Generation Fails

```bash
# Check cluster connectivity
redisctl enterprise cluster get

# Verify credentials
redisctl profile show
```

### Timeout on Large Clusters

```bash
# Increase timeout (1 hour)
redisctl enterprise support-package cluster --wait-timeout 3600
```

### Permission Denied

```bash
# Use a writable directory
redisctl enterprise support-package cluster -o /tmp/support.tar.gz
```

## File Naming

| Type | Pattern |
|------|---------|
| Cluster | `support-package-cluster-{timestamp}.tar.gz` |
| Database | `support-package-database-{uid}-{timestamp}.tar.gz` |
| Node | `support-package-node-{uid}-{timestamp}.tar.gz` |

Timestamps use ISO format: `YYYYMMDDTHHMMSS`
