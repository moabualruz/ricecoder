/// Code completion support for LSP
///
/// This module provides LSP handlers for code completion requests and item resolution.
use crate::types::{LspError, LspResult, Position};
use ricecoder_completion::{
    CompletionEngine, CompletionItem, CompletionItemKind, Position as CompletionPosition,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

/// Completion handler for LSP
pub struct CompletionHandler {
    engine: Arc<dyn CompletionEngine>,
}

impl CompletionHandler {
    /// Create a new completion handler
    pub fn new(engine: Arc<dyn CompletionEngine>) -> Self {
        Self { engine }
    }

    /// Handle textDocument/completion request
    pub async fn handle_completion(
        &self,
        code: &str,
        position: Position,
        language: &str,
    ) -> LspResult<Vec<Value>> {
        debug!(
            "Handling completion request at line={}, character={}",
            position.line, position.character
        );

        // Convert LSP position to completion position
        let completion_position = CompletionPosition::new(position.line, position.character);

        // Generate completions
        let completions = self
            .engine
            .generate_completions(code, completion_position, language)
            .await
            .map_err(|e| LspError::InternalError(format!("Completion generation failed: {}", e)))?;

        debug!("Generated {} completions", completions.len());

        // Convert to LSP format
        let items: Vec<Value> = completions
            .iter()
            .enumerate()
            .map(|(index, item)| self.completion_item_to_json(item, index as u32))
            .collect();

        Ok(items)
    }

    /// Handle completionItem/resolve request
    pub async fn handle_completion_resolve(&self, item: &Value) -> LspResult<Value> {
        debug!("Handling completion item resolve");

        // Validate that the item has a label
        let _label = item
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing label in completion item".to_string()))?;

        // Resolve additional details (documentation, additional edits, etc.)
        let mut resolved = item.clone();

        // Add resolved flag to indicate this item has been resolved
        resolved["resolved"] = json!(true);

        Ok(resolved)
    }

    /// Apply a completion item to code at the cursor position
    pub fn apply_completion(
        &self,
        code: &str,
        position: Position,
        insert_text: &str,
    ) -> LspResult<String> {
        debug!(
            "Applying completion at line={}, character={}",
            position.line, position.character
        );

        // Split code into lines
        let lines: Vec<&str> = code.lines().collect();

        // Validate position
        if position.line as usize >= lines.len() {
            return Err(LspError::InvalidParams(format!(
                "Line {} is out of bounds",
                position.line
            )));
        }

        let line = lines[position.line as usize];
        if position.character as usize > line.len() {
            return Err(LspError::InvalidParams(format!(
                "Character {} is out of bounds on line {}",
                position.character, position.line
            )));
        }

        // Build the new code
        let mut result = String::new();

        // Add lines before the insertion point
        for (i, l) in lines.iter().enumerate() {
            if i < position.line as usize {
                result.push_str(l);
                result.push('\n');
            }
        }

        // Add the modified line
        let line = lines[position.line as usize];
        let before = &line[..position.character as usize];
        let after = &line[position.character as usize..];

        result.push_str(before);
        result.push_str(insert_text);
        result.push_str(after);

        // Add lines after the insertion point
        if (position.line as usize + 1) < lines.len() {
            result.push('\n');
            for l in lines.iter().skip(position.line as usize + 1) {
                result.push_str(l);
                result.push('\n');
            }
            // Remove trailing newline if original didn't have it
            if !code.ends_with('\n') {
                result.pop();
            }
        }

        Ok(result)
    }

    /// Validate that code is still valid after completion insertion
    pub fn validate_code_validity(&self, code: &str) -> LspResult<bool> {
        debug!("Validating code validity");

        // Basic validation: check for balanced brackets
        let mut bracket_count = 0;
        let mut paren_count = 0;
        let mut brace_count = 0;

        for ch in code.chars() {
            match ch {
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }

            // Check for negative counts (unbalanced)
            if bracket_count < 0 || paren_count < 0 || brace_count < 0 {
                return Ok(false);
            }
        }

        // Check final counts
        Ok(bracket_count == 0 && paren_count == 0 && brace_count == 0)
    }

    /// Convert a completion item to LSP JSON format
    fn completion_item_to_json(&self, item: &CompletionItem, index: u32) -> Value {
        let kind = self.completion_kind_to_lsp(item.kind);

        json!({
            "label": item.label,
            "kind": kind,
            "detail": item.detail,
            "documentation": item.documentation,
            "sortText": item.sort_text.as_ref().unwrap_or(&format!("{:04}", index)),
            "filterText": item.filter_text.as_ref().unwrap_or(&item.label),
            "insertText": item.insert_text,
            "score": item.score,
        })
    }

    /// Convert completion item kind to LSP kind number
    fn completion_kind_to_lsp(&self, kind: CompletionItemKind) -> u32 {
        match kind {
            CompletionItemKind::Text => 1,
            CompletionItemKind::Method => 2,
            CompletionItemKind::Function => 3,
            CompletionItemKind::Constructor => 4,
            CompletionItemKind::Field => 5,
            CompletionItemKind::Variable => 6,
            CompletionItemKind::Class => 7,
            CompletionItemKind::Interface => 8,
            CompletionItemKind::Module => 9,
            CompletionItemKind::Property => 10,
            CompletionItemKind::Unit => 11,
            CompletionItemKind::Value => 12,
            CompletionItemKind::Enum => 13,
            CompletionItemKind::Keyword => 14,
            CompletionItemKind::Snippet => 15,
            CompletionItemKind::Color => 16,
            CompletionItemKind::File => 17,
            CompletionItemKind::Reference => 18,
            CompletionItemKind::Folder => 19,
            CompletionItemKind::EnumMember => 20,
            CompletionItemKind::Constant => 21,
            CompletionItemKind::Struct => 22,
            CompletionItemKind::EventListener => 23,
            CompletionItemKind::Operator => 24,
            CompletionItemKind::TypeParameter => 25,
            CompletionItemKind::Trait => 26,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_kind_to_lsp() {
        let handler = CompletionHandler::new(Arc::new(MockCompletionEngine));
        assert_eq!(handler.completion_kind_to_lsp(CompletionItemKind::Function), 3);
        assert_eq!(handler.completion_kind_to_lsp(CompletionItemKind::Variable), 6);
        assert_eq!(handler.completion_kind_to_lsp(CompletionItemKind::Keyword), 14);
    }

    #[test]
    fn test_apply_completion_simple() {
        let handler = CompletionHandler::new(Arc::new(MockCompletionEngine));
        let code = "fn main() {\n    let x = ";
        let position = Position::new(1, 12);
        let result = handler.apply_completion(code, position, "42");
        assert!(result.is_ok());
        let new_code = result.unwrap();
        assert!(new_code.contains("let x = 42"));
    }

    #[test]
    fn test_apply_completion_invalid_position() {
        let handler = CompletionHandler::new(Arc::new(MockCompletionEngine));
        let code = "fn main() {}";
        let position = Position::new(10, 0); // Line out of bounds
        let result = handler.apply_completion(code, position, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_code_validity_balanced() {
        let handler = CompletionHandler::new(Arc::new(MockCompletionEngine));
        let code = "fn main() { let x = [1, 2, 3]; }";
        let result = handler.validate_code_validity(code);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_validate_code_validity_unbalanced() {
        let handler = CompletionHandler::new(Arc::new(MockCompletionEngine));
        let code = "fn main() { let x = [1, 2, 3; }";
        let result = handler.validate_code_validity(code);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // Mock completion engine for testing
    struct MockCompletionEngine;

    #[async_trait::async_trait]
    impl CompletionEngine for MockCompletionEngine {
        async fn generate_completions(
            &self,
            _code: &str,
            _position: CompletionPosition,
            _language: &str,
        ) -> ricecoder_completion::CompletionResult<Vec<CompletionItem>> {
            Ok(vec![])
        }

        async fn resolve_completion(
            &self,
            item: &CompletionItem,
        ) -> ricecoder_completion::CompletionResult<CompletionItem> {
            Ok(item.clone())
        }
    }
}
