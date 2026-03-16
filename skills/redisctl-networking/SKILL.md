---
name: redisctl-networking
description: Configure network connectivity for Redis Cloud deployments via the redisctl CLI. Use when setting up VPC peering, Transit Gateway, Private Service Connect, or PrivateLink.
---

## Overview

Set up private network connectivity between your infrastructure and Redis Cloud deployments. Supports VPC Peering, Transit Gateway, Private Service Connect (GCP), and AWS PrivateLink.

## VPC Peering

```bash
# List VPC peering connections
redisctl cloud connectivity vpc-peering list --subscription-id 12345

# Get peering details
redisctl cloud connectivity vpc-peering get --subscription-id 12345 --id 1

# Create a peering connection
redisctl cloud connectivity vpc-peering create \
  --subscription-id 12345 \
  --data '{
    "region": "us-east-1",
    "aws_account_id": "123456789012",
    "vpc_id": "vpc-abc123",
    "vpc_cidr": "10.0.0.0/16"
  }'

# Delete a peering connection
redisctl cloud connectivity vpc-peering delete --subscription-id 12345 --id 1
```

## Transit Gateway (AWS)

```bash
# List TGW attachments
redisctl cloud connectivity tgw list --subscription-id 12345

# Create a TGW attachment
redisctl cloud connectivity tgw create \
  --subscription-id 12345 \
  --data '{...}'

# Accept a TGW invitation
redisctl cloud connectivity tgw accept --subscription-id 12345 --id 1
```

## Private Service Connect (GCP)

```bash
# List PSC endpoints
redisctl cloud connectivity psc list --subscription-id 12345

# Create a PSC endpoint
redisctl cloud connectivity psc create \
  --subscription-id 12345 \
  --data '{...}'

# Get connection scripts
redisctl cloud connectivity psc scripts --subscription-id 12345 --id 1
```

## AWS PrivateLink

```bash
# List PrivateLink endpoints
redisctl cloud connectivity privatelink list --subscription-id 12345

# Create a PrivateLink endpoint
redisctl cloud connectivity privatelink create \
  --subscription-id 12345 \
  --data '{...}'
```

## Cloud Provider Accounts

Required for some networking operations:

```bash
# List provider accounts
redisctl cloud provider-account list

# Create a provider account
redisctl cloud provider-account create --data '{...}'
```

## Tips

- VPC peering requires matching CIDR ranges -- ensure no overlap with Redis Cloud VPC
- Transit Gateway and PrivateLink are AWS-only features
- Private Service Connect is GCP-only
- Most connectivity operations are async -- use `redisctl cloud task wait` after creation
- Active-Active subscriptions have separate connectivity endpoints per region
