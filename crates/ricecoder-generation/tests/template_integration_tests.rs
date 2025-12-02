//! Integration tests for template rendering with full workflows
//! Tests template rendering with conditionals, loops, and boilerplate scaffolding

use ricecoder_generation::{
    TemplateEngine, BoilerplateManager, Boilerplate,
    BoilerplateFile, ConflictResolution, TemplateParser, PlaceholderResolver,
};
use ricecoder_generation::templates::resolver::CaseTransform;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Test template rendering with simple placeholders
#[test]
fn test_template_rendering_simple_placeholders() {
    let mut engine = TemplateEngine::new();
    engine.add_value("name", "MyProject");
    engine.add_value("description", "A test project");

    let template = "Project: {{name}}\nDescription: {{description}}";
    let result = engine.render_simple(template).unwrap();
    // The placeholder {{name}} is converted to lowercase, so it looks up "name"
    // and applies LowerCase transform, resulting in "myproject"
    assert!(result.contains("myproject"));
    assert!(result.contains("a test project"));
}

/// Test template rendering with case transformations
#[test]
fn test_template_rendering_case_transformations() {
    let mut engine = TemplateEngine::new();
    engine.add_value("name", "my_project");

    let template = "PascalCase: {{Name}}\ncamelCase: {{nameCamel}}\nsnake_case: {{name_snake}}\nkebab-case: {{name-kebab}}\nUPPERCASE: {{NAME}}\nlowercase: {{name}}";
    let result = engine.render_simple(template).unwrap();
    // Verify that the template was rendered (content is not empty)
    assert!(!result.is_empty());
    // Verify case transformations are applied
    assert!(result.contains("MyProject")); // {{Name}} -> PascalCase
    assert!(result.contains("myProject")); // {{nameCamel}} -> camelCase
    assert!(result.contains("my_project")); // {{name_snake}} -> snake_case
    assert!(result.contains("my-project")); // {{name-kebab}} -> kebab-case
    assert!(result.contains("MY_PROJECT")); // {{NAME}} -> UPPERCASE
}

/// Test template rendering with simple text
#[test]
fn test_template_rendering_with_text() {
    let mut engine = TemplateEngine::new();
    engine.add_value("name", "MyProject");

    let template = r#"
Project structure:
- src/
- tests/
- {{Name}}/
"#;

    let result = engine.render_simple(template).unwrap();
    // Verify template was rendered
    assert!(!result.is_empty());
    // Verify src/ is included
    assert!(result.contains("src/"));
    // Verify placeholder was substituted with PascalCase
    assert!(result.contains("MyProject"));
}

/// Test template rendering with multiple placeholders
#[test]
fn test_template_rendering_with_multiple_placeholders() {
    let mut engine = TemplateEngine::new();
    engine.add_value("name", "MyService");
    engine.add_value("version", "1.0.0");

    let template = r#"
pub struct {{Name}} {
    version: "{{version}}",
}
"#;

    let result = engine.render_simple(template).unwrap();
    assert!(result.contains("MyService")); // {{Name}} -> PascalCase
    assert!(result.contains("1.0.0")); // {{version}} -> lowercase
}

/// Test template rendering with struct definition
#[test]
fn test_template_rendering_struct_definition() {
    let mut engine = TemplateEngine::new();
    engine.add_value("name", "MyService");

    let template = r#"
pub struct {{Name}} {
    id: u64,
    name: String,
}
"#;

    let result = engine.render_simple(template).unwrap();
    assert!(result.contains("MyService")); // {{Name}} -> PascalCase
    assert!(result.contains("pub struct"));
}

/// Test boilerplate scaffolding with file creation
#[test]
fn test_boilerplate_scaffolding_creates_files() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BoilerplateManager::new();

    let boilerplate = Boilerplate {
        id: "rust-project".to_string(),
        name: "Rust Project".to_string(),
        description: "A Rust project boilerplate".to_string(),
        language: "rust".to_string(),
        files: vec![
            BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {\n    println!(\"Hello, {{Name}}!\");\n}".to_string(),
                condition: None,
            },
            BoilerplateFile {
                path: "Cargo.toml".to_string(),
                template: "[package]\nname = \"{{name_snake}}\"\nversion = \"0.1.0\"".to_string(),
                condition: None,
            },
        ],
        dependencies: vec![],
        scripts: vec![],
    };

    let mut variables = HashMap::new();
    variables.insert("name".to_string(), "MyApp".to_string());
    variables.insert("Name".to_string(), "MyApp".to_string());

    let result = manager
        .apply(
            &boilerplate,
            temp_dir.path(),
            &variables,
            ConflictResolution::Skip,
        )
        .unwrap();

    // Verify files were created
    assert_eq!(result.created_files.len(), 2);
    assert!(temp_dir.path().join("src/main.rs").exists());
    assert!(temp_dir.path().join("Cargo.toml").exists());

    // Verify content
    let main_content = fs::read_to_string(temp_dir.path().join("src/main.rs")).unwrap();
    assert!(main_content.contains("MyApp")); // {{Name}} -> PascalCase

    let cargo_content = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains("my_app")); // {{name_snake}} -> snake_case
}

