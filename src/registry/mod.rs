//! npm registry client

pub mod client;
pub mod types;

pub use client::RegistryClient;
pub use types::{PackageMetadata, VersionMetadata, DistInfo};
