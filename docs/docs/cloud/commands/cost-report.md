# Cost Reports

Generate and download billing data in FOCUS format for cost analysis.

!!! info "Beta Feature"
    Cost reports are currently in beta. The API and output format may change.

## Overview

Cost reports provide detailed billing data in the [FOCUS (FinOps Open Cost & Usage Specification)](https://focus.finops.org/) format—an industry standard that normalizes billing data across cloud providers. This makes it easy to analyze Redis Cloud costs alongside other infrastructure spending.

**Common use cases:**

- Monthly cost tracking and budgeting
- Team-based cost allocation using tags
- Integration with FinOps platforms (CloudHealth, Apptio, Kubecost, etc.)
- Automated cost reporting via scripts and cron jobs
- Export to spreadsheets for finance teams

## Commands

| Command | Description |
|---------|-------------|
| `export` | Generate and download in one step (recommended) |
| `generate` | Generate a cost report for a date range |
| `download` | Download a completed cost report |

## Export a Cost Report (Recommended)

The simplest way to get a cost report - combines generate, wait, and download into one command:

```bash
redisctl cloud cost-report export \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --file january-costs.csv
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--start-date` | Start date in YYYY-MM-DD format | required |
| `--end-date` | End date in YYYY-MM-DD format (max 40 days from start) | required |
| `--format` | Report format: `csv` or `json` | `csv` |
| `--file`, `-f` | Output file path (defaults to stdout) | - |
| `--timeout` | Maximum time to wait in seconds | 300 |
| `--subscription` | Filter by subscription ID (repeatable) | - |
| `--database` | Filter by database ID (repeatable) | - |
| `--subscription-type` | Filter by type: `pro` or `essentials` | - |
| `--region` | Filter by cloud region (repeatable) | - |
| `--tag` | Filter by tag in `key:value` format (repeatable) | - |

### Examples

```bash
# Export monthly CSV report to file
redisctl cloud cost-report export \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --file january-costs.csv

# Export as JSON for programmatic processing
redisctl cloud cost-report export \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --format json \
  --file january-costs.json

# Export filtered by team tag
redisctl cloud cost-report export \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --tag team:platform \
  --file team-costs.csv

# Export Pro subscriptions only
redisctl cloud cost-report export \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --subscription-type pro \
  --file pro-costs.csv

# Export to stdout and pipe to analysis tools
redisctl cloud cost-report export \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --format json | jq 'sum([].BilledCost)'
```

## Generate a Cost Report

Generate a cost report for a specific date range. Reports are created asynchronously.

```bash
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31
```

### Required Options

| Option | Description |
|--------|-------------|
| `--start-date` | Start date in YYYY-MM-DD format |
| `--end-date` | End date in YYYY-MM-DD format (max 40 days from start) |

### Filter Options

| Option | Description |
|--------|-------------|
| `--subscription` | Filter by subscription ID (repeatable) |
| `--database` | Filter by database ID (repeatable) |
| `--subscription-type` | Filter by type: `pro` or `essentials` |
| `--region` | Filter by cloud region (repeatable) |
| `--tag` | Filter by tag in `key:value` format (repeatable) |

### Output Options

| Option | Description | Default |
|--------|-------------|---------|
| `--format` | Report format: `csv` or `json` | `csv` |
| `--wait` | Wait for report generation to complete | false |
| `--wait-timeout` | Maximum wait time in seconds | 300 |

### Examples

```bash
# Generate a monthly report
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --wait

# Generate report for Pro subscriptions only
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --subscription-type pro \
  --wait

# Filter by specific subscriptions
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --subscription 12345 \
  --subscription 67890 \
  --wait

# Filter by team tag
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --tag team:marketing \
  --tag environment:production \
  --wait

# Generate JSON format for programmatic processing
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --format json \
  --wait
```

### Getting the Cost Report ID

When the generation completes, the task response includes the `costReportId` needed for download:

```bash
# Generate and wait, then extract the report ID
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --wait \
  -o json -q 'response.resource.costReportId'
```

If you don't use `--wait`, check the task status:

```bash
# Check task status to get the costReportId
redisctl cloud task get <task-id> -o json -q 'response.resource.costReportId'
```

## Download a Cost Report

Download a generated cost report by its ID.

```bash
redisctl cloud cost-report download <cost-report-id>
```

### Options

| Option | Description |
|--------|-------------|
| `--file`, `-f` | Output file path (defaults to stdout) |

### Examples

```bash
# Download to stdout
redisctl cloud cost-report download cost-report-abc123

# Save to a file
redisctl cloud cost-report download cost-report-abc123 \
  --file january-2025-costs.csv

# Download and process with other tools
redisctl cloud cost-report download cost-report-abc123 | \
  csvcut -c BilledCost,ResourceName | \
  csvstat
```

## Understanding FOCUS Format

FOCUS (FinOps Open Cost & Usage Specification) is an open standard that normalizes billing data across cloud providers. This allows you to analyze Redis Cloud costs using the same tools and queries you use for AWS, Azure, or GCP.

### Key FOCUS Columns

| Column | Description | Example |
|--------|-------------|---------|
| `BilledCost` | The amount charged | `125.50` |
| `BillingPeriodStart` | Start of billing period | `2025-01-01T00:00:00Z` |
| `BillingPeriodEnd` | End of billing period | `2025-02-01T00:00:00Z` |
| `ChargeType` | Type of charge | `Usage`, `Fee` |
| `ResourceId` | Unique resource identifier | `12345` |
| `ResourceName` | Human-readable name | `production-cache` |
| `ResourceType` | Type of resource | `Database`, `Subscription` |
| `ServiceName` | Service providing the resource | `Redis Cloud` |
| `Region` | Cloud region | `us-east-1` |
| `ProviderName` | Cloud provider | `AWS`, `GCP`, `Azure` |
| `Tags` | Resource tags | `{"team": "platform"}` |

### Benefits of FOCUS Format

- **Standardized**: Same column names and formats as other cloud providers
- **Tool-compatible**: Works with Excel, Google Sheets, FinOps platforms, and BI tools
- **Comparable**: Easily compare Redis costs with other cloud spending
- **Automatable**: Consistent format enables reliable scripting

## Workflow: Monthly Cost Report

A typical workflow for monthly cost analysis:

```bash
# 1. Generate the report for last month
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --wait \
  -o json > task-response.json

# 2. Extract the cost report ID
REPORT_ID=$(jq -r '.response.resource.costReportId' task-response.json)

# 3. Download the report
redisctl cloud cost-report download "$REPORT_ID" \
  --file costs-january-2025.csv

# 4. View summary (using csvkit)
csvstat costs-january-2025.csv --columns BilledCost
```

## Workflow: Team Cost Allocation

Generate reports filtered by team tags for chargeback:

```bash
# Generate reports for each team
for team in platform backend frontend; do
  redisctl cloud cost-report generate \
    --start-date 2025-01-01 \
    --end-date 2025-01-31 \
    --tag team:$team \
    --wait \
    -o json | jq -r '.response.resource.costReportId' > /tmp/${team}-report-id.txt
    
  REPORT_ID=$(cat /tmp/${team}-report-id.txt)
  redisctl cloud cost-report download "$REPORT_ID" \
    --file costs-${team}-january-2025.csv
done
```

## Limitations

- **Maximum date range**: 40 days per report
- **Processing time**: Large reports may take several minutes to generate
- **Rate limits**: API rate limits apply; use `--wait` to handle polling automatically

## Troubleshooting

### Report generation times out

Increase the timeout or check the task manually:

```bash
# Increase timeout
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --wait \
  --wait-timeout 600

# Or check task status manually
redisctl cloud task list -o json -q '[?status != `completed`]'
```

### No data in report

- Verify the date range includes dates with actual usage
- Check that filters (subscription, database, tags) match existing resources
- Ensure the account has billing data for the requested period

### Rate limit errors

The CLI automatically handles rate limits with retry and backoff. If you still encounter issues:

```bash
# Reduce polling frequency
redisctl cloud cost-report generate \
  --start-date 2025-01-01 \
  --end-date 2025-01-31 \
  --wait \
  --wait-interval 10
```

## Related

- [Tasks](tasks.md) - Monitor async operations
- [JMESPath Queries](../../common/jmespath.md) - Filter and transform output
- [Cost Report Cookbook](../../cookbook/cloud/cost-reports.md) - Step-by-step guides
