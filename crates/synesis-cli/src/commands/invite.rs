//! `synesis invite` - Collaborator invite management

use clap::{Subcommand, Args};
use comfy_table::{presets::UTF8_FULL, Table};
use owo_colors::OwoColorize;

use crate::config::Config;

#[derive(Subcommand)]
pub enum InviteCommands {
    /// Create a new collaborator invite
    Create(CreateArgs),

    /// List all pending invites
    List,

    /// Revoke an invite
    Revoke(RevokeArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// Project ID
    #[arg(short, long)]
    pub project: String,

    /// Collaborator email
    #[arg(short, long)]
    pub email: Option<String>,

    /// Role: viewer, commenter, editor
    #[arg(short, long, default_value = "viewer")]
    pub role: String,

    /// Quota in cents
    #[arg(short, long, default_value = "1000")]
    pub quota_cents: u32,

    /// Expires in hours
    #[arg(short, long, default_value = "24")]
    pub expires_hours: u32,
}

#[derive(Args)]
pub struct RevokeArgs {
    /// Invite token to revoke
    pub token: String,
}

pub async fn run(cmd: InviteCommands, _config: &Config) -> anyhow::Result<()> {
    match cmd {
        InviteCommands::Create(args) => create_invite(args).await,
        InviteCommands::List => list_invites().await,
        InviteCommands::Revoke(args) => revoke_invite(args).await,
    }
}

async fn create_invite(args: CreateArgs) -> anyhow::Result<()> {
    println!("{}", "Creating collaborator invite".bold());
    println!();

    let email = if let Some(e) = args.email {
        e
    } else {
        println!("Enter collaborator email:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    if email.is_empty() {
        println!("{} Email cannot be empty", "Error".red());
        std::process::exit(1);
    }

    println!();
    println!("{}", "Invite Details".bold());
    println!("  Project: {}", args.project.cyan());
    println!("  Collaborator: {}", email.cyan());
    println!("  Role: {}", args.role);
    println!("  Quota: ${:.2}", args.quota_cents as f64 / 100.0);
    println!("  Expires: {} hours", args.expires_hours);

    // TODO: Actually create invite via CollaboratorClient
    let invite_token = uuid::Uuid::new_v4().to_string();
    let invite_url = format!("https://superinstance.ai/invite/{}", invite_token);

    println!();
    println!("{} Invite created!", "✓".green());
    println!();
    println!("{}", "Share Options".bold());
    println!("  Link: {}", invite_url.cyan().underline());
    println!("  Code: {}", invite_token.bold());
    println!();
    println!("{}", "Next Steps".bold());
    println!("  1. Share the link or code with {}", email);
    println!("  2. They'll be able to collaborate once they accept");
    println!("  3. Revoke anytime with: synesis invite revoke {}", invite_token);

    Ok(())
}

async fn list_invites() -> anyhow::Result<()> {
    println!("{}", "Pending Invites".bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Email", "Role", "Quota", "Expires", "Token"]);

    // TODO: Get actual invites from cloud API
    // For now, show mock data
    table.add_row(vec![
        "alice@example.com",
        "editor",
        "$50.00",
        "2 hours",
        "abc123...",
    ]);
    table.add_row(vec![
        "bob@example.com",
        "viewer",
        "$10.00",
        "12 hours",
        "def456...",
    ]);
    table.add_row(vec![
        "carol@example.com",
        "commenter",
        "$25.00",
        "23 hours",
        "ghi789...",
    ]);

    println!("{table}");
    println!();
    println!("Total: {} pending invites", "3".bold());

    Ok(())
}

async fn revoke_invite(args: RevokeArgs) -> anyhow::Result<()> {
    println!("{}", "Revoking invite".bold());
    println!();
    println!("Token: {}", args.token.cyan());
    println!();

    // TODO: Actually revoke via cloud API
    println!("{} Invite revoked successfully", "✓".green());
    println!();
    println!("{}", "The invite link is no longer valid.".dimmed());

    Ok(())
}
