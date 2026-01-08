//! Collaborator system
//!
//! This module provides invite, guest sessions, and project handover functionality.

pub mod r#types;
pub mod client;

pub use r#types::{CollaboratorRole, Invite, InviteRequest, GuestSession, Handover, HandoverRequest, HandoverState};
pub use client::CollaboratorClient;
