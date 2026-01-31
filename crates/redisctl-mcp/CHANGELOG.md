# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/redis-developer/redisctl/compare/redisctl-mcp-v0.2.0...redisctl-mcp-v0.3.0) - 2026-01-31

### Added

- add MCP resources and prompts to redisctl-mcp ([#619](https://github.com/redis-developer/redisctl/pull/619))
- *(mcp)* add read-only tool filter using CapabilityFilter ([#618](https://github.com/redis-developer/redisctl/pull/618))
- *(mcp)* add historical stats, Cloud logs, debug info, and modules tools ([#617](https://github.com/redis-developer/redisctl/pull/617))
- *(mcp)* add Enterprise logs and aggregate stats tools ([#616](https://github.com/redis-developer/redisctl/pull/616))
- *(mcp)* add Enterprise license tools ([#615](https://github.com/redis-developer/redisctl/pull/615))
- *(mcp)* add mock testing support for cloud and enterprise tools ([#611](https://github.com/redis-developer/redisctl/pull/611))
- *(mcp)* add profile management tools ([#609](https://github.com/redis-developer/redisctl/pull/609))

### Fixed

- *(mcp)* normalize 'default' profile to use configured default ([#608](https://github.com/redis-developer/redisctl/pull/608))

### Other

- upgrade tower-mcp to 0.2.3 and use from_serialize() ([#607](https://github.com/redis-developer/redisctl/pull/607))

## [0.1.2](https://github.com/redis-developer/redisctl/compare/redisctl-mcp-v0.1.1...redisctl-mcp-v0.1.2) - 2026-01-23

### Added

- *(mcp)* add --database-url CLI option for direct Redis connections ([#574](https://github.com/redis-developer/redisctl/pull/574))
- *(mcp)* add database tools for direct Redis connections ([#572](https://github.com/redis-developer/redisctl/pull/572))

### Other

- add Python bindings documentation and update CHANGELOGs ([#581](https://github.com/redis-developer/redisctl/pull/581))

## [0.1.1](https://github.com/redis-developer/redisctl/compare/redisctl-mcp-v0.1.0...redisctl-mcp-v0.1.1) - 2026-01-14

### Added

- *(mcp)* add Private Link, Transit Gateway, BDB Groups, OCSP, and Suffixes tools ([#561](https://github.com/redis-developer/redisctl/pull/561))
- *(mcp)* add VPC Peering, Cloud Accounts, and CRDB Tasks tools ([#560](https://github.com/redis-developer/redisctl/pull/560))
- *(mcp)* add 25 new tools for enterprise and cloud operations ([#559](https://github.com/redis-developer/redisctl/pull/559))

### Other

- *(mcp)* add readme to redisctl-mcp crate ([#534](https://github.com/redis-developer/redisctl/pull/534))
- *(redisctl)* release v0.7.4 ([#517](https://github.com/redis-developer/redisctl/pull/517))

## [0.1.0](https://github.com/redis-developer/redisctl/releases/tag/redisctl-mcp-v0.1.0) - 2026-01-12

### Added

- add MCP server for AI integration ([#531](https://github.com/redis-developer/redisctl/pull/531))
