# Database Commands

Manage databases within Redis Cloud subscriptions.

## List Databases

```bash
redisctl cloud database list --subscription <id>
```

### Examples

```bash
# List all databases in a subscription
redisctl cloud database list --subscription 123456

# As JSON
redisctl cloud database list --subscription 123456 -o json

# Just names and endpoints
redisctl cloud database list --subscription 123456 -o json -q '[].{
  name: name,
  endpoint: publicEndpoint
}'
```

## Get Database Details

```bash
redisctl cloud database get <subscription-id>:<database-id>
```

### Examples

```bash
# Full details
redisctl cloud database get 123456:789

# Connection info
redisctl cloud database get 123456:789 -o json -q '{
  endpoint: publicEndpoint,
  password: password
}'

# Memory and status
redisctl cloud database get 123456:789 -o json -q '{
  name: name,
  memory_gb: memoryLimitInGb,
  status: status
}'
```

## Create Database

Create a database with first-class parameters for common options.

```bash
redisctl cloud database create \
  --subscription 123456 \
  --name mydb \
  --memory 1 \
  --wait
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--subscription` | Subscription ID (required) | - |
| `--name` | Database name (required) | - |
| `--memory` | Memory limit in GB | - |
| `--dataset-size` | Dataset size in GB (alternative to --memory) | - |
| `--protocol` | Database protocol | redis |
| `--replication` | Enable replication for HA | false |
| `--data-persistence` | Persistence policy | - |
| `--eviction-policy` | Eviction policy | volatile-lru |
| `--redis-version` | Redis version (e.g., "7.2") | - |
| `--oss-cluster` | Enable OSS Cluster API | false |
| `--port` | TCP port (10000-19999) | auto |
| `--data` | Full JSON configuration | - |

### Examples

```bash
# Simple database
redisctl cloud database create \
  --subscription 123456 \
  --name mydb \
  --memory 1

# Production database with high availability
redisctl cloud database create \
  --subscription 123456 \
  --name prod-cache \
  --memory 10 \
  --replication \
  --data-persistence aof-every-1-second

# Advanced: Mix flags with JSON for rare options
redisctl cloud database create \
  --subscription 123456 \
  --name mydb \
  --memory 5 \
  --data '{"modules": [{"name": "RedisJSON"}]}'
```

## Update Database

Update database configuration using first-class parameters.

```bash
redisctl cloud database update <subscription-id>:<database-id> \
  --memory 10 \
  --wait
```

### Options

| Option | Description |
|--------|-------------|
| `--name` | New database name |
| `--memory` | Memory limit in GB |
| `--replication` | Enable/disable replication |
| `--data-persistence` | Persistence policy |
| `--eviction-policy` | Eviction policy |
| `--oss-cluster` | Enable/disable OSS Cluster API |
| `--regex-rules` | Regular expression for allowed keys |
| `--data` | Full JSON with additional fields |

### Examples

```bash
# Update database name
redisctl cloud database update 123456:789 --name new-db-name

# Increase memory
redisctl cloud database update 123456:789 --memory 10

# Change eviction policy
redisctl cloud database update 123456:789 --eviction-policy allkeys-lru

# Enable replication
redisctl cloud database update 123456:789 --replication true

# Multiple changes at once
redisctl cloud database update 123456:789 \
  --memory 20 \
  --data-persistence aof-every-1-second \
  --wait

# Advanced: Use JSON for complex updates
redisctl cloud database update 123456:789 \
  --data '{"alerts": [{"name": "dataset-size", "value": 80}]}'
```

## Delete Database

```bash
redisctl cloud database delete <subscription-id>:<database-id> --wait
```

!!! warning
    This permanently deletes the database. Add `--force` to skip confirmation.

## Import Data

Import data into a database using first-class parameters.

```bash
redisctl cloud database import <subscription-id>:<database-id> \
  --source-type s3 \
  --import-from-uri s3://bucket/backup.rdb \
  --wait
```

### Options

| Option | Description |
|--------|-------------|
| `--source-type` | Source type: http, redis, ftp, aws-s3, gcs, azure-blob-storage |
| `--import-from-uri` | URI to import from |
| `--aws-access-key` | AWS access key ID (for aws-s3) |
| `--aws-secret-key` | AWS secret access key (for aws-s3) |
| `--gcs-client-email` | GCS client email (for gcs) |
| `--gcs-private-key` | GCS private key (for gcs) |
| `--azure-account-name` | Azure storage account name |
| `--azure-account-key` | Azure storage account key |
| `--data` | Full JSON configuration |

### Examples

```bash
# Import from S3
redisctl cloud database import 123456:789 \
  --source-type s3 \
  --import-from-uri s3://bucket/backup.rdb \
  --wait

# Import from FTP
redisctl cloud database import 123456:789 \
  --source-type ftp \
  --import-from-uri ftp://user:pass@server/backup.rdb

# Import from AWS S3 with credentials
redisctl cloud database import 123456:789 \
  --source-type aws-s3 \
  --import-from-uri s3://bucket/backup.rdb \
  --aws-access-key AKIA... \
  --aws-secret-key secret

# Import from Google Cloud Storage
redisctl cloud database import 123456:789 \
  --source-type gcs \
  --import-from-uri gs://bucket/backup.rdb
```

## Tags

### List Tags

```bash
redisctl cloud database list-tags <subscription-id>:<database-id>
```

### Add Tag

```bash
redisctl cloud database add-tag <subscription-id>:<database-id> \
  --key env \
  --value production
```

### Update Tags

Update multiple tags at once using first-class parameters.

```bash
redisctl cloud database update-tags <subscription-id>:<database-id> \
  --tag env=production \
  --tag team=backend \
  --tag cost-center=12345
```

### Delete Tag

```bash
redisctl cloud database delete-tag <subscription-id>:<database-id> --key env
```

## Common Queries

### Get Connection String

```bash
ENDPOINT=$(redisctl cloud database get 123456:789 -o json -q 'publicEndpoint')
PASSWORD=$(redisctl cloud database get 123456:789 -o json -q 'password')
echo "redis://default:$PASSWORD@$ENDPOINT"
```

### Find All Databases Across Subscriptions

```bash
for sub in $(redisctl cloud subscription list -o json -q '[].id' | jq -r '.[]'); do
  echo "=== Subscription $sub ==="
  redisctl cloud database list --subscription "$sub" -o json -q '[].name'
done
```

### Database Size Summary

```bash
redisctl cloud database list --subscription 123456 -o json -q '[].{
  name: name,
  memory_gb: memoryLimitInGb,
  status: status
}'
```

## Raw API Access

```bash
# All databases in subscription
redisctl api cloud get /subscriptions/123456/databases

# Specific database
redisctl api cloud get /subscriptions/123456/databases/789
```

## Related Commands

- [Subscriptions](subscriptions.md) - Manage subscriptions
- [Access Control](access-control.md) - Users and ACLs
- [Tasks](tasks.md) - Monitor async operations
