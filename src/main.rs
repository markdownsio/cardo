mod cli;
mod config;
mod dependency;
mod fetcher;
mod github;
mod utils;

use anyhow::{Context, Result};
use cli::{Cli, Commands};
use config::ConfigError;
use std::env;
use std::path::Path;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::try_parse()?;

    match cli.command {
        Commands::Init { name } => {
            handle_init(name).await?;
        }
        Commands::Fetch { force } => {
            handle_fetch(force).await?;
        }
        Commands::Update { force } => {
            handle_update(force).await?;
        }
        Commands::List => {
            handle_list().await?;
        }
        Commands::Clean => {
            handle_clean().await?;
        }
    }

    Ok(())
}

async fn handle_init(name: Option<String>) -> Result<()> {
    let config_file = "markdown.toml";

    if Path::new(config_file).exists() {
        anyhow::bail!("markdown.toml already exists");
    }

    let project_name = name.unwrap_or_else(|| {
        env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-project".to_string())
    });

    let config = config::MarkdownConfig::default_template(&project_name);
    let toml_content = config.to_toml_string()?;

    std::fs::write(config_file, toml_content)?;
    println!("Created {}", config_file);

    Ok(())
}

async fn handle_fetch(force: bool) -> Result<()> {
    let config_file = utils::find_config_file()
        .ok_or_else(|| ConfigError::NotFound("markdown.toml".to_string()))?;

    let config = config::MarkdownConfig::from_file(&config_file)
        .context("Failed to load markdown.toml")?;

    let dependencies = config
        .parse_dependencies()
        .context("Failed to parse dependencies")?;

    if dependencies.is_empty() {
        println!("No dependencies found in markdown.toml");
        return Ok(());
    }

    utils::ensure_output_dir("markdowns")?;

    let github_token = env::var("GITHUB_TOKEN").ok();
    let fetcher = fetcher::Fetcher::new("markdowns".to_string(), github_token);

    println!("Fetching {} dependencies...", dependencies.len());
    let results = fetcher.fetch_all(&dependencies, force).await?;

    let mut success_count = 0;
    let mut fail_count = 0;

    for result in &results {
        if result.success {
            success_count += 1;
            println!("  ✓ {} -> {}", result.name, result.path);
        } else {
            fail_count += 1;
            println!(
                "  ✗ {}: {}",
                result.name,
                result.error.as_deref().unwrap_or("Unknown error")
            );
        }
    }

    println!("\nSummary: {} succeeded, {} failed", success_count, fail_count);

    if fail_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_update(force: bool) -> Result<()> {
    // Update 和 Fetch 目前功能相同
    handle_fetch(force).await
}

async fn handle_list() -> Result<()> {
    let config_file = utils::find_config_file()
        .ok_or_else(|| ConfigError::NotFound("markdown.toml".to_string()))?;

    let config = config::MarkdownConfig::from_file(&config_file)
        .context("Failed to load markdown.toml")?;

    let dependencies = config
        .parse_dependencies()
        .context("Failed to parse dependencies")?;

    if dependencies.is_empty() {
        println!("No dependencies found in markdown.toml");
        return Ok(());
    }

    println!("Dependencies:");
    for (name, source) in &dependencies {
        match source {
            dependency::DependencySource::GitHub {
                owner,
                repo,
                path,
                version,
            } => {
                let version_str = match version {
                    Some(dependency::Version::Tag(t)) => format!("tag:{}", t),
                    Some(dependency::Version::Branch(b)) => format!("branch:{}", b),
                    Some(dependency::Version::Commit(c)) => format!("commit:{}", c),
                    None => "main".to_string(),
                };
                println!(
                    "  {}: github:{}/{}/{} ({})",
                    name, owner, repo, path, version_str
                );
            }
            dependency::DependencySource::Url(url) => {
                println!("  {}: {}", name, url);
            }
        }
    }

    Ok(())
}

async fn handle_clean() -> Result<()> {
    let fetcher = fetcher::Fetcher::new("markdowns".to_string(), None);
    fetcher.clean().await?;
    println!("Cleaned markdowns/ directory");
    Ok(())
}
