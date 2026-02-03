# Access Control Commands

Manage users, roles, and ACLs in Redis Cloud.

## Users

### List Users

```bash
redisctl cloud user list
```

### Get User

```bash
redisctl cloud user get <user-id>
```

### Create User

```bash
redisctl cloud user create --data '{
  "name": "app-user",
  "role": "member",
  "email": "user@example.com"
}'
```

## ACLs

### List ACL Rules

```bash
redisctl cloud acl list --subscription-id 123456
```

### Create ACL Rule

```bash
redisctl cloud acl create --subscription-id 123456 --data '{
  "name": "readonly",
  "redisRules": ["+@read", "-@write"]
}'
```

## Database Access

### Assign ACL to Database

```bash
redisctl cloud database update 123456 789 --data '{
  "security": {
    "defaultUserEnabled": false,
    "aclId": 12345
  }
}'
```

## Common Patterns

### List All ACLs

```bash
redisctl cloud acl list --subscription-id 123456 -o json -q '[].{
  id: id,
  name: name,
  rules: redisRules
}'
```

### Find Databases Using ACL

```bash
redisctl cloud database list --subscription-id 123456 -o json -q '[?security.aclId==`12345`].name'
```

## Raw API Access

```bash
# Users
redisctl api cloud get /users

# ACLs
redisctl api cloud get /subscriptions/123456/acls
```

## Related

- [Databases](databases.md) - Database management
- [Security Reference](../../reference/security.md) - Credential best practices
