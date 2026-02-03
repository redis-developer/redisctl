# Tools Reference

The redisctl MCP server exposes **237 tools** for managing Redis Cloud, Redis Enterprise, and direct database operations.

Tools marked with *(write)* require `--allow-writes` flag. Database tools require a `--database-url` connection.

## Redis Cloud Tools (29 tools)

### Account & Infrastructure

| Tool | Description |
|------|-------------|
| `cloud_account_get` | Get account information |
| `cloud_account_get_by_id` | Get cloud provider account by ID |
| `cloud_accounts_list` | List all cloud provider accounts |
| `cloud_account_delete` | Delete a cloud provider account *(write)* |
| `cloud_payment_methods_get` | List all payment methods |
| `cloud_database_modules_get` | List available database modules |
| `cloud_regions_get` | Get available regions (AWS, GCP, Azure) |

### Pro Subscriptions

| Tool | Description |
|------|-------------|
| `cloud_subscriptions_list` | List all Pro subscriptions |
| `cloud_subscription_get` | Get Pro subscription details |
| `cloud_pro_subscription_create` | Create a new Pro subscription *(write)* |
| `cloud_pro_subscription_delete` | Delete a Pro subscription *(write)* |

### Essentials Subscriptions

| Tool | Description |
|------|-------------|
| `cloud_essentials_subscriptions_list` | List all Essentials subscriptions |
| `cloud_essentials_subscription_get` | Get Essentials subscription details |
| `cloud_essentials_subscription_create` | Create Essentials subscription *(write)* |
| `cloud_essentials_subscription_delete` | Delete Essentials subscription *(write)* |
| `cloud_essentials_plans_list` | List available Essentials plans |
| `cloud_essentials_databases_list` | List databases in Essentials subscription |
| `cloud_essentials_database_get` | Get Essentials database details |
| `cloud_essentials_database_delete` | Delete Essentials database *(write)* |

### Database & Task Operations

| Tool | Description |
|------|-------------|
| `cloud_databases_list` | List databases in a subscription |
| `cloud_database_get` | Get database details |
| `cloud_tasks_list` | List recent async tasks |
| `cloud_task_get` | Get task status |

### Networking

| Tool | Description |
|------|-------------|
| `cloud_vpc_peerings_get` | Get VPC peerings for subscription |
| `cloud_vpc_peering_delete` | Delete a VPC peering *(write)* |
| `cloud_private_link_get` | Get AWS PrivateLink configuration |
| `cloud_private_link_delete` | Delete PrivateLink configuration *(write)* |
| `cloud_transit_gateway_attachments_get` | Get Transit Gateway attachments |
| `cloud_transit_gateway_attachment_delete` | Delete Transit Gateway attachment *(write)* |

## Redis Enterprise Tools (83 tools)

### Cluster Operations

| Tool | Description |
|------|-------------|
| `enterprise_cluster_get` | Get cluster information |
| `enterprise_cluster_stats` | Get cluster statistics |
| `enterprise_cluster_settings` | Get cluster settings |
| `enterprise_cluster_topology` | Get cluster topology |
| `enterprise_cluster_suffixes_get` | Get cluster DNS suffixes |
| `enterprise_cluster_update` | Update cluster configuration *(write)* |

### Database Operations

| Tool | Description |
|------|-------------|
| `enterprise_databases_list` | List all databases |
| `enterprise_database_get` | Get database details |
| `enterprise_database_stats` | Get database statistics |
| `enterprise_database_metrics` | Get database metrics |
| `enterprise_database_create` | Create a new database *(write)* |
| `enterprise_database_update` | Update database configuration *(write)* |
| `enterprise_database_delete` | Delete a database *(write)* |
| `enterprise_database_flush` | Flush all data *(write)* |
| `enterprise_database_export` | Export database *(write)* |
| `enterprise_database_import` | Import data *(write)* |
| `enterprise_database_backup` | Trigger backup *(write)* |
| `enterprise_database_restore` | Restore from backup *(write)* |

### Node Operations

| Tool | Description |
|------|-------------|
| `enterprise_nodes_list` | List all nodes |
| `enterprise_node_get` | Get node details |
| `enterprise_node_stats` | Get node statistics |
| `enterprise_node_update` | Update node configuration *(write)* |
| `enterprise_node_remove` | Remove node *(write)* |

### Shard & Endpoint Operations

| Tool | Description |
|------|-------------|
| `enterprise_shards_list` | List all shards |
| `enterprise_shard_get` | Get shard details |
| `enterprise_endpoints_list` | List all endpoints |
| `enterprise_endpoints_by_database` | List endpoints for database |
| `enterprise_endpoint_get` | Get endpoint details |
| `enterprise_endpoint_stats` | Get endpoint statistics |

### Proxy Operations

