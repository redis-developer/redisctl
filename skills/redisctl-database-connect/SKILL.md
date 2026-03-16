---
name: redisctl-database-connect
description: Connect to Redis databases using the redisctl CLI. Use when opening redis-cli sessions, managing connection profiles, or working with multiple Redis clusters.
---

## Overview

Connect to Redis databases using profile-based credential management. Supports direct Redis connections, Cloud databases, and Enterprise databases.

## Quick Connect

```bash
# Open redis-cli using the default database profile
redisctl db open

# Open redis-cli with a specific profile
redisctl db open --profile my-redis
```

## Profile-Based Connection Management

### Create Database Profiles

```bash
# Local Redis
redisctl profile set local --type database --url "redis://localhost:6379"

# Redis with password
redisctl profile set staging --type database --url "redis://:password@host:6379"

# Redis with TLS
redisctl profile set prod --type database --url "rediss://host:6380"

# Set as default
redisctl profile default-database local
```

### Multi-Cluster Workflows

Create profiles for each environment:

```bash
redisctl profile set dev-redis --type database --url "redis://dev:6379"
redisctl profile set staging-redis --type database --url "rediss://staging:6380"
redisctl profile set prod-redis --type database --url "rediss://prod:6380"
```

Switch between them:

```bash
# Connect to dev
redisctl db open --profile dev-redis

# Connect to staging
redisctl db open --profile staging-redis
```

## Raw API Access

For operations not covered by specific commands:

```bash
# Cloud API
redisctl api cloud GET /subscriptions
redisctl api cloud POST /subscriptions/12345/databases --data '{...}'

# Enterprise API
redisctl api enterprise GET /v1/bdbs
redisctl api enterprise PUT /v1/bdbs/1 --data '{...}'
```

## Tips

- Database profile URLs follow the standard Redis URI scheme: `redis://[user:password@]host[:port][/db]`
- Use `rediss://` (double s) for TLS connections
- Profile credentials support environment variable substitution: `--url "redis://:${REDIS_PASSWORD}@host:6379"`
- Run `redisctl profile validate` to test all profile connections
