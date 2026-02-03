# Presentation

Presentation materials for introducing redisctl to your team.

## Slide Deck

<a href="slides.html" target="_blank" class="md-button md-button--primary">
  :material-presentation-play: View Slides
</a>

The presentation covers:

- The problem: No CLI tooling for Redis Cloud/Enterprise
- The solution: redisctl's four-layer architecture
- Demo: Profile setup, Cloud operations, Enterprise operations
- Raw API access for any endpoint
- Support package automation
- Output formats and JMESPath queries
- Async operation handling
- Use cases by persona
- Installation and getting started

## Using the Slides

| Key | Action |
|-----|--------|
| Arrow keys | Navigate slides |
| `Space` | Next slide |
| `Esc` | Overview mode |
| `S` | Speaker notes |
| `F` | Fullscreen |
| `?` | Help |

## Sharing

Direct link to slides: `https://redis-field-engineering.github.io/redisctl-docs/presentation/slides.html`

## Customizing

The slides use [reveal.js](https://revealjs.com/). To customize:

1. Edit `docs/presentation/slides.html`
2. Slides are in `<section>` tags
3. Use standard HTML and reveal.js features

## Quick Pitch

Need a one-liner?

> **redisctl** is the first CLI for Redis Cloud and Enterprise. Type-safe API clients, async operation handling, support package automation, and structured output for scripting.

## Demo Script

### 1. Setup (30 seconds)

```bash
# Show installation
brew install redis-developer/homebrew-tap/redisctl

# Configure profile
redisctl profile set demo \
  --enterprise-url "https://cluster:9443" \
  --enterprise-user "admin@cluster.local" \
  --enterprise-password "$PASSWORD"
```

### 2. Basic Commands (1 minute)

```bash
# Cluster info
redisctl enterprise cluster get

# List databases
redisctl enterprise database list

# JSON output with filtering
redisctl enterprise database list -o json -q '[].{name: name, memory: memory_size}'
```

### 3. Support Package (30 seconds)

```bash
# The killer feature
redisctl enterprise support-package cluster --optimize --upload
```

### 4. Raw API Access (30 seconds)

```bash
# Any endpoint
redisctl api enterprise get /v1/nodes

# Compare to curl
curl -k -u "user:pass" https://cluster:9443/v1/nodes | jq
```

Total demo time: ~3 minutes
