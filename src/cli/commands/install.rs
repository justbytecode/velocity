//! velocity install - Install all dependencies

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use clap::Args;

use crate::cli::output;
use crate::core::{Engine, VelocityResult};

#[derive(Args)]
pub struct InstallArgs {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Only install production dependencies
    #[arg(long)]
    pub production: bool,

    /// Skip running install scripts
    #[arg(long)]
    pub ignore_scripts: bool,

    /// Force reinstall all packages
    #[arg(short, long)]
    pub force: bool,

    /// Install in workspace mode
    #[arg(short, long)]
    pub workspace: bool,

    /// Prefer offline mode (use cache when possible)
    #[arg(long)]
    pub prefer_offline: bool,

    /// Frozen lockfile mode (fail if lockfile needs update)
    #[arg(long)]
    pub frozen_lockfile: bool,
}

pub async fn execute(args: InstallArgs, json_output: bool) -> VelocityResult<()> {
    let start_time = Instant::now();

    let project_dir = if args.path.is_absolute() {
        args.path.clone()
    } else {
        env::current_dir()?.join(&args.path)
    };

    let engine = Engine::new(&project_dir).await?;
    engine.ensure_initialized()?;

    let package_json = engine.package_json()?;
    let existing_lockfile = engine.lockfile()?;

    if !json_output {
        output::info(&format!("Installing dependencies for '{}'...", package_json.name));
    }

    // Get dependencies to install
    let deps = if args.production {
        package_json.production_dependencies()
    } else {
        package_json.all_dependencies()
    };

    if deps.is_empty() {
        if json_output {
            output::json(&serde_json::json!({
                "success": true,
                "installed": 0,
                "duration_ms": start_time.elapsed().as_millis()
            }))?;
        } else {
            output::success("No dependencies to install");
        }
        return Ok(());
    }

    // Show progress
    let progress = if !json_output {
        Some(output::spinner("Resolving dependencies..."))
    } else {
        None
    };

    // Resolve dependencies
    let resolver = engine.resolver();
    let resolution = resolver.resolve(&deps).await?;

    if let Some(ref pb) = progress {
        pb.set_message("Downloading packages...");
    }

    // Check frozen lockfile mode
    if args.frozen_lockfile {
        if let Some(ref existing) = existing_lockfile {
            let diff = existing.diff(&resolution.lockfile);
            if !diff.is_empty() {
                if let Some(pb) = progress {
                    pb.finish_and_clear();
                }
                return Err(crate::core::VelocityError::other(
                    "Lockfile is out of date. Run 'velocity install' without --frozen-lockfile to update."
                ));
            }
        } else {
            if let Some(pb) = progress {
                pb.finish_and_clear();
            }
            return Err(crate::core::VelocityError::other(
                "No lockfile found. Run 'velocity install' without --frozen-lockfile to generate one."
            ));
        }
    }

    // Install packages
    let installer = engine.installer();
    let install_result = installer.install(
        &resolution,
        args.force,
        args.prefer_offline,
    ).await?;

    if let Some(ref pb) = progress {
        pb.set_message("Linking packages...");
    }

    // Link packages to node_modules
    installer.link(&resolution).await?;

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    // Save lockfile
    let mut lockfile = resolution.lockfile;
    lockfile.save(&project_dir)?;

    // Run install scripts if not ignored
    if !args.ignore_scripts && !engine.config.security.allow_scripts {
        // Scripts are disabled by default for security
        if !json_output {
            output::warning("Install scripts are disabled by default. Use --ignore-scripts=false to enable.");
        }
    }

    let duration = start_time.elapsed();

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "installed": install_result.installed_count,
            "cached": install_result.cached_count,
            "duration_ms": duration.as_millis()
        }))?;
    } else {
        output::success(&format!(
            "Installed {} packages in {}",
            install_result.installed_count,
            output::format_duration(duration.as_millis())
        ));

        if install_result.cached_count > 0 {
            output::info(&format!("{} packages restored from cache", install_result.cached_count));
        }
    }

    Ok(())
}
