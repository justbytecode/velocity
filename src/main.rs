//! Velocity - A next-generation frontend package manager
//!
//! Velocity is a high-performance, secure package manager for JavaScript/TypeScript
//! projects, written in Rust. It aims to be faster than pnpm while maintaining
//! full npm registry compatibility.

mod cli;
mod core;
mod resolver;
mod installer;
mod cache;
mod security;
mod workspace;
mod registry;
mod templates;
mod permissions;
mod utils;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use cli::{Cli, Commands};
use core::VelocityResult;

#[tokio::main]
async fn main() -> VelocityResult<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let cli = Cli::parse();

    // Set up output mode
    let json_output = cli.json;

    // Execute command
    let result = match cli.command {
        Commands::Init(args) => cli::commands::init::execute(args, json_output).await,
        Commands::Install(args) => cli::commands::install::execute(args, json_output).await,
        Commands::Add(args) => cli::commands::add::execute(args, json_output).await,
        Commands::Remove(args) => cli::commands::remove::execute(args, json_output).await,
        Commands::Update(args) => cli::commands::update::execute(args, json_output).await,
        Commands::Run(args) => cli::commands::run::execute(args, json_output).await,
        Commands::Doctor(args) => cli::commands::doctor::execute(args, json_output).await,
        Commands::Audit(args) => cli::commands::audit::execute(args, json_output).await,
        Commands::Cache(args) => cli::commands::cache::execute(args, json_output).await,
        Commands::Migrate(args) => cli::commands::migrate::execute(args, json_output).await,
        Commands::Upgrade(args) => cli::commands::upgrade::execute(args, json_output).await,
        Commands::Create(args) => cli::commands::create::execute(args, json_output).await,
        Commands::Workspace(args) => cli::commands::workspace::execute(args, json_output).await,
    };

    if let Err(ref e) = result {
        if json_output {
            let error_json = serde_json::json!({
                "error": true,
                "message": e.to_string()
            });
            eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap());
        } else {
            eprintln!("{} {}", console::style("error:").red().bold(), e);
        }
        std::process::exit(1);
    }

    Ok(())
}
