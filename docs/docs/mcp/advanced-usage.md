# Advanced Usage

This guide demonstrates advanced patterns for combining the redisctl MCP server with JMESPath queries for powerful data transformation and analytics.

## Combining MCP Servers

The real power emerges when you combine **redisctl** (Redis data source) with **jpx** (JMESPath query engine). This combination enables:

- Complex data transformations on Redis infrastructure data
- Statistical analysis of cluster metrics
- Time-series analysis for performance trending
- Multi-cluster reporting and comparison

### Setting Up Both Servers

Configure both MCP servers in your AI assistant:

=== "Claude Code (.mcp.json)"

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "redisctl",
          "args": ["-p", "my-profile", "mcp", "serve", "--allow-writes"]
        },
        "jpx": {
          "command": "jpx",
          "args": ["mcp"]
        }
      }
    }
    ```

=== "Claude Desktop"

    ```json
    {
      "mcpServers": {
        "redisctl": {
          "command": "/path/to/redisctl",
          "args": ["-p", "my-profile", "mcp", "serve"]
        },
        "jpx": {
          "command": "/path/to/jpx",
          "args": ["mcp"]
        }
      }
    }
    ```

## JMESPath Patterns for Redis Data

### Cluster Overview

Extract a clean cluster summary from raw API data:

```jmespath
{
  cluster_overview: {
    name: cluster.name,
    created: cluster.created_time,
    high_availability: cluster.slave_ha,
    rack_aware: cluster.rack_aware
  }
}
```

**Result:**
```json
{
  "cluster_overview": {
    "name": "docker-cluster",
    "created": "2026-01-12T04:50:45Z",
    "high_availability": true,
    "rack_aware": false
  }
}
```

### Database Summary with Module Information

```jmespath
{
  databases: databases[*].{
    name: name,
    memory_gb: round(divide(memory_size, `1073741824`), `2`),
    shards: shards_count,
    modules: module_list[*].module_name,
    status: status
  }
}
```

**Result:**
```json
{
  "databases": [
    {
      "name": "default-db",
      "memory_gb": 1.0,
      "shards": 1,
      "modules": ["timeseries", "bf", "ReJSON", "search"],
      "status": "active"
    }
  ]
}
```

### Resource Utilization Analysis

Calculate CPU and memory utilization from cluster stats:

```jmespath
{
  resource_analysis: {
    cpu_utilization_pct: round(multiply(subtract(`1`, stats.cpu_idle), `100`), `2`),
    memory_available_gb: round(divide(stats.available_memory, `1073741824`), `2`),
    total_cores: sum(nodes[*].cores),
    total_node_memory_gb: round(divide(sum(nodes[*].total_memory), `1073741824`), `2`)
  }
}
```

**Result:**
```json
{
  "resource_analysis": {
    "cpu_utilization_pct": 4.0,
    "memory_available_gb": 7.81,
    "total_cores": 8,
    "total_node_memory_gb": 11.67
  }
}
```

## Statistical Analysis

### CPU Analysis with Statistics

```jmespath
{
  cpu_analysis: {
    avg_idle_pct: round(multiply(avg(intervals[*].cpu_idle), `100`), `2`),
    cpu_utilization_pct: round(multiply(subtract(`1`, avg(intervals[*].cpu_idle)), `100`), `2`),
    stddev_idle: round(stddev(intervals[*].cpu_idle), `4`),
    min_idle: round(min(intervals[*].cpu_idle), `4`),
    max_idle: round(max(intervals[*].cpu_idle), `4`)
  }
}
```

### Memory Analysis with Percentiles

```jmespath
{
  memory_analysis: {
    avg_available_gb: round(divide(avg(intervals[*].available_memory), `1073741824`), `2`),
    p50_free_memory_gb: round(divide(percentile(intervals[*].free_memory, `50`), `1073741824`), `2`),
    p95_free_memory_gb: round(divide(percentile(intervals[*].free_memory, `95`), `1073741824`), `2`),
    memory_volatility: round(stddev(intervals[*].free_memory), `0`)
  }
}
```

## Time Series Analysis

### Moving Averages and Trend Detection

Use `moving_avg` and `ewma` (exponential weighted moving average) for smoothing:

```jmespath
{
  time_series_analysis: {
    cpu_trend: {
      moving_average_3h: moving_avg(hourly_stats[*].cpu_idle, `3`),
      ewma_smoothed: ewma(hourly_stats[*].cpu_idle, `0.3`),
      variance: round(variance(hourly_stats[*].cpu_idle), `6`)
    }
  }
}
```

### Peak Detection

Find periods of high utilization:

```jmespath
{
  peak_detection: {
    busiest_periods: intervals[?cpu_idle < `0.9`] | [*].{
      time: stime,
      cpu_busy_pct: round(multiply(subtract(`1`, cpu_idle), `100`), `1`)
    } | sort_by(@, &cpu_busy_pct) | reverse(@) | [:5]
  }
}
```

## Configuration Comparison

### Using json_diff for Config Drift Detection

Compare two database configurations using RFC 6902 JSON Patch:

```jmespath
json_diff(database_a, database_b)
```

**Result:**
```json
[
  {"op": "replace", "path": "/memoryLimitInGb", "value": 1.0},
  {"op": "replace", "path": "/name", "value": "cache-db"},
  {"op": "replace", "path": "/replication", "value": true}
]
```

This is useful for:

- Detecting configuration drift between environments
- Auditing changes between database snapshots
- Comparing production vs staging settings

## Batch Evaluation

Run multiple queries against the same data efficiently:

```jmespath
# Expression 1: Cluster name
{ cluster: cluster.name }