| Tool | Description |
|------|-------------|
| `enterprise_proxies_list` | List all proxies |
| `enterprise_proxies_by_database` | List proxies for database |
| `enterprise_proxy_get` | Get proxy details |
| `enterprise_proxy_stats` | Get proxy statistics |

### Alert Operations

| Tool | Description |
|------|-------------|
| `enterprise_alerts_list` | List active alerts |
| `enterprise_alert_get` | Get alert details |

### User Management

| Tool | Description |
|------|-------------|
| `enterprise_users_list` | List all users |
| `enterprise_user_get` | Get user details |
| `enterprise_user_create` | Create a user *(write)* |
| `enterprise_user_delete` | Delete a user *(write)* |

### Role Management

| Tool | Description |
|------|-------------|
| `enterprise_roles_list` | List all roles |
| `enterprise_role_get` | Get role details |
| `enterprise_role_create` | Create a role *(write)* |
| `enterprise_role_delete` | Delete a role *(write)* |

### ACL Management

| Tool | Description |
|------|-------------|
| `enterprise_acls_list` | List all Redis ACLs |
| `enterprise_acl_get` | Get ACL details |
| `enterprise_acl_create` | Create a Redis ACL *(write)* |
| `enterprise_acl_delete` | Delete a Redis ACL *(write)* |

### LDAP Mappings

| Tool | Description |
|------|-------------|
| `enterprise_ldap_mappings_list` | List LDAP mappings |
| `enterprise_ldap_mapping_get` | Get LDAP mapping details |
| `enterprise_ldap_mapping_create` | Create LDAP mapping *(write)* |
| `enterprise_ldap_mapping_delete` | Delete LDAP mapping *(write)* |

### License & Modules

| Tool | Description |
|------|-------------|
| `enterprise_license_get` | Get license information |
| `enterprise_modules_list` | List available modules |
| `enterprise_module_get` | Get module details |

### Active-Active (CRDB)

| Tool | Description |
|------|-------------|
| `enterprise_crdbs_list` | List Active-Active databases |
| `enterprise_crdb_get` | Get CRDB details |
| `enterprise_crdb_update` | Update CRDB *(write)* |
| `enterprise_crdb_delete` | Delete CRDB *(write)* |
| `enterprise_crdb_tasks_list` | List all CRDB tasks |
| `enterprise_crdb_tasks_by_crdb` | List tasks for specific CRDB |
| `enterprise_crdb_task_get` | Get CRDB task details |
| `enterprise_crdb_task_cancel` | Cancel CRDB task *(write)* |

### Database Groups

| Tool | Description |
|------|-------------|
| `enterprise_bdb_groups_list` | List database groups |
| `enterprise_bdb_group_get` | Get database group details |
| `enterprise_bdb_group_delete` | Delete database group *(write)* |

### DNS Suffixes

| Tool | Description |
|------|-------------|
| `enterprise_suffixes_list` | List DNS suffixes |
| `enterprise_suffix_get` | Get suffix details |
| `enterprise_suffix_delete` | Delete DNS suffix *(write)* |

### Jobs & Scheduling

| Tool | Description |
|------|-------------|
| `enterprise_jobs_list` | List scheduled jobs |
| `enterprise_job_get` | Get job details |
| `enterprise_job_history` | Get job execution history |
| `enterprise_job_trigger` | Trigger job immediately *(write)* |

### OCSP

| Tool | Description |
|------|-------------|
| `enterprise_ocsp_config_get` | Get OCSP configuration |
| `enterprise_ocsp_status_get` | Get OCSP status |
| `enterprise_ocsp_test` | Test OCSP connectivity |

### Logs & Diagnostics

| Tool | Description |
|------|-------------|
| `enterprise_logs_get` | Get cluster event logs |
| `enterprise_diagnostics_run` | Run diagnostics *(write)* |
| `enterprise_diagnostic_checks_list` | List diagnostic checks |
| `enterprise_diagnostic_reports_list` | List diagnostic reports |
| `enterprise_diagnostic_report_get` | Get diagnostic report |
| `enterprise_diagnostic_report_last` | Get most recent report |
| `enterprise_debuginfo_list` | List debug info tasks |
| `enterprise_debuginfo_status` | Get debug info status |

## Database Tools (125 tools)

Direct Redis database operations. Requires `--database-url` connection string.

### Server & Keys

| Tool | Description |
|------|-------------|
| `database_ping` | Ping Redis server |
| `database_info` | Get server information |
| `database_dbsize` | Get number of keys |
| `database_scan` | Scan keys by pattern |
| `database_type` | Get key type |
| `database_exists` | Check if key exists |
| `database_ttl` | Get key TTL |
| `database_memory_usage` | Get key memory usage |
| `database_rename` | Rename a key *(write)* |
| `database_del` | Delete keys *(write)* |
| `database_expire` | Set key expiration *(write)* |
| `database_persist` | Remove key expiration *(write)* |

