//! Property-based tests for impact analysis
//! **Feature: ricecoder-refactoring, Property 2: Impact Analysis Completeness**
//! **Validates: Requirements REF-1.1, REF-1.2**

use proptest::prelude::*;
use ricecoder_refactoring::impact::{
    Dependency, DependencyType, ImpactAnalyzer, Symbol, SymbolType,
};
use ricecoder_refactoring::types::{
    Refactoring, RefactoringOptions, RefactoringTarget, RefactoringType,
};
use std::path::PathBuf;

/// Strategy for generating symbol names
fn symbol_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    r"src/[a-z0-9_/]{1,20}\.rs".prop_map(|s| s.to_string())
}

/// Strategy for generating symbols
fn symbol_strategy() -> impl Strategy<Value = Symbol> {
    (symbol_name_strategy(), file_path_strategy()).prop_map(|(name, file)| Symbol {
        name,
        file,
        symbol_type: SymbolType::Function,
    })
}

/// Strategy for generating dependencies
fn dependency_strategy() -> impl Strategy<Value = (Symbol, Symbol)> {
    (symbol_strategy(), symbol_strategy()).prop_map(|(from, to)| (from, to))
}

/// Property: For any code change, impact analysis correctly identifies all affected symbols
/// This property tests that when we change a symbol, all symbols that transitively depend on it
/// are correctly identified as affected.
proptest! {
    #[test]
    fn prop_impact_analysis_identifies_all_affected_symbols(
        target_symbol in symbol_strategy(),
        dependent_symbols in prop::collection::vec(symbol_strategy(), 1..10),
        refactoring_type in prop_oneof![
            Just(RefactoringType::Rename),
            Just(RefactoringType::Extract),
            Just(RefactoringType::Inline),
        ]
    ) {
        // Create analyzer and add symbols
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_symbol(target_symbol.clone());

        // Add dependent symbols and create dependencies
        for dependent in &dependent_symbols {
            analyzer.add_symbol(dependent.clone());
            analyzer.add_dependency(Dependency {
                from: dependent.clone(),
                to: target_symbol.clone(),
                dep_type: DependencyType::Direct,
            });
        }

        // Create refactoring for the target symbol
        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type,
            target: RefactoringTarget {
                file: PathBuf::from(&target_symbol.file),
                symbol: target_symbol.name.clone(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        // Analyze impact
        let impact = analyzer.analyze(&refactoring).expect("Analysis should succeed");

        // Property: The target symbol should always be in affected symbols
        prop_assert!(
            impact.affected_symbols.contains(&target_symbol.name),
            "Target symbol should be affected"
        );

        // Property: All direct dependents should be in affected symbols
        for dependent in &dependent_symbols {
            prop_assert!(
                impact.affected_symbols.contains(&dependent.name),
                "Dependent symbol {} should be affected",
                dependent.name
            );
        }

        // Property: Number of affected symbols should be at least 1 (the target)
        prop_assert!(
            impact.affected_symbols.len() >= 1,
            "At least the target symbol should be affected"
        );

        // Property: Number of affected symbols should not exceed total symbols
        let total_symbols = 1 + dependent_symbols.len();
        prop_assert!(
            impact.affected_symbols.len() <= total_symbols,
            "Affected symbols should not exceed total symbols"
        );
    }
}

/// Property: Transitive dependencies are correctly identified
/// When symbol A depends on B, and B depends on C, changing C should affect both B and A
proptest! {
    #[test]
    fn prop_transitive_dependencies_are_complete(
        symbol_a in symbol_strategy(),
        symbol_b in symbol_strategy(),
        symbol_c in symbol_strategy(),
    ) {
        // Ensure symbols are different
        prop_assume!(symbol_a.name != symbol_b.name);
        prop_assume!(symbol_b.name != symbol_c.name);
        prop_assume!(symbol_a.name != symbol_c.name);

        let mut analyzer = ImpactAnalyzer::new();

        // Create chain: A -> B -> C (A depends on B, B depends on C)
        analyzer.add_symbol(symbol_a.clone());
        analyzer.add_symbol(symbol_b.clone());
        analyzer.add_symbol(symbol_c.clone());

        analyzer.add_dependency(Dependency {
            from: symbol_a.clone(),
            to: symbol_b.clone(),
            dep_type: DependencyType::Direct,
        });

        analyzer.add_dependency(Dependency {
            from: symbol_b.clone(),
            to: symbol_c.clone(),
            dep_type: DependencyType::Direct,
        });

        // Analyze impact of changing C
        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from(&symbol_c.file),
                symbol: symbol_c.name.clone(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let impact = analyzer.analyze(&refactoring).expect("Analysis should succeed");

        // Property: Changing C should affect C, B, and A (all in the chain)
        prop_assert!(
            impact.affected_symbols.contains(&symbol_c.name),
            "C should be affected"
        );
        prop_assert!(
            impact.affected_symbols.contains(&symbol_b.name),
            "B should be affected (depends on C)"
        );
        prop_assert!(
            impact.affected_symbols.contains(&symbol_a.name),
            "A should be affected (transitively depends on C)"
        );

        // Property: Exactly 3 symbols should be affected
        prop_assert_eq!(
            impact.affected_symbols.len(),
            3,
            "All three symbols should be affected"
        );
    }
}

/// Property: Impact analysis is deterministic
/// Running the same analysis twice should produce the same result
proptest! {
    #[test]
    fn prop_impact_analysis_is_deterministic(
        target_symbol in symbol_strategy(),
        dependent_symbols in prop::collection::vec(symbol_strategy(), 0..5),
    ) {
        // Create first analyzer
        let mut analyzer1 = ImpactAnalyzer::new();
        analyzer1.add_symbol(target_symbol.clone());
        for dependent in &dependent_symbols {
            analyzer1.add_symbol(dependent.clone());
            analyzer1.add_dependency(Dependency {
                from: dependent.clone(),
                to: target_symbol.clone(),
                dep_type: DependencyType::Direct,
            });
        }

        // Create second analyzer with same setup
        let mut analyzer2 = ImpactAnalyzer::new();
        analyzer2.add_symbol(target_symbol.clone());
        for dependent in &dependent_symbols {
            analyzer2.add_symbol(dependent.clone());
            analyzer2.add_dependency(Dependency {
                from: dependent.clone(),
                to: target_symbol.clone(),
                dep_type: DependencyType::Direct,
            });
        }

        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from(&target_symbol.file),
                symbol: target_symbol.name.clone(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let impact1 = analyzer1.analyze(&refactoring).expect("First analysis should succeed");
        let impact2 = analyzer2.analyze(&refactoring).expect("Second analysis should succeed");

        // Property: Both analyses should produce the same affected symbols
        prop_assert_eq!(
            impact1.affected_symbols.len(),
            impact2.affected_symbols.len(),
            "Both analyses should identify the same number of affected symbols"
        );

        // Property: The affected symbols should be identical
        for symbol in &impact1.affected_symbols {
            prop_assert!(
                impact2.affected_symbols.contains(symbol),
                "Symbol {} should be in both analyses",
                symbol
            );
        }
    }
}

/// Property: Risk level increases with impact scope
/// Higher number of affected symbols should result in higher or equal risk level
proptest! {
    #[test]
    fn prop_risk_level_correlates_with_impact(
        target_symbol in symbol_strategy(),
        num_dependents in 0usize..20,
    ) {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_symbol(target_symbol.clone());

        // Add dependent symbols
        for i in 0..num_dependents {
            let dependent = Symbol {
                name: format!("dep_{}", i),
                file: format!("src/file_{}.rs", i),
                symbol_type: SymbolType::Function,
            };
            analyzer.add_symbol(dependent.clone());
            analyzer.add_dependency(Dependency {
                from: dependent,
                to: target_symbol.clone(),
                dep_type: DependencyType::Direct,
            });
        }

        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from(&target_symbol.file),
                symbol: target_symbol.name.clone(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let impact = analyzer.analyze(&refactoring).expect("Analysis should succeed");

        // Property: Risk level should correlate with impact scope
        // Risk is calculated as: affected_files.len() + affected_symbols.len()
        // Low: 0-2, Medium: 3-10, High: 11+
        let total_impact = impact.affected_files.len() + impact.affected_symbols.len();

        if total_impact <= 2 {
            prop_assert_eq!(
                impact.risk_level,
                ricecoder_refactoring::types::RiskLevel::Low,
                "Impact {} should have Low risk",
                total_impact
            );
        } else if total_impact <= 10 {
            prop_assert_eq!(
                impact.risk_level,
                ricecoder_refactoring::types::RiskLevel::Medium,
                "Impact {} should have Medium risk",
                total_impact
            );
        } else {
            prop_assert_eq!(
                impact.risk_level,
                ricecoder_refactoring::types::RiskLevel::High,
                "Impact {} should have High risk",
                total_impact
            );
        }

        // Property: Estimated effort should be between 1 and 10
        prop_assert!(
            impact.estimated_effort >= 1 && impact.estimated_effort <= 10,
            "Estimated effort should be between 1 and 10"
        );
    }
}

/// Property: Affected files correspond to affected symbols
/// Every affected symbol should have a corresponding file in affected_files
proptest! {
    #[test]
    fn prop_affected_files_match_affected_symbols(
        target_symbol in symbol_strategy(),
        dependent_symbols in prop::collection::vec(symbol_strategy(), 0..5),
    ) {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_symbol(target_symbol.clone());

        for dependent in &dependent_symbols {
            analyzer.add_symbol(dependent.clone());
            analyzer.add_dependency(Dependency {
                from: dependent.clone(),
                to: target_symbol.clone(),
                dep_type: DependencyType::Direct,
            });
        }

        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from(&target_symbol.file),
                symbol: target_symbol.name.clone(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let impact = analyzer.analyze(&refactoring).expect("Analysis should succeed");

        // Property: Target symbol's file should be in affected files
        let target_file = PathBuf::from(&target_symbol.file);
        prop_assert!(
            impact.affected_files.contains(&target_file),
            "Target symbol's file should be affected"
        );

        // Property: All dependent symbols' files should be in affected files
        for dependent in &dependent_symbols {
            let dependent_file = PathBuf::from(&dependent.file);
            prop_assert!(
                impact.affected_files.contains(&dependent_file),
                "Dependent symbol's file should be affected"
            );
        }
    }
}
