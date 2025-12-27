//! Dependency graph with cycle detection

use std::collections::{HashMap, HashSet};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::is_cyclic_directed;
use petgraph::Direction;

/// Dependency graph for resolved packages
#[derive(Debug)]
pub struct DependencyGraph {
    /// The underlying graph
    graph: DiGraph<String, ()>,
    /// Map from package name to node index
    nodes: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    /// Add a package to the graph
    pub fn add_package(&mut self, name: &str, version: &str) {
        let key = format!("{}@{}", name, version);
        if !self.nodes.contains_key(name) {
            let idx = self.graph.add_node(key);
            self.nodes.insert(name.to_string(), idx);
        }
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        if let (Some(&from_idx), Some(&to_idx)) = (self.nodes.get(from), self.nodes.get(to)) {
            // Check if edge already exists
            if !self.graph.contains_edge(from_idx, to_idx) {
                self.graph.add_edge(from_idx, to_idx, ());
            }
        }
    }

    /// Check if the graph has a cycle
    pub fn has_cycle(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Find a cycle in the graph (if any)
    pub fn find_cycle(&self) -> Option<Vec<String>> {
        // Simple DFS-based cycle detection
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.graph.node_indices() {
            if !visited.contains(&node) {
                if self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut path) {
                    return Some(path);
                }
            }
        }

        None
    }

    fn dfs_cycle(
        &self,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        rec_stack: &mut HashSet<NodeIndex>,
        path: &mut Vec<String>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(self.graph[node].clone());

        for neighbor in self.graph.neighbors_directed(node, Direction::Outgoing) {
            if !visited.contains(&neighbor) {
                if self.dfs_cycle(neighbor, visited, rec_stack, path) {
                    return true;
                }
            } else if rec_stack.contains(&neighbor) {
                path.push(self.graph[neighbor].clone());
                return true;
            }
        }

        path.pop();
        rec_stack.remove(&node);
        false
    }

    /// Get all packages in topological order
    pub fn topological_order(&self) -> Vec<String> {
        use petgraph::algo::toposort;

        match toposort(&self.graph, None) {
            Ok(order) => order
                .into_iter()
                .map(|idx| self.graph[idx].clone())
                .collect(),
            Err(_) => {
                // Has cycle, return empty
                Vec::new()
            }
        }
    }

    /// Get direct dependencies of a package
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

    /// Get packages that depend on this package
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

    /// Get the number of packages
    pub fn package_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get all package names
    pub fn packages(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// Check if a package is in the graph
    pub fn has_package(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_package("a", "1.0.0");
        graph.add_package("b", "1.0.0");
        graph.add_package("c", "1.0.0");

        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");

        assert!(!graph.has_cycle());
        assert_eq!(graph.package_count(), 3);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_package("a", "1.0.0");
        graph.add_package("b", "1.0.0");
        graph.add_package("c", "1.0.0");

        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");
        graph.add_dependency("c", "a"); // Creates cycle

        assert!(graph.has_cycle());
        assert!(graph.find_cycle().is_some());
    }

    #[test]
    fn test_topological_order() {
        let mut graph = DependencyGraph::new();
        graph.add_package("c", "1.0.0");
        graph.add_package("b", "1.0.0");
        graph.add_package("a", "1.0.0");

        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");

        let order = graph.topological_order();
        assert!(!order.is_empty());
    }
}
