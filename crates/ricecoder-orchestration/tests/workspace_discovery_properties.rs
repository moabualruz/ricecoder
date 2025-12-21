//! Property-based tests for workspace discovery
//!
//! **Feature: ricecoder-orchestration, Property 1: Workspace Discovery Completeness**
//! **Validates: Requirements 1.1**

use proptest::prelude::*;
use ricecoder_orchestration::WorkspaceScanner;
use std::path::PathBuf;
use tempfile::TempDir;

/// Strategy for generating project types
fn project_type_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("rust"), Just("nodejs"), Just("python"),]
}

/// Strategy for generating project names
fn project_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating unique project specs
fn unique_project_specs_strategy() -> impl Strategy<Value = Vec<(String, String, String)>> {
    prop::collection::vec(
        (
            prop_oneof![
                Just("rust".to_string()),
                Just("nodejs".to_string()),
                Just("python".to_string()),
            ],
            "[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string()),
            r"[0-9]\.[0-9]\.[0-9]".prop_map(|s| s.to_string()),
        ),
        1..10,
    )
    .prop_map(|mut specs| {
        // Make project names unique by adding index
        for (i, spec) in specs.iter_mut().enumerate() {
            spec.1 = format!("{}-{}", spec.1, i);
        }
        specs
    })
}

/// Strategy for generating version strings
fn version_strategy() -> impl Strategy<Value = String> {
    r"[0-9]\.[0-9]\.[0-9]".prop_map(|s| s.to_string())
}