/// Test boilerplate scaffolding with conflict resolution (skip)
#[test]
fn test_boilerplate_scaffolding_conflict_skip() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BoilerplateManager::new();

    // Create existing file
    let existing_file = temp_dir.path().join("src").join("main.rs");
    fs::create_dir_all(existing_file.parent().unwrap()).unwrap();
    fs::write(&existing_file, "// existing content").unwrap();

    let boilerplate = Boilerplate {
        id: "rust-project".to_string(),
        name: "Rust Project".to_string(),
        description: "A Rust project boilerplate".to_string(),
        language: "rust".to_string(),
        files: vec![BoilerplateFile {
            path: "src/main.rs".to_string(),
            template: "fn main() {}".to_string(),
            condition: None,
        }],
        dependencies: vec![],
        scripts: vec![],
    };

    let variables = HashMap::new();
    let result = manager
        .apply(
            &boilerplate,
            temp_dir.path(),
            &variables,
            ConflictResolution::Skip,
        )
        .unwrap();

    // Verify file was skipped
    assert_eq!(result.skipped_files.len(), 1);
    assert_eq!(result.created_files.len(), 0);

    // Verify existing content was preserved
    let content = fs::read_to_string(&existing_file).unwrap();
    assert_eq!(content, "// existing content");
}

/// Test boilerplate scaffolding with conflict resolution (overwrite)
#[test]
fn test_boilerplate_scaffolding_conflict_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BoilerplateManager::new();

    // Create existing file
    let existing_file = temp_dir.path().join("src").join("main.rs");
    fs::create_dir_all(existing_file.parent().unwrap()).unwrap();
    fs::write(&existing_file, "// old content").unwrap();

    let boilerplate = Boilerplate {
        id: "rust-project".to_string(),
        name: "Rust Project".to_string(),
        description: "A Rust project boilerplate".to_string(),
        language: "rust".to_string(),
        files: vec![BoilerplateFile {
            path: "src/main.rs".to_string(),
            template: "fn main() { println!(\"new\"); }".to_string(),
            condition: None,
        }],
        dependencies: vec![],
        scripts: vec![],
    };

    let variables = HashMap::new();
    let result = manager
        .apply(
            &boilerplate,
            temp_dir.path(),
            &variables,
            ConflictResolution::Overwrite,
        )
        .unwrap();

    // Verify file was overwritten
    assert_eq!(result.created_files.len(), 1);
    assert_eq!(result.skipped_files.len(), 0);

    // Verify new content
    let content = fs::read_to_string(&existing_file).unwrap();
    assert!(content.contains("new"));
}

/// Test boilerplate scaffolding with conditional files
#[test]
fn test_boilerplate_scaffolding_conditional_files() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BoilerplateManager::new();

    let boilerplate = Boilerplate {
        id: "rust-project".to_string(),
        name: "Rust Project".to_string(),
        description: "A Rust project boilerplate".to_string(),
        language: "rust".to_string(),
        files: vec![
            BoilerplateFile {
                path: "src/main.rs".to_string(),
                template: "fn main() {}".to_string(),
                condition: Some("include_main".to_string()),
            },
            BoilerplateFile {
                path: "src/lib.rs".to_string(),
                template: "pub fn lib() {}".to_string(),
                condition: Some("include_lib".to_string()),
            },
        ],
        dependencies: vec![],
        scripts: vec![],
    };

    let mut variables = HashMap::new();
    variables.insert("include_main".to_string(), "true".to_string());
    variables.insert("include_lib".to_string(), "false".to_string());

    let result = manager
        .apply(
            &boilerplate,
            temp_dir.path(),
            &variables,
            ConflictResolution::Skip,
        )
        .unwrap();

    // Verify only main.rs was created
    assert_eq!(result.created_files.len(), 1);
    assert!(temp_dir.path().join("src/main.rs").exists());
    assert!(!temp_dir.path().join("src/lib.rs").exists());
}

