# Docker Usage

Run redisctl without installing anything.

## Quick Start

```bash
docker run ghcr.io/redis-developer/redisctl --help
```

## Passing Credentials

### Environment Variables

```bash
docker run --rm \
  -e REDIS_CLOUD_API_KEY \
  -e REDIS_CLOUD_SECRET_KEY \
  ghcr.io/redis-developer/redisctl cloud subscription list
```

### Mount Config File

If you have a local configuration:

```bash
docker run --rm \
  -v ~/.config/redisctl:/root/.config/redisctl:ro \
  ghcr.io/redis-developer/redisctl --profile prod cloud subscription list
```

## Convenient Aliases

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

=== "Redis Cloud"

    ``` bash
    alias redisctl-cloud='docker run --rm \
      -e REDIS_CLOUD_API_KEY \
      -e REDIS_CLOUD_SECRET_KEY \
      ghcr.io/redis-developer/redisctl'

    # Usage
    redisctl-cloud cloud subscription list
    ```

=== "Redis Enterprise"

    ``` bash
    alias redisctl-enterprise='docker run --rm \
      -e REDIS_ENTERPRISE_URL \
      -e REDIS_ENTERPRISE_USER \
      -e REDIS_ENTERPRISE_PASSWORD \
      -e REDIS_ENTERPRISE_INSECURE \
      ghcr.io/redis-developer/redisctl'

    # Usage
    redisctl-enterprise enterprise cluster get
    ```

=== "With Config"

    ``` bash
    alias redisctl='docker run --rm \
      -v ~/.config/redisctl:/root/.config/redisctl:ro \
      ghcr.io/redis-developer/redisctl'

    # Usage
    redisctl --profile prod cloud database list
    ```

## Saving Output to Files

Mount a volume to save command output:

```bash
docker run --rm \
  -e REDIS_ENTERPRISE_URL \
  -e REDIS_ENTERPRISE_USER \
  -e REDIS_ENTERPRISE_PASSWORD \
  -v $(pwd)/output:/output \
  ghcr.io/redis-developer/redisctl \
  enterprise support-package cluster --output-dir /output
```

## Image Tags

| Tag | Description |
|-----|-------------|
| `latest` | Most recent release |
| `0.7.3` | Specific version |
| `0.7` | Latest patch in minor version |

```bash
# Pin to specific version
docker run ghcr.io/redis-developer/redisctl:0.7.3 --version
```

## CI/CD Example

```yaml
# GitHub Actions
- name: List databases
  run: |
    docker run --rm \
      -e REDIS_CLOUD_API_KEY=${{ secrets.REDIS_CLOUD_API_KEY }} \
      -e REDIS_CLOUD_SECRET_KEY=${{ secrets.REDIS_CLOUD_SECRET_KEY }} \
      ghcr.io/redis-developer/redisctl \
      cloud database list --subscription-id ${{ vars.SUBSCRIPTION_ID }} -o json
```
