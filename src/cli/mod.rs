//! CLI module for Velocity
//!
//! Provides command-line interface using clap.

pub mod commands;
pub mod output;

use clap::{Parser, Subcommand};

use commands::*;

/// Velocity - A next-generation frontend package manager
#[derive(Parser)]
#[command(name = "velocity")]
#[command(author = "Velocity Contributors")]
#[command(version)]
#[command(about = "A fast, secure package manager for JavaScript projects", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project
    #[command(visible_alias = "i")]
    Init(init::InitArgs),

    /// Install all dependencies
    Install(install::InstallArgs),

    /// Add a package
    #[command(visible_alias = "a")]
    Add(add::AddArgs),

    /// Remove a package
    #[command(visible_aliases = ["rm", "uninstall"])]
    Remove(remove::RemoveArgs),

    /// Update packages to their latest versions
    #[command(visible_alias = "up")]
    Update(update::UpdateArgs),

    /// Run a script defined in package.json
    #[command(visible_alias = "r")]
    Run(run::RunArgs),

    /// Diagnose environment and configuration issues
    Doctor(doctor::DoctorArgs),

    /// Security audit for dependencies
    Audit(audit::AuditArgs),

    /// Manage the package cache
    Cache(cache::CacheArgs),

    /// Migrate from another package manager
    Migrate(migrate::MigrateArgs),

    /// Upgrade Velocity to the latest version
    Upgrade(upgrade::UpgradeArgs),

    /// Create a new project from a template
    #[command(visible_alias = "c")]
    Create(create::CreateArgs),

    /// Workspace commands
    #[command(visible_alias = "ws")]
    Workspace(workspace::WorkspaceArgs),
}

