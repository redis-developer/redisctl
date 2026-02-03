# Workflows

This guide covers real-world workflows and use cases for the MCP server, with a focus on Customer Success scenarios like multi-cluster license management.

## License Management

### Single Cluster License Status

Query license information and calculate key metrics:

```jmespath
{
  license_status: {
    is_trial: contains(license.features, 'trial'),
    days_remaining: date_diff(to_epoch(license.expiration_date), now(), 'days'),
    shards_used: add(license.ram_shards_in_use, license.flash_shards_in_use),
    shards_available: subtract(license.shards_limit, add(license.ram_shards_in_use, license.flash_shards_in_use)),
    utilization_pct: round(multiply(divide(
      add(license.ram_shards_in_use, license.flash_shards_in_use), 
      license.shards_limit
    ), `100`), `1`)
  }
}
```

**Result:**
```json
{
  "license_status": {
    "is_trial": true,
    "days_remaining": 30,
    "shards_used": 1,
    "shards_available": 3,
    "utilization_pct": 25.0
  }
}
```

### Multi-Cluster License Report

For Customer Success teams managing customers with multiple clusters, generate comprehensive license reports across all clusters:

```jmespath
{
  multi_cluster_license_report: {
    generated_at: now(),
    total_clusters: length(clusters),
    summary: {
      total_shards_in_use: add(
        sum(clusters[*].license.ram_shards_in_use), 
        sum(clusters[*].license.flash_shards_in_use)
      ),
      total_shards_available: sum(clusters[*].license.shards_limit),
      overall_utilization_pct: round(multiply(divide(
        add(
          sum(clusters[*].license.ram_shards_in_use), 
          sum(clusters[*].license.flash_shards_in_use)
        ),
        sum(clusters[*].license.shards_limit)
      ), `100`), `1`)
    },
    clusters_by_urgency: {
      critical: clusters[?date_diff(to_epoch(license.expiration_date), now(), 'days') < `14`] | [*].{
        name: name,
        expires: license.expiration_date,
        days_remaining: date_diff(to_epoch(license.expiration_date), now(), 'days'),
        status: date_diff(to_epoch(license.expiration_date), now(), 'days') < `0` && 'EXPIRED' || 'EXPIRING SOON'
      },
      warning: clusters[?date_diff(to_epoch(license.expiration_date), now(), 'days') >= `14` && date_diff(to_epoch(license.expiration_date), now(), 'days') < `30`] | [*].{
        name: name,
        expires: license.expiration_date,
        days_remaining: date_diff(to_epoch(license.expiration_date), now(), 'days')
      },
      healthy: clusters[?date_diff(to_epoch(license.expiration_date), now(), 'days') >= `30`] | [*].{
        name: name,
        expires: license.expiration_date,
        days_remaining: date_diff(to_epoch(license.expiration_date), now(), 'days')
      }
    },
    capacity_alerts: clusters[?multiply(divide(
      add(license.ram_shards_in_use, license.flash_shards_in_use), 
      license.shards_limit
    ), `100`) > `80`] | [*].{
      name: name,
      utilization_pct: round(multiply(divide(
        add(license.ram_shards_in_use, license.flash_shards_in_use), 
        license.shards_limit
      ), `100`), `1`),
      shards_remaining: subtract(license.shards_limit, add(license.ram_shards_in_use, license.flash_shards_in_use))
    },
    trial_licenses: clusters[?contains(license.features, 'trial')] | [*].{
      name: name,
      expires: license.expiration_date,
      recommendation: 'Convert to production license'
    }
  }
}
```

**Sample Result:**
```json
{
  "multi_cluster_license_report": {
    "generated_at": 1768256917,
    "total_clusters": 4,
    "summary": {
      "total_shards_in_use": 158,
      "total_shards_available": 260,
      "overall_utilization_pct": 60.8
    },
    "clusters_by_urgency": {
      "critical": [
        {
          "name": "staging",
          "expires": "2026-01-20T00:00:00Z",
          "days_remaining": 7.06,
          "status": "EXPIRING SOON"
        }
      ],
      "warning": [
        {
          "name": "production-west",
          "expires": "2026-02-01T00:00:00Z",
          "days_remaining": 19.06
        }
      ],
      "healthy": [
        {
          "name": "production-east",
          "expires": "2026-03-15T00:00:00Z",
          "days_remaining": 61.06
        },
        {
          "name": "dev-cluster",
          "expires": "2026-04-30T00:00:00Z",
          "days_remaining": 107.06
        }
      ]
    },
    "capacity_alerts": [
      {
        "name": "production-west",
        "utilization_pct": 93.0,
        "shards_remaining": 7
      }
    ],
    "trial_licenses": [
      {
        "name": "staging",
        "expires": "2026-01-20T00:00:00Z",
        "recommendation": "Convert to production license"
      }
    ]
  }
}
```

