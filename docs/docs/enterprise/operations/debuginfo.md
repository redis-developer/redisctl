# Debug Info

Collect detailed debugging information from Redis Enterprise clusters.

## Overview

Debug info provides lower-level diagnostic data than support packages. Use it for:

- Quick diagnostics without full support package
- Targeted debugging of specific components
- Automated collection in scripts

For support tickets, use [support-package](support-package.md) instead.

## Commands

### Cluster Debug Info

```bash
redisctl enterprise debuginfo cluster
```

### Node Debug Info

```bash
# All nodes
redisctl enterprise debuginfo node

# Specific node
redisctl enterprise debuginfo node 1
```

### Database Debug Info

```bash
redisctl enterprise debuginfo database 1
```

## Options

| Option | Description |
|--------|-------------|
| `-o, --output` | Output file path |
| `--use-new-api` | Use new API endpoints (7.4+) |

## Examples

### Save to File

```bash
redisctl enterprise debuginfo cluster -o cluster-debug.tar.gz
```

### Collect for Specific Issue

```bash
# Database issue
redisctl enterprise debuginfo database 1 -o db1-debug.tar.gz

# Node issue
redisctl enterprise debuginfo node 2 -o node2-debug.tar.gz
```

## Raw API Access

```bash
# Cluster debug info
redisctl api enterprise get /v1/debuginfo/all --output debug.tar.gz

# Node debug info
redisctl api enterprise get /v1/debuginfo/node/1 --output node1.tar.gz
```

## Debug Info vs Support Package

| Feature | Debug Info | Support Package |
|---------|-----------|-----------------|
| Size | Smaller | Larger (more complete) |
| Optimization | No | Yes (`--optimize`) |
| Upload | No | Yes (`--upload`) |
| Progress | Basic | Enhanced |
| Use case | Quick debug | Support tickets |

## Related

- [Support Package](support-package.md) - Full diagnostic packages for support
- [Cluster Health](../../cookbook/enterprise/cluster-health.md) - Health monitoring
