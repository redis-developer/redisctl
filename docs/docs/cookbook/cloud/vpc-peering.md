# Set Up VPC Peering

Connect your VPC to Redis Cloud for private networking.

## Prerequisites

- Redis Cloud subscription with VPC enabled
- AWS account ID and VPC details (or GCP project and network)
- IAM permissions to accept peering

## Step 1: Get Subscription VPC Info

```bash
redisctl cloud subscription get 123456 -o json -q '{
  vpcId: cloudDetails[0].vpcId,
  vpcCidr: cloudDetails[0].vpcCidr
}'
```

## Step 2: Create Peering Request

### AWS VPC Peering

```bash
redisctl cloud connectivity vpc-peering create \
  --subscription 123456 \
  --region us-east-1 \
  --aws-account-id 123456789012 \
  --vpc-id vpc-abc123def \
  --vpc-cidr 10.0.0.0/16 \
  --wait
```

### GCP VPC Peering

```bash
redisctl cloud connectivity vpc-peering create \
  --subscription 123456 \
  --gcp-project-id my-gcp-project \
  --gcp-network-name default \
  --wait
```

### With Multiple CIDR Blocks

```bash
redisctl cloud connectivity vpc-peering create \
  --subscription 123456 \
  --region us-east-1 \
  --aws-account-id 123456789012 \
  --vpc-id vpc-abc123def \
  --vpc-cidr 10.0.0.0/16 \
  --vpc-cidr 10.1.0.0/16 \
  --wait
```

## Step 3: Accept in AWS Console

The peering request appears in your AWS VPC console. Accept it:

1. Go to **VPC > Peering Connections**
2. Find the pending request from Redis
3. Click **Actions > Accept Request**

## Step 4: Update Route Tables

Add a route to your VPC route table:

| Destination | Target |
|-------------|--------|
| Redis VPC CIDR | Peering Connection |

## Step 5: Verify Connectivity

```bash
# Get database private endpoint
redisctl cloud database get 123456 789 -o json -q 'privateEndpoint'

# Test from within your VPC
redis-cli -h <private-endpoint> -p 12345 PING
```

## Complete Script

```bash
#!/bin/bash
set -e

SUB_ID="${1:?Usage: $0 <subscription-id> <aws-account-id> <vpc-id> <vpc-cidr>}"
AWS_ACCOUNT="${2:?}"
VPC_ID="${3:?}"
VPC_CIDR="${4:?}"

echo "Creating VPC peering..."

PEERING=$(redisctl cloud connectivity vpc-peering create \
  --subscription "$SUB_ID" \
  --region us-east-1 \
  --aws-account-id "$AWS_ACCOUNT" \
  --vpc-id "$VPC_ID" \
  --vpc-cidr "$VPC_CIDR" \
  --wait \
  -o json)

echo "Peering created!"
echo "$PEERING" | jq '{id: .id, status: .status}'

echo ""
echo "Next steps:"
echo "1. Accept the peering in AWS VPC console"
echo "2. Update your route tables"
echo "3. Test connectivity from your VPC"
```

## Update CIDR Blocks

To add additional CIDR blocks to an existing peering:

```bash
redisctl cloud connectivity vpc-peering update \
  --subscription 123456 \
  --peering-id 789 \
  --vpc-cidr 10.0.0.0/16 \
  --vpc-cidr 10.1.0.0/16 \
  --vpc-cidr 10.2.0.0/16 \
  --wait
```

## Active-Active VPC Peering

For Active-Active subscriptions with multiple regions:

```bash
# Create Active-Active VPC peering
redisctl cloud connectivity vpc-peering create-aa \
  --subscription 123456 \
  --source-region us-east-1 \
  --destination-region us-west-2 \
  --aws-account-id 123456789012 \
  --vpc-id vpc-abc123 \
  --vpc-cidr 10.0.0.0/16 \
  --wait
```

## Troubleshooting

### Peering Stuck in Pending

- Check AWS console for the peering request
- Verify IAM permissions to accept peerings
- Ensure CIDR ranges don't overlap

### Connection Refused

- Verify route table has correct route
- Check security group allows Redis port
- Ensure using private endpoint, not public

### Using JSON for Advanced Options

If you need options not available as CLI flags, use `--data`:

```bash
redisctl cloud connectivity vpc-peering create \
  --subscription 123456 \
  --data '{
    "region": "us-east-1",
    "awsAccountId": "123456789012",
    "vpcId": "vpc-abc123",
    "vpcCidrs": ["10.0.0.0/16"],
    "advancedOption": "value"
  }' \
  --wait
```

## Related

- [Networking Commands](../../cloud/commands/networking.md)
- [Database Commands](../../cloud/commands/databases.md)
