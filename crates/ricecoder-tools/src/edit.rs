//! File editing tool with multiple replacement strategies
//!
//! This module provides intelligent file editing capabilities with 9 different
//! strategies for applying changes to files, from simple string replacement
//! to advanced AST-based editing.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ToolError;

/// Input for file edit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditInput {
    /// Path to the file to edit
    pub file_path: String,
    /// Old content to replace
    pub old_string: String,
    /// New content to replace with
    pub new_string: String,
    /// Optional start line for context (1-indexed)
    pub start_line: Option<usize>,
    /// Optional end line for context (1-indexed)
    pub end_line: Option<usize>,
    /// Replace all occurrences (default: false)
    #[serde(default)]
    pub replace_all: bool,
}

/// Output from file edit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditOutput {
    /// Whether the edit was successfully applied
    pub success: bool,
    /// Strategy that succeeded (if any)
    pub strategy_used: Option<String>,
    /// All strategies that were attempted
    pub strategies_attempted: Vec<String>,
    /// Generated diff (unified format)
    pub diff: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Closest match information for debugging
    pub closest_match: Option<ClosestMatchInfo>,
}

/// Input for batch file edit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFileEditInput {
    /// List of file edits to apply
    pub edits: Vec<FileEditInput>,
    /// Whether to continue on individual failures
    pub continue_on_error: bool,
    /// Whether to create backups before editing
    pub create_backups: bool,
}

/// Output from batch file edit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFileEditOutput {
    /// Overall success status
    pub success: bool,
    /// Results for each edit operation
    pub results: Vec<BatchEditResult>,
    /// Number of successful edits
    pub successful_edits: usize,
    /// Number of failed edits
    pub failed_edits: usize,
    /// Backup files created (if any)
    pub backups_created: Vec<String>,
    /// OpenCode-compatible title (relative path from project root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Result of a single edit in a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEditResult {
    /// The edit input that was attempted
    pub input: FileEditInput,
    /// The result of the edit
    pub result: FileEditOutput,
    /// Backup file path (if created)
    pub backup_path: Option<String>,
}

/// Information about the closest match found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosestMatchInfo {
    /// Strategy that found the closest match
    pub strategy: String,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f64,
    /// Line number where match was found
    pub line_number: usize,
    /// Matched text snippet
    pub matched_text: String,
}

/// File edit tool with multiple strategies
pub struct FileEditTool;

impl FileEditTool {
    /// Apply batch edits with rollback capability
    pub fn batch_edit_files(input: &BatchFileEditInput) -> Result<BatchFileEditOutput, ToolError> {
        let mut results = Vec::new();
        let mut successful_edits = 0;
        let mut failed_edits = 0;
        let mut backups_created = Vec::new();

        // Create backups if requested
        let backups = if input.create_backups {
            Self::create_backups(&input.edits)?
        } else {
            Vec::new()
        };

        for edit_input in &input.edits {
            // Create backup for this specific file if not already done
            let backup_path = if input.create_backups {
                backups
                    .iter()
                    .find(|(path, _)| path == &edit_input.file_path)
                    .map(|(_, backup)| backup.clone())
            } else {
                None
            };

            match Self::edit_file(edit_input) {
                Ok(edit_result) => {
                    if edit_result.success {
                        successful_edits += 1;
                    } else {
                        failed_edits += 1;
                        if !input.continue_on_error {
                            // Rollback all successful edits
                            Self::rollback_batch(&results)?;
                            
                            // Generate title for failed batch
                            let title = if !input.edits.is_empty() {
                                Self::compute_relative_path(&input.edits[0].file_path)
                            } else {
                                None
                            };
                            
                            return Ok(BatchFileEditOutput {
                                success: false,
                                results,
                                successful_edits,
                                failed_edits,
                                backups_created: backups.into_iter().map(|(_, b)| b).collect(),
                                title,
                            });
                        }
                    }

                    results.push(BatchEditResult {
                        input: edit_input.clone(),
                        result: edit_result,
                        backup_path: backup_path.clone(),
                    });
                }
                Err(e) => {
                    failed_edits += 1;
                    if !input.continue_on_error {
                        // Rollback all successful edits
                        Self::rollback_batch(&results)?;
                        return Err(e);
                    }

                    // Add failed result
                    results.push(BatchEditResult {
                        input: edit_input.clone(),
                        result: FileEditOutput {
                            success: false,
                            strategy_used: None,
                            strategies_attempted: Vec::new(),
                            diff: None,
                            error: Some(e.to_string()),
                            closest_match: None,
                        },
                        backup_path,
                    });
                }
            }
        }

        // Collect backup paths
        for (original_path, backup_path) in backups {
            backups_created.push(backup_path);
        }

        // Generate OpenCode-compatible title (relative path from project root)
        let title = if !input.edits.is_empty() {
            Self::compute_relative_path(&input.edits[0].file_path)
        } else {
            None
        };

        Ok(BatchFileEditOutput {
            success: failed_edits == 0,
            results,
            successful_edits,
            failed_edits,
            backups_created,
            title,
        })
    }

