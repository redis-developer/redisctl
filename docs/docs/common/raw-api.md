# Raw API Access

Direct REST access to any endpoint.

## Why Raw API?

The `api` command gives you direct access to Redis REST APIs:

- **Exploration** - Discover what endpoints return
- **Debugging** - See exact API responses
- **Undocumented endpoints** - Access features before they have CLI commands
- **Scripting** - When you need specific fields or behaviors

## Basic Usage

### Redis Cloud

```bash
# GET request
redisctl api cloud get /subscriptions

# With path parameters
redisctl api cloud get /subscriptions/123456/databases

# POST with body
redisctl api cloud post /subscriptions/123/databases \
  --body '{"name": "test", "memoryLimitInGb": 1}'
```

### Redis Enterprise

```bash
# GET cluster info
redisctl api enterprise get /v1/cluster

# GET all databases
redisctl api enterprise get /v1/bdbs

# POST to create database
redisctl api enterprise post /v1/bdbs \
  --body '{"name": "test", "memory_size": 1073741824}'
```

## HTTP Methods

| Method | Usage |
|--------|-------|
| `get` | Retrieve resources |
| `post` | Create resources |
| `put` | Update resources (full replace) |
| `patch` | Update resources (partial) |
| `delete` | Remove resources |

```bash
redisctl api cloud get /subscriptions
redisctl api cloud post /subscriptions --body '{...}'
redisctl api cloud put /subscriptions/123 --body '{...}'
redisctl api cloud delete /subscriptions/123
```

## Query Parameters

Some endpoints accept query parameters:

```bash
# Cloud API with query params (append to path)
redisctl api cloud get "/subscriptions?limit=10&offset=0"
```

## Request Body

### Inline JSON

```bash
redisctl api cloud post /subscriptions/123/databases \
  --body '{"name": "mydb", "memoryLimitInGb": 1}'
```

### From File

```bash
# Create a JSON file
cat > database.json << 'EOF'
{
  "name": "mydb",
  "memoryLimitInGb": 1,
  "dataEvictionPolicy": "volatile-lru"
}
EOF

# Use it
redisctl api cloud post /subscriptions/123/databases \
  --body @database.json
```

## Output and Filtering

### Raw JSON

```bash
redisctl api cloud get /subscriptions -o json
```

### With JMESPath

```bash
# Get specific fields
redisctl api cloud get /subscriptions -q '[].{id: id, name: name}'

# Filter results
redisctl api cloud get /subscriptions -q "[?status == 'active']"

# Aggregate
redisctl api cloud get /subscriptions -q 'length(@)'
```

## Comparison: Raw vs Human Commands

<div class="grid" markdown>

**Raw API**
```bash
redisctl api cloud get /subscriptions/123456/databases \
  -q '[].{id: databaseId, name: name}'
```

**Human Command**
```bash
redisctl cloud database list \
  --subscription-id 123456 \
  -o json -q '[].{id: databaseId, name: name}'
```

</div>

Both work. Human commands add:

- Input validation
- Helpful error messages
- Default formatting
- Tab completion

Use raw API when you need:

- Access to endpoints without CLI commands
- Exact API response format
- Debugging API behavior

## Discovering Endpoints

### List What's Available

Check the official API documentation:

- [Redis Cloud API](https://api.redislabs.com/v1/swagger-ui.html)
- [Redis Enterprise API](https://redis.io/docs/latest/operate/rs/references/rest-api/)

### Explore Interactively

```bash
# Start broad
redisctl api cloud get /subscriptions -o json | head -50

# Drill down
redisctl api cloud get /subscriptions/123456 -o json

# Find nested resources
redisctl api cloud get /subscriptions/123456/databases -o json
```

## Real Examples

### Cloud: Get Account Info

```bash
redisctl api cloud get /accounts -o json -q '[0]'
```

### Cloud: List All Database Endpoints

```bash
redisctl api cloud get /subscriptions -o json -q '[].{
  sub: name,
  databases: databases[].{name: name, endpoint: publicEndpoint}
}'
```

### Enterprise: Node Status

```bash
redisctl api enterprise get /v1/nodes -o json -q '[].{
  id: uid,
  addr: addr,
  status: status,
  shards: shard_count
}'
```

### Enterprise: Cluster Alerts

```bash
redisctl api enterprise get /v1/cluster/alerts -o json
```
