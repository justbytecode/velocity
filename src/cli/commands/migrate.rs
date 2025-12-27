//! velocity migrate - Migrate from npm/pnpm

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use clap::Args;

use crate::cli::output;
use crate::core::{VelocityResult, VelocityError};

#[derive(Args)]
pub struct MigrateArgs {
    /// Source package manager (npm, pnpm, yarn)
    pub from: String,

    /// Project directory
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,

    /// Remove old lockfile after migration
    #[arg(long)]
    pub remove_old: bool,

    /// Dry run - show what would be migrated
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn execute(args: MigrateArgs, json_output: bool) -> VelocityResult<()> {
    let start_time = Instant::now();

    let project_dir = if args.cwd.is_absolute() {
        args.cwd.clone()
    } else {
        env::current_dir()?.join(&args.cwd)
    };

    let from = args.from.to_lowercase();
    
    // Validate source
    if !["npm", "pnpm", "yarn"].contains(&from.as_str()) {
        return Err(VelocityError::migration(format!(
            "Unsupported package manager '{}'. Supported: npm, pnpm, yarn",
            from
        )));
    }

    // Check for existing lockfile
    let source_lockfile = get_source_lockfile(&project_dir, &from);
    if !source_lockfile.exists() {
        return Err(VelocityError::migration(format!(
            "No {} lockfile found at {}",
            from,
            source_lockfile.display()
        )));
    }

    if !json_output {
        output::info(&format!("Migrating from {} to Velocity...", from));
    }

    let progress = if !json_output {
        Some(output::spinner("Analyzing lockfile..."))
    } else {
        None
    };

    // Parse the source lockfile
    let migration_info = parse_source_lockfile(&source_lockfile, &from)?;

    if let Some(ref pb) = progress {
        pb.set_message("Converting dependencies...");
    }

    if args.dry_run {
        if let Some(pb) = progress {
            pb.finish_and_clear();
        }

        if json_output {
            output::json(&serde_json::json!({
                "dry_run": true,
                "from": from,
                "packages": migration_info.packages.len(),
                "source_lockfile": source_lockfile
            }))?;
        } else {
            output::info("Dry run - no changes will be made");
            println!();
            println!("  Source: {}", source_lockfile.display());
            println!("  Packages: {}", migration_info.packages.len());
            println!();
            output::info("Run without --dry-run to perform migration");
        }
        return Ok(());
    }

    // Create Velocity lockfile
    let mut lockfile = crate::core::Lockfile::new();

    for pkg in &migration_info.packages {
        lockfile.add_package(crate::core::lockfile::LockedPackage {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            resolved: pkg.resolved.clone(),
            integrity: pkg.integrity.clone(),
            dependencies: pkg.dependencies.clone(),
            peer_dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            has_scripts: false,
            cpu: Vec::new(),
            os: Vec::new(),
        });
    }

    if let Some(ref pb) = progress {
        pb.set_message("Saving lockfile...");
    }

    // Save Velocity lockfile
    lockfile.save(&project_dir)?;

    // Update package.json to use Velocity
    let mut package_json = crate::core::PackageJson::load(&project_dir)?;
    package_json.package_manager = Some("velocity@0.1.0".to_string());
    package_json.save(&project_dir)?;

    // Optionally remove old lockfile
    if args.remove_old {
        std::fs::remove_file(&source_lockfile)?;
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let duration = start_time.elapsed();

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "from": from,
            "packages": migration_info.packages.len(),
            "duration_ms": duration.as_millis()
        }))?;
    } else {
        output::success(&format!(
            "Migrated {} packages from {} in {}",
            migration_info.packages.len(),
            from,
            output::format_duration(duration.as_millis())
        ));

        println!();
        output::info("Next steps:");
        println!("  1. Run 'velocity install' to reinstall packages");
        println!("  2. Test your project to ensure everything works");
        
        if !args.remove_old {
            println!("  3. Remove old lockfile: {}", source_lockfile.display());
        }
    }

    Ok(())
}

fn get_source_lockfile(project_dir: &PathBuf, from: &str) -> PathBuf {
    match from {
        "npm" => project_dir.join("package-lock.json"),
        "pnpm" => project_dir.join("pnpm-lock.yaml"),
        "yarn" => project_dir.join("yarn.lock"),
        _ => project_dir.join("package-lock.json"),
    }
}

struct MigrationInfo {
    packages: Vec<MigratedPackage>,
}

struct MigratedPackage {
    name: String,
    version: String,
    resolved: String,
    integrity: String,
    dependencies: Vec<String>,
}

fn parse_source_lockfile(path: &PathBuf, from: &str) -> VelocityResult<MigrationInfo> {
    let content = std::fs::read_to_string(path)?;

    match from {
        "npm" => parse_npm_lockfile(&content),
        "pnpm" => parse_pnpm_lockfile(&content),
        "yarn" => parse_yarn_lockfile(&content),
        _ => Err(VelocityError::migration("Unsupported lockfile format")),
    }
}

