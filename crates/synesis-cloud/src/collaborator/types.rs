//! Collaborator types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Collaborator role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollaboratorRole {
    /// Can view agent interactions
    Viewer,

    /// Can add comments/feedback
    Commenter,

    /// Can modify prompts and settings
    Editor,
}

/// Invite request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteRequest {
    /// Project to invite to
    pub project_id: String,

    /// Role for the collaborator
    pub role: CollaboratorRole,

    /// Usage quota in cents
    pub quota_cents: u32,

    /// Hours until invite expires
    pub expires_hours: u32,
}

/// Created invite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    /// Invite token
    pub token: String,

    /// Full invite URL
    pub url: String,

    /// Role
    pub role: CollaboratorRole,

    /// Quota
    pub quota_cents: u32,

    /// Quota used
    pub quota_used_cents: u32,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// Whether accepted
    pub accepted: bool,

    /// Guest email (if accepted)
    pub guest_email: Option<String>,
}

/// Active collaborator session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSession {
    /// Session ID
    pub id: String,

    /// Project ID
    pub project_id: String,

    /// Guest user ID
    pub guest_user_id: String,

    /// Guest email
    pub guest_email: String,

    /// Role
    pub role: CollaboratorRole,

    /// Remaining quota
    pub remaining_quota: u32,

    /// Host user ID
    pub host_user_id: String,

    /// Join date
    pub joined_at: DateTime<Utc>,

    /// Last active
    pub last_active_at: Option<DateTime<Utc>>,
}

/// Handover request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverRequest {
    /// Project to hand over
    pub project_id: String,

    /// New owner's email
    pub to_email: String,

    /// Include LoRAs in handover
    pub include_loras: bool,

    /// Include knowledge vault
    pub include_knowledge: bool,

    /// Optional message to new owner
    pub message: Option<String>,
}

/// Handover status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handover {
    /// Handover token
    pub token: String,

    /// Current state
    pub state: HandoverState,

    /// Recipient email
    pub to_email: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// Completion time (if completed)
    pub completed_at: Option<DateTime<Utc>>,

    /// Incentive pricing
    pub incentive: HandoverIncentive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoverState {
    Pending,
    EmailSent,
    Accepted,
    Completed,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverIncentive {
    /// First year price
    pub first_year_price: f64,

    /// Regular price after first year
    pub regular_price: f64,
}
