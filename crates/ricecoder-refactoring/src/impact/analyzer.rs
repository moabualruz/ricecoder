//! Impact analysis for refactoring operations

use std::{collections::HashSet, path::PathBuf};

#[allow(unused_imports)]
use super::graph::{Dependency, DependencyGraph, DependencyType, Symbol, SymbolType};
use crate::{
    error::Result,
    types::{ImpactAnalysis, Refactoring, RefactoringTarget, RiskLevel},
};

/// Analyzes the impact of refactoring operations
pub struct ImpactAnalyzer {
    /// Dependency graph for the codebase
    graph: DependencyGraph,
}

impl ImpactAnalyzer {
    /// Create a new impact analyzer
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    /// Create an analyzer with a pre-built dependency graph
    pub fn with_graph(graph: DependencyGraph) -> Self {
        Self { graph }
    }

    /// Add a symbol to the dependency graph
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.graph.add_symbol(symbol);
    }

    /// Add a dependency to the graph
    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.graph.add_dependency(dependency);
    }

    /// Analyze the impact of a refactoring
    pub fn analyze(&self, refactoring: &Refactoring) -> Result<ImpactAnalysis> {
        let affected_symbols = self.find_affected_symbols(&refactoring.target)?;
        let affected_files = self.find_affected_files(&affected_symbols)?;

        let affected_symbols_vec: Vec<String> = affected_symbols.iter().cloned().collect();
        let risk_level = Self::calculate_risk_level(&affected_files, &affected_symbols_vec);
        let estimated_effort = Self::estimate_effort(&affected_files, &affected_symbols_vec);

        Ok(ImpactAnalysis {
            affected_files,
            affected_symbols: affected_symbols_vec,
            risk_level,
            estimated_effort,
        })
    }

    /// Find all symbols affected by a refactoring
    /// This includes the target symbol and all symbols that transitively depend on it
    fn find_affected_symbols(&self, target: &RefactoringTarget) -> Result<HashSet<String>> {
        let mut affected = HashSet::new();

        // Add the target symbol itself
        affected.insert(target.symbol.clone());

        // Create a composite key for the target symbol (name:file)
        let target_key = format!("{}:{}", target.symbol, target.file.display());

        // Find all symbols that transitively depend on the target
        // get_transitive_dependents returns just symbol names (not composite keys)
        let transitive_dependents = self.graph.get_transitive_dependents(&target_key);
        affected.extend(transitive_dependents);

        Ok(affected)
    }

    /// Find files affected by the refactoring
    /// Returns unique files that contain affected symbols
    fn find_affected_files(&self, affected_symbols: &HashSet<String>) -> Result<Vec<PathBuf>> {
        let mut files = HashSet::new();

        // Get all symbols from the graph
        let all_symbols = self.graph.get_symbols();

        // Build a map of symbol names to their files
        let mut symbol_files: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for symbol in all_symbols {
            symbol_files
                .entry(symbol.name.clone())
                .or_default()
                .push(symbol.file.clone());
        }

        // For each affected symbol, add all files where it appears
        for symbol_name in affected_symbols {
            if let Some(file_list) = symbol_files.get(symbol_name) {
                for file in file_list {
                    files.insert(PathBuf::from(file));
                }
            }
        }

        Ok(files.into_iter().collect())
    }

    /// Calculate risk level based on impact
    fn calculate_risk_level(affected_files: &[PathBuf], affected_symbols: &[String]) -> RiskLevel {
        let total_impact = affected_files.len() + affected_symbols.len();

        match total_impact {
            0..=2 => RiskLevel::Low,
            3..=10 => RiskLevel::Medium,
            _ => RiskLevel::High,
        }
    }

    /// Estimate effort required for the refactoring
    fn estimate_effort(affected_files: &[PathBuf], affected_symbols: &[String]) -> u8 {
        let total_impact = affected_files.len() + affected_symbols.len();
        std::cmp::min(10, (total_impact as u8) + 1)
    }

    /// Generate a detailed impact report
    pub fn generate_report(&self, refactoring: &Refactoring) -> Result<ImpactReport> {
        let analysis = self.analyze(refactoring)?;
        let circular_deps = self.graph.find_circular_dependencies();
        let breaking_changes = self.detect_breaking_changes(&analysis)?;
        let recommendations = self.generate_recommendations(&analysis);

        Ok(ImpactReport {
            analysis,
            circular_dependencies: circular_deps,
            breaking_changes,
            recommendations,
        })
    }

    /// Detect potential breaking changes
    fn detect_breaking_changes(&self, analysis: &ImpactAnalysis) -> Result<Vec<String>> {
        let mut breaking_changes = Vec::new();

        // Check for high-risk changes
        if analysis.risk_level == RiskLevel::High {
            breaking_changes
                .push("High-risk refactoring: affects many symbols and files".to_string());
        }

        // Check for circular dependencies
        let cycles = self.graph.find_circular_dependencies();
        if !cycles.is_empty() {
            breaking_changes.push(format!(
                "Circular dependencies detected: {} cycles found",
                cycles.len()
            ));
        }

        Ok(breaking_changes)
    }

    /// Generate recommendations for the refactoring
    fn generate_recommendations(&self, analysis: &ImpactAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        match analysis.risk_level {
            RiskLevel::Low => {
                recommendations.push("Low-risk refactoring. Safe to proceed.".to_string());
            }
            RiskLevel::Medium => {
                recommendations.push(
                    "Medium-risk refactoring. Review affected symbols carefully.".to_string(),
                );
                recommendations.push("Consider running tests after refactoring.".to_string());
            }
            RiskLevel::High => {
                recommendations.push("High-risk refactoring. Proceed with caution.".to_string());
                recommendations.push("Create a backup before applying changes.".to_string());
                recommendations.push("Run comprehensive tests after refactoring.".to_string());
                recommendations
                    .push("Consider breaking the refactoring into smaller steps.".to_string());
            }
        }

        if analysis.affected_files.len() > 10 {
            recommendations.push(format!(
                "Many files affected ({}). Consider a phased approach.",
                analysis.affected_files.len()
            ));
        }

        recommendations
    }

    /// Get the dependency graph
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }
}

