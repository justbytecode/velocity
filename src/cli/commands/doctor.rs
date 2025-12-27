//! velocity doctor - Diagnose issues

use std::env;
use std::path::PathBuf;
use clap::Args;
use which::which;

use crate::cli::output;
use crate::core::VelocityResult;

#[derive(Args)]
pub struct DoctorArgs {
    /// Project directory
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,
}

pub async fn execute(args: DoctorArgs, json_output: bool) -> VelocityResult<()> {
    let project_dir = if args.cwd.is_absolute() {
        args.cwd.clone()
    } else {
        env::current_dir()?.join(&args.cwd)
    };

    let mut checks: Vec<DiagnosticCheck> = Vec::new();

    // Check Node.js
    let node_check = check_node().await;
    checks.push(node_check);

    // Check npm (for compatibility)
    let npm_check = check_npm().await;
    checks.push(npm_check);

    // Check project
    let project_check = check_project(&project_dir).await;
    checks.push(project_check);

    // Check cache
    let cache_check = check_cache(&project_dir).await;
    checks.push(cache_check);

    // Check network
    let network_check = check_network().await;
    checks.push(network_check);

    // Check node_modules
    let nm_check = check_node_modules(&project_dir).await;
    checks.push(nm_check);

    // Check lockfile
    let lockfile_check = check_lockfile(&project_dir).await;
    checks.push(lockfile_check);

    let all_passed = checks.iter().all(|c| c.passed);

    if json_output {
        output::json(&serde_json::json!({
            "success": all_passed,
            "checks": checks.iter().map(|c| serde_json::json!({
                "name": c.name,
                "passed": c.passed,
                "message": c.message,
                "details": c.details
            })).collect::<Vec<_>>()
        }))?;
    } else {
        println!();
        output::info("Velocity Doctor - System Diagnostics");
        output::divider();
        println!();

        for check in &checks {
            let status = if check.passed {
                console::style("✓").green().bold()
            } else {
                console::style("✗").red().bold()
            };

            println!(
                "{} {} - {}",
                status,
                console::style(&check.name).bold(),
                check.message
            );

            if let Some(ref details) = check.details {
                println!("  {}", console::style(details).dim());
            }
        }

        println!();
        output::divider();

        if all_passed {
            output::success("All checks passed! Your environment is ready.");
        } else {
            let failed_count = checks.iter().filter(|c| !c.passed).count();
            output::warning(&format!(
                "{} check(s) failed. Address the issues above.",
                failed_count
            ));
        }
    }

    Ok(())
}

struct DiagnosticCheck {
    name: String,
    passed: bool,
    message: String,
    details: Option<String>,
}

async fn check_node() -> DiagnosticCheck {
    match which("node") {
        Ok(path) => {
            // Get version
            let output = tokio::process::Command::new("node")
                .arg("--version")
                .output()
                .await;

            match output {
                Ok(out) => {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    DiagnosticCheck {
                        name: "Node.js".to_string(),
                        passed: true,
                        message: format!("Found {} at {}", version, path.display()),
                        details: None,
                    }
                }
                Err(_) => DiagnosticCheck {
                    name: "Node.js".to_string(),
                    passed: false,
                    message: "Could not get Node.js version".to_string(),
                    details: Some("Node.js is installed but version check failed".to_string()),
                },
            }
        }
        Err(_) => DiagnosticCheck {
            name: "Node.js".to_string(),
            passed: false,
            message: "Node.js not found".to_string(),
            details: Some("Install Node.js from https://nodejs.org".to_string()),
        },
    }
}

async fn check_npm() -> DiagnosticCheck {
    match which("npm") {
        Ok(path) => {
            let output = tokio::process::Command::new("npm")
                .arg("--version")
                .output()
                .await;

            match output {
                Ok(out) => {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    DiagnosticCheck {
                        name: "npm".to_string(),
                        passed: true,
                        message: format!("Found v{}", version),
                        details: Some("npm is optional but useful for compatibility".to_string()),
                    }
                }
                Err(_) => DiagnosticCheck {
                    name: "npm".to_string(),
                    passed: true,
                    message: "npm version check failed".to_string(),
                    details: None,
                },
            }
        }
        Err(_) => DiagnosticCheck {
            name: "npm".to_string(),
            passed: true, // npm is optional
            message: "npm not found (optional)".to_string(),
            details: None,
        },
    }
}

