# Security

Best practices for secure credential management.

## Credential Storage Options

### 1. Environment Variables (Recommended for CI/CD)

```bash
export REDIS_CLOUD_API_KEY="your-key"
export REDIS_CLOUD_SECRET_KEY="your-secret"
```

**Pros:** Not stored on disk, standard CI/CD pattern
**Cons:** Visible in process listings, shell history

### 2. OS Keyring (Recommended for Workstations)

```bash
redisctl profile set prod --type cloud \
  --api-key "$KEY" \
  --api-secret "$SECRET" \
  --use-keyring
```

Credentials are stored in:
- **macOS:** Keychain
- **Windows:** Credential Manager
- **Linux:** Secret Service (GNOME Keyring, KWallet)

**Pros:** Encrypted, OS-managed, survives reboots
**Cons:** Requires additional feature flag

### 3. Environment Variable References

```bash
redisctl profile set prod --type cloud \
  --api-key '${REDIS_CLOUD_API_KEY}' \
  --api-secret '${REDIS_CLOUD_SECRET_KEY}'
```

Config file stores the reference, not the value:
```toml
[profiles.prod]
deployment_type = "cloud"
api_key = "${REDIS_CLOUD_API_KEY}"
```

**Pros:** Credentials not in config file, flexible
**Cons:** Requires environment setup

### 4. Plaintext (Not Recommended)

```bash
redisctl profile set dev --type cloud \
  --api-key "actual-key" \
  --api-secret "actual-secret"
```

**Pros:** Simple
**Cons:** Credentials visible in config file

!!! danger "Avoid Plaintext for Production"
    Never store production credentials in plaintext. Use keyring or environment variables.

## Best Practices

### Development

```bash
# Use environment variables or plaintext profiles
redisctl profile set dev --type cloud \
  --api-key "dev-key" \
  --api-secret "dev-secret"
```

### CI/CD

```yaml
# GitHub Actions
env:
  REDIS_CLOUD_API_KEY: ${{ secrets.REDIS_CLOUD_API_KEY }}
  REDIS_CLOUD_SECRET_KEY: ${{ secrets.REDIS_CLOUD_SECRET_KEY }}

steps:
  - run: redisctl cloud subscription list
```

### Production Workstations

```bash
# Use OS keyring
redisctl profile set prod --type cloud \
  --api-key "$KEY" \
  --api-secret "$SECRET" \
  --use-keyring
```

## Files.com API Key

For support package uploads:

```bash
# Environment variable
export REDIS_ENTERPRISE_FILES_API_KEY="your-key"

# Or secure storage
redisctl files-key set "$KEY" --use-keyring
```

## Audit

### Check What's Stored

```bash
# List profiles (credentials redacted)
redisctl profile list

# Show profile details
redisctl profile show prod

# Validate all profiles
redisctl profile validate
```

### Config File Location

```bash
# Find your config file
cat ~/.config/redisctl/config.toml
```

## TLS Certificate Configuration

### Kubernetes Deployments

Redis Enterprise clusters deployed on Kubernetes typically use self-signed certificates. Instead of disabling TLS verification with `--insecure`, you can provide the cluster's CA certificate for secure connections:

```bash
# Extract CA cert from Kubernetes secret
kubectl get secret rec-proxy-cert -o jsonpath='{.data.ca\.crt}' | base64 -d > ca.crt

# Use the CA cert with a profile
redisctl profile set k8s-cluster --type enterprise \
  --url "https://rec-api.redis.svc:9443" \
  --username "admin@cluster.local" \
  --password "$PASSWORD" \
  --ca-cert "./ca.crt"

# Or use environment variables
export REDIS_ENTERPRISE_CA_CERT="./ca.crt"
export REDIS_ENTERPRISE_URL="https://rec-api.redis.svc:9443"
export REDIS_ENTERPRISE_USER="admin@cluster.local"
export REDIS_ENTERPRISE_PASSWORD="password"

redisctl enterprise cluster get
```

### When to Use Each Option

| Option | Use Case |
|--------|----------|
| `--ca-cert` / `REDIS_ENTERPRISE_CA_CERT` | Kubernetes, self-signed certs where you have the CA |
| `--insecure` / `REDIS_ENTERPRISE_INSECURE=true` | Local development, testing only |
| Neither | Production clusters with trusted certificates |

!!! warning "Avoid Insecure Mode in Production"
    Using `--insecure` disables all certificate verification and should only be used for local development. For Kubernetes deployments, always use `--ca-cert` with the cluster's CA certificate.

## Revoking Credentials

### Redis Cloud

1. Go to [Redis Cloud Console](https://cloud.redis.io)
2. Navigate to Access Management > API Keys
3. Delete the compromised key
4. Generate a new key
5. Update your profiles

### Redis Enterprise

1. Change the password in the cluster admin console
2. Update all profiles using that password

```bash
redisctl profile set prod --type enterprise \
  --password "$NEW_PASSWORD"
```
