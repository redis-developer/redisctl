//! Redis Cloud API tools

use std::sync::Arc;

use redis_cloud::flexible::{DatabaseHandler, SubscriptionHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::{CallToolResult, Tool, ToolBuilder, ToolError};

use crate::state::AppState;

/// Input for listing subscriptions
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSubscriptionsInput {}

/// Build the list_subscriptions tool
pub fn list_subscriptions(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_subscriptions")
        .description("List all Redis Cloud subscriptions accessible with the current credentials. Returns JSON with subscription details.")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, _input: ListSubscriptionsInput| async move {
            let client = state
                .cloud_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

            let handler = SubscriptionHandler::new(client);
            let account_subs = handler
                .get_all_subscriptions()
                .await
                .map_err(|e| ToolError::new(format!("Failed to list subscriptions: {}", e)))?;

            let output = serde_json::to_string_pretty(&account_subs)
                .map_err(|e| ToolError::new(format!("Failed to serialize: {}", e)))?;

            Ok(CallToolResult::text(output))
        })
        .build()
        .expect("valid tool")
}

/// Input for getting a specific subscription
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSubscriptionInput {
    /// Subscription ID
    pub subscription_id: i32,
}

/// Build the get_subscription tool
pub fn get_subscription(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_subscription")
        .description("Get detailed information about a specific Redis Cloud subscription. Returns JSON with full subscription details.")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetSubscriptionInput| async move {
            let client = state
                .cloud_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

            let handler = SubscriptionHandler::new(client);
            let subscription = handler
                .get_subscription_by_id(input.subscription_id)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get subscription: {}", e)))?;

            let output = serde_json::to_string_pretty(&subscription)
                .map_err(|e| ToolError::new(format!("Failed to serialize: {}", e)))?;

            Ok(CallToolResult::text(output))
        })
        .build()
        .expect("valid tool")
}

/// Input for listing databases
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabasesInput {
    /// Subscription ID
    pub subscription_id: i32,
}

/// Build the list_databases tool
pub fn list_databases(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("list_databases")
        .description(
            "List all databases in a Redis Cloud subscription. Returns JSON with database details.",
        )
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: ListDatabasesInput| async move {
            let client = state
                .cloud_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

            let handler = DatabaseHandler::new(client);
            let databases = handler
                .get_subscription_databases(input.subscription_id, None, None)
                .await
                .map_err(|e| ToolError::new(format!("Failed to list databases: {}", e)))?;

            let output = serde_json::to_string_pretty(&databases)
                .map_err(|e| ToolError::new(format!("Failed to serialize: {}", e)))?;

            Ok(CallToolResult::text(output))
        })
        .build()
        .expect("valid tool")
}

/// Input for getting a specific database
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDatabaseInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Database ID
    pub database_id: i32,
}

/// Build the get_database tool
pub fn get_database(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_database")
        .description("Get detailed information about a specific Redis Cloud database. Returns JSON with full database configuration.")
        .read_only()
        .idempotent()
        .handler_with_state(state, |state, input: GetDatabaseInput| async move {
            let client = state
                .cloud_client()
                .await
                .map_err(|e| ToolError::new(format!("Failed to get Cloud client: {}", e)))?;

            let handler = DatabaseHandler::new(client);
            let database = handler
                .get_subscription_database_by_id(input.subscription_id, input.database_id)
                .await
                .map_err(|e| ToolError::new(format!("Failed to get database: {}", e)))?;

            let output = serde_json::to_string_pretty(&database)
                .map_err(|e| ToolError::new(format!("Failed to serialize: {}", e)))?;

            Ok(CallToolResult::text(output))
        })
        .build()
        .expect("valid tool")
}
