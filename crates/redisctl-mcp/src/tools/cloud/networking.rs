//! Networking and connectivity tools for Redis Cloud
//!
//! VPC Peering, Transit Gateway, Private Service Connect (PSC), and PrivateLink tools.

use std::sync::Arc;

use redis_cloud::connectivity::psc::PscEndpointUpdateRequest;
use redis_cloud::connectivity::transit_gateway::TgwAttachmentRequest;
use redis_cloud::connectivity::vpc_peering::VpcPeeringCreateRequest;
use redis_cloud::{
    PrincipalType, PrivateLinkAddPrincipalRequest, PrivateLinkCreateRequest, PrivateLinkHandler,
    PscHandler, TransitGatewayHandler, VpcPeeringHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tower_mcp::extract::{Json, State};
use tower_mcp::{
    CallToolResult, Error as McpError, McpRouter, ResultExt, Tool, ToolBuilder, ToolError,
};

use crate::state::AppState;

// ============================================================================
// VPC Peering tools
// ============================================================================

/// Input for getting VPC peering details
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_vpc_peering tool
pub fn get_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_vpc_peering")
        .description("Get VPC peering details for a Redis Cloud subscription.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetVpcPeeringInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .get(input.subscription_id)
                    .await
                    .tool_context("Failed to get VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating a VPC peering
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Cloud provider (AWS, GCP, Azure)
    #[serde(default)]
    pub provider: Option<String>,
    /// AWS VPC ID
    #[serde(default)]
    pub vpc_id: Option<String>,
    /// AWS region
    #[serde(default)]
    pub aws_region: Option<String>,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// VPC CIDR block
    #[serde(default)]
    pub vpc_cidr: Option<String>,
    /// Multiple VPC CIDR blocks
    #[serde(default)]
    pub vpc_cidrs: Option<Vec<String>>,
    /// GCP project ID (for GCP peering)
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// GCP network name (for GCP peering)
    #[serde(default)]
    pub network_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_vpc_peering tool
pub fn create_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_vpc_peering")
        .description(
            "Create a VPC peering connection for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateVpcPeeringInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = VpcPeeringCreateRequest {
                    provider: input.provider,
                    vpc_id: input.vpc_id,
                    aws_region: input.aws_region,
                    aws_account_id: input.aws_account_id,
                    vpc_cidr: input.vpc_cidr,
                    vpc_cidrs: input.vpc_cidrs,
                    gcp_project_id: input.gcp_project_id,
                    network_name: input.network_name,
                    ..Default::default()
                };

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .create(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to create VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating a VPC peering
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// VPC Peering ID
    pub peering_id: i32,
    /// Cloud provider (AWS, GCP, Azure)
    #[serde(default)]
    pub provider: Option<String>,
    /// AWS VPC ID
    #[serde(default)]
    pub vpc_id: Option<String>,
    /// AWS region
    #[serde(default)]
    pub aws_region: Option<String>,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// VPC CIDR block
    #[serde(default)]
    pub vpc_cidr: Option<String>,
    /// Multiple VPC CIDR blocks
    #[serde(default)]
    pub vpc_cidrs: Option<Vec<String>>,
    /// GCP project ID (for GCP peering)
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// GCP network name (for GCP peering)
    #[serde(default)]
    pub network_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_vpc_peering tool
pub fn update_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_vpc_peering")
        .description(
            "Update a VPC peering connection for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateVpcPeeringInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = VpcPeeringCreateRequest {
                    provider: input.provider,
                    vpc_id: input.vpc_id,
                    aws_region: input.aws_region,
                    aws_account_id: input.aws_account_id,
                    vpc_cidr: input.vpc_cidr,
                    vpc_cidrs: input.vpc_cidrs,
                    gcp_project_id: input.gcp_project_id,
                    network_name: input.network_name,
                    ..Default::default()
                };

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .update(input.subscription_id, input.peering_id, &request)
                    .await
                    .tool_context("Failed to update VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a VPC peering
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// VPC Peering ID
    pub peering_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_vpc_peering tool
pub fn delete_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_vpc_peering")
        .description(
            "DANGEROUS: Permanently deletes a VPC peering connection. \
             Network connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteVpcPeeringInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .delete(input.subscription_id, input.peering_id)
                    .await
                    .tool_context("Failed to delete VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Active-Active VPC Peering tools
// ============================================================================

/// Input for getting Active-Active VPC peering details
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_vpc_peering tool
pub fn get_aa_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_vpc_peering")
        .description(
            "Get Active-Active VPC peering details for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaVpcPeeringInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .get_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to get AA VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating an Active-Active VPC peering
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAaVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Cloud provider (AWS, GCP, Azure)
    #[serde(default)]
    pub provider: Option<String>,
    /// AWS VPC ID
    #[serde(default)]
    pub vpc_id: Option<String>,
    /// AWS region
    #[serde(default)]
    pub aws_region: Option<String>,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// VPC CIDR block
    #[serde(default)]
    pub vpc_cidr: Option<String>,
    /// Multiple VPC CIDR blocks
    #[serde(default)]
    pub vpc_cidrs: Option<Vec<String>>,
    /// GCP project ID (for GCP peering)
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// GCP network name (for GCP peering)
    #[serde(default)]
    pub network_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_aa_vpc_peering tool
pub fn create_aa_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_aa_vpc_peering")
        .description(
            "Create an Active-Active VPC peering connection for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateAaVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateAaVpcPeeringInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = VpcPeeringCreateRequest {
                    provider: input.provider,
                    vpc_id: input.vpc_id,
                    aws_region: input.aws_region,
                    aws_account_id: input.aws_account_id,
                    vpc_cidr: input.vpc_cidr,
                    vpc_cidrs: input.vpc_cidrs,
                    gcp_project_id: input.gcp_project_id,
                    network_name: input.network_name,
                    ..Default::default()
                };

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .create_active_active(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to create AA VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating an Active-Active VPC peering
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAaVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// VPC Peering ID
    pub peering_id: i32,
    /// Cloud provider (AWS, GCP, Azure)
    #[serde(default)]
    pub provider: Option<String>,
    /// AWS VPC ID
    #[serde(default)]
    pub vpc_id: Option<String>,
    /// AWS region
    #[serde(default)]
    pub aws_region: Option<String>,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// VPC CIDR block
    #[serde(default)]
    pub vpc_cidr: Option<String>,
    /// Multiple VPC CIDR blocks
    #[serde(default)]
    pub vpc_cidrs: Option<Vec<String>>,
    /// GCP project ID (for GCP peering)
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// GCP network name (for GCP peering)
    #[serde(default)]
    pub network_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_aa_vpc_peering tool
pub fn update_aa_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_aa_vpc_peering")
        .description(
            "Update an Active-Active VPC peering connection for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateAaVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateAaVpcPeeringInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = VpcPeeringCreateRequest {
                    provider: input.provider,
                    vpc_id: input.vpc_id,
                    aws_region: input.aws_region,
                    aws_account_id: input.aws_account_id,
                    vpc_cidr: input.vpc_cidr,
                    vpc_cidrs: input.vpc_cidrs,
                    gcp_project_id: input.gcp_project_id,
                    network_name: input.network_name,
                    ..Default::default()
                };

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .update_active_active(input.subscription_id, input.peering_id, &request)
                    .await
                    .tool_context("Failed to update AA VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting an Active-Active VPC peering
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAaVpcPeeringInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// VPC Peering ID
    pub peering_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_aa_vpc_peering tool
pub fn delete_aa_vpc_peering(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_aa_vpc_peering")
        .description(
            "DANGEROUS: Permanently deletes an Active-Active VPC peering connection. \
             Network connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteAaVpcPeeringInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteAaVpcPeeringInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = VpcPeeringHandler::new(client);
                let result = handler
                    .delete_active_active(input.subscription_id, input.peering_id)
                    .await
                    .tool_context("Failed to delete AA VPC peering")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Transit Gateway tools
// ============================================================================

/// Input for getting Transit Gateway attachments
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTgwAttachmentsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_tgw_attachments tool
pub fn get_tgw_attachments(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_tgw_attachments")
        .description(
            "Get Transit Gateway attachments for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetTgwAttachmentsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetTgwAttachmentsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .get_attachments(input.subscription_id)
                    .await
                    .tool_context("Failed to get TGW attachments")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting Transit Gateway invitations
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTgwInvitationsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_tgw_invitations tool
pub fn get_tgw_invitations(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_tgw_invitations")
        .description(
            "Get Transit Gateway shared invitations for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetTgwInvitationsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetTgwInvitationsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .get_shared_invitations(input.subscription_id)
                    .await
                    .tool_context("Failed to get TGW invitations")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for accepting a Transit Gateway invitation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AcceptTgwInvitationInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Invitation ID
    pub invitation_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the accept_tgw_invitation tool
pub fn accept_tgw_invitation(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("accept_tgw_invitation")
        .description(
            "Accept a Transit Gateway resource share invitation. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, AcceptTgwInvitationInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<AcceptTgwInvitationInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .accept_resource_share(input.subscription_id, input.invitation_id)
                    .await
                    .tool_context("Failed to accept TGW invitation")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for rejecting a Transit Gateway invitation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RejectTgwInvitationInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Invitation ID
    pub invitation_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the reject_tgw_invitation tool
pub fn reject_tgw_invitation(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("reject_tgw_invitation")
        .description(
            "Reject a Transit Gateway resource share invitation. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RejectTgwInvitationInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RejectTgwInvitationInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .reject_resource_share(input.subscription_id, input.invitation_id)
                    .await
                    .tool_context("Failed to reject TGW invitation")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating a Transit Gateway attachment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateTgwAttachmentInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// Transit Gateway ID
    #[serde(default)]
    pub tgw_id: Option<String>,
    /// CIDR blocks to route through the TGW
    #[serde(default)]
    pub cidrs: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_tgw_attachment tool
pub fn create_tgw_attachment(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_tgw_attachment")
        .description(
            "Create a Transit Gateway attachment for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateTgwAttachmentInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateTgwAttachmentInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = TgwAttachmentRequest {
                    aws_account_id: input.aws_account_id,
                    tgw_id: input.tgw_id,
                    cidrs: input.cidrs,
                };

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .create_attachment(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to create TGW attachment")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating Transit Gateway attachment CIDRs
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTgwAttachmentCidrsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Attachment ID
    pub attachment_id: String,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// Transit Gateway ID
    #[serde(default)]
    pub tgw_id: Option<String>,
    /// CIDR blocks to route through the TGW
    #[serde(default)]
    pub cidrs: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_tgw_attachment_cidrs tool
pub fn update_tgw_attachment_cidrs(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_tgw_attachment_cidrs")
        .description(
            "Update CIDRs for a Transit Gateway attachment. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateTgwAttachmentCidrsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateTgwAttachmentCidrsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = TgwAttachmentRequest {
                    aws_account_id: input.aws_account_id,
                    tgw_id: input.tgw_id,
                    cidrs: input.cidrs,
                };

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .update_attachment_cidrs(input.subscription_id, input.attachment_id, &request)
                    .await
                    .tool_context("Failed to update TGW attachment CIDRs")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a Transit Gateway attachment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteTgwAttachmentInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Attachment ID
    pub attachment_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_tgw_attachment tool
pub fn delete_tgw_attachment(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_tgw_attachment")
        .description(
            "DANGEROUS: Permanently deletes a Transit Gateway attachment. \
             Network connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteTgwAttachmentInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteTgwAttachmentInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .delete_attachment(input.subscription_id, input.attachment_id)
                    .await
                    .tool_context("Failed to delete TGW attachment")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Active-Active Transit Gateway tools
// ============================================================================

/// Input for getting Active-Active Transit Gateway attachments
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaTgwAttachmentsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_tgw_attachments tool
pub fn get_aa_tgw_attachments(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_tgw_attachments")
        .description(
            "Get Active-Active Transit Gateway attachments for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaTgwAttachmentsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaTgwAttachmentsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .get_attachments_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to get AA TGW attachments")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting Active-Active Transit Gateway invitations
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaTgwInvitationsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_tgw_invitations tool
pub fn get_aa_tgw_invitations(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_tgw_invitations")
        .description(
            "Get Active-Active Transit Gateway shared invitations for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaTgwInvitationsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaTgwInvitationsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .get_shared_invitations_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to get AA TGW invitations")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for accepting an Active-Active Transit Gateway invitation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AcceptAaTgwInvitationInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Invitation ID
    pub invitation_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the accept_aa_tgw_invitation tool
pub fn accept_aa_tgw_invitation(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("accept_aa_tgw_invitation")
        .description(
            "Accept an Active-Active Transit Gateway resource share invitation. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, AcceptAaTgwInvitationInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<AcceptAaTgwInvitationInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .accept_resource_share_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.invitation_id,
                    )
                    .await
                    .tool_context("Failed to accept AA TGW invitation")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for rejecting an Active-Active Transit Gateway invitation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RejectAaTgwInvitationInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Invitation ID
    pub invitation_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the reject_aa_tgw_invitation tool
pub fn reject_aa_tgw_invitation(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("reject_aa_tgw_invitation")
        .description(
            "Reject an Active-Active Transit Gateway resource share invitation. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RejectAaTgwInvitationInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RejectAaTgwInvitationInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .reject_resource_share_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.invitation_id,
                    )
                    .await
                    .tool_context("Failed to reject AA TGW invitation")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating an Active-Active Transit Gateway attachment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAaTgwAttachmentInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// Transit Gateway ID
    #[serde(default)]
    pub tgw_id: Option<String>,
    /// CIDR blocks to route through the TGW
    #[serde(default)]
    pub cidrs: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_aa_tgw_attachment tool
pub fn create_aa_tgw_attachment(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_aa_tgw_attachment")
        .description(
            "Create an Active-Active Transit Gateway attachment. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateAaTgwAttachmentInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateAaTgwAttachmentInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = TgwAttachmentRequest {
                    aws_account_id: input.aws_account_id,
                    tgw_id: input.tgw_id,
                    cidrs: input.cidrs,
                };

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .create_attachment_active_active(
                        input.subscription_id,
                        input.region_id,
                        &request,
                    )
                    .await
                    .tool_context("Failed to create AA TGW attachment")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating Active-Active Transit Gateway attachment CIDRs
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAaTgwAttachmentCidrsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Attachment ID
    pub attachment_id: String,
    /// AWS account ID
    #[serde(default)]
    pub aws_account_id: Option<String>,
    /// Transit Gateway ID
    #[serde(default)]
    pub tgw_id: Option<String>,
    /// CIDR blocks to route through the TGW
    #[serde(default)]
    pub cidrs: Option<Vec<String>>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_aa_tgw_attachment_cidrs tool
pub fn update_aa_tgw_attachment_cidrs(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_aa_tgw_attachment_cidrs")
        .description(
            "Update CIDRs for an Active-Active Transit Gateway attachment. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateAaTgwAttachmentCidrsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateAaTgwAttachmentCidrsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = TgwAttachmentRequest {
                    aws_account_id: input.aws_account_id,
                    tgw_id: input.tgw_id,
                    cidrs: input.cidrs,
                };

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .update_attachment_cidrs_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.attachment_id,
                        &request,
                    )
                    .await
                    .tool_context("Failed to update AA TGW attachment CIDRs")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting an Active-Active Transit Gateway attachment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAaTgwAttachmentInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Attachment ID
    pub attachment_id: String,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_aa_tgw_attachment tool
pub fn delete_aa_tgw_attachment(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_aa_tgw_attachment")
        .description(
            "DANGEROUS: Permanently deletes an Active-Active Transit Gateway attachment. \
             Network connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteAaTgwAttachmentInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteAaTgwAttachmentInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = TransitGatewayHandler::new(client);
                let result = handler
                    .delete_attachment_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.attachment_id,
                    )
                    .await
                    .tool_context("Failed to delete AA TGW attachment")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Private Service Connect (PSC) tools
// ============================================================================

/// Input for getting PSC service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPscServiceInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_psc_service tool
pub fn get_psc_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_psc_service")
        .description("Get Private Service Connect service for a Redis Cloud subscription.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetPscServiceInput>(
            state,
            |State(state): State<Arc<AppState>>, Json(input): Json<GetPscServiceInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_service(input.subscription_id)
                    .await
                    .tool_context("Failed to get PSC service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating PSC service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePscServiceInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_psc_service tool
pub fn create_psc_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_psc_service")
        .description(
            "Create a Private Service Connect service for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreatePscServiceInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreatePscServiceInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .create_service(input.subscription_id)
                    .await
                    .tool_context("Failed to create PSC service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting PSC service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeletePscServiceInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_psc_service tool
pub fn delete_psc_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_psc_service")
        .description(
            "DANGEROUS: Permanently deletes a Private Service Connect service. \
             All endpoints will be disconnected. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeletePscServiceInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeletePscServiceInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .delete_service(input.subscription_id)
                    .await
                    .tool_context("Failed to delete PSC service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting PSC endpoints
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPscEndpointsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_psc_endpoints tool
pub fn get_psc_endpoints(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_psc_endpoints")
        .description(
            "Get Private Service Connect endpoints for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetPscEndpointsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetPscEndpointsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_endpoints(input.subscription_id)
                    .await
                    .tool_context("Failed to get PSC endpoints")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating a PSC endpoint
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePscEndpointInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// PSC service ID (used internally by the request)
    pub psc_service_id: i32,
    /// Endpoint ID (used internally by the request)
    pub endpoint_id: i32,
    /// Google Cloud project ID
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// Name of the Google Cloud VPC that hosts your application
    #[serde(default)]
    pub gcp_vpc_name: Option<String>,
    /// Name of your VPC's subnet of IP address ranges
    #[serde(default)]
    pub gcp_vpc_subnet_name: Option<String>,
    /// Prefix used to create PSC endpoints in the consumer application VPC
    #[serde(default)]
    pub endpoint_connection_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_psc_endpoint tool
pub fn create_psc_endpoint(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_psc_endpoint")
        .description(
            "Create a Private Service Connect endpoint. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreatePscEndpointInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreatePscEndpointInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = PscEndpointUpdateRequest {
                    subscription_id: input.subscription_id,
                    psc_service_id: input.psc_service_id,
                    endpoint_id: input.endpoint_id,
                    gcp_project_id: input.gcp_project_id,
                    gcp_vpc_name: input.gcp_vpc_name,
                    gcp_vpc_subnet_name: input.gcp_vpc_subnet_name,
                    endpoint_connection_name: input.endpoint_connection_name,
                };

                let handler = PscHandler::new(client);
                let result = handler
                    .create_endpoint(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to create PSC endpoint")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating a PSC endpoint
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdatePscEndpointInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// PSC service ID
    pub psc_service_id: i32,
    /// Google Cloud project ID
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// Name of the Google Cloud VPC that hosts your application
    #[serde(default)]
    pub gcp_vpc_name: Option<String>,
    /// Name of your VPC's subnet of IP address ranges
    #[serde(default)]
    pub gcp_vpc_subnet_name: Option<String>,
    /// Prefix used to create PSC endpoints in the consumer application VPC
    #[serde(default)]
    pub endpoint_connection_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_psc_endpoint tool
pub fn update_psc_endpoint(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_psc_endpoint")
        .description(
            "Update a Private Service Connect endpoint. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdatePscEndpointInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdatePscEndpointInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = PscEndpointUpdateRequest {
                    subscription_id: input.subscription_id,
                    psc_service_id: input.psc_service_id,
                    endpoint_id: input.endpoint_id,
                    gcp_project_id: input.gcp_project_id,
                    gcp_vpc_name: input.gcp_vpc_name,
                    gcp_vpc_subnet_name: input.gcp_vpc_subnet_name,
                    endpoint_connection_name: input.endpoint_connection_name,
                };

                let handler = PscHandler::new(client);
                let result = handler
                    .update_endpoint(input.subscription_id, input.endpoint_id, &request)
                    .await
                    .tool_context("Failed to update PSC endpoint")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a PSC endpoint
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeletePscEndpointInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_psc_endpoint tool
pub fn delete_psc_endpoint(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_psc_endpoint")
        .description(
            "DANGEROUS: Permanently deletes a Private Service Connect endpoint. \
             Connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeletePscEndpointInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeletePscEndpointInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .delete_endpoint(input.subscription_id, input.endpoint_id)
                    .await
                    .tool_context("Failed to delete PSC endpoint")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting PSC creation script
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPscCreationScriptInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_psc_creation_script tool
pub fn get_psc_creation_script(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_psc_creation_script")
        .description(
            "Get the creation script for a Private Service Connect endpoint.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetPscCreationScriptInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetPscCreationScriptInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_endpoint_creation_script(input.subscription_id, input.endpoint_id)
                    .await
                    .tool_context("Failed to get PSC creation script")?;

                Ok(CallToolResult::text(result))
            },
        )
        .build()
}

/// Input for getting PSC deletion script
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPscDeletionScriptInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_psc_deletion_script tool
pub fn get_psc_deletion_script(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_psc_deletion_script")
        .description(
            "Get the deletion script for a Private Service Connect endpoint.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetPscDeletionScriptInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetPscDeletionScriptInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_endpoint_deletion_script(input.subscription_id, input.endpoint_id)
                    .await
                    .tool_context("Failed to get PSC deletion script")?;

                Ok(CallToolResult::text(result))
            },
        )
        .build()
}

// ============================================================================
// Active-Active PSC tools
// ============================================================================

/// Input for getting Active-Active PSC service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaPscServiceInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_psc_service tool
pub fn get_aa_psc_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_psc_service")
        .description(
            "Get Active-Active Private Service Connect service for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaPscServiceInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaPscServiceInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_service_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to get AA PSC service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating Active-Active PSC service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAaPscServiceInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_aa_psc_service tool
pub fn create_aa_psc_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_aa_psc_service")
        .description(
            "Create an Active-Active Private Service Connect service. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateAaPscServiceInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateAaPscServiceInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .create_service_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to create AA PSC service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting Active-Active PSC service
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAaPscServiceInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_aa_psc_service tool
pub fn delete_aa_psc_service(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_aa_psc_service")
        .description(
            "DANGEROUS: Permanently deletes an Active-Active Private Service Connect service. \
             All endpoints will be disconnected. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteAaPscServiceInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteAaPscServiceInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .delete_service_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to delete AA PSC service")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting Active-Active PSC endpoints
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaPscEndpointsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_psc_endpoints tool
pub fn get_aa_psc_endpoints(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_psc_endpoints")
        .description(
            "Get Active-Active Private Service Connect endpoints for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaPscEndpointsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaPscEndpointsInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_endpoints_active_active(input.subscription_id)
                    .await
                    .tool_context("Failed to get AA PSC endpoints")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating an Active-Active PSC endpoint
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAaPscEndpointInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// PSC service ID
    pub psc_service_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Google Cloud project ID
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// Name of the Google Cloud VPC that hosts your application
    #[serde(default)]
    pub gcp_vpc_name: Option<String>,
    /// Name of your VPC's subnet of IP address ranges
    #[serde(default)]
    pub gcp_vpc_subnet_name: Option<String>,
    /// Prefix used to create PSC endpoints in the consumer application VPC
    #[serde(default)]
    pub endpoint_connection_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_aa_psc_endpoint tool
pub fn create_aa_psc_endpoint(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_aa_psc_endpoint")
        .description(
            "Create an Active-Active Private Service Connect endpoint. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateAaPscEndpointInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateAaPscEndpointInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = PscEndpointUpdateRequest {
                    subscription_id: input.subscription_id,
                    psc_service_id: input.psc_service_id,
                    endpoint_id: input.endpoint_id,
                    gcp_project_id: input.gcp_project_id,
                    gcp_vpc_name: input.gcp_vpc_name,
                    gcp_vpc_subnet_name: input.gcp_vpc_subnet_name,
                    endpoint_connection_name: input.endpoint_connection_name,
                };

                let handler = PscHandler::new(client);
                let result = handler
                    .create_endpoint_active_active(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to create AA PSC endpoint")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for updating an Active-Active PSC endpoint
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAaPscEndpointInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// PSC service ID
    pub psc_service_id: i32,
    /// Google Cloud project ID
    #[serde(default)]
    pub gcp_project_id: Option<String>,
    /// Name of the Google Cloud VPC that hosts your application
    #[serde(default)]
    pub gcp_vpc_name: Option<String>,
    /// Name of your VPC's subnet of IP address ranges
    #[serde(default)]
    pub gcp_vpc_subnet_name: Option<String>,
    /// Prefix used to create PSC endpoints in the consumer application VPC
    #[serde(default)]
    pub endpoint_connection_name: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the update_aa_psc_endpoint tool
pub fn update_aa_psc_endpoint(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("update_aa_psc_endpoint")
        .description(
            "Update an Active-Active Private Service Connect endpoint. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, UpdateAaPscEndpointInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<UpdateAaPscEndpointInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let request = PscEndpointUpdateRequest {
                    subscription_id: input.subscription_id,
                    psc_service_id: input.psc_service_id,
                    endpoint_id: input.endpoint_id,
                    gcp_project_id: input.gcp_project_id,
                    gcp_vpc_name: input.gcp_vpc_name,
                    gcp_vpc_subnet_name: input.gcp_vpc_subnet_name,
                    endpoint_connection_name: input.endpoint_connection_name,
                };

                let handler = PscHandler::new(client);
                let result = handler
                    .update_endpoint_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.endpoint_id,
                        &request,
                    )
                    .await
                    .tool_context("Failed to update AA PSC endpoint")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting an Active-Active PSC endpoint
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAaPscEndpointInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_aa_psc_endpoint tool
pub fn delete_aa_psc_endpoint(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_aa_psc_endpoint")
        .description(
            "DANGEROUS: Permanently deletes an Active-Active Private Service Connect endpoint. \
             Connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeleteAaPscEndpointInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeleteAaPscEndpointInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .delete_endpoint_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.endpoint_id,
                    )
                    .await
                    .tool_context("Failed to delete AA PSC endpoint")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting Active-Active PSC creation script
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaPscCreationScriptInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// PSC service ID
    pub psc_service_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_psc_creation_script tool
pub fn get_aa_psc_creation_script(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_psc_creation_script")
        .description(
            "Get the creation script for an Active-Active Private Service Connect endpoint.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaPscCreationScriptInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaPscCreationScriptInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_endpoint_creation_script_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.psc_service_id,
                        input.endpoint_id,
                    )
                    .await
                    .tool_context("Failed to get AA PSC creation script")?;

                Ok(CallToolResult::text(result))
            },
        )
        .build()
}

/// Input for getting Active-Active PSC deletion script
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaPscDeletionScriptInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// PSC service ID
    pub psc_service_id: i32,
    /// Endpoint ID
    pub endpoint_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_psc_deletion_script tool
pub fn get_aa_psc_deletion_script(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_psc_deletion_script")
        .description(
            "Get the deletion script for an Active-Active Private Service Connect endpoint.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaPscDeletionScriptInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaPscDeletionScriptInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PscHandler::new(client);
                let result = handler
                    .get_endpoint_deletion_script_active_active(
                        input.subscription_id,
                        input.region_id,
                        input.psc_service_id,
                        input.endpoint_id,
                    )
                    .await
                    .tool_context("Failed to get AA PSC deletion script")?;

                Ok(CallToolResult::text(result))
            },
        )
        .build()
}

// ============================================================================
// PrivateLink tools
// ============================================================================

/// Input for getting PrivateLink configuration
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPrivateLinkInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_private_link tool
pub fn get_private_link(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_private_link")
        .description(
            "Get AWS PrivateLink configuration for a Redis Cloud subscription.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetPrivateLinkInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetPrivateLinkInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .get(input.subscription_id)
                    .await
                    .tool_context("Failed to get PrivateLink")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating a PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePrivateLinkInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Share name for the PrivateLink service (max 64 characters)
    pub share_name: String,
    /// AWS principal (account ID, role ARN, etc.)
    pub principal: String,
    /// Principal type: aws_account, organization, organization_unit, iam_role, iam_user, service_principal
    pub principal_type: String,
    /// Optional alias for the PrivateLink
    #[serde(default)]
    pub alias: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

fn parse_principal_type(s: &str) -> Result<PrincipalType, ToolError> {
    match s.to_lowercase().as_str() {
        "aws_account" | "awsaccount" => Ok(PrincipalType::AwsAccount),
        "organization" => Ok(PrincipalType::Organization),
        "organization_unit" | "organizationunit" => Ok(PrincipalType::OrganizationUnit),
        "iam_role" | "iamrole" => Ok(PrincipalType::IamRole),
        "iam_user" | "iamuser" => Ok(PrincipalType::IamUser),
        "service_principal" | "serviceprincipal" => Ok(PrincipalType::ServicePrincipal),
        _ => Err(ToolError::new(format!(
            "Invalid principal type: {}. Expected one of: aws_account, organization, organization_unit, iam_role, iam_user, service_principal",
            s
        ))),
    }
}

/// Build the create_private_link tool
pub fn create_private_link(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_private_link")
        .description(
            "Create an AWS PrivateLink configuration for a Redis Cloud subscription. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreatePrivateLinkInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreatePrivateLinkInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let principal_type = parse_principal_type(&input.principal_type)?;

                let request = PrivateLinkCreateRequest {
                    share_name: input.share_name,
                    principal: input.principal,
                    principal_type,
                    alias: input.alias,
                };

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .create(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to create PrivateLink")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for deleting a PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeletePrivateLinkInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the delete_private_link tool
pub fn delete_private_link(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("delete_private_link")
        .description(
            "DANGEROUS: Permanently deletes an AWS PrivateLink configuration. \
             Connectivity will be immediately lost. Requires write permission.",
        )
        .destructive()
        .extractor_handler_typed::<_, _, _, DeletePrivateLinkInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<DeletePrivateLinkInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .delete(input.subscription_id)
                    .await
                    .tool_context("Failed to delete PrivateLink")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for adding principals to PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddPrivateLinkPrincipalsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// AWS principal (account ID, role ARN, etc.)
    pub principal: String,
    /// Principal type: aws_account, organization, organization_unit, iam_role, iam_user, service_principal
    #[serde(default)]
    pub principal_type: Option<String>,
    /// Optional alias for the principal
    #[serde(default)]
    pub alias: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the add_private_link_principals tool
pub fn add_private_link_principals(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("add_private_link_principals")
        .description(
            "Add AWS principals to a PrivateLink access list. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, AddPrivateLinkPrincipalsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<AddPrivateLinkPrincipalsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let principal_type = input
                    .principal_type
                    .map(|pt| parse_principal_type(&pt))
                    .transpose()?;

                let request = PrivateLinkAddPrincipalRequest {
                    principal: input.principal,
                    principal_type,
                    alias: input.alias,
                };

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .add_principals(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to add PrivateLink principals")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for removing principals from PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemovePrivateLinkPrincipalsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// AWS principal (account ID, role ARN, etc.) to remove
    pub principal: String,
    /// Principal type: aws_account, organization, organization_unit, iam_role, iam_user, service_principal
    #[serde(default)]
    pub principal_type: Option<String>,
    /// Alias of the principal
    #[serde(default)]
    pub alias: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the remove_private_link_principals tool
pub fn remove_private_link_principals(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("remove_private_link_principals")
        .description(
            "Remove AWS principals from a PrivateLink access list. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RemovePrivateLinkPrincipalsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RemovePrivateLinkPrincipalsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let principal_type = input
                    .principal_type
                    .map(|pt| parse_principal_type(&pt))
                    .transpose()?;

                let request =
                    redis_cloud::connectivity::private_link::PrivateLinkRemovePrincipalRequest {
                        principal: input.principal,
                        principal_type,
                        alias: input.alias,
                    };

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .remove_principals(input.subscription_id, &request)
                    .await
                    .tool_context("Failed to remove PrivateLink principals")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting PrivateLink endpoint script
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPrivateLinkEndpointScriptInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_private_link_endpoint_script tool
pub fn get_private_link_endpoint_script(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_private_link_endpoint_script")
        .description("Get the endpoint creation script for an AWS PrivateLink configuration.")
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetPrivateLinkEndpointScriptInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetPrivateLinkEndpointScriptInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .get_endpoint_script(input.subscription_id)
                    .await
                    .tool_context("Failed to get PrivateLink endpoint script")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Active-Active PrivateLink tools
// ============================================================================

/// Input for getting Active-Active PrivateLink configuration
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaPrivateLinkInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_private_link tool
pub fn get_aa_private_link(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_private_link")
        .description(
            "Get Active-Active AWS PrivateLink configuration for a Redis Cloud subscription region.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaPrivateLinkInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaPrivateLinkInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .get_active_active(input.subscription_id, input.region_id)
                    .await
                    .tool_context("Failed to get AA PrivateLink")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for creating an Active-Active PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAaPrivateLinkInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Share name for the PrivateLink service (max 64 characters)
    pub share_name: String,
    /// AWS principal (account ID, role ARN, etc.)
    pub principal: String,
    /// Principal type: aws_account, organization, organization_unit, iam_role, iam_user, service_principal
    pub principal_type: String,
    /// Optional alias for the PrivateLink
    #[serde(default)]
    pub alias: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the create_aa_private_link tool
pub fn create_aa_private_link(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("create_aa_private_link")
        .description(
            "Create an Active-Active AWS PrivateLink configuration. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, CreateAaPrivateLinkInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<CreateAaPrivateLinkInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let principal_type = parse_principal_type(&input.principal_type)?;

                let request = PrivateLinkCreateRequest {
                    share_name: input.share_name,
                    principal: input.principal,
                    principal_type,
                    alias: input.alias,
                };

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .create_active_active(input.subscription_id, input.region_id, &request)
                    .await
                    .tool_context("Failed to create AA PrivateLink")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for adding principals to Active-Active PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddAaPrivateLinkPrincipalsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// AWS principal (account ID, role ARN, etc.)
    pub principal: String,
    /// Principal type: aws_account, organization, organization_unit, iam_role, iam_user, service_principal
    #[serde(default)]
    pub principal_type: Option<String>,
    /// Optional alias for the principal
    #[serde(default)]
    pub alias: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the add_aa_private_link_principals tool
pub fn add_aa_private_link_principals(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("add_aa_private_link_principals")
        .description(
            "Add AWS principals to an Active-Active PrivateLink access list. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, AddAaPrivateLinkPrincipalsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<AddAaPrivateLinkPrincipalsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let principal_type = input
                    .principal_type
                    .map(|pt| parse_principal_type(&pt))
                    .transpose()?;

                let request = PrivateLinkAddPrincipalRequest {
                    principal: input.principal,
                    principal_type,
                    alias: input.alias,
                };

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .add_principals_active_active(input.subscription_id, input.region_id, &request)
                    .await
                    .tool_context("Failed to add AA PrivateLink principals")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for removing principals from Active-Active PrivateLink
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveAaPrivateLinkPrincipalsInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// AWS principal (account ID, role ARN, etc.) to remove
    pub principal: String,
    /// Principal type: aws_account, organization, organization_unit, iam_role, iam_user, service_principal
    #[serde(default)]
    pub principal_type: Option<String>,
    /// Alias of the principal
    #[serde(default)]
    pub alias: Option<String>,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the remove_aa_private_link_principals tool
pub fn remove_aa_private_link_principals(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("remove_aa_private_link_principals")
        .description(
            "Remove AWS principals from an Active-Active PrivateLink access list. \
             Requires write permission.",
        )
        .non_destructive()
        .extractor_handler_typed::<_, _, _, RemoveAaPrivateLinkPrincipalsInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<RemoveAaPrivateLinkPrincipalsInput>| async move {
                if !state.is_write_allowed() {
                    return Err(McpError::tool(
                        "Write operations not allowed in read-only mode",
                    ));
                }

                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let principal_type = input
                    .principal_type
                    .map(|pt| parse_principal_type(&pt))
                    .transpose()?;

                let request =
                    redis_cloud::connectivity::private_link::PrivateLinkRemovePrincipalRequest {
                        principal: input.principal,
                        principal_type,
                        alias: input.alias,
                    };

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .remove_principals_active_active(
                        input.subscription_id,
                        input.region_id,
                        &request,
                    )
                    .await
                    .tool_context("Failed to remove AA PrivateLink principals")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

/// Input for getting Active-Active PrivateLink endpoint script
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAaPrivateLinkEndpointScriptInput {
    /// Subscription ID
    pub subscription_id: i32,
    /// Region ID
    pub region_id: i32,
    /// Profile name for multi-account support. If not specified, uses the first configured profile or default.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Build the get_aa_private_link_endpoint_script tool
pub fn get_aa_private_link_endpoint_script(state: Arc<AppState>) -> Tool {
    ToolBuilder::new("get_aa_private_link_endpoint_script")
        .description(
            "Get the endpoint creation script for an Active-Active AWS PrivateLink configuration.",
        )
        .read_only_safe()
        .extractor_handler_typed::<_, _, _, GetAaPrivateLinkEndpointScriptInput>(
            state,
            |State(state): State<Arc<AppState>>,
             Json(input): Json<GetAaPrivateLinkEndpointScriptInput>| async move {
                let client = state
                    .cloud_client_for_profile(input.profile.as_deref())
                    .await
                    .map_err(|e| crate::tools::credential_error("cloud", e))?;

                let handler = PrivateLinkHandler::new(client);
                let result = handler
                    .get_endpoint_script_active_active(input.subscription_id, input.region_id)
                    .await
                    .tool_context("Failed to get AA PrivateLink endpoint script")?;

                CallToolResult::from_serialize(&result)
            },
        )
        .build()
}

// ============================================================================
// Instructions and Router
// ============================================================================

/// Build an MCP sub-router containing all networking tools
pub fn router(state: Arc<AppState>) -> McpRouter {
    McpRouter::new()
        // VPC Peering
        .tool(get_vpc_peering(state.clone()))
        .tool(create_vpc_peering(state.clone()))
        .tool(update_vpc_peering(state.clone()))
        .tool(delete_vpc_peering(state.clone()))
        .tool(get_aa_vpc_peering(state.clone()))
        .tool(create_aa_vpc_peering(state.clone()))
        .tool(update_aa_vpc_peering(state.clone()))
        .tool(delete_aa_vpc_peering(state.clone()))
        // Transit Gateway
        .tool(get_tgw_attachments(state.clone()))
        .tool(get_tgw_invitations(state.clone()))
        .tool(accept_tgw_invitation(state.clone()))
        .tool(reject_tgw_invitation(state.clone()))
        .tool(create_tgw_attachment(state.clone()))
        .tool(update_tgw_attachment_cidrs(state.clone()))
        .tool(delete_tgw_attachment(state.clone()))
        .tool(get_aa_tgw_attachments(state.clone()))
        .tool(get_aa_tgw_invitations(state.clone()))
        .tool(accept_aa_tgw_invitation(state.clone()))
        .tool(reject_aa_tgw_invitation(state.clone()))
        .tool(create_aa_tgw_attachment(state.clone()))
        .tool(update_aa_tgw_attachment_cidrs(state.clone()))
        .tool(delete_aa_tgw_attachment(state.clone()))
        // PSC
        .tool(get_psc_service(state.clone()))
        .tool(create_psc_service(state.clone()))
        .tool(delete_psc_service(state.clone()))
        .tool(get_psc_endpoints(state.clone()))
        .tool(create_psc_endpoint(state.clone()))
        .tool(update_psc_endpoint(state.clone()))
        .tool(delete_psc_endpoint(state.clone()))
        .tool(get_psc_creation_script(state.clone()))
        .tool(get_psc_deletion_script(state.clone()))
        .tool(get_aa_psc_service(state.clone()))
        .tool(create_aa_psc_service(state.clone()))
        .tool(delete_aa_psc_service(state.clone()))
        .tool(get_aa_psc_endpoints(state.clone()))
        .tool(create_aa_psc_endpoint(state.clone()))
        .tool(update_aa_psc_endpoint(state.clone()))
        .tool(delete_aa_psc_endpoint(state.clone()))
        .tool(get_aa_psc_creation_script(state.clone()))
        .tool(get_aa_psc_deletion_script(state.clone()))
        // PrivateLink
        .tool(get_private_link(state.clone()))
        .tool(create_private_link(state.clone()))
        .tool(delete_private_link(state.clone()))
        .tool(add_private_link_principals(state.clone()))
        .tool(remove_private_link_principals(state.clone()))
        .tool(get_private_link_endpoint_script(state.clone()))
        .tool(get_aa_private_link(state.clone()))
        .tool(create_aa_private_link(state.clone()))
        .tool(add_aa_private_link_principals(state.clone()))
        .tool(remove_aa_private_link_principals(state.clone()))
        .tool(get_aa_private_link_endpoint_script(state.clone()))
}