This report automatically identifies:

- **Critical** - Clusters expiring in less than 14 days
- **Warning** - Clusters expiring in 14-30 days
- **Healthy** - Clusters with 30+ days remaining
- **Capacity Alerts** - Clusters over 80% shard utilization
- **Trial Licenses** - Clusters that should be converted to production

## Daily Health Checks

### Cluster Health Dashboard

Generate a daily health summary:

```jmespath
{
  daily_health_check: {
    timestamp: format_date(now(), '%Y-%m-%d %H:%M:%S'),
    cluster: cluster.name,
    status: {
      license: {
        type: contains(license.features, 'trial') && 'Trial' || 'Production',
        days_remaining: round(date_diff(to_epoch(license.expiration_date), now(), 'days'), `0`),
        shard_utilization: join('', [
          to_string(round(multiply(divide(
            add(license.ram_shards_in_use, license.flash_shards_in_use),
            license.shards_limit
          ), `100`), `0`)),
          '%'
        ])
      },
      resources: {
        cpu_utilization: join('', [
          to_string(round(multiply(subtract(`1`, stats.cpu_idle), `100`), `1`)),
          '%'
        ]),
        memory_available_gb: round(divide(stats.available_memory, `1073741824`), `2`)
      },
      databases: {
        total: length(databases),
        active: length(databases[?status == 'active']),
        inactive: length(databases[?status != 'active'])
      },
      nodes: {
        total: length(nodes),
        active: length(nodes[?status == 'active'])
      }
    },
    alerts: alerts[*].{
      severity: severity,
      type: type,
      message: message
    }
  }
}
```

### Automated Alert Summary

Filter and prioritize alerts:

```jmespath
{
  alert_summary: {
    critical: alerts[?severity == 'critical'] | length(@),
    warning: alerts[?severity == 'warning'] | length(@),
    info: alerts[?severity == 'info'] | length(@),
    recent_critical: alerts[?severity == 'critical'] | sort_by(@, &created_time) | reverse(@) | [:5]
  }
}
```

## Capacity Planning

### Memory Utilization Trends

Analyze memory trends for capacity planning:

```jmespath
{
  capacity_analysis: {
    current_state: {
      total_memory_gb: round(divide(sum(nodes[*].total_memory), `1073741824`), `2`),
      available_memory_gb: round(divide(stats.available_memory, `1073741824`), `2`),
      utilization_pct: round(multiply(subtract(`1`, divide(stats.available_memory, sum(nodes[*].total_memory))), `100`), `1`)
    },
    memory_trend: {
      avg_available_gb: round(divide(avg(intervals[*].available_memory), `1073741824`), `2`),
      min_available_gb: round(divide(min(intervals[*].available_memory), `1073741824`), `2`),
      max_available_gb: round(divide(max(intervals[*].available_memory), `1073741824`), `2`),
      volatility: round(divide(stddev(intervals[*].available_memory), `1073741824`), `3`)
    },
    recommendations: {
      near_capacity: multiply(subtract(`1`, divide(stats.available_memory, sum(nodes[*].total_memory))), `100`) > `80`,
      high_volatility: divide(stddev(intervals[*].available_memory), avg(intervals[*].available_memory)) > `0.1`
    }
  }
}
```

### Shard Distribution Analysis

```jmespath
{
  shard_analysis: {
    total_shards: sum(nodes[*].shard_count),
    distribution: nodes[*].{
      node_id: uid,
      shard_count: shard_count,
      capacity: max_redis_servers
    },
    balance_score: round(divide(stddev(nodes[*].shard_count), avg(nodes[*].shard_count)), `3`),
    imbalanced_nodes: nodes[?shard_count > multiply(avg(nodes[*].shard_count), `1.5`)] | [*].uid
  }
}
```