async fn check_project(project_dir: &PathBuf) -> DiagnosticCheck {
    let package_json = project_dir.join("package.json");
    if package_json.exists() {
        match crate::core::PackageJson::load(project_dir) {
            Ok(pkg) => DiagnosticCheck {
                name: "Project".to_string(),
                passed: true,
                message: format!("Found '{}' v{}", pkg.name, pkg.version),
                details: None,
            },
            Err(e) => DiagnosticCheck {
                name: "Project".to_string(),
                passed: false,
                message: "Invalid package.json".to_string(),
                details: Some(e.to_string()),
            },
        }
    } else {
        DiagnosticCheck {
            name: "Project".to_string(),
            passed: false,
            message: "No package.json found".to_string(),
            details: Some("Run 'velocity init' to create a project".to_string()),
        }
    }
}

async fn check_cache(project_dir: &PathBuf) -> DiagnosticCheck {
    let config = crate::core::Config::load(project_dir).unwrap_or_default();
    match config.cache_dir() {
        Ok(cache_dir) => {
            let size = calculate_dir_size(&cache_dir).unwrap_or(0);
            DiagnosticCheck {
                name: "Cache".to_string(),
                passed: true,
                message: format!("Cache at {}", cache_dir.display()),
                details: Some(format!("Size: {}", output::format_bytes(size))),
            }
        }
        Err(e) => DiagnosticCheck {
            name: "Cache".to_string(),
            passed: false,
            message: "Could not access cache directory".to_string(),
            details: Some(e.to_string()),
        },
    }
}

async fn check_network() -> DiagnosticCheck {
    let client = reqwest::Client::new();
    let result = client
        .get("https://registry.npmjs.org")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match result {
        Ok(response) if response.status().is_success() => DiagnosticCheck {
            name: "Network".to_string(),
            passed: true,
            message: "npm registry is reachable".to_string(),
            details: None,
        },
        Ok(response) => DiagnosticCheck {
            name: "Network".to_string(),
            passed: false,
            message: format!("Registry returned status {}", response.status()),
            details: None,
        },
        Err(e) => DiagnosticCheck {
            name: "Network".to_string(),
            passed: false,
            message: "Cannot reach npm registry".to_string(),
            details: Some(format!("Error: {}", e)),
        },
    }
}

async fn check_node_modules(project_dir: &PathBuf) -> DiagnosticCheck {
    let node_modules = project_dir.join("node_modules");
    if node_modules.exists() {
        let count = std::fs::read_dir(&node_modules)
            .map(|entries| entries.count())
            .unwrap_or(0);
        DiagnosticCheck {
            name: "node_modules".to_string(),
            passed: true,
            message: format!("{} packages installed", count),
            details: None,
        }
    } else {
        DiagnosticCheck {
            name: "node_modules".to_string(),
            passed: true, // Not an error if not present
            message: "Not installed".to_string(),
            details: Some("Run 'velocity install' to install dependencies".to_string()),
        }
    }
}

async fn check_lockfile(project_dir: &PathBuf) -> DiagnosticCheck {
    let lockfile_path = project_dir.join("velocity.lock");
    if lockfile_path.exists() {
        match crate::core::Lockfile::load(project_dir) {
            Ok(Some(lockfile)) => DiagnosticCheck {
                name: "Lockfile".to_string(),
                passed: true,
                message: format!("{} packages locked", lockfile.packages.len()),
                details: None,
            },
            Ok(None) => DiagnosticCheck {
                name: "Lockfile".to_string(),
                passed: true,
                message: "No lockfile".to_string(),
                details: None,
            },
            Err(_) => DiagnosticCheck {
                name: "Lockfile".to_string(),
                passed: false,
                message: "Lockfile is corrupted".to_string(),
                details: Some("Delete velocity.lock and run 'velocity install'".to_string()),
            },
        }
    } else {
        DiagnosticCheck {
            name: "Lockfile".to_string(),
            passed: true,
            message: "No lockfile".to_string(),
            details: Some("Run 'velocity install' to create one".to_string()),
        }
    }
}

fn calculate_dir_size(path: &PathBuf) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += calculate_dir_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    }
    Ok(size)
}
