//! Unit tests for Documentation Generator
//!
//! These tests verify specific examples and edge cases

use ricecoder_github::{
    ApiDocumentation, ApiParameter, DocumentationCoverage, DocumentationGenerator,
    DocumentationOperations, DocumentationTemplate, MaintenanceStatus, MaintenanceTask,
    ReadmeConfig,
};
use std::collections::HashMap;

#[test]
fn test_readme_generation_with_all_sections() {
    let config = ReadmeConfig {
        project_name: "TestProject".to_string(),
        description: "A test project".to_string(),
        include_toc: true,
        include_installation: true,
        include_usage: true,
        include_api: true,
        include_contributing: true,
        include_license: true,
    };

    let generator = DocumentationGenerator::new(config);
    let readme = generator
        .generate_readme()
        .expect("Failed to generate README");

    assert!(readme.contains("# TestProject"));
    assert!(readme.contains("A test project"));
    assert!(readme.contains("## Table of Contents"));
    assert!(readme.contains("## Installation"));
    assert!(readme.contains("## Usage"));
    assert!(readme.contains("## Contributing"));
    assert!(readme.contains("## License"));
}

#[test]
fn test_readme_generation_with_minimal_sections() {
    let config = ReadmeConfig {
        project_name: "MinimalProject".to_string(),
        description: String::new(),
        include_toc: false,
        include_installation: false,
        include_usage: false,
        include_api: false,
        include_contributing: false,
        include_license: false,
    };

    let generator = DocumentationGenerator::new(config);
    let readme = generator
        .generate_readme()
        .expect("Failed to generate README");

    assert!(readme.contains("# MinimalProject"));
    assert!(!readme.contains("## Table of Contents"));
    assert!(!readme.contains("## Installation"));
}

#[test]
fn test_api_documentation_extraction_from_rust_code() {
    let code = r#"
/// Adds two numbers together
/// 
/// # Arguments
/// * `a` - First number
/// * `b` - Second number
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiplies two numbers
pub fn multiply(x: i32, y: i32) -> i32 {
    x * y
}
"#;

    let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
    let api_docs = generator
        .extract_api_documentation(code)
        .expect("Failed to extract API docs");

    assert!(!api_docs.is_empty());
    assert!(api_docs.iter().any(|doc| doc.name.contains("add")));
    assert!(api_docs.iter().any(|doc| doc.name.contains("multiply")));
}

#[test]
fn test_api_documentation_extraction_empty_code() {
    let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
    let api_docs = generator
        .extract_api_documentation("")
        .expect("Failed to extract API docs");

    assert!(api_docs.is_empty());
}

#[test]
fn test_api_documentation_extraction_no_docs() {
    let code = r#"
pub fn undocumented() {}
pub fn also_undocumented() {}
"#;

    let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
    let api_docs = generator
        .extract_api_documentation(code)
        .expect("Failed to extract API docs");

    // Functions without doc comments should not be extracted
    // (extraction only looks for documented functions)
    assert!(api_docs.is_empty());
}

#[test]
fn test_documentation_synchronization_with_changes() {
    let old_code = r#"
pub fn old_function() {}
"#;

    let new_code = r#"
pub fn old_function() {}
pub fn new_function() {}
"#;

    let generator = DocumentationGenerator::new(ReadmeConfig::default());
    let sync_result = generator
        .synchronize_documentation(old_code, new_code)
        .expect("Failed to synchronize");

    assert!(sync_result.success);
}

#[test]
fn test_documentation_synchronization_no_changes() {
    let code = r#"
pub fn function() {}
"#;

    let generator = DocumentationGenerator::new(ReadmeConfig::default());
    let sync_result = generator
        .synchronize_documentation(code, code)
        .expect("Failed to synchronize");

    assert!(sync_result.success);
    assert!(sync_result.files_updated.is_empty());
}

#[test]
fn test_documentation_coverage_calculation() {
    let code = r#"
/// Documented function
pub fn documented() {}

pub fn undocumented() {}
"#;

    let generator = DocumentationGenerator::new(ReadmeConfig::default());
    let coverage = generator
        .calculate_coverage(code)
        .expect("Failed to calculate coverage");

    assert!(coverage.total_items > 0);
    assert!(coverage.coverage_percentage >= 0.0 && coverage.coverage_percentage <= 100.0);
}

