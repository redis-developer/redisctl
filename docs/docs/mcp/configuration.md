# Configuration

The redisctl MCP server has three configuration axes:

1. **What to load** -- which toolsets and sub-modules are active (`--tools`)
2. **What to permit** -- which operations are allowed (`--read-only`, policy files)
3. **How to connect** -- transport and credentials (`--transport`, `--profile`, `--database-url`)

This page covers all three, with emphasis on `--tools` for controlling the tool surface.

## CLI Reference

| Flag | Short | Env Var | Default | Description |
|------|-------|---------|---------|-------------|
| `--transport` | `-t` | -- | `stdio` | Transport mode (`stdio` or `http`) |
| `--profile` | `-p` | `REDISCTL_PROFILE` | -- | Profile name(s) for credential resolution (repeatable) |
| `--read-only` | -- | -- | `true` | Read-only mode; use `--read-only=false` for writes. Ignored when a policy file is active |
| `--policy` | -- | `REDISCTL_MCP_POLICY` | -- | Path to TOML policy file for granular access control. Overrides `--read-only` |
| `--database-url` | -- | `REDIS_URL` | -- | Redis URL for direct database connections |
| `--tools` | -- | -- | -- | Comma-delimited toolset/sub-module selection (see below) |
| `--host` | -- | -- | `127.0.0.1` | HTTP bind host (HTTP transport only) |
| `--port` | -- | -- | `8080` | HTTP bind port (HTTP transport only) |
| `--oauth` | -- | -- | `false` | Enable OAuth authentication (HTTP transport only) |
| `--oauth-issuer` | -- | `OAUTH_ISSUER` | -- | OAuth issuer URL |
| `--oauth-audience` | -- | `OAUTH_AUDIENCE` | -- | OAuth audience |
| `--jwks-uri` | -- | `OAUTH_JWKS_URI` | -- | JWKS URI for token validation |
| `--max-concurrent` | -- | -- | `10` | Maximum concurrent requests |
| `--rate-limit-ms` | -- | -- | `100` | Rate limit interval in milliseconds |
| `--request-timeout-secs` | -- | -- | `30` | Request timeout in seconds (HTTP transport only) |
| `--log-level` | -- | `RUST_LOG` | `info` | Log level |

## The `--tools` Flag

By default, the MCP server loads all compiled-in toolsets. With `--tools`, you control exactly which toolsets and sub-modules are active. This is useful when you want to:

- Reduce the tool surface to what you actually need
- Keep token usage low by exposing fewer tool descriptions
- Scope an AI assistant to a specific domain (e.g., Cloud only)

### Syntax

```
--tools <spec>[,<spec>...]
```

Each `<spec>` is either:

- **Bare name** -- loads all sub-modules for that toolset: `cloud`, `enterprise`, `database`, `app`
- **Colon syntax** -- loads a single sub-module: `cloud:subscriptions`, `enterprise:observability`

Specs are comma-delimited. You can mix bare and colon forms freely.

### Resolution Priority

The server resolves which toolsets to load in this order:

1. **Explicit `--tools`** -- always wins when provided
2. **Auto-detect from profiles** -- infers toolsets from your configured profile types (e.g., a Cloud profile enables the `cloud` toolset)
3. **All compiled-in features** -- fallback when neither of the above applies

!!! note
    When `--tools` is not explicitly set, toolsets marked `enabled = false` in a policy file are also removed. When `--tools` is explicit, policy-based toolset disabling is skipped.

### Available Toolsets and Sub-Modules

| Toolset | Sub-modules | Total Tools |
|---------|-------------|-------------|
| `cloud` | `subscriptions`, `account`, `networking`, `fixed`, `raw` | 148 |
| `enterprise` | `cluster`, `databases`, `rbac`, `observability`, `proxy`, `services`, `raw` | 92 |
| `database` | `server`, `keys`, `structures`, `diagnostics`, `raw` | 55 |
| `app` | *(none -- flat toolset)* | 8 |
| *(system)* | *(always loaded)* | 2 |

The two system tools (`list_available_tools` and `show_policy`) are always registered regardless of `--tools` selection.

### Examples

**Cloud only** -- all Cloud sub-modules (148 tools + system):

```bash
redisctl-mcp --profile my-cloud --tools cloud
```

