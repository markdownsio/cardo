use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cardo")]
#[command(about = "A CLI tool for managing Markdown file dependencies", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Cardo project
    Init {
        /// Project name (default: current directory name)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Fetch all dependencies from markdown.toml
    Fetch {
        /// Force re-download even if files exist
        #[arg(short, long)]
        force: bool,
    },
    /// Update dependencies (same as fetch for now)
    Update {
        /// Force re-download even if files exist
        #[arg(short, long)]
        force: bool,
    },
    /// List all dependencies
    List,
    /// Clean the markdowns directory
    Clean,
}
