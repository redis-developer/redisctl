# Generate Support Package

Collect diagnostics for Redis Support in 30 seconds.

## Quick Start

```bash
redisctl enterprise support-package cluster
```

That's it. A tar.gz file is created in your current directory.

## Optimize and Upload

For the fastest support experience:

```bash
redisctl enterprise support-package cluster --optimize --upload
```

This:
1. Generates the package
2. Reduces size by 20-30%
3. Uploads directly to Redis Support

## Step-by-Step

### 1. Generate Package

```bash
redisctl enterprise support-package cluster
```

Output:
```
Redis Enterprise Support Package
================================
Cluster: prod-cluster
Nodes: 3
Databases: 5

Generating support package...

Support package created successfully
  File: support-package-cluster-20240115T143000.tar.gz
  Size: 487.3 MB
```

### 2. Optimize (Optional)

```bash
redisctl enterprise support-package cluster --optimize
```

Reduces package size by truncating logs and removing redundant data.

### 3. Upload

Set up your Files.com API key (get from Redis Support):

```bash
export REDIS_ENTERPRISE_FILES_API_KEY="your-key"
```

Then upload:

```bash
redisctl enterprise support-package cluster --upload
```

Or do everything at once:

```bash
redisctl enterprise support-package cluster --optimize --upload --no-save
```

## Package Types

### Cluster (Most Common)

Full cluster diagnostics:

```bash
redisctl enterprise support-package cluster
```

### Database-Specific

For issues with a specific database:

```bash
redisctl enterprise support-package database 1
```

### Node-Specific

For node issues:

```bash
redisctl enterprise support-package node 2
```

## Automation

### Before Maintenance

```bash
#!/bin/bash
DATE=$(date +%Y%m%d)

# Pre-maintenance baseline
redisctl enterprise support-package cluster \
  -o "pre-maintenance-$DATE.tar.gz"

# Do maintenance...

# Post-maintenance capture
redisctl enterprise support-package cluster \
  -o "post-maintenance-$DATE.tar.gz"
```

### On Failure in CI

```yaml
- name: Collect diagnostics on failure
  if: failure()
  run: |
    redisctl enterprise support-package cluster \
      --optimize \
      -o support-package-${{ github.run_id }}.tar.gz
  env:
    REDIS_ENTERPRISE_URL: ${{ secrets.REDIS_ENTERPRISE_URL }}
    REDIS_ENTERPRISE_USER: ${{ secrets.REDIS_ENTERPRISE_USER }}
    REDIS_ENTERPRISE_PASSWORD: ${{ secrets.REDIS_ENTERPRISE_PASSWORD }}
```

## The Old Way vs redisctl

**Before** (10+ minutes):
```bash
ssh admin@cluster-node
rladmin cluster debug_info
# Wait...
scp admin@node:/tmp/debug*.tar.gz ./
# Open browser, upload to support portal...
```

**Now** (30 seconds):
```bash
redisctl enterprise support-package cluster --optimize --upload
```

## Related

- [Cluster Health](cluster-health.md) - Monitor cluster status
- [Support Package Reference](../../enterprise/operations/support-package.md) - Full documentation