## Cloud Subscription Management

### Subscription Cost Analysis

```jmespath
{
  cost_analysis: {
    subscriptions: subscriptions[*].{
      id: id,
      name: name,
      provider: cloudDetails[0].provider,
      databases: numberOfDatabases,
      hourly_cost: sum(subscriptionPricing[?type == 'Shards'].pricePerUnit)
    },
    total_hourly_cost: sum(subscriptions[*].sum(subscriptionPricing[?type == 'Shards'].pricePerUnit)),
    by_provider: {
      aws: subscriptions[?cloudDetails[0].provider == 'AWS'] | length(@),
      gcp: subscriptions[?cloudDetails[0].provider == 'GCP'] | length(@),
      azure: subscriptions[?cloudDetails[0].provider == 'Azure'] | length(@)
    }
  }
}
```

### Database Inventory

```jmespath
{
  database_inventory: {
    total_databases: sum(subscriptions[*].numberOfDatabases),
    total_memory_gb: round(sum(subscriptions[*].cloudDetails[0].totalSizeInGb), `2`),
    by_subscription: subscriptions[*].{
      subscription: name,
      databases: numberOfDatabases,
      memory_gb: cloudDetails[0].totalSizeInGb,
      region: cloudDetails[0].regions[0].region
    }
  }
}
```

## Conversation Examples

### Customer Success: License Review

> **You**: Generate a license report for all of Acme Corp's clusters
>
> **AI**: *queries each cluster's license info, combines data, runs multi-cluster report*
>
> **Acme Corp License Report** (Generated: 2026-01-12)
>
> | Cluster | Type | Expires | Days Left | Utilization |
> |---------|------|---------|-----------|-------------|
> | production-east | Enterprise | Mar 15 | 61 | 55% |
> | production-west | Enterprise | Feb 1 | 19 | **93%** |
> | staging | **Trial** | Jan 20 | **7** | 30% |
> | dev-cluster | Enterprise | Apr 30 | 107 | 14% |
>
> **Action Items:**
>
> 1. **CRITICAL**: `staging` license expires in 7 days - convert to production
> 2. **WARNING**: `production-west` at 93% capacity - consider license upgrade
> 3. **WARNING**: `production-west` expires in 19 days - schedule renewal

### Ops: Daily Health Check

> **You**: Run the daily health check for production
>
> **AI**: *queries cluster, databases, nodes, alerts, stats*
>
> **Daily Health Check** - production-east (2026-01-12 10:30:00)
>
> | Category | Status |
> |----------|--------|
> | License | Production, 61 days remaining, 55% shards |
> | CPU | 4.2% utilization |
> | Memory | 7.81 GB available |
> | Databases | 3 active |
> | Nodes | 3 active |
> | Alerts | None |
>
> All systems healthy.

### DevOps: Capacity Planning

> **You**: Should we add more nodes to production-west?
>
> **AI**: *queries stats, trends, capacity metrics*
>
> **Capacity Analysis for production-west:**
>
> - Current memory utilization: 78%
> - Peak utilization (last 24h): 92%
> - Shard utilization: 93% (near limit)
> - Memory volatility: Low (stable workload)
>
> **Recommendation**: Yes, consider adding capacity. The cluster is at 93% shard utilization with limited headroom. Options:
>
> 1. Add 1-2 nodes to increase shard capacity
> 2. Upgrade license to higher shard limit
> 3. Review database configurations for optimization opportunities

## Automation Ideas

### Scheduled Reports

Set up automated reports using these patterns:

1. **Daily License Check** - Run at 8am, alert if any clusters < 14 days
2. **Weekly Capacity Report** - Summarize utilization trends across all clusters
3. **Monthly Cost Analysis** - Review Cloud subscription costs by provider

### Alert Workflows

Create alert-driven workflows:

1. **License Expiring** → Generate renewal quote → Notify account manager
2. **High Utilization** → Capacity analysis → Upgrade recommendations
3. **Trial Expiring** → Conversion reminder → Schedule customer call

## Database Operations Workflows

With the `--database-url` flag, you can work directly with Redis data through natural language.

### Data Exploration

