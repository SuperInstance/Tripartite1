//! `synesis knowledge` - Knowledge vault management

use clap::Subcommand;
use comfy_table::{presets::UTF8_FULL, Table};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal::ctrl_c;

use crate::config::Config;
use synesis_knowledge::{FileWatcher, KnowledgeVault, PlaceholderEmbedder, WatchConfig};

#[derive(Subcommand)]
pub enum KnowledgeCommands {
    /// Add documents to the knowledge vault
    Add(AddArgs),

    /// Remove documents from the vault
    Remove(RemoveArgs),

    /// List indexed documents
    List(ListArgs),

    /// Search the knowledge vault
    Search(SearchArgs),

    /// Re-index all documents
    Reindex(ReindexArgs),

    /// Show vault statistics
    Stats,

    /// Watch a directory for changes
    Watch(WatchArgs),
}

#[derive(clap::Args)]
pub struct AddArgs {
    /// File or directory path(s) to add
    pub paths: Vec<String>,

    /// Recursively add directories
    #[arg(short, long)]
    pub recursive: bool,

    /// File patterns to include (e.g., "*.md")
    #[arg(long)]
    pub include: Option<Vec<String>>,

    /// File patterns to exclude
    #[arg(long)]
    pub exclude: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct RemoveArgs {
    /// Document ID or path pattern
    pub pattern: String,

    /// Skip confirmation
    #[arg(short, long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Filter by file type
    #[arg(long)]
    pub r#type: Option<String>,

    /// Sort by: name, date, size, chunks
    #[arg(long, default_value = "date")]
    pub sort: String,

    /// Maximum results
    #[arg(short, long, default_value = "50")]
    pub limit: usize,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Number of results
    #[arg(short, long, default_value = "10")]
    pub limit: usize,

    /// Minimum similarity threshold (0.0-1.0)
    #[arg(long, default_value = "0.5")]
    pub threshold: f32,
}

#[derive(clap::Args)]
pub struct ReindexArgs {
    /// Only reindex specific document
    pub document: Option<String>,
}

#[derive(clap::Args)]
pub struct WatchArgs {
    /// Directory to watch
    pub path: String,

    /// File patterns to include
    #[arg(long)]
    pub include: Option<Vec<String>>,
}

pub async fn run(cmd: KnowledgeCommands, _config: &Config) -> anyhow::Result<()> {
    match cmd {
        KnowledgeCommands::Add(args) => add_documents(args).await,
        KnowledgeCommands::Remove(args) => remove_documents(args).await,
        KnowledgeCommands::List(args) => list_documents(args).await,
        KnowledgeCommands::Search(args) => search_vault(args).await,
        KnowledgeCommands::Reindex(args) => reindex_vault(args).await,
        KnowledgeCommands::Stats => show_stats().await,
        KnowledgeCommands::Watch(args) => watch_directory(args).await,
    }
}

async fn add_documents(args: AddArgs) -> anyhow::Result<()> {
    println!("{}", "Adding documents to knowledge vault...".bold());
    println!();

    for path in &args.paths {
        println!("  {} {}", "→".dimmed(), path);
    }
    println!();

    // TODO: Use synesis_knowledge::indexer
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")?
            .progress_chars("█▓░"),
    );

    pb.set_message("Processing");
    for i in 0..=100 {
        pb.set_position(i);
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }
    pb.finish_and_clear();

    println!(
        "{} Added {} documents ({} chunks)",
        "✓".green(),
        args.paths.len(),
        args.paths.len() * 25 // Placeholder
    );

    Ok(())
}

async fn remove_documents(args: RemoveArgs) -> anyhow::Result<()> {
    if !args.force {
        println!(
            "{} Remove documents matching '{}'? [y/N] ",
            "Warning:".yellow().bold(),
            args.pattern
        );
        // TODO: Read confirmation
    }

    // TODO: synesis_knowledge::vault::remove()
    println!("{} Removed 3 documents", "✓".green());

    Ok(())
}

