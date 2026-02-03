# Task Commands

Monitor async operations in Redis Cloud.

## Overview

Most Redis Cloud operations (create, update, delete) are asynchronous. They return a task ID that you can monitor.

## Using --wait

The easiest approach - just add `--wait` to any command:

```bash
redisctl cloud database create \
  --subscription-id 123456 \
  --name mydb \
  --memory-limit-in-gb 1 \
  --wait
```

This blocks until the operation completes or fails.

## List Tasks

```bash
redisctl cloud task list
```

### Filter Tasks

```bash
# Recent tasks as JSON
redisctl cloud task list -o json

# Failed tasks
redisctl cloud task list -o json -q '[?status==`failed`]'

# Tasks for a specific subscription
redisctl cloud task list --subscription-id 123456
```

## Get Task Status

```bash
redisctl cloud task get <task-id>
```

### Example

```bash
$ redisctl cloud task get abc123-def456

Task ID: abc123-def456
Status: completed
Progress: 100%
Description: Create database
Created: 2024-01-15T10:30:00Z
Completed: 2024-01-15T10:31:45Z
```

## Wait for Task

If you started an operation without `--wait`:

```bash
redisctl cloud task wait <task-id>
```

This blocks until the task completes.

## Common Patterns

### Fire and Forget, Check Later

```bash
# Start operation
TASK_ID=$(redisctl cloud database create \
  --subscription-id 123456 \
  --name mydb \
  -o json -q 'taskId')

echo "Started task: $TASK_ID"

# Do other work...

# Check status later
redisctl cloud task get "$TASK_ID"
```

### Wait with Timeout

```bash
redisctl cloud subscription create \
  --name production \
  --cloud-provider AWS \
  --region us-east-1 \
  --wait \
  --wait-timeout 600  # 10 minutes
```

### Check All Recent Tasks

```bash
redisctl cloud task list -o json -q '[].{
  id: taskId,
  status: status,
  description: description
}'
```

## Scripting with Tasks

### Create Database and Get Connection Info

```bash
#!/bin/bash
SUB_ID=123456

# Create and wait
redisctl cloud database create \
  --subscription-id "$SUB_ID" \
  --name mydb \
  --memory-limit-in-gb 1 \
  --wait

# Get connection info (database now exists)
ENDPOINT=$(redisctl cloud database list \
  --subscription-id "$SUB_ID" \
  -o json -q "[?name=='mydb'].publicEndpoint | [0]")

echo "Database ready at: $ENDPOINT"
```

## Related Commands

- [Databases](databases.md) - Database operations
- [Subscriptions](subscriptions.md) - Subscription operations
