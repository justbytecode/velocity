//! Dependency resolver for Velocity
//!
//! Implements deterministic version resolution with conflict detection.

pub mod version;
pub mod graph;

use std::collections::HashMap;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::core::{Lockfile, lockfile::LockedPackage, VelocityError, VelocityResult};
use crate::registry::RegistryClient;

pub use graph::DependencyGraph;
pub use version::VersionConstraint;

/// Resolution result containing the dependency graph and lockfile
pub struct Resolution {
    /// The resolved dependency graph
    pub graph: DependencyGraph,

    /// Generated lockfile
    pub lockfile: Lockfile,

    /// Packages to install (not in cache)
    pub to_install: Vec<ResolvedPackage>,

    /// Packages available in cache
    pub from_cache: Vec<ResolvedPackage>,
}

/// A resolved package with all metadata
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub tarball_url: String,
    pub integrity: String,
    pub dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub optional_dependencies: HashMap<String, String>,
    pub has_scripts: bool,
}

/// Dependency resolver
pub struct Resolver {
    registry: Arc<RegistryClient>,
    cache: Arc<CacheManager>,
}

impl Resolver {
    /// Create a new resolver
    pub fn new(registry: Arc<RegistryClient>, cache: Arc<CacheManager>) -> Self {
        Self { registry, cache }
    }

    /// Resolve dependencies from a dependency map
    pub async fn resolve(
        &self,
        dependencies: &HashMap<String, String>,
    ) -> VelocityResult<Resolution> {
        let mut graph = DependencyGraph::new();
        let mut lockfile = Lockfile::new();
        let mut to_install = Vec::new();
        let mut from_cache = Vec::new();
        let mut resolved_versions: HashMap<String, String> = HashMap::new();

        // Queue of (name, constraint, depth)
        let mut queue: Vec<(String, String, usize)> = dependencies
            .iter()
            .map(|(n, v)| (n.clone(), v.clone(), 0))
            .collect();

        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

        while let Some((name, constraint_str, depth)) = queue.pop() {
            let cache_key = format!("{}@{}", name, constraint_str);
            if visited.contains(&cache_key) {
                continue;
            }
            visited.insert(cache_key);

            // Get package metadata from registry
            let metadata = self.registry.get_package_metadata(&name).await?;

            // Parse constraint and find best matching version
            let constraint = VersionConstraint::parse(&constraint_str)?;
            let matching_version = self.find_matching_version(&metadata.versions, &constraint)?;

            // Check for conflicts
            if let Some(existing) = resolved_versions.get(&name) {
                if *existing != matching_version {
                    // Try to find a version that satisfies both
                    // For now, use the higher version
                    let existing_semver = semver::Version::parse(existing).ok();
                    let new_semver = semver::Version::parse(&matching_version).ok();

                    match (existing_semver, new_semver) {
                        (Some(e), Some(n)) if e >= n => continue,
                        _ => {}
                    }
                }
            }

            resolved_versions.insert(name.clone(), matching_version.clone());

            // Get version-specific metadata
            let version_meta = metadata.versions.get(&matching_version)
                .ok_or_else(|| VelocityError::VersionNotFound {
                    package: name.clone(),
                    version: matching_version.clone(),
                })?;

            let resolved = ResolvedPackage {
                name: name.clone(),
                version: matching_version.clone(),
                tarball_url: version_meta.dist.tarball.clone(),
                integrity: version_meta.dist.integrity.clone().unwrap_or_default(),
                dependencies: version_meta.dependencies.clone(),
                peer_dependencies: version_meta.peer_dependencies.clone(),
                optional_dependencies: version_meta.optional_dependencies.clone(),
                has_scripts: version_meta.has_install_scripts(),
            };

            // Add to graph
            graph.add_package(&name, &matching_version);
            for (dep_name, _) in &resolved.dependencies {
                graph.add_dependency(&name, dep_name);
            }

            // Check cache
            if self.cache.has_package(&name, &matching_version)? {
                from_cache.push(resolved.clone());
            } else {
                to_install.push(resolved.clone());
            }

            // Add to lockfile
            lockfile.add_package(LockedPackage {
                name: name.clone(),
                version: matching_version.clone(),
                resolved: resolved.tarball_url.clone(),
                integrity: resolved.integrity.clone(),
                dependencies: resolved.dependencies.keys().map(|k| {
                    format!("{}@{}", k, resolved.dependencies.get(k).unwrap())
                }).collect(),
                peer_dependencies: resolved.peer_dependencies.keys().cloned().collect(),
                optional_dependencies: resolved.optional_dependencies.keys().cloned().collect(),
                has_scripts: resolved.has_scripts,
                cpu: vec![],
                os: vec![],
            });

            // Queue dependencies (limit depth to prevent infinite loops)
            if depth < 100 {
                for (dep_name, dep_constraint) in &resolved.dependencies {
                    queue.push((dep_name.clone(), dep_constraint.clone(), depth + 1));
                }

                // Optional dependencies are best-effort
                for (dep_name, dep_constraint) in &resolved.optional_dependencies {
                    queue.push((dep_name.clone(), dep_constraint.clone(), depth + 1));
                }
            }
        }

        // Check for cycles
        if graph.has_cycle() {
            let cycle = graph.find_cycle().unwrap_or_default();
            return Err(VelocityError::CircularDependency(cycle.join(" -> ")));
        }

        Ok(Resolution {
            graph,
            lockfile,
            to_install,
            from_cache,
        })
    }

    /// Find the best matching version for a constraint
    fn find_matching_version(
        &self,
        versions: &HashMap<String, crate::registry::types::VersionMetadata>,
        constraint: &VersionConstraint,
    ) -> VelocityResult<String> {
        let mut matching: Vec<semver::Version> = versions
            .keys()
            .filter_map(|v| semver::Version::parse(v).ok())
            .filter(|v| constraint.matches(v))
            .collect();

        matching.sort();
        matching.reverse();

        matching
            .first()
            .map(|v| v.to_string())
            .ok_or_else(|| VelocityError::InvalidVersionConstraint(constraint.to_string()))
    }
}
