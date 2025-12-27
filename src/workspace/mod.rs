//! Workspace manager for monorepos

pub mod graph;

use std::path::{Path, PathBuf};

use crate::core::{VelocityResult, VelocityError, PackageJson};
use crate::core::config::WorkspaceConfig;

pub use graph::WorkspaceGraph;

/// Workspace manager
pub struct WorkspaceManager {
    /// Workspace root directory
    root: PathBuf,

    /// Workspace configuration
    config: WorkspaceConfig,

    /// Discovered packages
    packages: Vec<PathBuf>,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new(root: &Path, config: &WorkspaceConfig) -> VelocityResult<Self> {
        let root = root.to_path_buf();
        let packages = Self::discover_packages(&root, &config.packages)?;

        Ok(Self {
            root,
            config: config.clone(),
            packages,
        })
    }

    /// Discover packages matching workspace patterns
    fn discover_packages(root: &Path, patterns: &[String]) -> VelocityResult<Vec<PathBuf>> {
        let mut packages = Vec::new();

        for pattern in patterns {
            let full_pattern = root.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();

            // Use glob to match patterns
            for entry in glob::glob(&pattern_str).map_err(|e| VelocityError::workspace(e.to_string()))? {
                match entry {
                    Ok(path) => {
                        // Check if it's a valid package (has package.json)
                        if path.join("package.json").exists() {
                            packages.push(path);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error matching workspace pattern: {}", e);
                    }
                }
            }
        }

        packages.sort();
        Ok(packages)
    }

    /// Get all package paths
    pub fn find_packages(&self) -> VelocityResult<Vec<PathBuf>> {
        Ok(self.packages.clone())
    }

    /// Get the workspace root
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if a path is a workspace package
    pub fn is_package(&self, path: &Path) -> bool {
        self.packages.iter().any(|p| p == path)
    }

    /// Get package.json for all packages
    pub fn package_jsons(&self) -> VelocityResult<Vec<(PathBuf, PackageJson)>> {
        let mut result = Vec::new();

        for pkg_path in &self.packages {
            let pkg = PackageJson::load(pkg_path)?;
            result.push((pkg_path.clone(), pkg));
        }

        Ok(result)
    }

    /// Build a workspace dependency graph
    pub fn build_graph(&self) -> VelocityResult<WorkspaceGraph> {
        let mut graph = WorkspaceGraph::new();

        let packages = self.package_jsons()?;

        // Add all packages
        for (path, pkg) in &packages {
            graph.add_package(&pkg.name, path.clone());
        }

        // Add dependencies
        let package_names: Vec<String> = packages.iter().map(|(_, p)| p.name.clone()).collect();

        for (_, pkg) in &packages {
            for dep_name in pkg.all_dependencies().keys() {
                if package_names.contains(dep_name) {
                    graph.add_dependency(&pkg.name, dep_name);
                }
            }
        }

        Ok(graph)
    }

    /// Get topological order for building
    pub fn build_order(&self) -> VelocityResult<Vec<PathBuf>> {
        let graph = self.build_graph()?;
        let order = graph.topological_order()?;

        let mut paths = Vec::new();
        for name in order {
            if let Some(path) = self.packages.iter().find(|p| {
                PackageJson::load(p).map(|pkg| pkg.name == name).unwrap_or(false)
            }) {
                paths.push(path.clone());
            }
        }

        Ok(paths)
    }

    /// Check if hoisting is enabled
    pub fn should_hoist(&self) -> bool {
        self.config.hoist
    }

    /// Check if shared lockfile is enabled
    pub fn shared_lockfile(&self) -> bool {
        self.config.shared_lockfile
    }
}
