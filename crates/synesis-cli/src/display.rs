//! Display utilities for CLI output

use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use owo_colors::OwoColorize;

use crate::commands::ask::CouncilResponse;

/// Print a consensus summary after a query
pub fn print_consensus_summary(response: &CouncilResponse) {
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "Consensus Summary".bold());
    println!();

    // Processing location
    let location = if response.used_cloud {
        "☁️  Cloud".cyan().to_string()
    } else {
        "💻 Local".green().to_string()
    };
    println!("  {} {}", "Processing:".dimmed(), location);

    // Rounds
    println!("  {} {}/3", "Rounds:".dimmed(), response.rounds);

    // Overall confidence
    let confidence_bar = render_confidence_bar(response.confidence, 20);
    println!(
        "  {} {} ({:.0}%)",
        "Confidence:".dimmed(),
        confidence_bar,
        response.confidence * 100.0
    );
    println!();

    // Agent votes
    println!("  {}", "Agent Votes:".dimmed());
    print_agent_vote("Pathos", response.agent_votes.pathos, "🎭");
    print_agent_vote("Logos", response.agent_votes.logos, "🔬");
    print_agent_vote("Ethos", response.agent_votes.ethos, "⚖️");
}

fn print_agent_vote(name: &str, score: f32, emoji: &str) {
    let bar = render_confidence_bar(score, 15);
    let score_colored = if score >= 0.9 {
        format!("{:.0}%", score * 100.0).green().to_string()
    } else if score >= 0.7 {
        format!("{:.0}%", score * 100.0).yellow().to_string()
    } else {
        format!("{:.0}%", score * 100.0).red().to_string()
    };

    println!("    {} {:8} {} {}", emoji, name, bar, score_colored);
}

/// Render a progress/confidence bar
pub fn render_confidence_bar(value: f32, width: usize) -> String {
    let filled = ((value * width as f32) as usize).min(width);
    let empty = width - filled;

    let bar = format!(
        "{}{}",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed()
    );

    format!("[{}]", bar)
}

/// Print a formatted error message
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}

/// Print a formatted warning message  
pub fn print_warning(msg: &str) {
    eprintln!("{} {}", "Warning:".yellow().bold(), msg);
}

/// Print a formatted success message
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

/// Print a formatted info message
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Create a styled table with standard preset
pub fn create_table() -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table
}

/// Format bytes as human readable
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human readable
pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Format a timestamp as relative time
pub fn format_relative_time(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} min ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else {
        timestamp.format("%Y-%m-%d").to_string()
    }
}

/// Print streaming response chunks
pub struct StreamingDisplay {
    chars_printed: usize,
}

impl StreamingDisplay {
    pub fn new() -> Self {
        Self { chars_printed: 0 }
    }

    pub fn print_chunk(&mut self, chunk: &str) {
        print!("{}", chunk);
        self.chars_printed += chunk.len();
        // Flush to ensure immediate display
        use std::io::Write;
        std::io::stdout().flush().ok();
    }

    pub fn finish(&self) {
        println!();
    }
}

impl Default for StreamingDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3700), "1h 1m");
    }

    #[test]
    fn test_confidence_bar() {
        let bar = render_confidence_bar(0.5, 10);
        assert!(bar.contains('['));
        assert!(bar.contains(']'));
    }
}