/// Creates a manifest file for a given project type
fn create_manifest(
    project_dir: &PathBuf,
    project_type: &str,
    version: &str,
) -> std::io::Result<()> {
    match project_type {
        "rust" => {
            std::fs::write(
                project_dir.join("Cargo.toml"),
                format!("[package]\nname = \"test\"\nversion = \"{}\"\n", version),
            )?;
        }
        "nodejs" => {
            std::fs::write(
                project_dir.join("package.json"),
                format!(r#"{{"name": "test", "version": "{}"}}"#, version),
            )?;
        }
        "python" => {
            std::fs::write(
                project_dir.join("pyproject.toml"),
                format!("[project]\nname = \"test\"\nversion = \"{}\"\n", version),
            )?;
        }
        "go" => {
            std::fs::write(project_dir.join("go.mod"), "module example.com/test\n")?;
        }
        "java" => {
            std::fs::write(
                project_dir.join("pom.xml"),
                format!("<project><version>{}</version></project>", version),
            )?;
        }
        "gradle" => {
            std::fs::write(
                project_dir.join("build.gradle"),
                format!("plugins {{}}\nversion = \"{}\"\n", version),
            )?;
        }
        _ => {}
    }
    Ok(())
}

proptest! {
    /// Property 1: Workspace Discovery Completeness
    ///
    /// For any workspace with projects in standard locations, the WorkspaceScanner
    /// SHALL discover all projects and their metadata without missing any.
    ///
    /// **Validates: Requirements 1.1**
    #[test]
    fn prop_workspace_discovery_discovers_all_projects(
        project_specs in unique_project_specs_strategy()
    ) {
        // Create a temporary workspace
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();
        let projects_dir = workspace_root.join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        // Create projects according to the generated specs
        for (project_type, project_name, version) in &project_specs {
            let project_dir = projects_dir.join(project_name);
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            create_manifest(&project_dir, project_type, version)
                .expect("failed to create manifest");
        }

        // Scan the workspace
        let scanner = WorkspaceScanner::new(workspace_root);
        let discovered_projects = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(scanner.scan_workspace())
            .expect("scan failed");

        // Verify all projects were discovered
        prop_assert_eq!(
            discovered_projects.len(),
            project_specs.len(),
            "Not all projects were discovered"
        );

        // Sort both lists by project name for comparison
        let mut discovered_sorted = discovered_projects.clone();
        discovered_sorted.sort_by(|a, b| a.name.cmp(&b.name));

        let mut specs_sorted: Vec<_> = project_specs.iter().map(|(t, n, v)| (t.as_str(), n.as_str(), v.as_str())).collect();
        specs_sorted.sort_by(|a, b| a.1.cmp(b.1));

        // Verify each project has correct metadata
        for (i, (expected_type, expected_name, expected_version)) in specs_sorted.iter().enumerate() {
            let project = &discovered_sorted[i];
            prop_assert_eq!(
                &project.name,
                expected_name,
                "Project name mismatch at index {}", i
            );
            prop_assert_eq!(
                &project.project_type,
                expected_type,
                "Project type mismatch for {}", expected_name
            );
            prop_assert_eq!(
                &project.version,
                expected_version,
                "Project version mismatch for {}", expected_name
            );
        }
    }

    /// Property: No projects are missed during discovery
    ///
    /// For any workspace with projects, the scanner should discover exactly
    /// the number of projects that were created.
    #[test]
    fn prop_no_projects_missed(
        num_projects in 1usize..20
    ) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();
        let projects_dir = workspace_root.join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        // Create projects
        for i in 0..num_projects {
            let project_dir = projects_dir.join(format!("project-{}", i));
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            std::fs::write(
                project_dir.join("Cargo.toml"),
                format!("[package]\nname = \"project-{}\"\nversion = \"0.{}.0\"\n", i, i),
            ).expect("failed to write Cargo.toml");
        }

        // Scan the workspace
        let scanner = WorkspaceScanner::new(workspace_root);
        let discovered_projects = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(scanner.scan_workspace())
            .expect("scan failed");

        // Verify count matches
        prop_assert_eq!(
            discovered_projects.len(),
            num_projects,
            "Expected {} projects but discovered {}", num_projects, discovered_projects.len()
        );
    }

    /// Property: All discovered projects have valid metadata
    ///
    /// For any discovered project, it should have:
    /// - Non-empty name
    /// - Valid project type
    /// - Non-empty version
    /// - Valid path
    #[test]
    fn prop_discovered_projects_have_valid_metadata(
        project_specs in unique_project_specs_strategy()
    ) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();
        let projects_dir = workspace_root.join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        // Create projects
        for (project_type, project_name, version) in &project_specs {
            let project_dir = projects_dir.join(project_name);
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            create_manifest(&project_dir, project_type, version)
                .expect("failed to create manifest");
        }

        // Scan the workspace
        let scanner = WorkspaceScanner::new(workspace_root);
        let discovered_projects = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(scanner.scan_workspace())
            .expect("scan failed");

        // Verify metadata validity
        for project in discovered_projects {
            prop_assert!(!project.name.is_empty(), "Project name is empty");
            prop_assert!(!project.project_type.is_empty(), "Project type is empty");
            prop_assert!(!project.version.is_empty(), "Project version is empty");
            prop_assert!(project.path.exists(), "Project path does not exist: {:?}", project.path);
        }
    }

    /// Property: Mixed project types are all discovered
    ///
    /// For any workspace with mixed project types, all projects should be discovered
    /// regardless of their type.
    #[test]
    fn prop_mixed_project_types_discovered(
        rust_count in 0usize..5,
        nodejs_count in 0usize..5,
        python_count in 0usize..5,
    ) {
        prop_assume!(rust_count + nodejs_count + python_count > 0);

        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();
        let projects_dir = workspace_root.join("projects");
        std::fs::create_dir(&projects_dir).expect("failed to create projects dir");

        let mut total_projects = 0;

        // Create Rust projects
        for i in 0..rust_count {
            let project_dir = projects_dir.join(format!("rust-project-{}", i));
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            std::fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"\n")
                .expect("failed to write Cargo.toml");
            total_projects += 1;
        }

        // Create Node.js projects
        for i in 0..nodejs_count {
            let project_dir = projects_dir.join(format!("node-project-{}", i));
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            std::fs::write(project_dir.join("package.json"), "{}")
                .expect("failed to write package.json");
            total_projects += 1;
        }

        // Create Python projects
        for i in 0..python_count {
            let project_dir = projects_dir.join(format!("python-project-{}", i));
            std::fs::create_dir(&project_dir).expect("failed to create project dir");
            std::fs::write(project_dir.join("pyproject.toml"), "[build-system]\n")
                .expect("failed to write pyproject.toml");
            total_projects += 1;
        }

        // Scan the workspace
        let scanner = WorkspaceScanner::new(workspace_root);
        let discovered_projects = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(scanner.scan_workspace())
            .expect("scan failed");

        // Verify all projects were discovered
        prop_assert_eq!(
            discovered_projects.len(),
            total_projects,
            "Not all projects were discovered"
        );

        // Verify project types
        let rust_projects = discovered_projects.iter().filter(|p| p.project_type == "rust").count();
        let nodejs_projects = discovered_projects.iter().filter(|p| p.project_type == "nodejs").count();
        let python_projects = discovered_projects.iter().filter(|p| p.project_type == "python").count();

        prop_assert_eq!(rust_projects, rust_count, "Rust project count mismatch");
        prop_assert_eq!(nodejs_projects, nodejs_count, "Node.js project count mismatch");
        prop_assert_eq!(python_projects, python_count, "Python project count mismatch");
    }
}
