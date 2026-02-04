//! # redisctl-core - SPIKE/EXPERIMENTAL
//!
//! This crate explores different patterns for a shared "engine" layer that both
//! CLI and MCP can consume. The goal is to find the right abstraction level that:
//!
//! 1. Reduces duplication between CLI and MCP
//! 2. Provides a clean, testable core
//! 3. Doesn't over-abstract what the client libraries already provide
//!
//! ## The Problem
//!
//! Currently both CLI and MCP:
//! - Call redis-cloud/redis-enterprise clients directly
//! - Duplicate async task polling logic
//! - Duplicate input validation
//! - Handle progress/status differently (CLI: spinners, MCP: nothing)
//!
//! ## What Would Live Here
//!
//! - **Config** - could absorb redisctl-config
//! - **Operations** - higher-level ops like "create and wait"
//! - **Workflows** - multi-step operations (subscription setup, cluster init)
//! - **Shared types** - common result types, error types
//! - **Tracing/metrics** - unified observability
//!
//! ## Patterns Explored
//!
//! ### [Approach A](approach_a): Direct Struct Wrapper
//! Simple structs that wrap clients and provide higher-level operations.
//! - Pros: Simple, familiar, low ceremony
//! - Cons: Doesn't solve progress callback cleanly
//!
//! ### [Approach B](approach_b): Operation-Centric
//! Each operation is a struct you configure and execute.
//! - Pros: Very flexible, optional callbacks, self-documenting
//! - Cons: More boilerplate
//!
//! ### [Approach C](approach_c): Trait-Based with Hooks
//! Core engine is generic over a "hooks" trait for presentation concerns.
//! - Pros: Clean separation, type-safe, composable
//! - Cons: Complex type signatures, maybe overengineered
//!
//! ## Recommendation
//!
//! Start with **Approach A** for simplicity, with elements of **B** for operations
//! that need progress callbacks. We can evolve toward **C** if we find ourselves
//! duplicating hook patterns across many operations.
//!
//! ## Crate Structure (Proposed)
//!
//! ```text
//! redisctl-core/
//! ├── src/
//! │   ├── lib.rs
//! │   ├── config.rs       # absorb redisctl-config
//! │   ├── cloud/
//! │   │   ├── mod.rs
//! │   │   ├── engine.rs   # CloudEngine
//! │   │   ├── types.rs    # DatabaseInfo, SubscriptionInfo, etc.
//! │   │   └── operations/ # complex ops if using Approach B
//! │   ├── enterprise/
//! │   │   ├── mod.rs
//! │   │   ├── engine.rs   # EnterpriseEngine
//! │   │   └── types.rs
//! │   ├── error.rs        # unified error types
//! │   └── tracing.rs      # shared tracing setup
//! ```

pub mod approach_a;
pub mod approach_b;
pub mod approach_c;
