# Enterprise Commands

All Redis Enterprise CLI commands.

## Command Reference

| Command Group | Description |
|---------------|-------------|
| [cluster](cluster.md) | Cluster configuration |
| [database](databases.md) | Database management |
| [node](nodes.md) | Node operations |
| [user](access-control.md) | User management |
| [role](access-control.md) | Role management |
| [alert](monitoring.md) | Alert configuration |
| [crdb](active-active.md) | Active-Active databases |

## Getting Help

```bash
# List all enterprise commands
redisctl enterprise --help

# Help for specific command
redisctl enterprise cluster --help
redisctl enterprise database create --help
```

## Common Options

All commands support:

| Option | Description |
|--------|-------------|
| `-o, --output` | Output format: `table`, `json`, `yaml` |
| `-q, --query` | JMESPath query to filter output |
| `--profile` | Use specific profile |

## Examples

```bash
# Get cluster info
redisctl enterprise cluster get

# List databases as JSON
redisctl enterprise database list -o json

# Filter with JMESPath
redisctl enterprise node list -o json -q '[].{id: uid, status: status}'
```
