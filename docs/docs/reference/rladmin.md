# rladmin vs redisctl

A comparison guide for Redis Enterprise users familiar with rladmin.

## Overview

| Tool | Purpose |
|------|---------|
| **rladmin** | Redis Enterprise's built-in CLI for node-local management |
| **redisctl** | Remote cluster management via REST API |

They're **complementary tools** - use both!

## Quick Comparison

| Feature | rladmin | redisctl |
|---------|---------|----------|
| Deployment | Pre-installed on nodes | Single binary, any platform |
| Access | Local (SSH required) | Remote (REST API) |
| Platforms | Linux only (on nodes) | macOS, Windows, Linux |
| Output | Text only | JSON, YAML, Table |
| Scripting | Text parsing required | Native JSON + JMESPath |
| Multi-cluster | One at a time | Profile system |

## Where Each Tool Excels

<div class="grid" markdown>

:material-server:{ .lg } **rladmin**

- Local node operations
- Works when API is down
- Low-level node commands
- Already installed on nodes

:material-laptop:{ .lg } **redisctl**

- Remote management (no SSH)
- Structured output for automation
- Works on developer laptops
- Multi-cluster profile system
- Support package automation

</div>

## Example Comparison

### Get Database Info

=== "rladmin"

    ```bash
    # 1. SSH to node
    ssh admin@cluster-node

    # 2. Get info (text output, need to parse)
    rladmin info bdb 1 | grep memory | awk '{print $2}'
    ```

=== "redisctl"

    ```bash
    # From your laptop (no SSH)
    redisctl enterprise database get 1 -o json -q 'memory_size'
    ```

### List Databases

=== "rladmin"

    ```bash
    ssh admin@cluster-node
    rladmin status databases
    ```

=== "redisctl"

    ```bash
    redisctl enterprise database list

    # Or with filtering
    redisctl enterprise database list -o json -q '[?status==`active`].name'
    ```

### Generate Support Package

=== "rladmin"

    ```bash
    # 1. SSH to node
    ssh admin@cluster-node

    # 2. Generate package
    rladmin cluster debug_info

    # 3. Copy to local machine
    scp admin@node:/tmp/debug*.tar.gz ./

    # 4. Manually upload via web browser
    # Total time: 10+ minutes
    ```

=== "redisctl"

    ```bash
    # One command from your laptop
    redisctl enterprise support-package cluster --optimize --upload

    # Total time: 30 seconds
    ```

### Check Cluster Status

=== "rladmin"

    ```bash
    ssh admin@cluster-node
    rladmin status
    ```

=== "redisctl"

    ```bash
    redisctl enterprise cluster get
    redisctl enterprise node list
    ```

## When to Use Which

### Use rladmin when:

- You're SSH'd into a cluster node
- Need low-level node operations
- The REST API is unavailable
- Debugging directly on nodes
- Performing node-specific maintenance

### Use redisctl when:

- Managing clusters remotely
- Building CI/CD automation
- Managing multiple clusters
- Need structured output (JSON/YAML)
- Generating support packages
- Working from your laptop

## Command Mapping

| Task | rladmin | redisctl |
|------|---------|----------|
| Cluster status | `rladmin status` | `redisctl enterprise cluster get` |
| List databases | `rladmin status databases` | `redisctl enterprise database list` |
| Database info | `rladmin info bdb <id>` | `redisctl enterprise database get <id>` |
| Node status | `rladmin status nodes` | `redisctl enterprise node list` |
| Support package | `rladmin cluster debug_info` | `redisctl enterprise support-package cluster` |

## Best Practice

!!! tip "Use Both"
    - **redisctl** for day-to-day operations, automation, and remote management
    - **rladmin** for emergency troubleshooting and node-level operations

## Migration Path

If you're currently using rladmin scripts:

1. Start by using redisctl for new automation
2. Replace SSH + rladmin + text parsing with redisctl + JSON
3. Keep rladmin skills for emergency troubleshooting
4. Use redisctl profiles for multi-cluster management
