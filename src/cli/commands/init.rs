//! velocity init - Initialize a new project

use std::env;
use std::path::PathBuf;
use clap::Args;
use dialoguer::{Input, Confirm};

use crate::cli::output;
use crate::core::{PackageJson, VelocityResult};

#[derive(Args)]
pub struct InitArgs {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Skip interactive prompts
    #[arg(short, long)]
    pub yes: bool,

    /// Initialize as a workspace
    #[arg(short, long)]
    pub workspace: bool,

    /// Project name
    #[arg(long)]
    pub name: Option<String>,
}

pub async fn execute(args: InitArgs, json_output: bool) -> VelocityResult<()> {
    let project_dir = if args.path.is_absolute() {
        args.path.clone()
    } else {
        env::current_dir()?.join(&args.path)
    };

    // Create directory if it doesn't exist
    if !project_dir.exists() {
        std::fs::create_dir_all(&project_dir)?;
    }

    // Check if package.json already exists
    let package_json_path = project_dir.join("package.json");
    if package_json_path.exists() {
        if json_output {
            output::json(&serde_json::json!({
                "success": false,
                "error": "package.json already exists"
            }))?;
        } else {
            output::warning("package.json already exists. Use 'velocity install' to install dependencies.");
        }
        return Ok(());
    }

    // Get project name
    let default_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let project_name = if let Some(name) = args.name {
        name
    } else if args.yes {
        default_name
    } else {
        Input::new()
            .with_prompt("Package name")
            .default(default_name)
            .interact_text()?
    };

    // Get version
    let version = if args.yes {
        "1.0.0".to_string()
    } else {
        Input::new()
            .with_prompt("Version")
            .default("1.0.0".to_string())
            .interact_text()?
    };

    // Get description
    let description = if args.yes {
        String::new()
    } else {
        Input::new()
            .with_prompt("Description")
            .default(String::new())
            .allow_empty(true)
            .interact_text()?
    };

    // Create package.json
    let mut package_json = PackageJson::new(&project_name);
    package_json.version = version;
    package_json.description = description;

    // Set up as workspace if requested
    if args.workspace {
        package_json.private = true;
        package_json.workspaces = Some(crate::core::package::WorkspacesConfig::Patterns(vec![
            "packages/*".to_string(),
        ]));

        // Create packages directory
        let packages_dir = project_dir.join("packages");
        if !packages_dir.exists() {
            std::fs::create_dir_all(&packages_dir)?;
        }
    }

    // Add default scripts
    package_json.scripts.insert("test".to_string(), "echo \"Error: no test specified\" && exit 1".to_string());

    // Ask about TypeScript
    let use_typescript = if args.yes {
        false
    } else {
        Confirm::new()
            .with_prompt("Use TypeScript?")
            .default(false)
            .interact()?
    };

    if use_typescript {
        package_json.dev_dependencies.insert("typescript".to_string(), "^5.0.0".to_string());
        package_json.scripts.insert("build".to_string(), "tsc".to_string());
    }

    // Save package.json
    package_json.save(&project_dir)?;

    // Create .gitignore if it doesn't exist
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = r#"# Dependencies
node_modules/

# Build output
dist/
build/

# Velocity
velocity.lock

# Environment
.env
.env.local
.env.*.local

# IDE
.idea/
.vscode/
*.swp

# OS
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*
"#;
        std::fs::write(&gitignore_path, gitignore_content)?;
    }

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "name": project_name,
            "path": project_dir,
            "workspace": args.workspace
        }))?;
    } else {
        output::success(&format!("Initialized project '{}' in {}", project_name, project_dir.display()));
        
        if args.workspace {
            output::info("Workspace mode enabled. Add packages to packages/ directory.");
        }

        println!();
        output::info("Next steps:");
        println!("  1. Add dependencies: velocity add <package>");
        println!("  2. Install dependencies: velocity install");
        println!("  3. Run scripts: velocity run <script>");
    }

    Ok(())
}
