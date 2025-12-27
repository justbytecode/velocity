//! velocity run - Run scripts

use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use clap::Args;
use tokio::process::Command;

use crate::cli::output;
use crate::core::{Engine, VelocityResult, VelocityError};

#[derive(Args)]
pub struct RunArgs {
    /// Script name to run
    pub script: Option<String>,

    /// Arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Project directory
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,

    /// List available scripts
    #[arg(short, long)]
    pub list: bool,
}

pub async fn execute(args: RunArgs, json_output: bool) -> VelocityResult<()> {
    let project_dir = if args.cwd.is_absolute() {
        args.cwd.clone()
    } else {
        env::current_dir()?.join(&args.cwd)
    };

    let engine = Engine::new(&project_dir).await?;
    engine.ensure_initialized()?;

    let package_json = engine.package_json()?;

    // List scripts
    if args.list || args.script.is_none() {
        if json_output {
            output::json(&serde_json::json!({
                "scripts": package_json.scripts
            }))?;
        } else {
            if package_json.scripts.is_empty() {
                output::info("No scripts defined in package.json");
            } else {
                output::info("Available scripts:");
                for (name, command) in &package_json.scripts {
                    println!(
                        "  {} → {}",
                        console::style(name).cyan().bold(),
                        console::style(command).dim()
                    );
                }
            }
        }
        return Ok(());
    }

    let script_name = args.script.unwrap();

    // Find the script
    let script_command = package_json.scripts.get(&script_name)
        .ok_or_else(|| VelocityError::other(format!(
            "Script '{}' not found. Available scripts: {}",
            script_name,
            package_json.scripts.keys().cloned().collect::<Vec<_>>().join(", ")
        )))?;

    if !json_output {
        output::info(&format!("Running script '{}'...", script_name));
        println!("{} {}", console::style("$").dim(), console::style(script_command).dim());
        println!();
    }

    // Build the command
    let shell = get_shell();
    let shell_arg = get_shell_arg();

    // Add node_modules/.bin to PATH
    let node_modules_bin = project_dir.join("node_modules").join(".bin");
    let path_env = env::var("PATH").unwrap_or_default();
    let new_path = format!(
        "{}{}{}",
        node_modules_bin.display(),
        if cfg!(windows) { ";" } else { ":" },
        path_env
    );

    // Build command with args
    let full_command = if args.args.is_empty() {
        script_command.clone()
    } else {
        format!("{} {}", script_command, args.args.join(" "))
    };

    // Execute
    let status = Command::new(&shell)
        .arg(&shell_arg)
        .arg(&full_command)
        .current_dir(&project_dir)
        .env("PATH", &new_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if json_output {
        output::json(&serde_json::json!({
            "script": script_name,
            "command": script_command,
            "success": status.success(),
            "exit_code": status.code()
        }))?;
    }

    if !status.success() {
        let exit_code = status.code().unwrap_or(1);
        return Err(VelocityError::ScriptFailed {
            package: package_json.name,
            script: script_name,
        });
    }

    Ok(())
}

/// Get the shell to use for running scripts
fn get_shell() -> String {
    if cfg!(windows) {
        env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    } else {
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

/// Get the shell argument for running commands
fn get_shell_arg() -> String {
    if cfg!(windows) {
        "/c".to_string()
    } else {
        "-c".to_string()
    }
}
