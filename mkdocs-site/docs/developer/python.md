# Python Bindings

Use the Redis API client libraries from Python.

## Installation

```bash
pip install redis-cloud redis-enterprise
```

## Quick Start

=== "Redis Cloud"

    ```python
    from redis_cloud import CloudClient

    # Create client with API credentials
    client = CloudClient(
        api_key="your-api-key",
        api_secret="your-api-secret"
    )

    # Or use environment variables
    client = CloudClient.from_env()

    # List subscriptions (sync)
    subs = client.subscriptions_sync()
    for sub in subs:
        print(f"{sub['id']}: {sub['name']}")

    # Async usage
    async def main():
        subs = await client.subscriptions()
    ```

=== "Redis Enterprise"

    ```python
    from redis_enterprise import EnterpriseClient

    # Create client
    client = EnterpriseClient(
        base_url="https://cluster:9443",
        username="admin@redis.local",
        password="secret",
        insecure=True  # For self-signed certs
    )

    # Or use environment variables
    client = EnterpriseClient.from_env()

    # Get cluster info (sync)
    cluster = client.cluster_info_sync()
    print(f"Cluster: {cluster['name']}")

    # List databases
    for db in client.databases_sync():
        print(f"{db['uid']}: {db['name']}")
    ```

## CloudClient API

### Constructor

```python
CloudClient(
    api_key: str,
    api_secret: str,
    base_url: str | None = None,
    timeout_secs: int | None = None
)
```

### Methods

| Method | Description |
|--------|-------------|
| `subscriptions()` / `subscriptions_sync()` | List all subscriptions |
| `subscription(id)` / `subscription_sync(id)` | Get subscription by ID |
| `databases(subscription_id)` / `databases_sync(subscription_id)` | List databases |
| `database(subscription_id, database_id)` / `database_sync(...)` | Get database |
| `get(path)` / `get_sync(path)` | Raw GET request |
| `post(path, body)` / `post_sync(path, body)` | Raw POST request |
| `delete(path)` / `delete_sync(path)` | Raw DELETE request |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `REDIS_CLOUD_API_KEY` | API key |
| `REDIS_CLOUD_API_SECRET` | API secret |
| `REDIS_CLOUD_BASE_URL` | Base URL (optional) |

## EnterpriseClient API

### Constructor

```python
EnterpriseClient(
    base_url: str,
    username: str,
    password: str,
    insecure: bool = False,
    timeout_secs: int | None = None
)
```

### Methods

| Method | Description |
|--------|-------------|
| `cluster_info()` / `cluster_info_sync()` | Get cluster info |
| `cluster_stats()` / `cluster_stats_sync()` | Get cluster statistics |
| `license()` / `license_sync()` | Get license info |
| `databases()` / `databases_sync()` | List all databases |
| `database(uid)` / `database_sync(uid)` | Get database by UID |
| `nodes()` / `nodes_sync()` | List all nodes |
| `node(uid)` / `node_sync(uid)` | Get node by UID |
| `users()` / `users_sync()` | List all users |
| `get(path)` / `get_sync(path)` | Raw GET request |
| `post(path, body)` / `post_sync(path, body)` | Raw POST request |
| `delete(path)` / `delete_sync(path)` | Raw DELETE request |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `REDIS_ENTERPRISE_URL` | Cluster URL |
| `REDIS_ENTERPRISE_USER` | Username |
| `REDIS_ENTERPRISE_PASSWORD` | Password |
| `REDIS_ENTERPRISE_INSECURE` | Set to `true` for self-signed certs |

## Platform Support

Pre-built wheels are available for:

- **Linux**: x86_64, aarch64
- **macOS**: Intel (x86_64), Apple Silicon (arm64)
- **Windows**: x86_64

Python versions: 3.9, 3.10, 3.11, 3.12, 3.13

## Links

- [redis-cloud on PyPI](https://pypi.org/project/redis-cloud/)
- [redis-enterprise on PyPI](https://pypi.org/project/redis-enterprise/)
- [redis-cloud-rs GitHub](https://github.com/redis-developer/redis-cloud-rs)
- [redis-enterprise-rs GitHub](https://github.com/redis-developer/redis-enterprise-rs)
