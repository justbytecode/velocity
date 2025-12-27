//! velocity cache - Manage the package cache

use std::env;
use std::path::PathBuf;
use clap::{Args, Subcommand};

use crate::cli::output;
use crate::core::{Config, VelocityResult};

#[derive(Args)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache location and size
    Info,

    /// Clean the entire cache
    Clean {
        /// Force clean without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List cached packages
    List {
        /// Filter by package name
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Verify cache integrity
    Verify,
}

pub async fn execute(args: CacheArgs, json_output: bool) -> VelocityResult<()> {
    let project_dir = env::current_dir()?;
    let config = Config::load(&project_dir)?;
    let cache_dir = config.cache_dir()?;

    match args.command {
        CacheCommands::Info => info(&cache_dir, json_output).await,
        CacheCommands::Clean { force } => clean(&cache_dir, force, json_output).await,
        CacheCommands::List { filter } => list(&cache_dir, filter, json_output).await,
        CacheCommands::Verify => verify(&cache_dir, json_output).await,
    }
}

async fn info(cache_dir: &PathBuf, json_output: bool) -> VelocityResult<()> {
    let size = calculate_dir_size(cache_dir)?;
    let package_count = count_packages(cache_dir)?;

    if json_output {
        output::json(&serde_json::json!({
            "path": cache_dir,
            "size_bytes": size,
            "size_human": output::format_bytes(size),
            "package_count": package_count
        }))?;
    } else {
        output::info("Cache Information");
        output::divider();
        println!("  Path: {}", cache_dir.display());
        println!("  Size: {}", output::format_bytes(size));
        println!("  Packages: {}", package_count);
    }

    Ok(())
}

async fn clean(cache_dir: &PathBuf, force: bool, json_output: bool) -> VelocityResult<()> {
    if !cache_dir.exists() {
        if json_output {
            output::json(&serde_json::json!({
                "success": true,
                "message": "Cache is already empty"
            }))?;
        } else {
            output::info("Cache is already empty");
        }
        return Ok(());
    }

    let size = calculate_dir_size(cache_dir)?;

    if !force && !json_output {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!(
                "Delete cache ({})? This cannot be undone.",
                output::format_bytes(size)
            ))
            .default(false)
            .interact()?;

        if !confirm {
            output::info("Cancelled");
            return Ok(());
        }
    }

    // Remove cache contents
    if cache_dir.exists() {
        std::fs::remove_dir_all(cache_dir)?;
        std::fs::create_dir_all(cache_dir)?;
    }

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "freed_bytes": size,
            "freed_human": output::format_bytes(size)
        }))?;
    } else {
        output::success(&format!("Cleared {} from cache", output::format_bytes(size)));
    }

    Ok(())
}

async fn list(cache_dir: &PathBuf, filter: Option<String>, json_output: bool) -> VelocityResult<()> {
    let packages = list_cached_packages(cache_dir, filter.as_deref())?;

    if json_output {
        output::json(&serde_json::json!({
            "packages": packages.iter().map(|(name, versions)| serde_json::json!({
                "name": name,
                "versions": versions
            })).collect::<Vec<_>>()
        }))?;
    } else {
        if packages.is_empty() {
            output::info("No packages in cache");
        } else {
            output::info(&format!("{} packages in cache:", packages.len()));
            for (name, versions) in &packages {
                println!(
                    "  {} ({})",
                    console::style(name).cyan(),
                    versions.join(", ")
                );
            }
        }
    }

    Ok(())
}

async fn verify(cache_dir: &PathBuf, json_output: bool) -> VelocityResult<()> {
    let progress = if !json_output {
        Some(output::spinner("Verifying cache integrity..."))
    } else {
        None
    };

    let mut verified = 0;
    let mut failed = 0;
    let mut errors: Vec<String> = Vec::new();

    // Walk the cache directory and verify integrity
    if cache_dir.exists() {
        for entry in walkdir::WalkDir::new(cache_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                
                // Check if file is readable
                if std::fs::read(path).is_ok() {
                    verified += 1;
                } else {
                    failed += 1;
                    errors.push(format!("Cannot read: {}", path.display()));
                }
            }
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    if json_output {
        output::json(&serde_json::json!({
            "success": failed == 0,
            "verified": verified,
            "failed": failed,
            "errors": errors
        }))?;
    } else {
        if failed == 0 {
            output::success(&format!("Verified {} cached files", verified));
        } else {
            output::warning(&format!(
                "Verified {} files, {} failed",
                verified, failed
            ));
            for error in errors.iter().take(10) {
                println!("  {}", console::style(error).red());
            }
            if errors.len() > 10 {
                println!("  ... and {} more", errors.len() - 10);
            }
        }
    }

    Ok(())
}

fn calculate_dir_size(path: &PathBuf) -> std::io::Result<u64> {
    let mut size = 0;
    if path.exists() && path.is_dir() {
        for entry in walkdir::WalkDir::new(path) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
    }
    Ok(size)
}

fn count_packages(cache_dir: &PathBuf) -> std::io::Result<usize> {
    let content_dir = cache_dir.join("content");
    if !content_dir.exists() {
        return Ok(0);
    }

    let count = std::fs::read_dir(&content_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count();

    Ok(count)
}

fn list_cached_packages(cache_dir: &PathBuf, filter: Option<&str>) -> std::io::Result<Vec<(String, Vec<String>)>> {
    let content_dir = cache_dir.join("content");
    if !content_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packages: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for entry in std::fs::read_dir(&content_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Apply filter
            if let Some(f) = filter {
                if !name.contains(f) {
                    continue;
                }
            }

            // Get versions
            let mut versions = Vec::new();
            for version_entry in std::fs::read_dir(entry.path())? {
                let version_entry = version_entry?;
                if version_entry.file_type()?.is_dir() {
                    versions.push(version_entry.file_name().to_string_lossy().to_string());
                }
            }

            if !versions.is_empty() {
                packages.insert(name, versions);
            }
        }
    }

    let mut result: Vec<_> = packages.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