    /// Create backups for all files before editing
    fn create_backups(edits: &[FileEditInput]) -> Result<Vec<(String, String)>, ToolError> {
        let mut backups = Vec::new();
        let mut unique_files = std::collections::HashSet::new();

        // Collect unique file paths
        for edit in edits {
            unique_files.insert(&edit.file_path);
        }

        for file_path in unique_files {
            let path = std::path::Path::new(file_path);
            if path.exists() {
                let backup_path = format!("{}.backup", file_path);
                std::fs::copy(file_path, &backup_path).map_err(|e| {
                    ToolError::new("BACKUP_ERROR", format!("Failed to create backup: {}", e))
                })?;
                backups.push((file_path.clone(), backup_path));
            }
        }

        Ok(backups)
    }

    /// Rollback a batch of edits using backups
    fn rollback_batch(results: &[BatchEditResult]) -> Result<(), ToolError> {
        for result in results {
            if result.result.success {
                if let Some(backup_path) = &result.backup_path {
                    std::fs::copy(backup_path, &result.input.file_path).map_err(|e| {
                        ToolError::new("ROLLBACK_ERROR", format!("Failed to rollback: {}", e))
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Compute relative path from project root (OpenCode compatibility)
    fn compute_relative_path(file_path: &str) -> Option<String> {
        let path = Path::new(file_path);
        
        // Try to find project root by walking up the directory tree
        let mut current = path.parent()?;
        while let Some(parent) = current.parent() {
            // Check for common project root markers
            if parent.join(".git").exists()
                || parent.join("Cargo.toml").exists()
                || parent.join("package.json").exists()
                || parent.join("pyproject.toml").exists()
            {
                // Found project root, compute relative path
                return path.strip_prefix(parent).ok()?.to_str().map(|s| s.to_string());
            }
            current = parent;
        }
        
        // Fallback: use filename if no project root found
        path.file_name()?.to_str().map(|s| s.to_string())
    }

    /// Apply edit using the best available strategy
    pub fn edit_file(input: &FileEditInput) -> Result<FileEditOutput, ToolError> {
        // GAP-3: Enforce old_string != new_string
        if input.old_string == input.new_string {
            return Err(ToolError::new(
                "IDENTICAL_STRINGS",
                "oldString and newString must be different".to_string(),
            ));
        }

        let file_path = Path::new(&input.file_path);
        
        // GAP-10: File type/existence checks with specific error messages
        if !file_path.exists() {
            return Err(ToolError::new(
                "FILE_NOT_FOUND",
                format!("File {} not found", input.file_path),
            ));
        }
        
        if file_path.is_dir() {
            return Err(ToolError::new(
                "PATH_IS_DIRECTORY",
                format!("Path is a directory, not a file: {}", input.file_path),
            ));
        }

        // GAP-4: Handle empty oldString = overwrite file
        if input.old_string.is_empty() {
            std::fs::write(file_path, &input.new_string).map_err(|e| {
                ToolError::new("FILE_WRITE_ERROR", format!("Failed to write file: {}", e))
            })?;
            
            let diff = generate_diff("", &input.new_string);
            return Ok(FileEditOutput {
                success: true,
                strategy_used: Some("FullOverwrite".to_string()),
                strategies_attempted: vec!["FullOverwrite".to_string()],
                diff: Some(diff),
                error: None,
                closest_match: None,
            });
        }

        let content = std::fs::read_to_string(file_path).map_err(|e| {
            ToolError::new("FILE_READ_ERROR", format!("Failed to read file: {}", e))
        })?;

        let mut strategies_attempted = Vec::new();
        let mut closest_match: Option<ClosestMatchInfo> = None;

        // GAP-5: Port OpenCode replacement engine - 9 strategies in OpenCode order
        let strategies: Vec<Box<dyn EditStrategy>> = vec![
            Box::new(SimpleStrategy),
            Box::new(LineTrimmedStrategy),
            Box::new(BlockAnchorStrategy),
            Box::new(WhitespaceNormalizedStrategy),
            Box::new(IndentNormalizedStrategy),
            Box::new(EscapeNormalizedStrategy),
            Box::new(TrimmedBoundaryStrategy),
            Box::new(ContextAwareStrategy),
            Box::new(MultiOccurrenceStrategy { replace_all: input.replace_all }),
        ];

        for strategy in strategies {
            let strategy_name = strategy.name();
            strategies_attempted.push(strategy_name.clone());

            match strategy.apply(&content, input) {
                Ok(new_content) => {
                    // GAP-1: CRITICAL FIX - Actually write the file instead of calling stub
                    std::fs::write(file_path, &new_content).map_err(|e| {
                        ToolError::new("FILE_WRITE_ERROR", format!("Failed to write file: {}", e))
                    })?;

                    let diff = generate_diff(&content, &new_content);

                    return Ok(FileEditOutput {
                        success: true,
                        strategy_used: Some(strategy_name),
                        strategies_attempted,
                        diff: Some(diff),
                        error: None,
                        closest_match,
                    });
                }
                Err(EditError::NoMatch) => {
                    // Continue to next strategy
                    continue;
                }
                Err(EditError::ClosestMatch(info)) => {
                    // Update closest match if better
                    if let Some(ref current) = closest_match {
                        if info.similarity > current.similarity {
                            closest_match = Some(info);
                        }
                    } else {
                        closest_match = Some(info);
                    }
                    continue;
                }
                Err(EditError::MultipleMatches) => {
                    // GAP-2: Handle multiple matches error with OpenCode message
                    return Ok(FileEditOutput {
                        success: false,
                        strategy_used: None,
                        strategies_attempted,
                        diff: None,
                        error: Some("Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.".to_string()),
                        closest_match,
                    });
                }
            }
        }

        // All strategies failed - use OpenCode error message
        Ok(FileEditOutput {
            success: false,
            strategy_used: None,
            strategies_attempted,
            diff: None,
            error: Some("oldString not found in content".to_string()),
            closest_match,
        })
    }
}

/// Trait for edit strategies
trait EditStrategy {
    fn name(&self) -> String;
    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError>;
}

/// GAP-5: EscapeNormalizedStrategy (from OpenCode)
struct EscapeNormalizedStrategy;

impl EditStrategy for EscapeNormalizedStrategy {
    fn name(&self) -> String {
        "EscapeNormalized".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let unescape = |s: &str| -> String {
            s.replace("\\n", "\n")
             .replace("\\t", "\t")
             .replace("\\r", "\r")
             .replace("\\'", "'")
             .replace("\\\"", "\"")
             .replace("\\`", "`")
             .replace("\\\\", "\\")
             .replace("\\$", "$")
        };

        let unescaped_old = unescape(&input.old_string);
        
        if content.contains(&unescaped_old) {
            let new_content = content.replace(&unescaped_old, &input.new_string);
            Ok(new_content)
        } else {
            Err(EditError::NoMatch)
        }
    }
}

/// GAP-5: TrimmedBoundaryStrategy (from OpenCode)
struct TrimmedBoundaryStrategy;

impl EditStrategy for TrimmedBoundaryStrategy {
    fn name(&self) -> String {
        "TrimmedBoundary".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let trimmed_old = input.old_string.trim();
        
        if trimmed_old == input.old_string {
            // Already trimmed, no point trying
            return Err(EditError::NoMatch);
        }

        if content.contains(trimmed_old) {
            let new_content = content.replace(trimmed_old, &input.new_string);
            Ok(new_content)
        } else {
            Err(EditError::NoMatch)
        }
    }
}

/// GAP-5: ContextAwareStrategy (from OpenCode)
struct ContextAwareStrategy;

impl EditStrategy for ContextAwareStrategy {
    fn name(&self) -> String {
        "ContextAware".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let old_lines: Vec<&str> = input.old_string.lines().collect();
        if old_lines.len() < 3 {
            return Err(EditError::NoMatch);
        }

        let first_line = old_lines[0].trim();
        let last_line = old_lines[old_lines.len() - 1].trim();
        let content_lines: Vec<&str> = content.lines().collect();

        for i in 0..content_lines.len() {
            if content_lines[i].trim() == first_line {
                for j in (i + 2)..content_lines.len() {
                    if content_lines[j].trim() == last_line {
                        let block_lines = &content_lines[i..=j];
                        
                        if block_lines.len() == old_lines.len() {
                            let mut matching_lines = 0;
                            let mut total_non_empty = 0;

                            for k in 1..(block_lines.len() - 1) {
                                let block_line = block_lines[k].trim();
                                let old_line = old_lines[k].trim();

                                if !block_line.is_empty() || !old_line.is_empty() {
                                    total_non_empty += 1;
                                    if block_line == old_line {
                                        matching_lines += 1;
                                    }
                                }
                            }

                            if total_non_empty == 0 || (matching_lines as f64 / total_non_empty as f64) >= 0.5 {
                                let mut new_lines = content_lines.clone();
                                let new_block_lines: Vec<&str> = input.new_string.lines().collect();
                                
                                for k in 0..=j-i {
                                    if k < new_block_lines.len() {
                                        new_lines[i + k] = new_block_lines[k];
                                    }
                                }
                                
                                let new_content = new_lines.join("\n");
                                return Ok(new_content);
                            }
                        }
                        break;
                    }
                }
            }
        }

        Err(EditError::NoMatch)
    }
}

/// GAP-5: MultiOccurrenceStrategy (from OpenCode) with replaceAll support
struct MultiOccurrenceStrategy {
    replace_all: bool,
}

impl EditStrategy for MultiOccurrenceStrategy {
    fn name(&self) -> String {
        "MultiOccurrence".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let match_count = content.matches(&input.old_string).count();

        if match_count == 0 {
            return Err(EditError::NoMatch);
        }

        if match_count == 1 || self.replace_all {
            // Single match or replaceAll=true: do the replacement
            let new_content = content.replace(&input.old_string, &input.new_string);
            Ok(new_content)
        } else {
            // Multiple matches without replaceAll=true: error
            Err(EditError::MultipleMatches)
        }
    }
}

/// Errors that can occur during editing
#[derive(Debug, Clone)]
enum EditError {
    NoMatch,
    ClosestMatch(ClosestMatchInfo),
    MultipleMatches,
}

/// Simple exact string match and replace
struct SimpleStrategy;

impl EditStrategy for SimpleStrategy {
    fn name(&self) -> String {
        "Simple".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        if content.contains(&input.old_string) {
            let new_content = content.replace(&input.old_string, &input.new_string);
            Ok(new_content)
        } else {
            Err(EditError::NoMatch)
        }
    }
}

/// Match with leading/trailing whitespace trimmed
struct LineTrimmedStrategy;

impl EditStrategy for LineTrimmedStrategy {
    fn name(&self) -> String {
        "Line-trimmed".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let old_trimmed = input.old_string.trim();
        let new_trimmed = input.new_string.trim();

        if content.contains(old_trimmed) {
            let new_content = content.replace(old_trimmed, new_trimmed);
            Ok(new_content)
        } else {
            Err(EditError::NoMatch)
        }
    }
}

/// Match using first and last lines as anchors
struct BlockAnchorStrategy;

impl EditStrategy for BlockAnchorStrategy {
    fn name(&self) -> String {
        "Block-anchor".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let old_lines: Vec<&str> = input.old_string.lines().collect();
        if old_lines.len() < 2 {
            return Err(EditError::NoMatch);
        }

        let first_line = old_lines[0];
        let last_line = old_lines[old_lines.len() - 1];

        let content_lines: Vec<&str> = content.lines().collect();

        for (i, &line) in content_lines.iter().enumerate() {
            if line.contains(first_line) {
                // Look for the last line
                for j in i..content_lines.len() {
                    if content_lines[j].contains(last_line) {
                        // Found the block - replace it
                        let start_idx = content_lines[..i].iter().map(|l| l.len() + 1).sum();
                        let end_idx = content_lines[..=j].iter().map(|l| l.len() + 1).sum();

                        let before = &content[..start_idx];
                        let after = &content[end_idx..];
                        let new_content = format!("{}{}{}", before, input.new_string, after);

                        return Ok(new_content);
                    }
                }
            }
        }

        Err(EditError::NoMatch)
    }
}

/// Match with normalized indentation
struct IndentNormalizedStrategy;

impl EditStrategy for IndentNormalizedStrategy {
    fn name(&self) -> String {
        "Indent-normalized".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let old_normalized = normalize_indentation(&input.old_string);
        let new_normalized = normalize_indentation(&input.new_string);

        if content.contains(&old_normalized) {
            let new_content = content.replace(&old_normalized, &new_normalized);
            Ok(new_content)
        } else {
            Err(EditError::NoMatch)
        }
    }
}

/// Match with all whitespace normalized
struct WhitespaceNormalizedStrategy;

impl EditStrategy for WhitespaceNormalizedStrategy {
    fn name(&self) -> String {
        "Whitespace-normalized".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let old_normalized = normalize_whitespace(&input.old_string);
        let new_normalized = normalize_whitespace(&input.new_string);

        if content.contains(&old_normalized) {
            let new_content = content.replace(&old_normalized, &new_normalized);
            Ok(new_content)
        } else {
            Err(EditError::NoMatch)
        }
    }
}

/// Fuzzy match using Levenshtein distance
struct LevenshteinStrategy;

impl EditStrategy for LevenshteinStrategy {
    fn name(&self) -> String {
        "Levenshtein".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        const THRESHOLD: f64 = 0.9; // 90% similarity

        let mut best_match: Option<(usize, f64, String)> = None;

        // Find the best fuzzy match
        for (line_num, line) in content.lines().enumerate() {
            let similarity = fuzzy_similarity(&input.old_string, line);
            if similarity >= THRESHOLD {
                if let Some((_, best_sim, _)) = best_match {
                    if similarity > best_sim {
                        best_match = Some((line_num, similarity, line.to_string()));
                    }
                } else {
                    best_match = Some((line_num, similarity, line.to_string()));
                }
            }
        }

        if let Some((line_num, similarity, matched_line)) = best_match {
            // Replace the matched line
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = lines.clone();
            new_lines[line_num] = &input.new_string;

            let new_content = new_lines.join("\n");

            Ok(new_content)
        } else {
            // Return closest match info
            let mut closest: Option<(usize, f64, String)> = None;
            for (line_num, line) in content.lines().enumerate() {
                let similarity = fuzzy_similarity(&input.old_string, line);
                if let Some((_, best_sim, _)) = closest {
                    if similarity > best_sim {
                        closest = Some((line_num, similarity, line.to_string()));
                    }
                } else {
                    closest = Some((line_num, similarity, line.to_string()));
                }
            }

            if let Some((line_num, similarity, matched_line)) = closest {
                Err(EditError::ClosestMatch(ClosestMatchInfo {
                    strategy: self.name(),
                    similarity,
                    line_number: line_num + 1, // 1-indexed
                    matched_text: matched_line,
                }))
            } else {
                Err(EditError::NoMatch)
            }
        }
    }
}

/// Match and replace line by line
struct LineByLineStrategy;

impl EditStrategy for LineByLineStrategy {
    fn name(&self) -> String {
        "Line-by-line".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        let old_lines: Vec<&str> = input.old_string.lines().collect();
        let new_lines: Vec<&str> = input.new_string.lines().collect();

        if old_lines.is_empty() {
            return Err(EditError::NoMatch);
        }

        let content_lines: Vec<&str> = content.lines().collect();

        // Find the starting line
        for (i, &content_line) in content_lines.iter().enumerate() {
            if content_line.trim() == old_lines[0].trim() {
                // Check if subsequent lines match
                let mut matches = true;
                for j in 1..old_lines.len() {
                    if i + j >= content_lines.len()
                        || content_lines[i + j].trim() != old_lines[j].trim()
                    {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    // Replace the lines
                    let mut new_content_lines = content_lines.clone();
                    for j in 0..old_lines.len() {
                        if i + j < new_content_lines.len() && j < new_lines.len() {
                            new_content_lines[i + j] = new_lines[j];
                        }
                    }

                    let new_content = new_content_lines.join("\n");
                    return Ok(new_content);
                }
            }
        }

        Err(EditError::NoMatch)
    }
}

/// Match using regular expression
struct RegexStrategy;

impl EditStrategy for RegexStrategy {
    fn name(&self) -> String {
        "Regex".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        // Escape special regex characters in the old string to treat it as literal
        let escaped = regex::escape(&input.old_string);

        match regex::Regex::new(&escaped) {
            Ok(re) => {
                if re.is_match(content) {
                    let new_content = re.replace_all(content, &input.new_string).to_string();
                    Ok(new_content)
                } else {
                    Err(EditError::NoMatch)
                }
            }
            Err(_) => Err(EditError::NoMatch),
        }
    }
}

/// Match using abstract syntax tree (language-specific)
struct AstBasedStrategy;

impl EditStrategy for AstBasedStrategy {
    fn name(&self) -> String {
        "AST-based".to_string()
    }

    fn apply(&self, content: &str, input: &FileEditInput) -> Result<String, EditError> {
        // This is a placeholder for AST-based editing
        // In a real implementation, this would parse the code into an AST
        // and perform intelligent matching based on code structure

        // For now, fall back to simple strategy
        SimpleStrategy.apply(content, input)
    }
}

/// Generate a simple unified diff between two strings
fn generate_diff(old_content: &str, new_content: &str) -> String {
    // This is a simplified diff generation
    // In a real implementation, you'd use a proper diff library
    format!("--- a/file\n+++ b/file\n@@ -1,{old_lines} +1,{new_lines} @@\n-{old_content}\n+{new_content}\n",
            old_lines = old_content.lines().count(),
            new_lines = new_content.lines().count(),
            old_content = old_content,
            new_content = new_content)
}

/// Normalize indentation by removing common leading whitespace
fn normalize_indentation(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    // Find the minimum indentation
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove the common indentation
    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Normalize all whitespace to single spaces
fn normalize_whitespace(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Calculate fuzzy similarity between two strings (0.0 to 1.0)
fn fuzzy_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let max_len = a_chars.len().max(b_chars.len());
    if max_len == 0 {
        return 1.0;
    }

    // Simple character-based similarity (case-sensitive)
    // Each character in b can only be used once
    let mut used_positions = std::collections::HashSet::new();
    let mut matches = 0;

    for &ca in &a_chars {
        for (i, &cb) in b_chars.iter().enumerate() {
            if ca == cb && !used_positions.contains(&i) {
                matches += 1;
                used_positions.insert(i);
                break;
            }
        }
    }

    matches as f64 / max_len as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_strategy() {
        let content = "Hello world\nThis is a test\nGoodbye world";
        let input = FileEditInput {
            file_path: "test.txt".to_string(),
            old_string: "Hello world".to_string(),
            new_string: "Hi world".to_string(),
            start_line: None,
            end_line: None,
            replace_all: false,
        };

        let result = SimpleStrategy.apply(content, &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_line_trimmed_strategy() {
        let content = "Hello world\nThis is a test\nGoodbye world";
        let input = FileEditInput {
            file_path: "test.txt".to_string(),
            old_string: "  Hello world  ".to_string(),
            new_string: "Hi world".to_string(),
            start_line: None,
            end_line: None,
            replace_all: false,
        };

        let result = LineTrimmedStrategy.apply(content, &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fuzzy_similarity() {
        assert_eq!(fuzzy_similarity("hello", "hello"), 1.0);
        assert_eq!(fuzzy_similarity("hello", "world"), 2.0 / 5.0); // 2 matching chars (l, o) out of 5
        assert_eq!(fuzzy_similarity("", ""), 1.0);
    }

    #[test]
    fn test_normalize_indentation() {
        let input = "  line1\n    line2\n  line3";
        let expected = "line1\n  line2\nline3";
        assert_eq!(normalize_indentation(input), expected);
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "hello   world\n\ttest";
        let expected = "hello world test";
        assert_eq!(normalize_whitespace(input), expected);
    }
}
