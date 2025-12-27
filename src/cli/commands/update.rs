//! velocity update - Update packages

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use clap::Args;

use crate::cli::output;
use crate::core::{Engine, VelocityResult};

#[derive(Args)]
pub struct UpdateArgs {
    /// Specific packages to update (all if omitted)
    pub packages: Vec<String>,

    /// Update to latest versions (ignore version constraints)
    #[arg(long)]
    pub latest: bool,

    /// Project directory
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,

    /// Dry run - show what would be updated
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn execute(args: UpdateArgs, json_output: bool) -> VelocityResult<()> {
    let start_time = Instant::now();

    let project_dir = if args.cwd.is_absolute() {
        args.cwd.clone()
    } else {
        env::current_dir()?.join(&args.cwd)
    };

    let engine = Engine::new(&project_dir).await?;
    engine.ensure_initialized()?;

    let mut package_json = engine.package_json()?;
    let existing_lockfile = engine.lockfile()?;

    if !json_output {
        if args.packages.is_empty() {
            output::info("Checking for updates...");
        } else {
            output::info(&format!("Updating {} package(s)...", args.packages.len()));
        }
    }

    let progress = if !json_output {
        Some(output::spinner("Resolving versions..."))
    } else {
        None
    };

    let mut updates = Vec::new();

    // Get packages to update
    let packages_to_check: Vec<String> = if args.packages.is_empty() {
        package_json.all_dependencies().keys().cloned().collect()
    } else {
        args.packages.clone()
    };

    // Check for updates
    for name in &packages_to_check {
        let current_version = package_json.dependencies.get(name)
            .or_else(|| package_json.dev_dependencies.get(name))
            .or_else(|| package_json.optional_dependencies.get(name));

        if let Some(current) = current_version {
            let metadata = engine.registry.get_package_metadata(name).await?;
            let latest = metadata.dist_tags.get("latest").cloned().unwrap_or_default();

            // Check if update is available
            let current_semver = extract_version(current);
            if latest != current_semver {
                updates.push((name.clone(), current.clone(), latest.clone()));

                if args.latest {
                    // Update to latest
                    let new_version = format!("^{}", latest);
                    if package_json.dependencies.contains_key(name) {
                        package_json.dependencies.insert(name.clone(), new_version);
                    } else if package_json.dev_dependencies.contains_key(name) {
                        package_json.dev_dependencies.insert(name.clone(), new_version);
                    } else if package_json.optional_dependencies.contains_key(name) {
                        package_json.optional_dependencies.insert(name.clone(), new_version);
                    }
                }
            }
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    if updates.is_empty() {
        if json_output {
            output::json(&serde_json::json!({
                "success": true,
                "updates": [],
                "message": "All packages are up to date"
            }))?;
        } else {
            output::success("All packages are up to date!");
        }
        return Ok(());
    }

    if args.dry_run {
        if json_output {
            output::json(&serde_json::json!({
                "success": true,
                "dry_run": true,
                "updates": updates.iter().map(|(name, from, to)| serde_json::json!({
                    "name": name,
                    "from": from,
                    "to": to
                })).collect::<Vec<_>>()
            }))?;
        } else {
            output::info("Available updates (dry run):");
            for (name, from, to) in &updates {
                println!(
                    "  {} {} → {}",
                    console::style(name).cyan(),
                    console::style(from).red(),
                    console::style(to).green()
                );
            }
        }
        return Ok(());
    }

    // Apply updates
    package_json.save(&project_dir)?;

    let progress = if !json_output {
        Some(output::spinner("Installing updates..."))
    } else {
        None
    };

    // Reinstall
    let deps = package_json.all_dependencies();
    let resolver = engine.resolver();
    let resolution = resolver.resolve(&deps).await?;

    let installer = engine.installer();
    installer.install(&resolution, false, false).await?;
    installer.link(&resolution).await?;

    let mut lockfile = resolution.lockfile;
    lockfile.save(&project_dir)?;

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let duration = start_time.elapsed();

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "updates": updates.iter().map(|(name, from, to)| serde_json::json!({
                "name": name,
                "from": from,
                "to": to
            })).collect::<Vec<_>>(),
            "duration_ms": duration.as_millis()
        }))?;
    } else {
        output::success(&format!("Updated {} package(s)", updates.len()));
        for (name, from, to) in &updates {
            println!(
                "  {} {} → {}",
                console::style(name).cyan(),
                console::style(from).red(),
                console::style(to).green()
            );
        }
        output::info(&format!(
            "Completed in {}",
            output::format_duration(duration.as_millis())
        ));
    }

    Ok(())
}

/// Extract the actual version from a constraint (^1.0.0 -> 1.0.0)
fn extract_version(constraint: &str) -> String {
    constraint
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches('=')
        .trim_start_matches('>')
        .trim_start_matches('<')
        .to_string()
}
