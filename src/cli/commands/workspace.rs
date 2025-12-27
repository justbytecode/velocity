//! velocity workspace - Workspace commands

use std::env;

use clap::{Args, Subcommand};

use crate::cli::output;
use crate::core::{Engine, PackageJson, VelocityResult};


#[derive(Args)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// Initialize a new workspace
    Init {
        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
    },

    /// List all packages in the workspace
    List,

    /// Run a command in all packages
    Run {
        /// Command to run
        command: String,

        /// Arguments to pass
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// Filter by package name
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Add a new package to the workspace
    Add {
        /// Package name
        name: String,

        /// Package directory (relative to packages/)
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Show dependency graph
    Graph,
}

pub async fn execute(args: WorkspaceArgs, json_output: bool) -> VelocityResult<()> {
    match args.command {
        WorkspaceCommands::Init { yes } => init_workspace(yes, json_output).await,
        WorkspaceCommands::List => list_packages(json_output).await,
        WorkspaceCommands::Run { command, args, filter } => {
            run_in_packages(&command, &args, filter, json_output).await
        }
        WorkspaceCommands::Add { name, dir } => add_package(&name, dir, json_output).await,
        WorkspaceCommands::Graph => show_graph(json_output).await,
    }
}

async fn init_workspace(yes: bool, json_output: bool) -> VelocityResult<()> {
    let project_dir = env::current_dir()?;

    // Check if already a workspace
    if let Ok(pkg) = PackageJson::load(&project_dir) {
        if pkg.is_workspace_root() {
            if json_output {
                output::json(&serde_json::json!({
                    "success": false,
                    "error": "Already a workspace"
                }))?;
            } else {
                output::warning("This directory is already a workspace");
            }
            return Ok(());
        }
    }

    if !json_output {
        output::info("Initializing workspace...");
    }

    // Create or update package.json
    let mut package_json = PackageJson::load(&project_dir)
        .unwrap_or_else(|_| PackageJson::new("my-workspace"));

    package_json.private = true;
    package_json.workspaces = Some(crate::core::package::WorkspacesConfig::Patterns(vec![
        "packages/*".to_string(),
    ]));

    package_json.save(&project_dir)?;

    // Create packages directory
    let packages_dir = project_dir.join("packages");
    if !packages_dir.exists() {
        std::fs::create_dir_all(&packages_dir)?;
    }

    // Create .gitignore if needed
    let gitignore = project_dir.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(&gitignore, "node_modules/\nvelocity.lock\n")?;
    }

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "path": project_dir
        }))?;
    } else {
        output::success("Workspace initialized");
        output::info("Add packages to the packages/ directory");
    }

    Ok(())
}

async fn list_packages(json_output: bool) -> VelocityResult<()> {
    let project_dir = env::current_dir()?;
    let engine = Engine::new(&project_dir).await?;

    let packages = engine.workspace_packages()?;

    if packages.is_empty() {
        if json_output {
            output::json(&serde_json::json!({
                "packages": []
            }))?;
        } else {
            output::info("No packages in workspace");
            output::info("Add packages to the packages/ directory");
        }
        return Ok(());
    }

    let mut package_info = Vec::new();

    for pkg_path in &packages {
        if let Ok(pkg) = PackageJson::load(pkg_path) {
            package_info.push((
                pkg.name.clone(),
                pkg.version.clone(),
                pkg_path.strip_prefix(&project_dir).unwrap_or(pkg_path).to_path_buf(),
            ));
        }
    }

    if json_output {
        output::json(&serde_json::json!({
            "packages": package_info.iter().map(|(name, version, path)| {
                serde_json::json!({
                    "name": name,
                    "version": version,
                    "path": path
                })
            }).collect::<Vec<_>>()
        }))?;
    } else {
        output::info(&format!("Workspace packages ({}):", package_info.len()));
        output::divider();

        for (name, version, path) in &package_info {
            println!(
                "  {} {} ({})",
                console::style(name).cyan().bold(),
                console::style(format!("v{}", version)).dim(),
                console::style(path.display()).dim()
            );
        }
    }

    Ok(())
}