impl Default for ImpactAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed impact report for a refactoring
#[derive(Debug, Clone)]
pub struct ImpactReport {
    /// Impact analysis
    pub analysis: ImpactAnalysis,
    /// Circular dependencies found
    pub circular_dependencies: Vec<Vec<String>>,
    /// Potential breaking changes
    pub breaking_changes: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Refactoring, RefactoringOptions, RefactoringType};

    #[test]
    fn test_analyze_refactoring() -> Result<()> {
        let mut analyzer = ImpactAnalyzer::new();

        // Add symbols to the graph
        let target_symbol = Symbol {
            name: "old_name".to_string(),
            file: "src/main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        analyzer.add_symbol(target_symbol);

        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from("src/main.rs"),
                symbol: "old_name".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let impact = analyzer.analyze(&refactoring)?;
        assert_eq!(impact.affected_files.len(), 1);
        assert_eq!(impact.affected_symbols.len(), 1);

        Ok(())
    }

    #[test]
    fn test_transitive_impact() -> Result<()> {
        let mut analyzer = ImpactAnalyzer::new();

        // Create a chain: a -> b -> c
        let a = Symbol {
            name: "a".to_string(),
            file: "src/main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let b = Symbol {
            name: "b".to_string(),
            file: "src/lib.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        let c = Symbol {
            name: "c".to_string(),
            file: "src/utils.rs".to_string(),
            symbol_type: SymbolType::Function,
        };

        analyzer.add_symbol(a.clone());
        analyzer.add_symbol(b.clone());
        analyzer.add_symbol(c.clone());

        // b depends on a, c depends on b
        analyzer.add_dependency(Dependency {
            from: b.clone(),
            to: a.clone(),
            dep_type: DependencyType::Direct,
        });
        analyzer.add_dependency(Dependency {
            from: c.clone(),
            to: b.clone(),
            dep_type: DependencyType::Direct,
        });

        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from("src/main.rs"),
                symbol: "a".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let impact = analyzer.analyze(&refactoring)?;
        // Should affect a, b, and c
        assert_eq!(impact.affected_symbols.len(), 3);
        assert_eq!(impact.affected_files.len(), 3);

        Ok(())
    }

    #[test]
    fn test_risk_level_low() {
        let files = vec![PathBuf::from("src/main.rs")];
        let symbols = vec!["symbol1".to_string()];
        let risk = ImpactAnalyzer::calculate_risk_level(&files, &symbols);
        assert_eq!(risk, RiskLevel::Low);
    }

    #[test]
    fn test_risk_level_medium() {
        let files = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            PathBuf::from("src/utils.rs"),
        ];
        let symbols = vec!["symbol1".to_string(), "symbol2".to_string()];
        let risk = ImpactAnalyzer::calculate_risk_level(&files, &symbols);
        assert_eq!(risk, RiskLevel::Medium);
    }

    #[test]
    fn test_risk_level_high() {
        let files: Vec<PathBuf> = (0..15)
            .map(|i| PathBuf::from(format!("src/file{}.rs", i)))
            .collect();
        let symbols: Vec<String> = (0..10).map(|i| format!("symbol{}", i)).collect();
        let risk = ImpactAnalyzer::calculate_risk_level(&files, &symbols);
        assert_eq!(risk, RiskLevel::High);
    }

    #[test]
    fn test_estimate_effort() {
        let files = vec![PathBuf::from("src/main.rs")];
        let symbols = vec!["symbol1".to_string()];
        let effort = ImpactAnalyzer::estimate_effort(&files, &symbols);
        assert!(effort > 0 && effort <= 10);
    }

    #[test]
    fn test_generate_report() -> Result<()> {
        let mut analyzer = ImpactAnalyzer::new();

        let target_symbol = Symbol {
            name: "old_name".to_string(),
            file: "src/main.rs".to_string(),
            symbol_type: SymbolType::Function,
        };
        analyzer.add_symbol(target_symbol);

        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from("src/main.rs"),
                symbol: "old_name".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let report = analyzer.generate_report(&refactoring)?;
        assert!(!report.recommendations.is_empty());

        Ok(())
    }
}