### Strings

| Tool | Description |
|------|-------------|
| `database_get` | Get string value |
| `database_set` | Set string value *(write)* |
| `database_incr` | Increment by 1 *(write)* |
| `database_decr` | Decrement by 1 *(write)* |
| `database_incrby` | Increment by amount *(write)* |

### Hashes

| Tool | Description |
|------|-------------|
| `database_hgetall` | Get all hash fields |
| `database_hget` | Get hash field |
| `database_hlen` | Get hash length |
| `database_hset` | Set hash field *(write)* |
| `database_hset_multiple` | Set multiple hash fields *(write)* |
| `database_hdel` | Delete hash fields *(write)* |

### Lists

| Tool | Description |
|------|-------------|
| `database_lrange` | Get list range |
| `database_lindex` | Get element by index |
| `database_llen` | Get list length |
| `database_lpush` | Push to head *(write)* |
| `database_rpush` | Push to tail *(write)* |
| `database_lpop` | Pop from head *(write)* |
| `database_rpop` | Pop from tail *(write)* |
| `database_lset` | Set element at index *(write)* |

### Sets

| Tool | Description |
|------|-------------|
| `database_smembers` | Get all members |
| `database_sismember` | Check membership |
| `database_scard` | Get set size |
| `database_sadd` | Add members *(write)* |
| `database_srem` | Remove members *(write)* |

### Sorted Sets

| Tool | Description |
|------|-------------|
| `database_zrange` | Get range (low to high) |
| `database_zrevrange` | Get range (high to low) |
| `database_zrange_withscores` | Get range with scores |
| `database_zrevrange_withscores` | Get reverse range with scores |
| `database_zrangebyscore` | Get by score range |
| `database_zscore` | Get member score |
| `database_zrank` | Get member rank |
| `database_zrevrank` | Get reverse rank |
| `database_zcard` | Get sorted set size |
| `database_zadd` | Add members with scores *(write)* |
| `database_zrem` | Remove members *(write)* |
| `database_zincrby` | Increment member score *(write)* |

### Monitoring & Config

| Tool | Description |
|------|-------------|
| `database_slowlog` | Get slow log entries |
| `database_slowlog_len` | Get slow log length |
| `database_client_list` | List connected clients |
| `database_config_get` | Get config values |
| `database_module_list` | List loaded modules |

### Generic & Pipeline

| Tool | Description |
|------|-------------|
| `database_execute` | Execute any Redis command |
| `database_pipeline` | Execute multiple commands in pipeline |

### RediSearch

| Tool | Description |
|------|-------------|
| `database_ft_list` | List all indexes |
| `database_ft_info` | Get index information |
| `database_ft_search` | Search an index |
| `database_ft_aggregate` | Run aggregation query |
| `database_ft_create` | Create an index *(write)* |
| `database_ft_alter` | Add field to index *(write)* |
| `database_ft_dropindex` | Delete an index *(write)* |
| `database_ft_aliasadd` | Create index alias *(write)* |
| `database_ft_aliasupdate` | Update index alias *(write)* |
| `database_ft_aliasdel` | Delete index alias *(write)* |
| `database_ft_synupdate` | Update synonym group *(write)* |
| `database_ft_syndump` | Dump synonym groups |
| `database_ft_spellcheck` | Get spelling suggestions |
| `database_ft_explain` | Explain query execution plan |
| `database_ft_tagvals` | Get unique tag values |
| `database_ft_sugadd` | Add autocomplete suggestion *(write)* |
| `database_ft_sugget` | Get autocomplete suggestions |
| `database_ft_sugdel` | Delete autocomplete suggestion *(write)* |
| `database_ft_suglen` | Get suggestion count |

### RedisJSON

| Tool | Description |
|------|-------------|
| `database_json_get` | Get JSON value |
| `database_json_mget` | Get JSON from multiple keys |
| `database_json_set` | Set JSON value *(write)* |
| `database_json_del` | Delete JSON path *(write)* |
| `database_json_type` | Get JSON value type |
| `database_json_arrappend` | Append to JSON array *(write)* |
| `database_json_arrinsert` | Insert into JSON array *(write)* |
| `database_json_arrindex` | Find index in JSON array |
| `database_json_arrlen` | Get JSON array length |
| `database_json_arrpop` | Pop from JSON array *(write)* |
| `database_json_arrtrim` | Trim JSON array *(write)* |
| `database_json_objkeys` | Get JSON object keys |
| `database_json_objlen` | Get JSON object length |
| `database_json_numincrby` | Increment JSON number *(write)* |
| `database_json_toggle` | Toggle JSON boolean *(write)* |
| `database_json_clear` | Clear JSON container *(write)* |
| `database_json_strlen` | Get JSON string length |

### RedisTimeSeries

