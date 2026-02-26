# Troubleshooting

Common issues and how to resolve them.

## First Steps

Before diving into specific issues, run profile validation:

```bash
# Check profile configuration for errors
redisctl profile validate

# Also test actual API/database connectivity
redisctl profile validate --connect
```

This catches most configuration problems -- missing fields, invalid credential references, and unreachable endpoints.

## Authentication Failures

### Wrong or Expired Credentials

**Symptom:**

```
Error: 401 Unauthorized
```

or

```
Error: 403 Forbidden
```

**Cause:** API key/secret is incorrect, expired, or lacks the required permissions.

**Fix:**

1. Verify your credentials are correct:

    ```bash
    redisctl profile show my-profile
    ```

2. For Cloud profiles, confirm your API key is active in the [Redis Cloud console](https://cloud.redis.io) under **Access Management > API Keys**.

3. Update credentials if needed:

    ```bash
    redisctl profile set my-profile --type cloud \
      --api-key "$NEW_KEY" \
      --api-secret "$NEW_SECRET"
    ```

### Environment Variable Not Set

**Symptom:**

```
Error: missing required credential: api_key
```

**Cause:** Profile uses an environment variable reference (`${VAR_NAME}`) but the variable is not set.

**Fix:**

```bash
# Check which variable the profile references
redisctl profile show my-profile

# Set the missing variable
export REDIS_CLOUD_API_KEY="your-key"
```

### Wrong Profile Used

**Symptom:** Command succeeds but returns unexpected data (wrong account, wrong cluster).

**Cause:** The default profile or `--profile` flag points to the wrong environment.

**Fix:**

```bash
# Check which profile is active
redisctl profile current --type cloud

# Check defaults
redisctl profile list

# Use explicit profile
redisctl --profile correct-profile cloud subscription list
```

## Connection Errors

### Connection Refused

**Symptom:**

```
Error: connection refused (os error 61)
```

**Cause:** The target host is not accepting connections on the specified port. Common when the Enterprise cluster URL is wrong or the cluster is down.

**Fix:**

1. Verify the URL in your profile:

    ```bash
    redisctl profile show my-profile
    ```

2. Confirm the host and port are reachable:

    ```bash
    curl -k https://cluster.example.com:9443/v1/cluster
    ```

3. For Enterprise, the default API port is `9443`. Make sure your URL includes it.

### Connection Timeout

**Symptom:**

```
Error: operation timed out
```

**Cause:** Network connectivity issue -- firewall, VPN not connected, or DNS resolution failure.

**Fix:**

- Check network connectivity to the target host
- Verify VPN is connected if the cluster is on a private network
- Check DNS resolution: `nslookup cluster.example.com`

### DNS Resolution Failure

**Symptom:**

```
Error: failed to resolve host
```

**Cause:** Hostname cannot be resolved.

**Fix:**

- Verify the hostname is correct in your profile
- Check DNS configuration
- For Kubernetes clusters, ensure you're using the correct service name and namespace

## TLS / Certificate Issues

### Self-Signed Certificate Error

**Symptom:**

```
Error: invalid peer certificate: UnknownIssuer
```

**Cause:** The server uses a self-signed or private CA certificate that your system doesn't trust.

**Fix (recommended):** Provide the CA certificate:

```bash
redisctl profile set my-cluster --type enterprise \
  --url "https://cluster:9443" \
  --username "admin@cluster.local" \
  --password "$PASSWORD" \
  --ca-cert "/path/to/ca.crt"
```

**Fix (development only):** Disable TLS verification:

```bash
redisctl profile set my-cluster --type enterprise \
  --url "https://cluster:9443" \
  --username "admin@cluster.local" \
  --password "$PASSWORD" \
  --insecure
```

!!! warning
    Only use `--insecure` for local development. For Kubernetes deployments, extract the CA cert and use `--ca-cert` instead.

### Extracting Kubernetes CA Certificates

```bash
# Extract CA cert from the cluster's proxy secret
kubectl get secret rec-proxy-cert -o jsonpath='{.data.ca\.crt}' | base64 -d > ca.crt

# Use it with your profile
redisctl profile set k8s-cluster --type enterprise \
  --ca-cert "./ca.crt" \
  --url "https://rec-api.redis.svc:9443" \
  --username "admin@cluster.local" \
  --password "$PASSWORD"
```

## Profile Type Mismatch

### Using the Wrong Profile Type for a Command

**Symptom:**

```
Error: profile 'my-profile' is not a cloud profile
```

or unexpected missing-credential errors when the profile looks correct.

**Cause:** The profile's `deployment_type` doesn't match the command you're running (e.g., using an Enterprise profile with `cloud subscription list`).

**Fix:**

1. Check the profile type:

    ```bash
    redisctl profile show my-profile
    ```

2. Use the correct profile:

    ```bash
    # Cloud commands need a cloud profile
    redisctl --profile my-cloud-profile cloud subscription list

    # Enterprise commands need an enterprise profile
    redisctl --profile my-enterprise-profile enterprise cluster get
    ```

3. Verify your defaults are set correctly:

    ```bash
    redisctl profile list
    # Check which profile is marked as default for each type
    ```

## Database Connection Issues

### Authentication Required

**Symptom:**

```
Error: NOAUTH Authentication required
```

**Cause:** The Redis server requires a password but none was provided in the profile.

**Fix:**

```bash
redisctl profile set my-db --type database \
  --host "hostname" \
  --port 12345 \
  --password "$PASSWORD"
```

### TLS Required

**Symptom:**

Connection hangs or fails immediately when connecting to a cloud-hosted Redis database.

**Cause:** Most cloud Redis databases require TLS but the profile has TLS disabled.

**Fix:**

TLS is enabled by default. If you previously used `--no-tls`, remove it:

```bash
redisctl profile set my-db --type database \
  --host "hostname" \
  --port 12345 \
  --password "$PASSWORD"
```

### Wrong Database Number

**Symptom:** Connected successfully but data is missing or unexpected.

**Cause:** The profile targets a different Redis database number than expected.

**Fix:**

```bash
redisctl profile set my-db --type database \
  --host "hostname" \
  --port 12345 \
  --db 0
```