# Expression 2: License info
{ license_shards: license.shards_limit }

# Expression 3: CPU utilization
{ cpu: round(multiply(stats.cpu_idle, `100`), `1`) }

# Expression 4: Database count
{ databases: length(databases) }

# Expression 5: Node count
{ nodes: length(nodes) }
```

The `batch_evaluate` tool runs all expressions against the same input in a single call, returning results for each.

## Health Dashboard

Combine multiple data sources into a unified view:

```jmespath
{
  health_dashboard: {
    cluster: cluster.name,
    license_type: contains(license.features, 'trial') && 'Trial' || 'Production',
    license_utilization: join('', [
      to_string(round(multiply(divide(
        add(license.ram_shards_in_use, license.flash_shards_in_use),
        license.shards_limit
      ), `100`), `0`)),
      '%'
    ]),
    cpu_utilization: join('', [
      to_string(round(multiply(subtract(`1`, stats.cpu_idle), `100`), `1`)),
      '%'
    ]),
    databases_active: length(databases[?status == 'active']),
    nodes_active: length(nodes[?status == 'active'])
  }
}
```

**Result:**
```json
{
  "health_dashboard": {
    "cluster": "docker-cluster",
    "license_type": "Trial",
    "license_utilization": "25%",
    "cpu_utilization": "4.0%",
    "databases_active": 1,
    "nodes_active": 1
  }
}
```

## JMESPath Function Reference

### Math Functions

| Function | Description | Example |
|----------|-------------|---------|
| `add(a, b)` | Addition | `add(5, 3)` → `8` |
| `subtract(a, b)` | Subtraction | `subtract(10, 4)` → `6` |
| `multiply(a, b)` | Multiplication | `multiply(3, 4)` → `12` |
| `divide(a, b)` | Division | `divide(10, 2)` → `5` |
| `round(n, decimals)` | Round to decimals | `round(3.14159, 2)` → `3.14` |

### Statistical Functions

| Function | Description | Example |
|----------|-------------|---------|
| `avg(array)` | Average/mean | `avg([1,2,3,4,5])` → `3` |
| `sum(array)` | Sum all values | `sum([1,2,3])` → `6` |
| `min(array)` | Minimum value | `min([3,1,4])` → `1` |
| `max(array)` | Maximum value | `max([3,1,4])` → `4` |
| `stddev(array)` | Standard deviation | `stddev([1,2,3,4,5])` → `1.414` |
| `variance(array)` | Variance | `variance([1,2,3,4,5])` → `2` |
| `percentile(array, p)` | Percentile | `percentile(data, 95)` |
| `median(array)` | Median value | `median([1,2,3,4,5])` → `3` |

### Time Series Functions

| Function | Description | Example |
|----------|-------------|---------|
| `moving_avg(array, window)` | Moving average | `moving_avg(data, 3)` |
| `ewma(array, alpha)` | Exponential weighted MA | `ewma(data, 0.3)` |

### Datetime Functions

| Function | Description | Example |
|----------|-------------|---------|
| `now()` | Current Unix timestamp | `now()` |
| `to_epoch(datetime)` | Convert ISO to epoch | `to_epoch('2026-01-12T00:00:00Z')` |
| `date_diff(t1, t2, unit)` | Time difference | `date_diff(exp, now, 'days')` |
| `time_ago(datetime)` | Human-readable diff | `time_ago(date)` → `"in 29 days"` |
| `format_date(epoch, fmt)` | Format epoch | `format_date(now(), '%Y-%m-%d')` |

### Comparison Functions

| Function | Description | Example |
|----------|-------------|---------|
| `json_diff(a, b)` | RFC 6902 JSON Patch | `json_diff(config1, config2)` |

## Next Steps

- [Workflows](workflows.md) - Real-world use cases including license management
- [Tools Reference](tools-reference.md) - Complete list of available tools
