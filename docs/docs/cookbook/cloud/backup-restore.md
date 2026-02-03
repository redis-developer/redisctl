# Backup and Restore

Manage database backups in Redis Cloud.

## View Backup Settings

```bash
redisctl cloud database get 123456 789 -o json -q '{
  name: name,
  backup: backup
}'
```

## Configure Automatic Backups

```bash
redisctl cloud database update 123456 789 --data '{
  "backup": {
    "enabled": true,
    "interval": 24,
    "destination": "s3://my-bucket/redis-backups"
  }
}' --wait
```

### Backup Options

| Field | Description |
|-------|-------------|
| `enabled` | Enable/disable backups |
| `interval` | Hours between backups (12 or 24) |
| `destination` | S3 or GCS bucket URL |

## Manual Backup

Trigger an immediate backup:

```bash
redisctl cloud database backup 123456 789 --wait
```

## List Backups

```bash
redisctl api cloud get /subscriptions/123456/databases/789/backups
```

## Restore from Backup

### To Same Database

```bash
redisctl cloud database restore 123456 789 \
  --backup-id <backup-id> \
  --wait
```

### To New Database

1. Create new database
2. Import from backup location

```bash
redisctl cloud database create \
  --subscription-id 123456 \
  --name restored-db \
  --data '{
    "memoryLimitInGb": 1,
    "dataImport": {
      "sourceUri": "s3://bucket/backup.rdb"
    }
  }' \
  --wait
```

## Backup to S3 Script

```bash
#!/bin/bash
SUB_ID="${1:?Usage: $0 <subscription-id> <database-id>}"
DB_ID="${2:?}"

echo "Triggering backup for database $DB_ID..."

# Configure backup destination if not set
redisctl cloud database update "$SUB_ID" "$DB_ID" --data '{
  "backup": {
    "enabled": true,
    "interval": 24,
    "destination": "s3://my-backup-bucket/redis"
  }
}' --wait

# Trigger immediate backup
redisctl cloud database backup "$SUB_ID" "$DB_ID" --wait

echo "Backup complete!"
```

## Verify Backup Configuration

```bash
redisctl cloud database list --subscription-id 123456 -o json -q '[].{
  name: name,
  backup_enabled: backup.enabled,
  backup_interval: backup.interval
}'
```

## Related

- [Database Commands](../../cloud/commands/databases.md)
- [Create First Database](first-database.md)
