//! velocity create - Create projects from templates

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use clap::Args;
use dialoguer::{Input, Select};

use crate::cli::output;
use crate::core::{VelocityResult, VelocityError};
use crate::templates::TemplateManager;
use crate::security::ecosystem::TemplateFlags;

#[derive(Args)]
pub struct CreateArgs {
    /// Framework to use (react, next, vue, svelte, solid, astro)
    pub framework: Option<String>,

    /// Project name/directory
    #[arg(short, long)]
    pub name: Option<String>,

    /// Use TypeScript
    #[arg(long)]
    pub typescript: bool,

    /// Add Web3/Blockchain support (wagmi, viem, rainbowkit)
    #[arg(long)]
    pub web3: bool,

    /// Add AI support (ai-sdk, openai)
    #[arg(long)]
    pub ai: bool,

    /// Skip git initialization
    #[arg(long)]
    pub no_git: bool,

    /// Skip dependency installation
    #[arg(long)]
    pub no_install: bool,

    /// Use default options (no prompts)
    #[arg(short, long)]
    pub yes: bool,
}

const SUPPORTED_FRAMEWORKS: &[(&str, &str)] = &[
    ("react", "React - A JavaScript library for building user interfaces"),
    ("next", "Next.js - The React framework for production"),
    ("vue", "Vue - The Progressive JavaScript Framework"),
    ("svelte", "Svelte - Cybernetically enhanced web apps"),
    ("solid", "Solid - Simple and performant reactivity"),
    ("astro", "Astro - Build fast websites, faster"),
];

