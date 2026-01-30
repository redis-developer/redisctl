//! Cloud tools implementation
//!
//! Wraps Redis Cloud API client operations for MCP tool invocation.

use redis_cloud::fixed::subscriptions::FixedSubscriptionCreateRequest;
use redis_cloud::{
    AccountHandler, CloudAccountHandler, CloudClient, DatabaseHandler, FixedDatabaseHandler,
    FixedSubscriptionHandler, PrivateLinkHandler, SubscriptionHandler, TaskHandler,
    TransitGatewayHandler, VpcPeeringHandler,
};
use redisctl_config::Config;
use rmcp::{ErrorData as RmcpError, model::*};
use serde_json::Value;
use tracing::debug;

/// Cloud tools wrapper
#[derive(Clone)]
pub struct CloudTools {
    client: CloudClient,
}

impl CloudTools {
    /// Create new Cloud tools instance
    pub fn new(profile: Option<&str>) -> anyhow::Result<Self> {
        let config = Config::load()?;

        // Resolve profile name: explicit > default > error
        let profile_name = match profile {
            Some(name) => name.to_string(),
            None => config.resolve_cloud_profile(None)?,
        };

        debug!(profile = %profile_name, "Loading Cloud client from profile");

        let profile_config = config
            .profiles
            .get(&profile_name)
            .ok_or_else(|| anyhow::anyhow!("Cloud profile '{}' not found", profile_name))?;

        let (api_key, api_secret, api_url) = profile_config
            .cloud_credentials()
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' is not a Cloud profile", profile_name))?;

        let client = CloudClient::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .base_url(api_url.to_string())
            .build()?;

        Ok(Self { client })
    }

