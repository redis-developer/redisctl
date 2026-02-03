# Access Control Commands

Manage users, roles, and LDAP in Redis Enterprise.

## Users

### List Users

```bash
redisctl enterprise user list
```

### Get User

```bash
redisctl enterprise user get <uid>
```

### Create User

```bash
redisctl enterprise user create --data '{
  "name": "operator",
  "email": "operator@company.com",
  "password": "secure-password",
  "role": "db_viewer"
}'
```

### Update User

```bash
redisctl enterprise user update 1 --data '{
  "role": "db_member"
}'
```

### Delete User

```bash
redisctl enterprise user delete <uid>
```

## Roles

### List Roles

```bash
redisctl enterprise role list
```

### Built-in Roles

| Role | Description |
|------|-------------|
| `admin` | Full cluster access |
| `cluster_member` | Cluster management |
| `cluster_viewer` | Read-only cluster |
| `db_member` | Database management |
| `db_viewer` | Read-only databases |

## LDAP

### Get LDAP Config

```bash
redisctl enterprise ldap get
```

### Update LDAP

```bash
redisctl enterprise ldap update --data '{
  "enabled": true,
  "server": "ldap://ldap.company.com:389",
  "bindDn": "cn=admin,dc=company,dc=com"
}'
```

## Common Patterns

### List All Users with Roles

```bash
redisctl enterprise user list -o json -q '[].{
  name: name,
  email: email,
  role: role
}'
```

### Find Admin Users

```bash
redisctl enterprise user list -o json -q '[?role==`admin`]'
```

## Raw API Access

```bash
# Users
redisctl api enterprise get /v1/users

# Roles
redisctl api enterprise get /v1/roles

# LDAP
redisctl api enterprise get /v1/ldap
```

## Related

- [Security Reference](../../reference/security.md) - Best practices
- [Cluster Commands](cluster.md) - Cluster management
