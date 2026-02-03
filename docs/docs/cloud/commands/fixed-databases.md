# Essentials Database Commands

Manage databases within Redis Cloud Essentials (fixed) subscriptions.

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all databases in a subscription |
| `get` | Get database details |
| `create` | Create a new database |
| `update` | Update database configuration |
| `delete` | Delete a database |
| `import` | Import data into a database |
| `list-tags` | List database tags |
| `update-tags` | Update database tags |
| `delete-tag` | Delete a database tag |

## List Databases

```bash
redisctl cloud fixed-database list --subscription <id>
```

### Examples

```bash
# List all databases in a subscription
redisctl cloud fixed-database list --subscription 123456

# As JSON
redisctl cloud fixed-database list --subscription 123456 -o json

# Just names and endpoints
redisctl cloud fixed-database list --subscription 123456 -o json -q '[].{
  name: name,
  endpoint: publicEndpoint
}'
```

## Get Database Details

```bash
redisctl cloud fixed-database get <subscription-id>:<database-id>
```

### Examples

```bash
# Full details
redisctl cloud fixed-database get 123456:789

# Connection info
redisctl cloud fixed-database get 123456:789 -o json -q '{
  endpoint: publicEndpoint,
  password: security.password
}'
```

## Create Database

Create a database with first-class parameters for common options.

```bash
redisctl cloud fixed-database create <subscription-id> \
  --name mydb \
  --wait
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--name` | Database name | - |
| `--password` | Database password | - |
| `--enable-tls` | Enable TLS encryption | - |
| `--eviction-policy` | Eviction policy | volatile-lru |
| `--replication` | Enable replication | false |
| `--data-persistence` | Persistence policy | - |
| `--data` | Full JSON configuration | - |

### Examples

```bash
# Simple database with name
redisctl cloud fixed-database create 123456 --name mydb --wait

# Database with password and TLS
redisctl cloud fixed-database create 123456 \
  --name secure-cache \
  --password mysecretpass \
  --enable-tls true

# Database with persistence
redisctl cloud fixed-database create 123456 \
  --name persistent-db \
  --data-persistence aof-every-1-second \
  --replication true

# Advanced: Use JSON for full control
redisctl cloud fixed-database create 123456 \
  --data '{"name": "mydb", "memoryLimitInGb": 1}'
```

## Update Database

Update database configuration using first-class parameters.

```bash
redisctl cloud fixed-database update <subscription-id>:<database-id> \
  --name new-name \
  --wait
```

### Options

| Option | Description |
|--------|-------------|
| `--name` | New database name |
| `--password` | New database password |
| `--enable-tls` | Enable/disable TLS |
| `--eviction-policy` | Eviction policy |
| `--replication` | Enable/disable replication |
| `--data-persistence` | Persistence policy |
| `--data` | Full JSON with additional fields |

### Examples

```bash
# Update database name
redisctl cloud fixed-database update 123456:789 --name new-db-name

# Change password
redisctl cloud fixed-database update 123456:789 --password newsecret

# Enable replication
redisctl cloud fixed-database update 123456:789 --replication true

# Multiple changes at once
redisctl cloud fixed-database update 123456:789 \
  --enable-tls true \
  --data-persistence aof-every-1-second \
  --wait
```

## Delete Database

```bash
redisctl cloud fixed-database delete <subscription-id>:<database-id> --wait
```

!!! warning
    This permanently deletes the database. Add `--force` to skip confirmation.

## Import Data

Import data into a database using first-class parameters.

```bash
redisctl cloud fixed-database import <subscription-id>:<database-id> \
  --source-type http \
  --import-from-uri https://example.com/backup.rdb \
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
# Import from HTTP URL
redisctl cloud fixed-database import 123456:789 \
  --source-type http \
  --import-from-uri https://example.com/backup.rdb \
  --wait

# Import from AWS S3 with credentials
redisctl cloud fixed-database import 123456:789 \
  --source-type aws-s3 \
  --import-from-uri s3://bucket/backup.rdb \
  --aws-access-key AKIA... \
  --aws-secret-key secret

# Import from Google Cloud Storage
redisctl cloud fixed-database import 123456:789 \
  --source-type gcs \
  --import-from-uri gs://bucket/backup.rdb \
  --gcs-client-email service@project.iam.gserviceaccount.com \
  --gcs-private-key @/path/to/key.pem

# Import from Azure Blob Storage
redisctl cloud fixed-database import 123456:789 \
  --source-type azure-blob-storage \
  --import-from-uri https://account.blob.core.windows.net/container/backup.rdb \
  --azure-account-name myaccount \
  --azure-account-key mykey
```

## Tags

### List Tags

```bash
redisctl cloud fixed-database list-tags <subscription-id>:<database-id>
```

### Update Tags

Update multiple tags at once using first-class parameters.

```bash
redisctl cloud fixed-database update-tags <subscription-id>:<database-id> \
  --tag env=production \
  --tag team=backend \
  --tag cost-center=12345
```

#### Options

| Option | Description |
|--------|-------------|
| `--tag` | Tag in key=value format (repeatable) |
| `--data` | JSON array of tags |

#### Examples

```bash
# Set multiple tags
redisctl cloud fixed-database update-tags 123456:789 \
  --tag env=production \
  --tag owner=team-a

# Use JSON for complex tag values
redisctl cloud fixed-database update-tags 123456:789 \
  --data '[{"key": "env", "value": "prod"}, {"key": "team", "value": "backend"}]'
```

### Delete Tag

```bash
redisctl cloud fixed-database delete-tag <subscription-id>:<database-id> --key env
```

## Related Commands

- [Essentials Subscriptions](fixed-subscriptions.md) - Manage Essentials subscriptions
- [Pro Databases](databases.md) - Manage Pro databases
- [Tasks](tasks.md) - Monitor async operations