**Cloud subscriptions and networking only** (87 tools + system):

```bash
redisctl-mcp --profile my-cloud --tools cloud:subscriptions,cloud:networking
```

**Enterprise monitoring** -- cluster info + observability (40 tools + system):

```bash
redisctl-mcp --profile my-re --tools enterprise:cluster,enterprise:observability
```

**Database only** -- direct Redis operations (55 tools + system):

```bash
redisctl-mcp --database-url redis://localhost:6379 --tools database
```

**Minimal Cloud** -- just account info and subscriptions (69 tools + system):

```bash
redisctl-mcp --profile my-cloud --tools cloud:account,cloud:subscriptions
```

**Everything except database** -- Cloud + Enterprise + profile management:

```bash
redisctl-mcp --profile my-cloud --profile my-re --tools cloud,enterprise,app
```

**Mixed selective and bare** -- all Enterprise + specific Cloud sub-modules:

```bash
redisctl-mcp --tools enterprise,cloud:subscriptions,cloud:account
```

### Bare-Overrides-Selective Rule

If you specify both a bare name and colon-syntax for the same toolset, the bare name wins. For example:

```bash
# "cloud" overrides "cloud:subscriptions" -- all Cloud sub-modules are loaded
--tools cloud:subscriptions,cloud
```

This makes it easy to "upgrade" a selective choice to the full toolset without removing the specific entries.

### Error Behavior

The server exits with an error if:

- An unknown toolset name is used (e.g., `--tools nosuch`)
- An unknown sub-module is used (e.g., `--tools cloud:nosuch`)
- A sub-module is specified for `app`, which has no sub-modules (e.g., `--tools app:anything`)

Error messages include the list of valid toolset or sub-module names.

## Safety Tiers

The MCP server enforces three safety tiers that control which categories of operations are permitted:

| Tier | Flag / Policy Value | Behavior |
|------|-------------------|----------|
| **Read-only** | `--read-only` (default) / `"read-only"` | Only tools marked as read-only are allowed |
| **Read-write** | -- / `"read-write"` | Reads + non-destructive writes (e.g., create, update) |
| **Full** | `--read-only=false` / `"full"` | All operations including destructive ones (delete, flush) |

The `--read-only` flag maps to the read-only and full tiers. For the intermediate read-write tier, use a policy file:

```toml
# mcp-policy.toml
tier = "read-write"
```

```bash
redisctl-mcp --profile my-profile --policy mcp-policy.toml
```

!!! warning
    When a policy file is active, it overrides `--read-only`. The policy file is the authoritative source for safety tier configuration.

## Presets

Presets control tool **visibility** -- which tools are presented to the AI model. This is independent of `--tools` (which controls what is *loaded*) and safety tiers (which control what is *permitted*).

Two presets are available:

| Preset | Description |
|--------|-------------|
| `"all"` | Every loaded tool is visible (default) |
| `"essentials"` | A curated subset per toolset: ~20 Cloud, ~18 Enterprise, ~15 Database, all 8 App tools |

Configure presets in the policy file:

```toml
[tools]
preset = "essentials"
include = ["enterprise_raw_api"]  # add tools on top of the preset
exclude = ["flush_database"]      # remove tools from the resolved set
```

!!! tip
    Use the `list_available_tools` system tool at runtime to see which tools are active vs. hidden under the current preset. This lets you discover tools you might want to add to the `include` list.

## Choosing the Right Approach

| Goal | Approach |
|------|----------|
| Limit to one product (Cloud or Enterprise) | `--tools cloud` or `--tools enterprise` |
| Reduce tool surface within a product | `--tools cloud:subscriptions,cloud:account` |
| Allow writes but not destructive ops | Policy file with `tier = "read-write"` |
| Curate which tools the AI sees | Policy file with `preset = "essentials"` |
| Quick read-only exploration | Default settings (no extra flags) |
| Full access for development | `--read-only=false --tools cloud,enterprise,database` |
| Fine-grained per-tool control | Policy file with `include` / `exclude` lists |

## Next Steps

- [Tools Reference](tools-reference.md) -- see what tools are available per toolset and sub-module
- [Getting Started](getting-started.md) -- installation and IDE setup
- [Advanced Usage](advanced-usage.md) -- JMESPath integration and analytics