> **You**: What keys are in my database?
>
> **AI**: *uses `database_scan` to list keys*
>
> Found 247 keys. Here's a sample:
>
> | Pattern | Count | Type |
> |---------|-------|------|
> | `user:*` | 150 | hash |
> | `session:*` | 45 | string |
> | `product:*` | 32 | JSON |
> | `leaderboard:*` | 5 | sorted set |
> | `events` | 1 | stream |

### Leaderboard Management

> **You**: Show me the top 10 players on the global leaderboard
>
> **AI**: *uses `database_zrevrange_withscores` with key="leaderboard:global"*
>
> | Rank | Player | Score |
> |------|--------|-------|
> | 1 | alice | 15,230 |
> | 2 | bob | 14,100 |
> | 3 | charlie | 12,890 |
> | ... | ... | ... |

> **You**: Add 500 points to alice's score
>
> **AI**: *uses `database_zincrby` with key="leaderboard:global", member="alice", increment=500*
>
> Updated alice's score: 15,230 → 15,730

### Full-Text Search

> **You**: Create a search index for my products
>
> **AI**: *uses `database_ft_create`*
>
> Created index `idx:products` with schema:
> - `name` (TEXT, weight 2.0)
> - `description` (TEXT)
> - `category` (TAG)
> - `price` (NUMERIC, sortable)

> **You**: Search for wireless headphones under $100
>
> **AI**: *uses `database_ft_search` with query="@name:wireless headphones @price:[0 100]"*
>
> Found 3 products:
>
> 1. **Sony WH-1000XM4** - $89.99 - Premium noise-cancelling headphones
> 2. **JBL Tune 500BT** - $49.99 - Wireless on-ear headphones
> 3. **Anker Soundcore Life Q20** - $59.99 - Hybrid active noise cancelling

### Stream Processing

> **You**: Show me the last 10 orders from my orders stream
>
> **AI**: *uses `database_xrevrange` with key="orders", count=10*
>
> | ID | Time | Order | Status |
> |----|------|-------|--------|
> | 1705432100000-0 | 12:35:00 | ORD-789 | shipped |
> | 1705431900000-0 | 12:31:40 | ORD-788 | processing |
> | 1705431700000-0 | 12:28:20 | ORD-787 | completed |
> | ... | ... | ... | ... |

> **You**: What's the status of consumer group "order-processors"?
>
> **AI**: *uses `database_xinfo_groups` and `database_xpending`*
>
> **Consumer Group: order-processors**
>
> - Consumers: 3 active (worker-1, worker-2, worker-3)
> - Pending entries: 5
> - Last delivered: 1705432100000-0
>
> **Pending Messages:**
> | Consumer | Pending | Idle Time |
> |----------|---------|-----------|
> | worker-1 | 2 | 5s |
> | worker-2 | 3 | 12s |
> | worker-3 | 0 | 1s |

### JSON Document Operations

> **You**: Get the user profile for user:123
>
> **AI**: *uses `database_json_get` with key="user:123"*
>
> ```json
> {
>   "name": "Alice Smith",
>   "email": "alice@example.com",
>   "preferences": {
>     "theme": "dark",
>     "notifications": true
>   },
>   "orders": ["ORD-100", "ORD-150", "ORD-200"]
> }
> ```

> **You**: Add order ORD-250 to their orders array
>
> **AI**: *uses `database_json_arrappend` with path="$.orders", value="ORD-250"*
>
> Appended "ORD-250" to orders array. New length: 4 orders.

### Time Series Analytics

> **You**: What's the average CPU usage over the last hour?
>
> **AI**: *uses `database_ts_range` with aggregation="avg", bucket_duration=3600000*
>
> **CPU Metrics (Last Hour)**
>
> - Average: 42.3%
> - Min: 12.1%
> - Max: 87.5%
> - Samples: 3,600

### Bulk Operations with Pipeline

> **You**: Add these 5 users to the active_users set and increment the user_count
>
> **AI**: *uses `database_pipeline` with atomic=true*
>
> Executed 6 commands atomically:
> - SADD active_users user:1 user:2 user:3 user:4 user:5 → 5 added
> - INCR user_count → 105
>
> All commands succeeded in 0.8ms (380x faster than individual calls).

## Next Steps

- [Advanced Usage](advanced-usage.md) - More JMESPath patterns and functions
- [Tools Reference](tools-reference.md) - Complete list of available tools
- [Getting Started](getting-started.md) - Installation and setup
