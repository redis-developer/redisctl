//! Enterprise-specific workflows and helpers
//!
//! This module provides higher-level operations that compose Layer 1 calls
//! for Redis Enterprise.
//!
//! ## Overview
//!
//! Enterprise API has some async operations that return an `Action` which must be
//! polled for completion. This module provides:
//!
//! - `poll_action` - Generic action polling with progress callbacks
//! - `upgrade_database_and_wait` - Upgrade a database and wait for completion
//!
//! ## Example
//!
//! ```rust,ignore
//! use redisctl_core::enterprise::{poll_action, EnterpriseProgressEvent};
//! use std::time::Duration;
//!
//! let action = poll_action(
//!     &client,
//!     "action-uid",
//!     Duration::from_secs(600),
//!     Duration::from_secs(5),
//!     Some(Box::new(|event| {
//!         if let EnterpriseProgressEvent::Polling { progress, .. } = event {
//!             println!("Progress: {:?}%", progress);
//!         }
//!     })),
//! ).await?;
//! ```

pub mod progress;
pub mod workflows;

// Re-export key types for convenience
pub use progress::{EnterpriseProgressCallback, EnterpriseProgressEvent, poll_action};
pub use workflows::{
    DEFAULT_INTERVAL, DEFAULT_TIMEOUT, backup_database_and_wait, import_database_and_wait,
    upgrade_database_and_wait, upgrade_module_and_wait,
};