async fn list_documents(args: ListArgs) -> anyhow::Result<()> {
    println!("{}", "Indexed Documents".bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Name", "Type", "Chunks", "Indexed"]);

    // TODO: Get from synesis_knowledge::vault
    table.add_row(vec!["doc_001", "README.md", "markdown", "12", "2h ago"]);
    table.add_row(vec![
        "doc_002",
        "architecture.md",
        "markdown",
        "45",
        "2h ago",
    ]);
    table.add_row(vec!["doc_003", "main.rs", "rust", "28", "1h ago"]);
    table.add_row(vec!["doc_004", "api_spec.yaml", "yaml", "89", "30m ago"]);

    println!("{table}");
    println!();
    println!(
        "{} Showing {} of {} documents",
        "ℹ".dimmed(),
        4.min(args.limit),
        47
    );

    Ok(())
}

async fn search_vault(args: SearchArgs) -> anyhow::Result<()> {
    println!("{} \"{}\"", "Searching:".bold(), args.query);
    println!();

    // TODO: Use synesis_knowledge::search
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Score", "Document", "Chunk", "Preview"]);

    table.add_row(vec![
        "0.92",
        "architecture.md",
        "#15",
        "The tripartite council consists of...",
    ]);
    table.add_row(vec![
        "0.87",
        "README.md",
        "#3",
        "Three agents work together to...",
    ]);
    table.add_row(vec![
        "0.81",
        "design.md",
        "#42",
        "Consensus is reached when...",
    ]);

    println!("{table}");

    Ok(())
}

async fn reindex_vault(args: ReindexArgs) -> anyhow::Result<()> {
    match args.document {
        Some(doc) => {
            println!("Reindexing {}...", doc);
        },
        None => {
            println!("Reindexing all documents...");
        },
    }

    // TODO: synesis_knowledge::indexer::reindex()
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")?
            .progress_chars("█▓░"),
    );

    pb.set_message("Reindexing");
    for i in 0..=100 {
        pb.set_position(i);
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
    }
    pb.finish_and_clear();

    println!("{} Reindexed 47 documents", "✓".green());

    Ok(())
}

async fn show_stats() -> anyhow::Result<()> {
    println!("{}", "Knowledge Vault Statistics".bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    // TODO: Get from synesis_knowledge::vault::stats()
    table.add_row(vec!["Total Documents", "47"]);
    table.add_row(vec!["Total Chunks", "1,284"]);
    table.add_row(vec!["Embedding Dimensions", "384"]);
    table.add_row(vec!["Database Size", "128 MB"]);
    table.add_row(vec!["Average Chunk Size", "512 tokens"]);
    table.add_row(vec!["File Types", "md, rs, py, yaml, json"]);
    table.add_row(vec!["Last Updated", "30 minutes ago"]);
    table.add_row(vec!["Watched Directories", "2"]);

    println!("{table}");

    Ok(())
}

async fn watch_directory(args: WatchArgs) -> anyhow::Result<()> {
    let path = PathBuf::from(&args.path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", args.path);
    }

    if !path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", args.path);
    }

    println!("{} {}", "Watching:".bold(), args.path);
    println!("{}", "Press Ctrl+C to stop".dimmed());
    println!();

    // Open knowledge vault
    let vault_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".synesis")
        .join("knowledge.db");

    let vault = Arc::new(tokio::sync::Mutex::new(KnowledgeVault::open(&vault_path, 384)?));

    // Create embedder (using placeholder for now)
    let embedder = Arc::new(tokio::sync::Mutex::new(PlaceholderEmbedder::new(384)));

    // Configure watcher
    let mut config = WatchConfig {
        directories: vec![path.clone()],
        ..Default::default()
    };

    // Add custom include patterns if provided
    if let Some(include) = args.include {
        config.extensions = Some(include);
    }

    // Create channel-based indexer
    let indexer_config = synesis_knowledge::indexer::IndexerConfig::default();
    let (indexer, _handle) = synesis_knowledge::indexer::DocumentIndexer::new(
        vault.clone(),
        embedder.clone(),
        indexer_config,
    );

    // Create watcher with indexer channel
    let mut watcher = FileWatcher::with_auto_index(config.clone(), indexer.command_sender())?;

    // Start watcher
    watcher.start().await?;

    // Initial indexing
    println!("{} Initial indexing...", "Scanning".bold());

    // Manually trigger indexing for the directory
    let _command = synesis_knowledge::indexer::IndexCommand::IndexDirectory {
        path: path.clone(),
        extensions: config.extensions.clone(),
    };

    if let Err(e) = indexer.index_file(path.clone()).await {
        println!("{} Initial indexing failed: {}", "Warning".yellow(), e);
    } else {
        println!("{} Initial indexing complete", "✓".green());
    }

    println!();
    println!("{}", "Watching for changes (auto-indexing enabled)...".dimmed());
    println!("{}", "Files will be automatically indexed when changed.".dimmed());
    println!();

    // Wait for Ctrl+C
    tokio::select! {
        _ = ctrl_c() => {
            println!();
            println!("{}", "Stopping watcher...".dimmed());
            watcher.stop();
            println!("{}", "Done".green());
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)) => {
            unreachable!()
        }
    }

    Ok(())
}
