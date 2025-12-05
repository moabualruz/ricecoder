//! Impact analysis for refactoring operations

pub mod analyzer;
pub mod graph;

pub use analyzer::ImpactAnalyzer;
pub use graph::{DependencyGraph, Symbol, SymbolType, Dependency, DependencyType};