    fn to_result(&self, value: serde_json::Value) -> Result<CallToolResult, RmcpError> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
        )]))
    }

    fn to_error(&self, err: impl std::fmt::Display) -> RmcpError {
        RmcpError::internal_error(err.to_string(), None)
    }

    // =========================================================================
    // Account Operations
    // =========================================================================

    /// Get account information
    pub async fn get_account(&self) -> Result<CallToolResult, RmcpError> {
        let handler = AccountHandler::new(self.client.clone());
        let account = handler
            .get_current_account()
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(account).map_err(|e| self.to_error(e))?)
    }

    /// Get payment methods
    pub async fn get_payment_methods(&self) -> Result<CallToolResult, RmcpError> {
        let handler = AccountHandler::new(self.client.clone());
        let methods = handler
            .get_account_payment_methods()
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(methods).map_err(|e| self.to_error(e))?)
    }

    /// Get supported database modules
    pub async fn get_database_modules(&self) -> Result<CallToolResult, RmcpError> {
        let handler = AccountHandler::new(self.client.clone());
        let modules = handler
            .get_supported_database_modules()
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(modules).map_err(|e| self.to_error(e))?)
    }

    /// Get supported regions
    pub async fn get_regions(&self, provider: Option<&str>) -> Result<CallToolResult, RmcpError> {
        let handler = AccountHandler::new(self.client.clone());
        let regions = handler
            .get_supported_regions(provider.map(String::from))
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(regions).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // Pro Subscription Operations
    // =========================================================================

    /// List all Pro subscriptions
    pub async fn list_subscriptions(&self) -> Result<CallToolResult, RmcpError> {
        let handler = SubscriptionHandler::new(self.client.clone());
        let subs = handler
            .get_all_subscriptions()
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(subs).map_err(|e| self.to_error(e))?)
    }

    /// Get a specific Pro subscription
    pub async fn get_subscription(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = SubscriptionHandler::new(self.client.clone());
        let sub = handler
            .get_subscription_by_id(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(sub).map_err(|e| self.to_error(e))?)
    }

    /// Create a Pro subscription (accepts JSON payload)
    pub async fn create_subscription(&self, request: Value) -> Result<CallToolResult, RmcpError> {
        let result = self
            .client
            .post_raw("/subscriptions", request)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(result)
    }

    /// Delete a Pro subscription
    pub async fn delete_subscription(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = SubscriptionHandler::new(self.client.clone());
        let result = handler
            .delete_subscription_by_id(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // Essentials (Fixed) Subscription Operations
    // =========================================================================

    /// List all Essentials subscriptions
    pub async fn list_essentials_subscriptions(&self) -> Result<CallToolResult, RmcpError> {
        let handler = FixedSubscriptionHandler::new(self.client.clone());
        let subs = handler.list().await.map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(subs).map_err(|e| self.to_error(e))?)
    }

    /// Get a specific Essentials subscription
    pub async fn get_essentials_subscription(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedSubscriptionHandler::new(self.client.clone());
        let sub = handler
            .get_by_id(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(sub).map_err(|e| self.to_error(e))?)
    }

    /// Create an Essentials subscription
    pub async fn create_essentials_subscription(
        &self,
        name: &str,
        plan_id: i64,
        payment_method_id: Option<i64>,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedSubscriptionHandler::new(self.client.clone());
        let request = FixedSubscriptionCreateRequest {
            name: name.to_string(),
            plan_id: plan_id as i32,
            payment_method: None,
            payment_method_id: payment_method_id.map(|id| id as i32),
            command_type: None,
        };
        let result = handler
            .create(&request)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }

    /// Delete an Essentials subscription
    pub async fn delete_essentials_subscription(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedSubscriptionHandler::new(self.client.clone());
        let result = handler
            .delete_by_id(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }

    /// List Essentials plans
    pub async fn list_essentials_plans(
        &self,
        provider: Option<&str>,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedSubscriptionHandler::new(self.client.clone());
        let plans = handler
            .list_plans(provider.map(String::from), None)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(plans).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // Database Operations
    // =========================================================================

    /// List databases in a subscription
    pub async fn list_databases(&self, subscription_id: i64) -> Result<CallToolResult, RmcpError> {
        let handler = DatabaseHandler::new(self.client.clone());
        let dbs = handler
            .get_subscription_databases(subscription_id as i32, None, None)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(dbs).map_err(|e| self.to_error(e))?)
    }

    /// Get a specific database
    pub async fn get_database(
        &self,
        subscription_id: i64,
        database_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = DatabaseHandler::new(self.client.clone());
        let db = handler
            .get_subscription_database_by_id(subscription_id as i32, database_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(db).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // Task Operations
    // =========================================================================

    /// List tasks
    pub async fn list_tasks(&self) -> Result<CallToolResult, RmcpError> {
        let handler = TaskHandler::new(self.client.clone());
        let tasks = handler
            .get_all_tasks()
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(tasks).map_err(|e| self.to_error(e))?)
    }

    /// Get a specific task
    pub async fn get_task(&self, task_id: &str) -> Result<CallToolResult, RmcpError> {
        let handler = TaskHandler::new(self.client.clone());
        let task = handler
            .get_task_by_id(task_id.to_string())
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(task).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // Essentials (Fixed) Database Operations
    // =========================================================================

    /// List databases in an Essentials subscription
    pub async fn list_essentials_databases(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedDatabaseHandler::new(self.client.clone());
        let dbs = handler
            .list(subscription_id as i32, None, None)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(dbs).map_err(|e| self.to_error(e))?)
    }

    /// Get a specific Essentials database
    pub async fn get_essentials_database(
        &self,
        subscription_id: i64,
        database_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedDatabaseHandler::new(self.client.clone());
        let db = handler
            .get_by_id(subscription_id as i32, database_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(db).map_err(|e| self.to_error(e))?)
    }

    /// Delete an Essentials database
    pub async fn delete_essentials_database(
        &self,
        subscription_id: i64,
        database_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = FixedDatabaseHandler::new(self.client.clone());
        let result = handler
            .delete_by_id(subscription_id as i32, database_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // VPC Peering Operations
    // =========================================================================

    /// Get VPC peerings for a subscription
    pub async fn get_vpc_peerings(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = VpcPeeringHandler::new(self.client.clone());
        let peerings = handler
            .get(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(peerings).map_err(|e| self.to_error(e))?)
    }

    /// Delete a VPC peering
    pub async fn delete_vpc_peering(
        &self,
        subscription_id: i64,
        peering_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = VpcPeeringHandler::new(self.client.clone());
        handler
            .delete(subscription_id as i32, peering_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::json!({
            "success": true,
            "message": format!("VPC peering {} deleted from subscription {}", peering_id, subscription_id)
        }))
    }

    // =========================================================================
    // Cloud Account Operations
    // =========================================================================

    /// List all cloud accounts
    pub async fn list_cloud_accounts(&self) -> Result<CallToolResult, RmcpError> {
        let handler = CloudAccountHandler::new(self.client.clone());
        let accounts = handler
            .get_cloud_accounts()
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(accounts).map_err(|e| self.to_error(e))?)
    }

    /// Get a specific cloud account
    pub async fn get_cloud_account(&self, account_id: i64) -> Result<CallToolResult, RmcpError> {
        let handler = CloudAccountHandler::new(self.client.clone());
        let account = handler
            .get_cloud_account_by_id(account_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(account).map_err(|e| self.to_error(e))?)
    }

    /// Delete a cloud account
    pub async fn delete_cloud_account(&self, account_id: i64) -> Result<CallToolResult, RmcpError> {
        let handler = CloudAccountHandler::new(self.client.clone());
        let result = handler
            .delete_cloud_account(account_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }

    // =========================================================================
    // Private Link Operations (AWS PrivateLink)
    // =========================================================================

    /// Get Private Link configuration for a subscription
    pub async fn get_private_link(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = PrivateLinkHandler::new(self.client.clone());
        let result = handler
            .get(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(result)
    }

    /// Delete Private Link configuration for a subscription
    pub async fn delete_private_link(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = PrivateLinkHandler::new(self.client.clone());
        let result = handler
            .delete(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(result)
    }

    // =========================================================================
    // Transit Gateway Operations (AWS Transit Gateway)
    // =========================================================================

    /// Get Transit Gateway attachments for a subscription
    pub async fn get_transit_gateway_attachments(
        &self,
        subscription_id: i64,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = TransitGatewayHandler::new(self.client.clone());
        let result = handler
            .get_attachments(subscription_id as i32)
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }

    /// Delete a Transit Gateway attachment
    pub async fn delete_transit_gateway_attachment(
        &self,
        subscription_id: i64,
        attachment_id: &str,
    ) -> Result<CallToolResult, RmcpError> {
        let handler = TransitGatewayHandler::new(self.client.clone());
        let result = handler
            .delete_attachment(subscription_id as i32, attachment_id.to_string())
            .await
            .map_err(|e| self.to_error(e))?;
        self.to_result(serde_json::to_value(result).map_err(|e| self.to_error(e))?)
    }
}
