# JMESPath Queries

Filter, transform, and reshape command output.

## What is JMESPath?

[JMESPath](https://jmespath.org/) is a query language for JSON. redisctl includes an extended implementation with 300+ functions.

```bash
# Basic: get all subscription names
redisctl cloud subscription list -o json -q '[].name'

# Advanced: aggregate statistics
redisctl cloud subscription list -o json -q '{
  total: length(@),
  providers: [].cloudDetails[0].provider | unique(@)
}'
```

## Basic Queries

### Select Fields

```bash
# Single field from each item
redisctl cloud subscription list -o json -q '[].name'
["prod-sub", "dev-sub", "staging-sub"]

# Multiple fields as objects
redisctl cloud subscription list -o json -q '[].{name: name, id: id}'
[{"name": "prod-sub", "id": 123}, {"name": "dev-sub", "id": 456}]
```

### Filter Results

```bash
# Databases over 1GB
redisctl enterprise database list -o json -q '[?memory_size > `1073741824`]'

# Active subscriptions only
redisctl cloud subscription list -o json -q '[?status == `active`]'

# Name contains "prod"
redisctl cloud subscription list -o json -q "[?contains(name, 'prod')]"
```

### Slice Arrays

```bash
# First 3 results
redisctl cloud subscription list -o json -q '[:3]'

# Last 2 results
redisctl cloud subscription list -o json -q '[-2:]'

# Skip first 5, take next 10
redisctl cloud subscription list -o json -q '[5:15]'
```

## Aggregations

### Count

```bash
# Total subscriptions
redisctl cloud subscription list -o json -q 'length(@)'
42

# Count by status
redisctl enterprise database list -o json -q '{
  active: length([?status == `active`]),
  inactive: length([?status != `active`])
}'
```

### Sum, Min, Max, Avg

```bash
# Total memory across all databases
redisctl enterprise database list -o json -q 'sum([].memory_size)'

# Statistics
redisctl enterprise database list -o json -q '{
  total_gb: sum([].memory_size) / `1073741824`,
  max_gb: max([].memory_size) / `1073741824`,
  avg_gb: avg([].memory_size) / `1073741824`
}'
```

### Unique Values

```bash
# Unique cloud providers
redisctl cloud subscription list -o json -q '[].cloudDetails[0].provider | unique(@)'
["AWS", "GCP", "Azure"]

# Unique regions
redisctl cloud subscription list -o json -q '[].cloudDetails[0].regions[0].region | unique(@) | sort(@)'
```

## Pipelines

Chain operations with `|`:

```bash
# Filter -> Project -> Sort -> Limit
redisctl cloud subscription list -o json -q "
  [?status == 'active']
  | [].{name: name, provider: cloudDetails[0].provider}
  | sort_by(@, &name)
  | [:5]
"
```

## String Functions

```bash
# Uppercase names
redisctl cloud subscription list -o json -q '[].name | map(&upper(@), @)'

# Find by prefix
redisctl cloud subscription list -o json -q "[?starts_with(name, 'prod-')]"

# Replace in strings
redisctl cloud subscription list -o json -q "[].{
  original: name,
  modified: replace(name, 'prod', 'PROD')
}"
```

## Extended Functions

redisctl uses [jmespath-community](https://jmespath.site/) with 300+ functions:

### Formatting

```bash
# Human-readable bytes
redisctl enterprise database list -o json -q '[].{
  name: name,
  memory: format_bytes(memory_size)
}'
[{"name": "cache", "memory": "1.0 GB"}]
```

### Date/Time

```bash
# Add timestamp to output
redisctl enterprise cluster get -o json -q '{
  cluster: name,
  checked_at: now()
}'
```

### Fuzzy Matching

```bash
# Find similar names (Levenshtein distance)
redisctl cloud subscription list -o json -q "[].{
  name: name,
  similarity: levenshtein(name, 'production')
} | sort_by(@, &similarity) | [:3]"
```

### Type Checking

```bash
# Inspect field types
redisctl enterprise database list -o json -q '[0] | {
  uid_type: type_of(uid),
  name_type: type_of(name),
  endpoints_type: type_of(endpoints)
}'
```

## Real-World Examples

### Cost Analysis

```bash
# Subscriptions with total size
redisctl cloud subscription list -o json -q '[].{
  name: name,
  size_gb: cloudDetails[0].totalSizeInGb,
  provider: cloudDetails[0].provider
} | sort_by(@, &size_gb) | reverse(@)'
```

### Health Check

```bash
# Nodes with issues
redisctl enterprise node list -o json -q "[?status != 'active'].{
  id: uid,
  addr: addr,
  status: status
}"
```

### Inventory Report

```bash
# Database summary by type
redisctl enterprise database list -o json -q '{
  total: length(@),
  by_status: {
    active: length([?status == `active`]),
    other: length([?status != `active`])
  },
  total_memory_gb: sum([].memory_size) / `1073741824`
}'
```

## Learning More

- [JMESPath Tutorial](https://jmespath.org/tutorial.html) - Official tutorial
- [JMESPath Community](https://jmespath.site/) - Extended functions documentation
- Use `redisctl ... -o json` without `-q` first to see the full structure
