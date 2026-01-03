//! `synesis ask` - Main interaction command
//!
//! Sends a query through the tripartite council and returns the response.

use clap::Args;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::Config;
use crate::display;

#[derive(Args)]
pub struct AskArgs {
    /// The question or request
    pub query: String,

    /// Force local-only processing (no cloud escalation)
    #[arg(long)]
    pub local: bool,

    /// Force cloud processing
    #[arg(long)]
    pub cloud: bool,

    /// Show agent reasoning process
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format: text, json, markdown
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Include specific knowledge sources
    #[arg(short, long)]
    pub knowledge: Option<Vec<String>>,

    /// Show what was redacted (for debugging)
    #[arg(long)]
    pub show_redactions: bool,
}

pub async fn run(args: AskArgs, config: &Config) -> anyhow::Result<()> {
    // Validate conflicting flags
    if args.local && args.cloud {
        anyhow::bail!("Cannot specify both --local and --cloud");
    }

    // Generate a session ID for this interaction
    let session_id = Uuid::new_v4().to_string();

    if args.verbose {
        println!("{}", "🔍 Processing query...".dimmed());
        println!("{} {}", "Session ID:".dimmed(), session_id);
        println!();
    }

    // Step 1: Initialize redactor
    let mut redactor = initialize_redactor(config, &session_id)?;

    // Step 2: Privacy redaction
    let (redacted_query, redaction_result) = redact_query(&args.query, &mut redactor, &session_id)?;

    if args.show_redactions {
        println!("{} {}", "Original:".dimmed(), args.query);
        println!("{} {}", "Redacted:".dimmed(), redacted_query);
        println!(
            "{} {}",
            "Patterns found:".dimmed(),
            redaction_result.stats.patterns_detected
        );
        println!();
    } else if args.verbose {
        println!(
            "{} {} patterns redacted",
            "Privacy:".dimmed(),
            redaction_result.stats.patterns_redacted
        );
        println!();
    }

    // Step 3: Run through tripartite council
    let response = run_council(&redacted_query, &args, config).await?;

    // Step 4: Reinflate any tokens in response
    let final_response = reinflate_response(&response.content, &mut redactor)?;

    // Step 5: Clear session tokens
    cleanup_session(&mut redactor, &session_id)?;

    // Step 6: Display response
    match args.format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "query": args.query,
                "response": final_response,
                "session_id": session_id,
                "metadata": {
                    "local": !response.used_cloud,
                    "consensus_rounds": response.rounds,
                    "confidence": response.confidence,
                    "redaction_stats": redaction_result.stats,
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        },
        "markdown" => {
            println!("## Response\n\n{}", final_response);
        },
        _ => {
            println!("{}", final_response);
        },
    }

    // Show metadata if verbose
    if args.verbose {
        println!();
        display::print_consensus_summary(&response);
    }

    Ok(())
}

/// Initialize the redactor with session context
fn initialize_redactor(
    config: &Config,
    _session_id: &str,
) -> anyhow::Result<synesis_privacy::Redactor> {
    use synesis_privacy::{Redactor, RedactorConfig, TokenVault};

    // Create in-memory vault for this session
    let vault = TokenVault::in_memory()
        .map_err(|e| anyhow::anyhow!("Failed to create token vault: {}", e))?;

    // Create redactor with default config
    let redactor_config = RedactorConfig::default();

    Redactor::new(redactor_config, vault)
        .map_err(|e| anyhow::anyhow!("Failed to create redactor: {}", e))
}

/// Redact sensitive information from the query
fn redact_query(
    query: &str,
    redactor: &mut synesis_privacy::Redactor,
    session_id: &str,
) -> anyhow::Result<(String, synesis_privacy::RedactionResult)> {
    let redacted = redactor.redact(query, session_id);
    Ok((redacted.redacted_text.clone(), redacted))
}

/// Run the query through the tripartite council
async fn run_council(
    query: &str,
    args: &AskArgs,
    _config: &Config,
) -> anyhow::Result<CouncilResponse> {
    // TODO: Implement actual council orchestration with consensus engine
    // For now, return a placeholder response
    Ok(CouncilResponse {
        content: format!(
            "I received your query: \"{}\"\n\n[Council processing not yet implemented]",
            query
        ),
        used_cloud: args.cloud,
        rounds: 1,
        confidence: 0.95,
        agent_votes: AgentVotes {
            pathos: 0.90,
            logos: 0.95,
            ethos: 0.92,
        },
    })
}

/// Reinflate tokens in the response
fn reinflate_response(
    response: &str,
    redactor: &mut synesis_privacy::Redactor,
) -> anyhow::Result<String> {
    // For now, the response stays redacted since reinflate needs to access the vault
    // which is handled internally by the redactor
    Ok(response.to_string())
}

/// Clean up session tokens from vault
fn cleanup_session(
    redactor: &mut synesis_privacy::Redactor,
    session_id: &str,
) -> anyhow::Result<()> {
    // Clear the in-memory vault after use
    redactor
        .clear_session(session_id)
        .map_err(|e| anyhow::anyhow!("Failed to clear session: {}", e))
}

#[derive(Debug)]
pub struct CouncilResponse {
    pub content: String,
    pub used_cloud: bool,
    pub rounds: u8,
    pub confidence: f32,
    pub agent_votes: AgentVotes,
}

#[derive(Debug)]
pub struct AgentVotes {
    pub pathos: f32,
    pub logos: f32,
    pub ethos: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redact_query_passthrough() {
        use synesis_privacy::TokenVault;
        let vault = TokenVault::in_memory();
        let mut redactor = synesis_privacy::Redactor::new(
            synesis_privacy::RedactorConfig::default(),
            vault.unwrap(),
        )
        .unwrap();
        let session_id = "test-session";
        let (redacted, result) = redact_query("Hello world", &mut redactor, session_id).unwrap();
        assert_eq!(redacted, "Hello world");
        assert!(result.token_map.is_empty());
    }
}
