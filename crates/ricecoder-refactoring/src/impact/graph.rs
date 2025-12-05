//! Dependency graph construction and analysis

use std::collections::{HashMap, HashSet, VecDeque};

/// Represents a symbol in the dependency graph
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// File path where the symbol is defined
    pub file: String,
    /// Symbol type (function, struct, enum, etc.)
    pub symbol_type: SymbolType,
}

/// Type of symbol
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolType {
    /// Function or method
    Function,
    /// Struct or class
    Struct,
    /// Enum
    Enum,
    /// Trait or interface
    Trait,
    /// Module
    Module,
    /// Variable or constant
    Variable,
    /// Type alias
    TypeAlias,
    /// Macro
    Macro,
    /// Other
    Other,
}

/// Represents a dependency between two symbols
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// Source symbol
    pub from: Symbol,
    /// Target symbol
    pub to: Symbol,
    /// Type of dependency
    pub dep_type: DependencyType,
}

/// Type of dependency
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    /// Direct call or reference
    Direct,
    /// Trait implementation
    TraitImpl,
    /// Type reference
    TypeRef,
    /// Module import
    Import,
    /// Generic type parameter
    Generic,
}

/// Dependency graph for code analysis
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// All symbols in the graph (keyed by "name:file" to handle duplicate names in different files)
    symbols: HashMap<String, Symbol>,
    /// Dependencies between symbols
    dependencies: Vec<Dependency>,
    /// Adjacency list for efficient traversal (keyed by "name:file")
    adjacency: HashMap<String, Vec<String>>,
    /// Reverse adjacency list (who depends on this symbol) (keyed by "name:file")
    reverse_adjacency: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            dependencies: Vec::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    /// Add a symbol to the graph
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let key = format!("{}:{}", symbol.name, symbol.file);
        self.symbols.insert(key, symbol);
    }

    /// Add a dependency between two symbols
    pub fn add_dependency(&mut self, dependency: Dependency) {
        let from_key = format!("{}:{}", dependency.from.name, dependency.from.file);
        let to_key = format!("{}:{}", dependency.to.name, dependency.to.file);

        // Add symbols if they don't exist
        self.add_symbol(dependency.from.clone());
        self.add_symbol(dependency.to.clone());

        // Add to adjacency list
        self.adjacency
            .entry(from_key.clone())
            .or_default()
            .push(to_key.clone());

        // Add to reverse adjacency list
        self.reverse_adjacency
            .entry(to_key)
            .or_default()
            .push(from_key);

        self.dependencies.push(dependency);
    }

    /// Get all symbols that depend on a given symbol (direct dependents)
    /// Returns just the symbol names (without file paths)
    pub fn get_direct_dependents(&self, symbol_key: &str) -> Vec<String> {
        let dependents = self.reverse_adjacency
            .get(symbol_key)
            .cloned()
            .unwrap_or_default();
        
        // Extract symbol names from composite keys (name:file)
        dependents.iter().map(|key| {
            if let Some(colon_pos) = key.find(':') {
                key[..colon_pos].to_string()
            } else {
                key.clone()
            }
        }).collect()
    }

    /// Get all symbols that a given symbol depends on (direct dependencies)
    /// Returns just the symbol names (without file paths)
    pub fn get_direct_dependencies(&self, symbol_key: &str) -> Vec<String> {
        let dependencies = self.adjacency
            .get(symbol_key)
            .cloned()
            .unwrap_or_default();
        
        // Extract symbol names from composite keys (name:file)
        dependencies.iter().map(|key| {
            if let Some(colon_pos) = key.find(':') {
                key[..colon_pos].to_string()
            } else {
                key.clone()
            }
        }).collect()
    }

    /// Get all symbols transitively affected by a change to a given symbol
    /// This includes all symbols that depend on the changed symbol, directly or indirectly
    /// Returns just the symbol names (without file paths)
    pub fn get_transitive_dependents(&self, symbol_key: &str) -> HashSet<String> {
        let mut affected_keys = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with direct dependents
        if let Some(dependents) = self.reverse_adjacency.get(symbol_key) {
            for dependent in dependents {
                queue.push_back(dependent.clone());
                affected_keys.insert(dependent.clone());
            }
        }

        // BFS to find all transitive dependents
        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = self.reverse_adjacency.get(&current) {
                for dependent in dependents {
                    if !affected_keys.contains(dependent) {
                        affected_keys.insert(dependent.clone());
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        // Extract symbol names from composite keys (name:file)
        affected_keys.iter().map(|key| {
            if let Some(colon_pos) = key.find(':') {
                key[..colon_pos].to_string()
            } else {
                key.clone()
            }
        }).collect()
    }

    /// Get all symbols that a given symbol transitively depends on
    pub fn get_transitive_dependencies(&self, symbol_name: &str) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with direct dependencies
        if let Some(deps) = self.adjacency.get(symbol_name) {
            for dep in deps {
                queue.push_back(dep.clone());
                dependencies.insert(dep.clone());
            }
        }

        // BFS to find all transitive dependencies
        while let Some(current) = queue.pop_front() {
            if let Some(deps) = self.adjacency.get(&current) {
                for dep in deps {
                    if !dependencies.contains(dep) {
                        dependencies.insert(dep.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        dependencies
    }

    /// Detect circular dependencies
    pub fn find_circular_dependencies(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for symbol_name in self.symbols.keys() {
            if !visited.contains(symbol_name) {
                self.dfs_cycles(symbol_name, &mut visited, &mut rec_stack, &mut cycles, Vec::new());
            }
        }

        cycles
    }

    /// DFS helper for cycle detection
    fn dfs_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
        mut path: Vec<String>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycles(neighbor, visited, rec_stack, cycles, path.clone());
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|x| x == neighbor) {
                        let cycle: Vec<String> = path[start_idx..].to_vec();
                        if !cycles.contains(&cycle) {
                            cycles.push(cycle);
                        }
                    }
                }
            }
        }

        rec_stack.remove(node);
    }

    /// Get all symbols in the graph
    pub fn get_symbols(&self) -> Vec<Symbol> {
        self.symbols.values().cloned().collect()
    }

    /// Get all dependencies in the graph
    pub fn get_dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// Get the number of symbols in the graph
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Get the number of dependencies in the graph
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
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
    fn test_add_symbol() {
        let mut graph = DependencyGraph::new();
        let symbol = Symbol {
            name: "foo".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };

        graph.add_symbol(symbol.clone());
        assert_eq!(graph.symbol_count(), 1);
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let from = Symbol {
            name: "foo".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let to = Symbol {
            name: "bar".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };

        let dep = Dependency {
            from,
            to,
            dep_type: DependencyType::Direct,
        };

        graph.add_dependency(dep);
        assert_eq!(graph.symbol_count(), 2);
        assert_eq!(graph.dependency_count(), 1);
    }

    #[test]
    fn test_get_direct_dependents() {
        let mut graph = DependencyGraph::new();
        let foo = Symbol {
            name: "foo".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let bar = Symbol {
            name: "bar".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };

        graph.add_dependency(Dependency {
            from: bar.clone(),
            to: foo.clone(),
            dep_type: DependencyType::Direct,
        });

        let dependents = graph.get_direct_dependents("foo:main.rs");
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], "bar");
    }

    #[test]
    fn test_transitive_dependents() {
        let mut graph = DependencyGraph::new();
        let a = Symbol {
            name: "a".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let b = Symbol {
            name: "b".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let c = Symbol {
            name: "c".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };

        // a -> b -> c (c depends on b, b depends on a)
        graph.add_dependency(Dependency {
            from: b.clone(),
            to: a.clone(),
            dep_type: DependencyType::Direct,
        });
        graph.add_dependency(Dependency {
            from: c.clone(),
            to: b.clone(),
            dep_type: DependencyType::Direct,
        });

        let affected = graph.get_transitive_dependents("a:main.rs");
        assert_eq!(affected.len(), 2);
        assert!(affected.contains("b"));
        assert!(affected.contains("c"));
    }

    #[test]
    fn test_circular_dependencies() {
        let mut graph = DependencyGraph::new();
        let a = Symbol {
            name: "a".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let b = Symbol {
            name: "b".to_string(),
            file: "main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };

        // Create a cycle: a -> b -> a
        graph.add_dependency(Dependency {
            from: a.clone(),
            to: b.clone(),
            dep_type: DependencyType::Direct,
        });
        graph.add_dependency(Dependency {
            from: b.clone(),
            to: a.clone(),
            dep_type: DependencyType::Direct,
        });

        let cycles = graph.find_circular_dependencies();
        assert!(!cycles.is_empty());
    }
}
