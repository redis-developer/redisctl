# Subscriptions

Manage Redis Cloud subscriptions.

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all subscriptions |
| `get` | Get subscription details |
| `create` | Create a new subscription |
| `update` | Update subscription settings |
| `delete` | Delete a subscription |
| `get-cidr-allowlist` | Get CIDR allowlist |
| `update-cidr-allowlist` | Update CIDR allowlist |
| `get-maintenance-windows` | Get maintenance windows |
| `update-maintenance-windows` | Update maintenance windows |
| `list-aa-regions` | List Active-Active regions |
| `add-aa-region` | Add Active-Active region |
| `delete-aa-regions` | Delete Active-Active regions |

## List Subscriptions

```bash
redisctl cloud subscription list
```

### Examples

```bash
# List all subscriptions
redisctl cloud subscription list

# Get just names
redisctl cloud subscription list -o json -q '[].name'

# Filter active only
redisctl cloud subscription list -o json -q '[?status == `active`]'
```

## Get Subscription

```bash
redisctl cloud subscription get <subscription-id>
```

## Create Subscription

Create a new Pro subscription with first-class parameters for common options.

```bash
redisctl cloud subscription create \
  --name "my-subscription" \
  --payment-method credit-card \
  --payment-method-id 12345 \
  --data @subscription.json \
  --wait
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--name` | Subscription name | - |
| `--dry-run` | Validate without creating | false |
| `--deployment-type` | single-region or active-active | - |
| `--payment-method` | credit-card or marketplace | credit-card |
| `--payment-method-id` | Payment method ID (required for credit-card) | - |
| `--memory-storage` | ram or ram-and-flash | ram |
| `--persistent-storage-encryption` | true or false | false |
| `--data` | JSON file or string with full request body | - |

!!! note
    The `--data` option is required and must include `cloudProviders` and `databases` arrays. First-class parameters override values in the JSON.

### Examples

```bash
# Create with JSON file
redisctl cloud subscription create \
  --name "prod-subscription" \
  --data @subscription.json \
  --wait

# Dry run to validate
redisctl cloud subscription create \
  --name "test-subscription" \
  --dry-run \
  --data @subscription.json
```

## Update Subscription

Update subscription settings using first-class parameters.

```bash
redisctl cloud subscription update <subscription-id> \
  --name "new-name" \
  --wait
```

### Options

| Option | Description |
|--------|-------------|
| `--name` | New subscription name |
| `--payment-method` | credit-card or marketplace |
| `--payment-method-id` | Payment method ID |
| `--data` | JSON with additional fields |

### Examples

```bash
# Rename subscription
redisctl cloud subscription update 12345 --name "production-redis"

# Change payment method
redisctl cloud subscription update 12345 \
  --payment-method credit-card \
  --payment-method-id 67890
```

## Delete Subscription

```bash
redisctl cloud subscription delete <subscription-id> --wait
```

!!! warning
    Deleting a subscription removes all databases within it.

## CIDR Allowlist

### Get CIDR Allowlist

```bash
redisctl cloud subscription get-cidr-allowlist <subscription-id>
```

### Update CIDR Allowlist

Update allowed CIDR blocks using first-class parameters.

```bash
redisctl cloud subscription update-cidr-allowlist <subscription-id> \
  --cidr "10.0.0.0/24" \
  --cidr "192.168.1.0/24"
```

#### Options

| Option | Description |
|--------|-------------|
| `--cidr` | CIDR block to allow (repeatable) |
| `--security-group` | AWS security group ID (repeatable) |
| `--data` | JSON with full request body |

#### Examples

```bash
# Allow multiple CIDR blocks
redisctl cloud subscription update-cidr-allowlist 12345 \
  --cidr "10.0.0.0/16" \
  --cidr "172.16.0.0/12"

# Allow AWS security groups
redisctl cloud subscription update-cidr-allowlist 12345 \
  --security-group "sg-12345678" \
  --security-group "sg-87654321"

# Use JSON for complex configurations
redisctl cloud subscription update-cidr-allowlist 12345 \
  --data '{"cidrIps": [{"cidr": "10.0.0.0/24", "description": "Office"}]}'
```

## Maintenance Windows

### Get Maintenance Windows

```bash
redisctl cloud subscription get-maintenance-windows <subscription-id>
```

### Update Maintenance Windows

Configure maintenance windows using first-class parameters.

```bash
redisctl cloud subscription update-maintenance-windows <subscription-id> \
  --mode manual \
  --window "Monday:03-07"
```

#### Options

| Option | Description |
|--------|-------------|
| `--mode` | Maintenance mode (automatic or manual) |
| `--window` | Maintenance window in DAY:HH-HH format (repeatable) |
| `--data` | JSON with full request body |

#### Examples

```bash
# Set automatic maintenance
redisctl cloud subscription update-maintenance-windows 12345 \
  --mode automatic

# Set manual maintenance with specific windows
redisctl cloud subscription update-maintenance-windows 12345 \
  --mode manual \
  --window "Sunday:02-06" \
  --window "Wednesday:02-06"
```

## Active-Active Regions

### List Active-Active Regions

```bash
redisctl cloud subscription list-aa-regions <subscription-id>
```

### Add Active-Active Region

Add a new region to an Active-Active subscription.

```bash
redisctl cloud subscription add-aa-region <subscription-id> \
  --region "us-west-2" \
  --deployment-cidr "10.1.0.0/24"
```

#### Options

| Option | Description |
|--------|-------------|
| `--region` | Cloud region to add (required) |
| `--deployment-cidr` | CIDR for the deployment |
| `--vpc-id` | VPC ID for the region |
| `--resp-version` | RESP protocol version |
| `--dry-run` | Validate without creating |
| `--data` | JSON with additional fields |

#### Examples

```bash
# Add region with CIDR
redisctl cloud subscription add-aa-region 12345 \
  --region "eu-west-1" \
  --deployment-cidr "10.2.0.0/24" \
  --wait

# Dry run to validate
redisctl cloud subscription add-aa-region 12345 \
  --region "ap-southeast-1" \
  --deployment-cidr "10.3.0.0/24" \
  --dry-run
```

### Delete Active-Active Regions

Remove regions from an Active-Active subscription.

```bash
redisctl cloud subscription delete-aa-regions <subscription-id> \
  --region "us-west-2" \
  --force
```

#### Options

| Option | Description |
|--------|-------------|
| `--region` | Region to delete (repeatable, at least one required) |
| `--dry-run` | Validate without deleting |
| `--data` | JSON with additional fields |
| `--force` | Skip confirmation prompt |

#### Examples

```bash
# Delete single region
redisctl cloud subscription delete-aa-regions 12345 \
  --region "eu-west-1" \
  --force \
  --wait

# Delete multiple regions
redisctl cloud subscription delete-aa-regions 12345 \
  --region "ap-southeast-1" \
  --region "ap-northeast-1" \
  --force
```
