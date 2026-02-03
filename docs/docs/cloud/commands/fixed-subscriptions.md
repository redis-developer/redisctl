# Essentials Subscription Commands

Manage Redis Cloud Essentials (fixed) subscriptions.

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all Essentials subscriptions |
| `get` | Get subscription details |
| `create` | Create a new subscription |
| `update` | Update subscription settings |
| `delete` | Delete a subscription |
| `list-plans` | List available plans |

## List Subscriptions

```bash
redisctl cloud fixed-subscription list
```

### Examples

```bash
# List all Essentials subscriptions
redisctl cloud fixed-subscription list

# As JSON
redisctl cloud fixed-subscription list -o json

# Get just IDs and names
redisctl cloud fixed-subscription list -o json -q '[].{id: id, name: name}'
```

## Get Subscription

```bash
redisctl cloud fixed-subscription get <subscription-id>
```

### Examples

```bash
# Full details
redisctl cloud fixed-subscription get 123456

# Just status
redisctl cloud fixed-subscription get 123456 -o json -q 'status'
```

## List Plans

List available Essentials plans with pricing.

```bash
redisctl cloud fixed-subscription list-plans
```

### Options

| Option | Description |
|--------|-------------|
| `--provider` | Filter by cloud provider (AWS, GCP, Azure) |

### Examples

```bash
# List all plans
redisctl cloud fixed-subscription list-plans

# Filter by provider
redisctl cloud fixed-subscription list-plans --provider AWS

# Show plan details
redisctl cloud fixed-subscription list-plans -o json -q '[].{
  id: id,
  name: name,
  size: size,
  price: price
}'
```

## Create Subscription

Create a new Essentials subscription with first-class parameters.

```bash
redisctl cloud fixed-subscription create \
  --name my-subscription \
  --plan-id 12345 \
  --wait
```

### Options

| Option | Description |
|--------|-------------|
| `--name` | Subscription name (required) |
| `--plan-id` | Plan ID from list-plans (required) |
| `--payment-method` | Payment method (credit-card or marketplace) |
| `--payment-method-id` | Payment method ID (required for credit-card) |
| `--data` | JSON with additional fields |

### Examples

```bash
# Create with plan ID
redisctl cloud fixed-subscription create \
  --name my-cache \
  --plan-id 12345 \
  --wait

# With payment method
redisctl cloud fixed-subscription create \
  --name prod-cache \
  --plan-id 12345 \
  --payment-method credit-card \
  --payment-method-id 67890

# Use JSON for full control
redisctl cloud fixed-subscription create \
  --data '{"name": "my-cache", "planId": 12345}'
```

## Update Subscription

Update subscription settings using first-class parameters.

```bash
redisctl cloud fixed-subscription update <subscription-id> \
  --name new-name \
  --wait
```

### Options

| Option | Description |
|--------|-------------|
| `--name` | New subscription name |
| `--plan-id` | New plan ID |
| `--payment-method` | Payment method (credit-card or marketplace) |
| `--payment-method-id` | Payment method ID |
| `--data` | JSON with additional fields |

### Examples

```bash
# Rename subscription
redisctl cloud fixed-subscription update 123456 --name new-name

# Change plan
redisctl cloud fixed-subscription update 123456 --plan-id 67890 --wait

# Change payment method
redisctl cloud fixed-subscription update 123456 \
  --payment-method credit-card \
  --payment-method-id 11111

# Update using JSON
redisctl cloud fixed-subscription update 123456 \
  --data '{"name": "new-name"}'
```

## Delete Subscription

```bash
redisctl cloud fixed-subscription delete <subscription-id> --wait
```

!!! warning
    Deleting a subscription removes all databases within it. Add `--force` to skip confirmation.

## Related Commands

- [Essentials Databases](fixed-databases.md) - Manage databases
- [Pro Subscriptions](subscriptions.md) - Manage Pro subscriptions
- [Tasks](tasks.md) - Monitor async operations
