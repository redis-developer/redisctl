//! # Approach B: Operation-Centric
//!
//! Each operation is a struct that you configure and execute. This gives more
//! flexibility for complex operations.
//!
//! ## Usage Pattern
//!
//! ```rust,ignore
//! // CLI usage - with progress callback
//! let result = CloudOperations::create_database(&client)
//!     .subscription_id(123)
//!     .name("my-db")
//!     .memory_gb(1.0)
//!     .wait(true)
//!     .on_progress(|status| {
//!         spinner.set_message(status);  // CLI updates spinner
//!     })
//!     .execute()
//!     .await?;
//!
//! // MCP usage - same operation, no progress callback
//! let result = CloudOperations::create_database(&client)
//!     .subscription_id(123)
//!     .name("my-db")
//!     .memory_gb(1.0)
//!     .wait(true)
//!     .execute()
//!     .await?;
//! ```
//!
//! ## Pros
//! - Very flexible - easy to add new params without breaking changes
//! - Callbacks allow presentation layer to hook in (progress bars, etc.)
//! - Each operation is self-documenting
//! - Easy to test individual operations
//! - Optional params with sensible defaults
//!
//! ## Cons
//! - More boilerplate than Approach A
//! - Might be overkill for simple operations
//! - Callback types can get complex
//!
//! ## What Would Live Here
//!
//! ```rust,ignore
//! pub struct CloudOperations;
//!
//! impl CloudOperations {
//!     pub fn list_databases(client: &CloudClient) -> ListDatabasesOp;
//!     pub fn create_database(client: &CloudClient) -> CreateDatabaseOp;
//!     pub fn delete_database(client: &CloudClient) -> DeleteDatabaseOp;
//! }
//!
//! pub struct CreateDatabaseOp<'a> {
//!     client: &'a CloudClient,
//!     subscription_id: Option<u64>,
//!     name: Option<String>,
//!     memory_gb: Option<f64>,
//!     wait: bool,
//!     on_progress: Option<Box<dyn Fn(&str)>>,
//! }
//!
//! impl CreateDatabaseOp<'_> {
//!     pub fn subscription_id(self, id: u64) -> Self;
//!     pub fn name(self, name: impl Into<String>) -> Self;
//!     pub fn memory_gb(self, gb: f64) -> Self;
//!     pub fn wait(self, wait: bool) -> Self;
//!     pub fn on_progress<F: Fn(&str)>(self, f: F) -> Self;
//!     pub async fn execute(self) -> Result<CreateResult>;
//! }
//! ```
//!
//! ## Progress Callback Pattern
//!
//! The key insight here is that CLI needs to update spinners/progress bars,
//! but MCP doesn't care. The callback is optional:
//!
//! ```rust,ignore
//! // Inside execute():
//! fn report_progress(&self, msg: &str) {
//!     if let Some(cb) = &self.on_progress {
//!         cb(msg);
//!     }
//! }
//!
//! // Called during polling:
//! self.report_progress(&format!("Status: {}", task.status));
//! ```

// Placeholder to make the module compile
pub struct CloudOperations;