pub async fn execute(args: CreateArgs, json_output: bool) -> VelocityResult<()> {
    let start_time = Instant::now();

    // Get framework
    let framework = if let Some(f) = args.framework {
        validate_framework(&f)?;
        f
    } else if args.yes {
        "react".to_string()
    } else {
        let items: Vec<&str> = SUPPORTED_FRAMEWORKS.iter().map(|(_, desc)| *desc).collect();
        let selection = Select::new()
            .with_prompt("Which framework would you like to use?")
            .items(&items)
            .default(0)
            .interact()?;
        SUPPORTED_FRAMEWORKS[selection].0.to_string()
    };

    // Get project name
    let project_name = if let Some(name) = args.name {
        name
    } else if args.yes {
        format!("my-{}-app", framework)
    } else {
        Input::new()
            .with_prompt("Project name")
            .default(format!("my-{}-app", framework))
            .interact_text()?
    };

    // Validate project name
    if project_name.contains(std::path::is_separator) {
        return Err(VelocityError::other("Project name cannot contain path separators"));
    }

    let project_dir = env::current_dir()?.join(&project_name);

    // Check if directory exists
    if project_dir.exists() {
        return Err(VelocityError::other(format!(
            "Directory '{}' already exists",
            project_name
        )));
    }

    // Determine TypeScript
    let use_typescript = args.typescript || (!args.yes && {
        dialoguer::Confirm::new()
            .with_prompt("Use TypeScript?")
            .default(true)
            .interact()?
    });

    // Ecosystem flags
    let template_flags = TemplateFlags {
        web3: args.web3,
        ai: args.ai,
        typescript: use_typescript,
    };

    if !json_output {
        let mut extras = vec![];
        if args.web3 { extras.push("Web3"); }
        if args.ai { extras.push("AI"); }
        
        let extra_str = if extras.is_empty() {
            String::new()
        } else {
            format!(" with {}", extras.join(" + "))
        };

        output::info(&format!(
            "Creating {} project '{}'{}...",
            console::style(&framework).cyan(),
            console::style(&project_name).green(),
            extra_str
        ));
    }

    let progress = if !json_output {
        Some(output::spinner("Scaffolding project..."))
    } else {
        None
    };

    // Create project directory
    std::fs::create_dir_all(&project_dir)?;

    // Generate template
    let template_manager = TemplateManager::new();
    let template = template_manager.get_template(&framework, use_typescript)?;
    
    template.generate(&project_dir)?;

    // Add Web3/AI dependencies to package.json if requested
    if args.web3 || args.ai {
        add_ecosystem_deps(&project_dir, &template_flags)?;
    }

    if let Some(ref pb) = progress {
        pb.set_message("Initializing git...");
    }

    // Initialize git
    if !args.no_git {
        init_git(&project_dir).await?;
    }

    // Install dependencies
    if !args.no_install {
        if let Some(ref pb) = progress {
            pb.set_message("Installing dependencies...");
        }

        install_dependencies(&project_dir).await?;
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let duration = start_time.elapsed();

    if json_output {
        output::json(&serde_json::json!({
            "success": true,
            "framework": framework,
            "name": project_name,
            "path": project_dir,
            "typescript": use_typescript,
            "web3": args.web3,
            "ai": args.ai,
            "duration_ms": duration.as_millis()
        }))?;
    } else {
        println!();
        output::success(&format!(
            "Created {} project in {}",
            console::style(&framework).cyan(),
            output::format_duration(duration.as_millis())
        ));

        println!();
        output::info("Next steps:");
        println!("  cd {}", project_name);
        if args.no_install {
            println!("  velocity install");
        }
        println!("  velocity run dev");
        println!();
    }

    Ok(())
}

fn validate_framework(framework: &str) -> VelocityResult<()> {
    let valid = SUPPORTED_FRAMEWORKS.iter().any(|(f, _)| *f == framework);
    if !valid {
        return Err(VelocityError::template(format!(
            "Unknown framework '{}'. Supported: {}",
            framework,
            SUPPORTED_FRAMEWORKS.iter().map(|(f, _)| *f).collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(())
}

async fn init_git(project_dir: &PathBuf) -> VelocityResult<()> {
    let status = tokio::process::Command::new("git")
        .args(["init"])
        .current_dir(project_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    // Git init is optional, don't fail if git isn't available
    if let Err(_) = status {
        tracing::debug!("git init failed, continuing without git");
    }

    Ok(())
}

async fn install_dependencies(project_dir: &PathBuf) -> VelocityResult<()> {
    use crate::core::Engine;

    let engine = Engine::new(project_dir).await?;
    let package_json = engine.package_json()?;
    let deps = package_json.all_dependencies();

    if deps.is_empty() {
        return Ok(());
    }

    let resolver = engine.resolver();
    let resolution = resolver.resolve(&deps).await?;

    let installer = engine.installer();
    installer.install(&resolution, false, false).await?;
    installer.link(&resolution).await?;

    let mut lockfile = resolution.lockfile;
    lockfile.save(project_dir)?;

    Ok(())
}

fn add_ecosystem_deps(project_dir: &PathBuf, flags: &TemplateFlags) -> VelocityResult<()> {
    let pkg_json_path = project_dir.join("package.json");
    let content = std::fs::read_to_string(&pkg_json_path)?;
    let mut pkg: serde_json::Value = serde_json::from_str(&content)?;

    // Ensure dependencies object exists
    if pkg.get("dependencies").is_none() {
        pkg["dependencies"] = serde_json::json!({});
    }

    let deps = pkg["dependencies"].as_object_mut().unwrap();

    // Add Web3 dependencies
    if flags.web3 {
        deps.insert("wagmi".to_string(), serde_json::json!("^2.0.0"));
        deps.insert("viem".to_string(), serde_json::json!("^2.0.0"));
        deps.insert("@tanstack/react-query".to_string(), serde_json::json!("^5.0.0"));
    }

    // Add AI dependencies
    if flags.ai {
        deps.insert("ai".to_string(), serde_json::json!("^3.0.0"));
        deps.insert("@ai-sdk/openai".to_string(), serde_json::json!("^0.0.1"));
    }

    // Write back
    let updated = serde_json::to_string_pretty(&pkg)?;
    std::fs::write(&pkg_json_path, updated)?;

    Ok(())
}
