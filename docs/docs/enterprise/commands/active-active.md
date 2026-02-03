# Active-Active Commands

Manage Active-Active (CRDB) databases in Redis Enterprise.

## Overview

Active-Active databases replicate across multiple clusters for geo-distributed deployments.

## List CRDBs

```bash
redisctl enterprise crdb list
```

## Get CRDB Details

```bash
redisctl enterprise crdb get <guid>
```

## CRDB Tasks

### List Tasks

```bash
redisctl enterprise crdb-task list
```

### Get Task Status

```bash
redisctl enterprise crdb-task get <task-id>
```

## Common Patterns

### Check Replication Status

```bash
redisctl enterprise crdb list -o json -q '[].{
  guid: guid,
  name: name,
  status: status
}'
```

### Find Active-Active Databases

```bash
redisctl enterprise database list -o json -q '[?crdt==`true`].{
  uid: uid,
  name: name
}'
```

## Raw API Access

```bash
# List CRDBs
redisctl api enterprise get /v1/crdbs

# Specific CRDB
redisctl api enterprise get /v1/crdbs/<guid>

# CRDB tasks
redisctl api enterprise get /v1/crdb_tasks
```

## Related

- [Databases](databases.md) - Standard database management
- [Cluster](cluster.md) - Cluster configuration
