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
# Requires secure-storage feature
cargo install redisctl --features secure-storage

# Store in keyring
redisctl profile set prod \
  --cloud-api-key "$KEY" \
  --cloud-secret-key "$SECRET" \
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
redisctl profile set prod \
  --cloud-api-key '${REDIS_CLOUD_API_KEY}' \
  --cloud-secret-key '${REDIS_CLOUD_SECRET_KEY}'
```

Config file stores the reference, not the value:
```toml
[profiles.prod]
cloud_api_key = "${REDIS_CLOUD_API_KEY}"
```

**Pros:** Credentials not in config file, flexible
**Cons:** Requires environment setup

### 4. Plaintext (Not Recommended)

```bash
redisctl profile set dev \
  --cloud-api-key "actual-key"
```

**Pros:** Simple
**Cons:** Credentials visible in config file

!!! danger "Avoid Plaintext for Production"
    Never store production credentials in plaintext. Use keyring or environment variables.

## Best Practices

### Development

```bash
# Use environment variables or plaintext profiles
redisctl profile set dev --cloud-api-key "dev-key"
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
redisctl profile set prod \
  --cloud-api-key "$KEY" \
  --cloud-secret-key "$SECRET" \
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
```

### Config File Location

```bash
# Find your config file
cat ~/.config/redisctl/config.toml
```

## Revoking Credentials

### Redis Cloud

1. Go to [Redis Cloud Console](https://app.redislabs.com/)
2. Navigate to Access Management > API Keys
3. Delete the compromised key
4. Generate a new key
5. Update your profiles

### Redis Enterprise

1. Change the password in the cluster admin console
2. Update all profiles using that password

```bash
redisctl profile set prod --enterprise-password "$NEW_PASSWORD"
```
