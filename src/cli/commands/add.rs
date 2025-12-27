//! velocity add - Add packages

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use clap::Args;

use crate::cli::output;
use crate::core::{Engine, VelocityResult};

#[derive(Args)]
pub struct AddArgs {
    /// Packages to add (name or name@version)
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// Add as dev dependency
    #[arg(short = 'D', long)]
    pub dev: bool,

    /// Add as peer dependency
    #[arg(short = 'P', long)]
    pub peer: bool,

    /// Add as optional dependency
    #[arg(short = 'O', long)]
    pub optional: bool,

    /// Add to specific workspace package
    #[arg(short, long)]
    pub workspace: Option<String>,

    /// Exact version (no ^ or ~)
    #[arg(short = 'E', long)]
    pub exact: bool,

    /// Project directory
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,
}

pub async fn execute(args: AddArgs, json_output: bool) -> VelocityResult<()> {
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
        output::info(&format!("Adding {} package(s)...", args.packages.len()));
    }

    let progress = if !json_output {
        Some(output::spinner("Resolving packages..."))
    } else {
        None
    };

    let mut added_packages = Vec::new();

    for package_spec in &args.packages {
        // Parse package@version format
        let (name, version_spec) = parse_package_spec(package_spec);

        // Resolve the package version
        let resolved_version = if let Some(v) = version_spec {
            v.to_string()
        } else {
            // Fetch latest version from registry
            let metadata = engine.registry.get_package_metadata(&name).await?;
            let latest = metadata.dist_tags.get("latest")
                .ok_or_else(|| crate::core::VelocityError::PackageNotFound(name.clone()))?;
            
            if args.exact {
                latest.clone()
            } else {
                format!("^{}", latest)
            }
        };

        // Add to appropriate dependency section
        if args.dev {
            package_json.dev_dependencies.insert(name.clone(), resolved_version.clone());
        } else if args.peer {
            package_json.peer_dependencies.insert(name.clone(), resolved_version.clone());
        } else if args.optional {
            package_json.optional_dependencies.insert(name.clone(), resolved_version.clone());
        } else {
            package_json.dependencies.insert(name.clone(), resolved_version.clone());
        }

        added_packages.push((name, resolved_version));
    }

    if let Some(ref pb) = progress {
        pb.set_message("Saving package.json...");
    }

    // Save package.json
    package_json.save(&project_dir)?;

    if let Some(ref pb) = progress {
        pb.set_message("Installing packages...");
    }

    // Install the new packages
    let deps = package_json.all_dependencies();
    let resolver = engine.resolver();
    let resolution = resolver.resolve(&deps).await?;

    let installer = engine.installer();
    let install_result = installer.install(&resolution, false, false).await?;
    installer.link(&resolution).await?;

    // Save lockfile
    let mut lockfile = resolution.lockfile;
    lockfile.save(&project_dir)?;

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let duration = start_time.elapsed();

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "added": added_packages.iter().map(|(n, v)| serde_json::json!({
                "name": n,
                "version": v
            })).collect::<Vec<_>>(),
            "duration_ms": duration.as_millis()
        }))?;
    } else {
        for (name, version) in &added_packages {
            output::success(&format!("Added {}", output::package_version(name, version)));
        }

        output::info(&format!(
            "Installed in {}",
            output::format_duration(duration.as_millis())
        ));
    }

    Ok(())
}

/// Parse a package specification (name@version)
fn parse_package_spec(spec: &str) -> (String, Option<&str>) {
    // Handle scoped packages (@org/name@version)
    if spec.starts_with('@') {
        if let Some(at_idx) = spec[1..].find('@') {
            let idx = at_idx + 1;
            return (spec[..idx].to_string(), Some(&spec[idx + 1..]));
        }
        return (spec.to_string(), None);
    }

    // Regular package (name@version)
    if let Some(at_idx) = spec.find('@') {
        return (spec[..at_idx].to_string(), Some(&spec[at_idx + 1..]));
    }

    (spec.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        assert_eq!(parse_package_spec("react"), ("react".to_string(), None));
        assert_eq!(parse_package_spec("react@18.0.0"), ("react".to_string(), Some("18.0.0")));
        assert_eq!(parse_package_spec("@types/node"), ("@types/node".to_string(), None));
        assert_eq!(parse_package_spec("@types/node@18.0.0"), ("@types/node".to_string(), Some("18.0.0")));
    }
}