#[test]
fn test_documentation_coverage_100_percent() {
    let coverage = DocumentationCoverage::new(10, 10);
    assert!((coverage.coverage_percentage - 100.0).abs() < 0.1);
}

#[test]
fn test_documentation_coverage_0_percent() {
    let coverage = DocumentationCoverage::new(10, 0);
    assert!(coverage.coverage_percentage < 0.1);
}

#[test]
fn test_documentation_coverage_50_percent() {
    let coverage = DocumentationCoverage::new(10, 5);
    assert!((coverage.coverage_percentage - 50.0).abs() < 0.1);
}

#[test]
fn test_api_documentation_builder() {
    let api_doc = ApiDocumentation::new("test_func", "pub fn test_func() {}")
        .with_documentation("This is a test function")
        .with_parameter(ApiParameter::new("x", "i32", "First parameter"))
        .with_return_type("i32")
        .with_example("test_func()");

    assert_eq!(api_doc.name, "test_func");
    assert_eq!(api_doc.documentation, "This is a test function");
    assert_eq!(api_doc.parameters.len(), 1);
    assert_eq!(api_doc.return_type, "i32");
    assert_eq!(api_doc.examples.len(), 1);
}

#[test]
fn test_api_parameter_creation() {
    let param = ApiParameter::new("count", "usize", "Number of items");

    assert_eq!(param.name, "count");
    assert_eq!(param.param_type, "usize");
    assert_eq!(param.description, "Number of items");
}

#[test]
fn test_documentation_coverage_with_gaps() {
    let mut coverage = DocumentationCoverage::new(5, 2);
    coverage = coverage.with_gap("Function 'foo' is not documented");
    coverage = coverage.with_gap("Function 'bar' is not documented");

    assert_eq!(coverage.gaps.len(), 2);
    assert!(coverage.gaps.iter().any(|g| g.contains("foo")));
    assert!(coverage.gaps.iter().any(|g| g.contains("bar")));
}

#[test]
fn test_documentation_operations_commit() {
    use ricecoder_github::DocumentationCommit;

    let mut ops = DocumentationOperations::new();
    let commit = DocumentationCommit::new("Update documentation")
        .with_file("README.md")
        .with_file("API.md");

    let result = ops.commit_documentation(commit).expect("Failed to commit");

    assert!(result.success);
    assert!(result.commit_hash.is_some());
    assert_eq!(result.files_published.len(), 2);
}

#[test]
fn test_documentation_operations_template_rendering() {
    let mut ops = DocumentationOperations::new();
    let template = DocumentationTemplate::new("readme", "# {{project}}\n\nVersion: {{version}}");

    ops.add_template(template).expect("Failed to add template");

    let mut values = HashMap::new();
    values.insert("project".to_string(), "MyProject".to_string());
    values.insert("version".to_string(), "1.0.0".to_string());

    let rendered = ops
        .render_template("readme", &values)
        .expect("Failed to render");

    assert!(rendered.contains("# MyProject"));
    assert!(rendered.contains("Version: 1.0.0"));
}

#[test]
fn test_documentation_operations_maintenance_task() {
    let mut ops = DocumentationOperations::new();
    let task = MaintenanceTask::new("Update API docs", "Update API documentation")
        .with_file("API.md")
        .with_status(MaintenanceStatus::InProgress)
        .with_progress(50);

    ops.track_coverage(task).expect("Failed to track");

    let retrieved = ops
        .get_maintenance_task("Update API docs")
        .expect("Task not found");

    assert_eq!(retrieved.status, MaintenanceStatus::InProgress);
    assert_eq!(retrieved.progress, 50);
}

#[test]
fn test_documentation_operations_update_task_status() {
    let mut ops = DocumentationOperations::new();
    let task = MaintenanceTask::new("Task 1", "Description");

    ops.track_coverage(task).expect("Failed to track");
    ops.update_task_status("Task 1", MaintenanceStatus::Completed)
        .expect("Failed to update");

    let updated = ops.get_maintenance_task("Task 1").expect("Task not found");

    assert_eq!(updated.status, MaintenanceStatus::Completed);
}