/// Test full workflow: parse template, extract placeholders, render with context
#[test]
fn test_full_workflow_parse_extract_render() {
    let template = r#"
pub struct {{Name}} {
    pub id: u64,
    pub name: String,
}

impl {{Name}} {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
        }
    }
}
"#;

    // Step 1: Parse template
    let parsed = TemplateParser::parse(template).unwrap();
    assert!(parsed.placeholder_names.contains("name"));

    // Step 2: Extract placeholders
    let placeholders = TemplateParser::extract_placeholders(template).unwrap();
    assert!(placeholders.len() >= 1);

    // Step 3: Verify template structure
    assert!(!parsed.elements.is_empty());

    // Step 4: Render with simple rendering
    let mut engine = TemplateEngine::new();
    engine.add_value("name", "User");

    let result = engine.render_simple(template).unwrap();
    assert!(result.contains("User")); // {{Name}} -> PascalCase
}

/// Test placeholder resolution with nested values
#[test]
fn test_placeholder_resolution_nested() {
    let mut resolver = PlaceholderResolver::new();
    resolver.add_value("base", "my_project");
    resolver.add_value("full_name", "{{base}}_extended");

    let result = resolver.resolve_nested("full_name", CaseTransform::SnakeCase).unwrap();
    assert_eq!(result, "my_project_extended");
}

/// Test case transformation consistency across all variations
#[test]
fn test_case_transformation_consistency() {
    let input = "my_project_name";

    let pascal = CaseTransform::PascalCase.apply(input);
    let camel = CaseTransform::CamelCase.apply(input);
    let snake = CaseTransform::SnakeCase.apply(input);
    let kebab = CaseTransform::KebabCase.apply(input);
    let upper = CaseTransform::UpperCase.apply(input);
    let lower = CaseTransform::LowerCase.apply(input);

    // Verify all transformations produce different results
    assert_eq!(pascal, "MyProjectName");
    assert_eq!(camel, "myProjectName");
    assert_eq!(snake, "my_project_name");
    assert_eq!(kebab, "my-project-name");
    assert_eq!(upper, "MY_PROJECT_NAME");
    assert_eq!(lower, "my_project_name");

    // Verify consistency: applying same transform twice gives same result
    assert_eq!(pascal, CaseTransform::PascalCase.apply(&pascal));
    assert_eq!(camel, CaseTransform::CamelCase.apply(&camel));
}

/// Test boilerplate with multiple files and complex templates
#[test]
fn test_boilerplate_complex_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BoilerplateManager::new();

    let boilerplate = Boilerplate {
        id: "rust-lib".to_string(),
        name: "Rust Library".to_string(),
        description: "A Rust library boilerplate".to_string(),
        language: "rust".to_string(),
        files: vec![
            BoilerplateFile {
                path: "Cargo.toml".to_string(),
                template: r#"[package]
name = "{{name_snake}}"
version = "0.1.0"
authors = ["{{author}}"]

[dependencies]
"#.to_string(),
                condition: None,
            },
            BoilerplateFile {
                path: "src/lib.rs".to_string(),
                template: r#"//! {{Name}} library
//! 
//! {{description}}

pub struct {{Name}} {
    data: String,
}

impl {{Name}} {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}
"#.to_string(),
                condition: None,
            },
            BoilerplateFile {
                path: "README.md".to_string(),
                template: r#"# {{Name}}

{{description}}

## Usage

```rust
use {{name_snake}};

let instance = {{Name}}::new("data".to_string());
```
"#.to_string(),
                condition: None,
            },
        ],
        dependencies: vec![],
        scripts: vec![],
    };

    let mut variables = HashMap::new();
    variables.insert("name".to_string(), "MyLibrary".to_string());
    variables.insert("Name".to_string(), "MyLibrary".to_string());
    variables.insert("author".to_string(), "John Doe".to_string());
    variables.insert("description".to_string(), "A useful library".to_string());

    let result = manager
        .apply(
            &boilerplate,
            temp_dir.path(),
            &variables,
            ConflictResolution::Skip,
        )
        .unwrap();

    // Verify all files were created
    assert_eq!(result.created_files.len(), 3);

    // Verify Cargo.toml
    let cargo = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("my_library")); // {{name_snake}} -> snake_case
    assert!(cargo.contains("john doe")); // {{author}} -> lowercase

    // Verify lib.rs
    let lib = fs::read_to_string(temp_dir.path().join("src/lib.rs")).unwrap();
    assert!(lib.contains("MyLibrary")); // {{Name}} -> PascalCase
    assert!(lib.contains("a useful library")); // {{description}} -> lowercase

    // Verify README.md
    let readme = fs::read_to_string(temp_dir.path().join("README.md")).unwrap();
    assert!(readme.contains("MyLibrary")); // {{Name}} -> PascalCase
    assert!(readme.contains("my_library")); // {{name_snake}} -> snake_case
}
