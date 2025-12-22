//! Refactoring patterns for reusable transformations

pub mod exporter;
pub mod matcher;
pub mod store;
pub mod validator;

use std::collections::HashMap;

pub use exporter::PatternExporter;
pub use matcher::PatternMatcher;
use serde::{Deserialize, Serialize};
pub use store::PatternStore;
pub use validator::PatternValidator;

/// A reusable refactoring pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringPattern {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Pattern template with placeholders
    pub template: String,
    /// Pattern parameters
    pub parameters: Vec<PatternParameter>,
    /// Pattern scope (global or project)
    pub scope: PatternScope,
}

/// A pattern parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternParameter {
    /// Parameter name
    pub name: String,
    /// Placeholder in template (e.g., {{old_name}})
    pub placeholder: String,
    /// Parameter description
    pub description: String,
}

/// Pattern scope
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternScope {
    /// Global pattern
    Global,
    /// Project-specific pattern
    Project,
}

/// Pattern application with parameter values
#[derive(Debug, Clone)]
pub struct PatternApplication {
    /// The pattern being applied
    pub pattern: RefactoringPattern,
    /// Parameter values
    pub parameters: HashMap<String, String>,
}

impl PatternApplication {
    /// Apply the pattern to code
    pub fn apply(&self, _code: &str) -> crate::error::Result<String> {
        let mut result = self.pattern.template.clone();

        for (name, value) in &self.parameters {
            let placeholder = format!("{{{{{}}}}}", name);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_application() -> crate::error::Result<()> {
        let pattern = RefactoringPattern {
            name: "rename".to_string(),
            description: "Rename a function".to_string(),
            template: "fn {{old_name}}() {} -> fn {{new_name}}() {}".to_string(),
            parameters: vec![
                PatternParameter {
                    name: "old_name".to_string(),
                    placeholder: "{{old_name}}".to_string(),
                    description: "Old function name".to_string(),
                },
                PatternParameter {
                    name: "new_name".to_string(),
                    placeholder: "{{new_name}}".to_string(),
                    description: "New function name".to_string(),
                },
            ],
            scope: PatternScope::Global,
        };

        let mut params = HashMap::new();
        params.insert("old_name".to_string(), "foo".to_string());
        params.insert("new_name".to_string(), "bar".to_string());

        let app = PatternApplication {
            pattern,
            parameters: params,
        };

        let result = app.apply("fn foo() {} -> fn bar() {}")?;
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));

        Ok(())
    }
}