fn parse_npm_lockfile(content: &str) -> VelocityResult<MigrationInfo> {
    let lockfile: serde_json::Value = serde_json::from_str(content)?;
    let mut packages = Vec::new();

    // npm v2/v3 format
    if let Some(deps) = lockfile.get("packages").and_then(|p| p.as_object()) {
        for (key, value) in deps {
            // Skip root package
            if key.is_empty() || key == "." {
                continue;
            }

            // Extract package name from path
            let name = key
                .trim_start_matches("node_modules/")
                .to_string();

            if let Some(obj) = value.as_object() {
                let version = obj.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0")
                    .to_string();

                let resolved = obj.get("resolved")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let integrity = obj.get("integrity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut dependencies = Vec::new();
                if let Some(deps) = obj.get("dependencies").and_then(|d| d.as_object()) {
                    for (dep_name, dep_version) in deps {
                        dependencies.push(format!("{}@{}", dep_name, dep_version.as_str().unwrap_or("*")));
                    }
                }

                packages.push(MigratedPackage {
                    name,
                    version,
                    resolved,
                    integrity,
                    dependencies,
                });
            }
        }
    }

    Ok(MigrationInfo { packages })
}

fn parse_pnpm_lockfile(content: &str) -> VelocityResult<MigrationInfo> {
    // Basic YAML parsing for pnpm lockfile
    // In production, use a proper YAML parser
    let mut packages = Vec::new();

    let mut current_package: Option<String> = None;
    let mut current_version = String::new();
    let mut current_resolved = String::new();
    let mut current_integrity = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        
        // Package entry
        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.ends_with(':') {
            // Save previous package
            if let Some(ref name) = current_package {
                if !current_version.is_empty() {
                    packages.push(MigratedPackage {
                        name: name.clone(),
                        version: current_version.clone(),
                        resolved: current_resolved.clone(),
                        integrity: current_integrity.clone(),
                        dependencies: Vec::new(),
                    });
                }
            }

            let entry = trimmed.trim_end_matches(':');
            // Parse package@version format
            if let Some(at_idx) = entry.rfind('@') {
                current_package = Some(entry[..at_idx].to_string());
                current_version = entry[at_idx + 1..].to_string();
            } else {
                current_package = Some(entry.to_string());
                current_version.clear();
            }
            current_resolved.clear();
            current_integrity.clear();
        }

        // Parse properties
        if trimmed.starts_with("resolution:") {
            current_resolved = trimmed
                .trim_start_matches("resolution:")
                .trim()
                .trim_matches('{')
                .trim_matches('}')
                .to_string();
        }

        if trimmed.starts_with("integrity:") {
            current_integrity = trimmed
                .trim_start_matches("integrity:")
                .trim()
                .to_string();
        }
    }

    // Don't forget the last package
    if let Some(ref name) = current_package {
        if !current_version.is_empty() {
            packages.push(MigratedPackage {
                name: name.clone(),
                version: current_version,
                resolved: current_resolved,
                integrity: current_integrity,
                dependencies: Vec::new(),
            });
        }
    }

    Ok(MigrationInfo { packages })
}

fn parse_yarn_lockfile(content: &str) -> VelocityResult<MigrationInfo> {
    let mut packages = Vec::new();
    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_resolved = String::new();
    let mut current_integrity = String::new();

    for line in content.lines() {
        // Package entry (e.g., "package-name@^1.0.0:")
        if !line.starts_with(' ') && !line.starts_with('\t') && !line.starts_with('#') && line.contains('@') {
            // Save previous
            if !current_name.is_empty() && !current_version.is_empty() {
                packages.push(MigratedPackage {
                    name: current_name.clone(),
                    version: current_version.clone(),
                    resolved: current_resolved.clone(),
                    integrity: current_integrity.clone(),
                    dependencies: Vec::new(),
                });
            }

            // Parse entry
            let entry = line.trim().trim_end_matches(':');
            // Handle quoted entries
            let entry = entry.trim_matches('"');
            
            if let Some(at_idx) = entry.rfind('@') {
                if at_idx > 0 {
                    current_name = entry[..at_idx].to_string();
                }
            }
            current_version.clear();
            current_resolved.clear();
            current_integrity.clear();
        }

        let trimmed = line.trim();

        if trimmed.starts_with("version ") {
            current_version = trimmed
                .trim_start_matches("version ")
                .trim_matches('"')
                .to_string();
        }

        if trimmed.starts_with("resolved ") {
            current_resolved = trimmed
                .trim_start_matches("resolved ")
                .trim_matches('"')
                .to_string();
        }

        if trimmed.starts_with("integrity ") {
            current_integrity = trimmed
                .trim_start_matches("integrity ")
                .to_string();
        }
    }

    // Last package
    if !current_name.is_empty() && !current_version.is_empty() {
        packages.push(MigratedPackage {
            name: current_name,
            version: current_version,
            resolved: current_resolved,
            integrity: current_integrity,
            dependencies: Vec::new(),
        });
    }

    Ok(MigrationInfo { packages })
}
