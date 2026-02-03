# Configure ACL Security

Secure database access with ACL rules.

## Overview

ACLs (Access Control Lists) let you:
- Restrict commands users can run
- Limit key patterns users can access
- Create read-only or write-only users

## Step 1: Create ACL Rule

```bash
redisctl cloud acl create --subscription-id 123456 --data '{
  "name": "readonly",
  "redisRules": ["+@read", "-@write", "-@admin"]
}'
```

### Common ACL Patterns

| Pattern | Rules |
|---------|-------|
| Read-only | `["+@read", "-@write", "-@admin"]` |
| Write-only | `["-@read", "+@write", "-@admin"]` |
| No dangerous | `["+@all", "-@dangerous"]` |
| Specific keys | `["+@all", "~cache:*"]` |

## Step 2: Create User with ACL

```bash
# Get ACL ID
ACL_ID=$(redisctl cloud acl list --subscription-id 123456 \
  -o json -q "[?name=='readonly'].id | [0]")

# Create user
redisctl cloud user create --subscription-id 123456 --data "{
  \"name\": \"app-reader\",
  \"password\": \"secure-password\",
  \"aclId\": $ACL_ID
}"
```

## Step 3: Apply to Database

```bash
redisctl cloud database update 123456 789 --data '{
  "security": {
    "defaultUserEnabled": false
  }
}' --wait
```

## Verify Configuration

### List ACLs

```bash
redisctl cloud acl list --subscription-id 123456 -o json -q '[].{
  id: id,
  name: name,
  rules: redisRules
}'
```

### Test Access

```bash
# Connect as the new user
redis-cli -u "redis://app-reader:password@endpoint:port"

# Try a read command (should work)
> GET key

# Try a write command (should fail)
> SET key value
(error) NOPERM this user has no permissions...
```

## Complete Example

```bash
#!/bin/bash
set -e

SUB_ID="${1:?Usage: $0 <subscription-id>}"

echo "Creating read-only ACL..."
redisctl cloud acl create --subscription-id "$SUB_ID" --data '{
  "name": "app-readonly",
  "redisRules": ["+@read", "-@write", "-@admin", "-@dangerous"]
}'

echo "Creating write ACL..."
redisctl cloud acl create --subscription-id "$SUB_ID" --data '{
  "name": "app-writer",
  "redisRules": ["+@all", "-@admin", "-@dangerous"]
}'

echo "ACLs created:"
redisctl cloud acl list --subscription-id "$SUB_ID" -o json -q '[].{name: name, id: id}'
```

## Related

- [Access Control Commands](../../cloud/commands/access-control.md)
- [Security Reference](../../reference/security.md)
