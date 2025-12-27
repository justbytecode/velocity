//! Workspace dependency graph

use std::collections::HashMap;
use std::path::PathBuf;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, is_cyclic_directed};
use petgraph::Direction;

use crate::core::{VelocityResult, VelocityError};

/// Workspace package dependency graph
pub struct WorkspaceGraph {
    /// The underlying graph
    graph: DiGraph<String, ()>,
    /// Map from package name to node index
    nodes: HashMap<String, NodeIndex>,
    /// Map from package name to path
    paths: HashMap<String, PathBuf>,
}

impl WorkspaceGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// Add a package to the graph
    pub fn add_package(&mut self, name: &str, path: PathBuf) {
        if !self.nodes.contains_key(name) {
            let idx = self.graph.add_node(name.to_string());
            self.nodes.insert(name.to_string(), idx);
            self.paths.insert(name.to_string(), path);
        }
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        if let (Some(&from_idx), Some(&to_idx)) = (self.nodes.get(from), self.nodes.get(to)) {
            if !self.graph.contains_edge(from_idx, to_idx) {
                self.graph.add_edge(from_idx, to_idx, ());
            }
        }
    }

    /// Get topological order (dependencies first)
    pub fn topological_order(&self) -> VelocityResult<Vec<String>> {
        match toposort(&self.graph, None) {
            Ok(order) => Ok(order
                .into_iter()
                .rev() // Reverse to get dependencies first
                .map(|idx| self.graph[idx].clone())
                .collect()),
            Err(_) => Err(VelocityError::workspace(
                "Circular dependency detected in workspace",
            )),
        }
    }

    /// Check for cycles
    pub fn has_cycle(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Get dependencies of a package
    pub fn dependencies(&self, name: &str) -> Vec<String> {
        if let Some(&idx) = self.nodes.get(name) {
            self.graph
                .neighbors_directed(idx, Direction::Outgoing)
                .map(|n| self.graph[n].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get dependents (reverse dependencies) of a package
    pub fn dependents(&self, name: &str) -> Vec<String> {
        if let Some(&idx) = self.nodes.get(name) {
            self.graph
                .neighbors_directed(idx, Direction::Incoming)
                .map(|n| self.graph[n].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get the path for a package
    pub fn get_path(&self, name: &str) -> Option<&PathBuf> {
        self.paths.get(name)
    }

    /// Get all package names
    pub fn packages(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// Get the number of packages
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for WorkspaceGraph {
    fn default() -> Self {
        Self::new()
    }
}