async fn run_in_packages(
    command: &str,
    args: &[String],
    filter: Option<String>,
    json_output: bool,
) -> VelocityResult<()> {
    let project_dir = env::current_dir()?;
    let engine = Engine::new(&project_dir).await?;

    let packages = engine.workspace_packages()?;

    if packages.is_empty() {
        if !json_output {
            output::warning("No packages in workspace");
        }
        return Ok(());
    }

    let mut results = Vec::new();

    for pkg_path in &packages {
        let pkg = match PackageJson::load(pkg_path) {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Apply filter
        if let Some(ref f) = filter {
            if !pkg.name.contains(f) {
                continue;
            }
        }

        if !json_output {
            output::info(&format!("Running in {}...", console::style(&pkg.name).cyan()));
        }

        // Check if script exists
        if let Some(script) = pkg.scripts.get(command) {
            let full_args: Vec<String> = args.to_vec();

            let shell = if cfg!(windows) { "cmd" } else { "sh" };
            let shell_arg = if cfg!(windows) { "/c" } else { "-c" };

            let full_command = if full_args.is_empty() {
                script.clone()
            } else {
                format!("{} {}", script, full_args.join(" "))
            };

            let status = tokio::process::Command::new(shell)
                .arg(shell_arg)
                .arg(&full_command)
                .current_dir(pkg_path)
                .status()
                .await?;

            results.push((pkg.name.clone(), status.success()));

            if !json_output && !status.success() {
                output::warning(&format!("Command failed in {}", pkg.name));
            }
        } else if !json_output {
            output::warning(&format!("Script '{}' not found in {}", command, pkg.name));
        }
    }

    if json_output {
        output::json(&serde_json::json!({
            "command": command,
            "results": results.iter().map(|(name, success)| {
                serde_json::json!({
                    "package": name,
                    "success": success
                })
            }).collect::<Vec<_>>()
        }))?;
    } else {
        let success_count = results.iter().filter(|(_, s)| *s).count();
        let total = results.len();

        if success_count == total {
            output::success(&format!("Completed in all {} packages", total));
        } else {
            output::warning(&format!(
                "Completed in {}/{} packages",
                success_count, total
            ));
        }
    }

    Ok(())
}

async fn add_package(name: &str, dir: Option<String>, json_output: bool) -> VelocityResult<()> {
    let project_dir = env::current_dir()?;

    // Ensure we're in a workspace
    let root_pkg = PackageJson::load(&project_dir)?;
    if !root_pkg.is_workspace_root() {
        return Err(crate::core::VelocityError::workspace(
            "Not in a workspace root. Run 'velocity workspace init' first.",
        ));
    }

    let package_dir_name = dir.unwrap_or_else(|| name.replace('@', "").replace('/', "-"));
    let package_path = project_dir.join("packages").join(&package_dir_name);

    if package_path.exists() {
        return Err(crate::core::VelocityError::workspace(format!(
            "Package directory '{}' already exists",
            package_dir_name
        )));
    }

    if !json_output {
        output::info(&format!("Creating package '{}'...", name));
    }

    // Create package directory
    std::fs::create_dir_all(&package_path)?;

    // Create package.json
    let mut pkg = PackageJson::new(name);
    pkg.version = "0.1.0".to_string();
    pkg.private = true;
    pkg.save(&package_path)?;

    // Create src directory
    std::fs::create_dir_all(package_path.join("src"))?;

    // Create index file
    std::fs::write(
        package_path.join("src").join("index.ts"),
        "// Package entry point\nexport {};\n",
    )?;

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "name": name,
            "path": package_path
        }))?;
    } else {
        output::success(&format!(
            "Created package '{}' at packages/{}",
            console::style(name).cyan(),
            package_dir_name
        ));
    }

    Ok(())
}

async fn show_graph(json_output: bool) -> VelocityResult<()> {
    let project_dir = env::current_dir()?;
    let engine = Engine::new(&project_dir).await?;

    let packages = engine.workspace_packages()?;

    if packages.is_empty() {
        if !json_output {
            output::info("No packages in workspace");
        }
        return Ok(());
    }

    // Build dependency graph
    let mut graph: Vec<(String, Vec<String>)> = Vec::new();

    let workspace_package_names: Vec<String> = packages
        .iter()
        .filter_map(|p| PackageJson::load(p).ok().map(|pkg| pkg.name))
        .collect();

    for pkg_path in &packages {
        if let Ok(pkg) = PackageJson::load(pkg_path) {
            let deps: Vec<String> = pkg
                .all_dependencies()
                .keys()
                .filter(|d| workspace_package_names.contains(d))
                .cloned()
                .collect();

            graph.push((pkg.name, deps));
        }
    }

    if json_output {
        output::json(&serde_json::json!({
            "packages": graph.iter().map(|(name, deps)| {
                serde_json::json!({
                    "name": name,
                    "workspace_dependencies": deps
                })
            }).collect::<Vec<_>>()
        }))?;
    } else {
        output::info("Workspace dependency graph:");
        output::divider();

        for (name, deps) in &graph {
            if deps.is_empty() {
                println!("  {} (no workspace dependencies)", console::style(name).cyan());
            } else {
                println!("  {} → {}", 
                    console::style(name).cyan(),
                    deps.iter()
                        .map(|d| console::style(d).green().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
    }

    Ok(())
}
