//! Core module for Velocity package manager
//!
//! This module contains the central engine, configuration, error types,
//! and lockfile handling.

pub mod config;
pub mod error;
pub mod lockfile;
pub mod engine;
pub mod package;

pub use config::Config;
pub use error::{VelocityError, VelocityResult};
pub use lockfile::Lockfile;
pub use engine::Engine;
pub use package::PackageJson;