#[test]
fn test_documentation_operations_commit_history() {
    use ricecoder_github::DocumentationCommit;

    let mut ops = DocumentationOperations::new();

    let commit1 = DocumentationCommit::new("First commit").with_file("README.md");
    let commit2 = DocumentationCommit::new("Second commit").with_file("API.md");

    ops.commit_documentation(commit1).expect("Failed to commit");
    ops.commit_documentation(commit2).expect("Failed to commit");

    let history = ops.get_commit_history();
    assert_eq!(history.len(), 2);

    let latest = ops.get_latest_commit().expect("No commits");
    assert_eq!(latest.message, "Second commit");
}

#[test]
fn test_documentation_generator_add_section() {
    use ricecoder_github::DocumentationSection;

    let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
    let section = DocumentationSection::new("Getting Started", "Follow these steps...", 1);

    generator.add_section("getting_started", section);

    let sections = generator.get_sorted_sections();
    assert!(!sections.is_empty());
}

#[test]
fn test_documentation_generator_add_api_documentation() {
    let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
    let api_doc = ApiDocumentation::new("test_func", "pub fn test_func() {}");

    generator.add_api_documentation("test", api_doc);

    assert!(generator.api_docs.contains_key("test"));
}

#[test]
fn test_sync_result_builder() {
    use ricecoder_github::SyncResult;

    let result = SyncResult::success()
        .with_updated_file("file1.md")
        .with_added_file("file2.md")
        .with_deleted_file("file3.md");

    assert!(result.success);
    assert_eq!(result.files_updated.len(), 1);
    assert_eq!(result.files_added.len(), 1);
    assert_eq!(result.files_deleted.len(), 1);
}

#[test]
fn test_sync_result_failure() {
    use ricecoder_github::SyncResult;

    let result = SyncResult::failure("Something went wrong");

    assert!(!result.success);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Something went wrong");
}

#[test]
fn test_publishing_result_builder() {
    use ricecoder_github::PublishingResult;

    let result = PublishingResult::success("abc123")
        .with_file("README.md")
        .with_file("API.md");

    assert!(result.success);
    assert_eq!(result.commit_hash, Some("abc123".to_string()));
    assert_eq!(result.files_published.len(), 2);
}

#[test]
fn test_publishing_result_failure() {
    use ricecoder_github::PublishingResult;

    let result = PublishingResult::failure("Commit failed");

    assert!(!result.success);
    assert!(result.error.is_some());
}

#[test]
fn test_readme_config_default() {
    let config = ReadmeConfig::default();

    assert_eq!(config.project_name, "Project");
    assert!(config.include_toc);
    assert!(config.include_installation);
    assert!(config.include_usage);
    assert!(config.include_api);
    assert!(config.include_contributing);
    assert!(config.include_license);
}

#[test]
fn test_documentation_coverage_zero_items() {
    let coverage = DocumentationCoverage::new(0, 0);

    // Zero items should result in 100% coverage (vacuous truth)
    assert!((coverage.coverage_percentage - 100.0).abs() < 0.1);
}

#[test]
fn test_documentation_operations_default() {
    let ops = DocumentationOperations::default();

    assert!(ops.templates.is_empty());
    assert!(ops.maintenance_tasks.is_empty());
    assert!(ops.commit_history.is_empty());
}

#[test]
fn test_documentation_operations_get_all_maintenance_tasks() {
    let mut ops = DocumentationOperations::new();

    let task1 = MaintenanceTask::new("Task 1", "Description 1");
    let task2 = MaintenanceTask::new("Task 2", "Description 2");

    ops.track_coverage(task1).expect("Failed to track");
    ops.track_coverage(task2).expect("Failed to track");

    let tasks = ops.get_maintenance_tasks();
    assert_eq!(tasks.len(), 2);
}

#[test]
fn test_documentation_template_with_variables() {
    let template = DocumentationTemplate::new("test", "Hello {{name}}").with_variable("name");

    assert_eq!(template.variables.len(), 1);
    assert!(template.variables.contains(&"name".to_string()));
}

#[test]
fn test_maintenance_task_builder() {
    let task = MaintenanceTask::new("Task", "Description")
        .with_file("file1.md")
        .with_file("file2.md")
        .with_status(MaintenanceStatus::InProgress)
        .with_progress(75);

    assert_eq!(task.files.len(), 2);
    assert_eq!(task.status, MaintenanceStatus::InProgress);
    assert_eq!(task.progress, 75);
}

#[test]
fn test_maintenance_task_progress_capped_at_100() {
    let task = MaintenanceTask::new("Task", "Description").with_progress(150);

    assert_eq!(task.progress, 100);
}