| Tool | Description |
|------|-------------|
| `database_ts_create` | Create time series *(write)* |
| `database_ts_add` | Add sample *(write)* |
| `database_ts_get` | Get latest sample |
| `database_ts_range` | Query time range |
| `database_ts_info` | Get time series info |

### RedisBloom

| Tool | Description |
|------|-------------|
| `database_bf_reserve` | Create Bloom filter *(write)* |
| `database_bf_add` | Add to Bloom filter *(write)* |
| `database_bf_madd` | Add multiple to Bloom filter *(write)* |
| `database_bf_exists` | Check Bloom filter |
| `database_bf_mexists` | Check multiple in Bloom filter |
| `database_bf_info` | Get Bloom filter info |

### Redis Streams

| Tool | Description |
|------|-------------|
| `database_xadd` | Add entry to stream *(write)* |
| `database_xread` | Read from streams |
| `database_xrange` | Get entries by ID range |
| `database_xrevrange` | Get entries in reverse |
| `database_xlen` | Get stream length |
| `database_xinfo_stream` | Get stream info |
| `database_xinfo_groups` | List consumer groups |
| `database_xinfo_consumers` | List consumers in group |
| `database_xgroup_create` | Create consumer group *(write)* |
| `database_xgroup_destroy` | Delete consumer group *(write)* |
| `database_xgroup_delconsumer` | Remove consumer *(write)* |
| `database_xgroup_setid` | Set group last ID *(write)* |
| `database_xreadgroup` | Read as consumer group |
| `database_xack` | Acknowledge entries *(write)* |
| `database_xdel` | Delete entries *(write)* |
| `database_xtrim` | Trim stream *(write)* |
| `database_xpending` | Get pending entries |
| `database_xclaim` | Claim pending entries *(write)* |
| `database_xautoclaim` | Auto-claim pending entries *(write)* |

### Pub/Sub

| Tool | Description |
|------|-------------|
| `database_publish` | Publish message *(write)* |
| `database_pubsub_channels` | List active channels |
| `database_pubsub_numsub` | Get subscriber counts |
| `database_pubsub_numpat` | Get pattern subscription count |

## Tool Categories Summary

| Category | Read | Write | Total |
|----------|------|-------|-------|
| **Redis Cloud** | 19 | 10 | 29 |
| **Redis Enterprise** | 56 | 27 | 83 |
| **Database - Core** | 18 | 24 | 42 |
| **Database - RediSearch** | 9 | 10 | 19 |
| **Database - RedisJSON** | 6 | 11 | 17 |
| **Database - RedisTimeSeries** | 3 | 2 | 5 |
| **Database - RedisBloom** | 3 | 3 | 6 |
| **Database - Streams** | 7 | 12 | 19 |
| **Database - Pub/Sub** | 3 | 1 | 4 |
| **Total** | **124** | **100** | **237** |

## Example Tool Usage

### Working with Data

> **You**: Add a user to my users hash with name "Alice" and email "alice@example.com"
>
> **AI**: *uses `database_hset_multiple`*
>
> Added 2 fields to hash `user:123`:
> - name: Alice
> - email: alice@example.com

### Leaderboard Operations

> **You**: Show me the top 5 players on the leaderboard
>
> **AI**: *uses `database_zrevrange_withscores` with key="leaderboard", start=0, stop=4*
>
> | Rank | Player | Score |
> |------|--------|-------|
> | 1 | alice | 15000 |
> | 2 | bob | 12500 |
> | 3 | charlie | 11000 |
> | 4 | diana | 9500 |
> | 5 | eve | 8000 |

### Full-Text Search

> **You**: Search for products containing "wireless headphones" under $100
>
> **AI**: *uses `database_ft_search` with index="products", query="@name:wireless headphones @price:[0 100]"*
>
> Found 3 products:
> - Sony WH-1000XM4 - $89.99
> - JBL Tune 500BT - $49.99
> - Anker Soundcore - $39.99

### Stream Processing

> **You**: Show me the last 5 events from my orders stream
>
> **AI**: *uses `database_xrevrange` with key="orders", end="+", start="-", count=5*
>
> Recent orders:
> - 1705432100000-0: order_id=ORD-789, status=shipped
> - 1705431900000-0: order_id=ORD-788, status=processing
> - 1705431700000-0: order_id=ORD-787, status=completed
> ...

### Infrastructure Management

> **You**: What's the cluster health?
>
> **AI**: *uses `enterprise_cluster_get`, `enterprise_cluster_stats`, `enterprise_nodes_list`*
>
> **Cluster**: docker-cluster (v8.0.2-17)
>
> | Metric | Value |
> |--------|-------|
> | CPU Utilization | 4% |
> | Memory Available | 7.81 GB |
> | Nodes | 1 active |
> | Databases | 1 active |
| Alerts | None |
