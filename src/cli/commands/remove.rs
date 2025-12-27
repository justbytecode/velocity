//! velocity remove - Remove packages

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use clap::Args;

use crate::cli::output;
use crate::core::{Engine, VelocityResult};

#[derive(Args)]
pub struct RemoveArgs {
    /// Packages to remove
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// Project directory
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,
}

pub async fn execute(args: RemoveArgs, json_output: bool) -> VelocityResult<()> {
    let start_time = Instant::now();

    let project_dir = if args.cwd.is_absolute() {
        args.cwd.clone()
    } else {
        env::current_dir()?.join(&args.cwd)
    };

    let engine = Engine::new(&project_dir).await?;
    engine.ensure_initialized()?;

    let mut package_json = engine.package_json()?;

    if !json_output {
        output::info(&format!("Removing {} package(s)...", args.packages.len()));
    }

    let mut removed_packages = Vec::new();

    for name in &args.packages {
        if package_json.remove_dependency(name) {
            removed_packages.push(name.clone());
        } else if !json_output {
            output::warning(&format!("Package '{}' not found in dependencies", name));
        }
    }

    if removed_packages.is_empty() {
        if json_output {
            output::json(&serde_json::json!({
                "success": true,
                "removed": []
            }))?;
        } else {
            output::info("No packages were removed");
        }
        return Ok(());
    }

    // Save package.json
    package_json.save(&project_dir)?;

    // Reinstall to update node_modules and lockfile
    let progress = if !json_output {
        Some(output::spinner("Updating dependencies..."))
    } else {
        None
    };

    let deps = package_json.all_dependencies();
    
    if !deps.is_empty() {
        let resolver = engine.resolver();
        let resolution = resolver.resolve(&deps).await?;

        let installer = engine.installer();
        installer.install(&resolution, false, false).await?;
        installer.link(&resolution).await?;

        let mut lockfile = resolution.lockfile;
        lockfile.save(&project_dir)?;
    } else {
        // Remove lockfile if no deps remain
        let lockfile_path = project_dir.join("velocity.lock");
        if lockfile_path.exists() {
            std::fs::remove_file(&lockfile_path)?;
        }

        // Clean node_modules
        let node_modules = project_dir.join("node_modules");
        if node_modules.exists() {
            std::fs::remove_dir_all(&node_modules)?;
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let duration = start_time.elapsed();

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "removed": removed_packages,
            "duration_ms": duration.as_millis()
        }))?;
    } else {
        for name in &removed_packages {
            output::success(&format!("Removed {}", console::style(name).cyan()));
        }

        output::info(&format!(
            "Completed in {}",
            output::format_duration(duration.as_millis())
        ));
    }

    Ok(())
}
