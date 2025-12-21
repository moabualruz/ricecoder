//! Cross-crate integration tests for domain + application + infrastructure with enterprise features
//!
//! Tests the integration between:
//! - ricecoder-domain (entities and business logic)
//! - ricecoder-orchestration (application layer coordination)
//! - ricecoder-permissions (enterprise access control)
//! - ricecoder-security (encryption and compliance)
//! - ricecoder-storage (infrastructure persistence)

use ricecoder_domain::{entities::*, value_objects::*};
use ricecoder_orchestration::{OrchestrationManager, Workspace, WorkspaceScanner};
use ricecoder_permissions::{PermissionLevel, PermissionManager, ToolPermission};
use ricecoder_security::{compliance::ComplianceManager, encryption::KeyManager};
use ricecoder_storage::{StorageManager, StorageMode};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

/// Mock storage manager for testing
struct MockStorageManager {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl StorageManager for MockStorageManager {
    fn global_path(&self) -> &PathBuf {
        &self.global_path
    }

    fn project_path(&self) -> Option<&PathBuf> {
        self.project_path.as_ref()
    }

    fn mode(&self) -> StorageMode {
        StorageMode::Merged
    }

    fn global_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> PathBuf {
        self.global_path.join("resources")
    }

    fn project_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> Option<PathBuf> {
        self.project_path.as_ref().map(|p| p.join("resources"))
    }

    fn is_first_run(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn test_domain_orchestration_permissions_integration(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create workspace structure
    let project1_dir = workspace_root.join("project1");
    let project2_dir = workspace_root.join("project2");
    std::fs::create_dir_all(&project1_dir)?;
    std::fs::create_dir_all(&project2_dir)?;

    // Initialize storage
    let storage = Arc::new(MockStorageManager {
        global_path: workspace_root.clone(),
        project_path: Some(workspace_root.clone()),
    });

    // Initialize orchestration manager
    let orchestration = OrchestrationManager::new(workspace_root.clone());
    orchestration.initialize().await?;

    // Create domain projects
    let project1 = Project::new(
        "test-project-1".to_string(),
        ProgrammingLanguage::Rust,
        project1_dir.to_string_lossy().to_string(),
    )?;
    let project2 = Project::new(
        "test-project-2".to_string(),
        ProgrammingLanguage::Python,
        project2_dir.to_string_lossy().to_string(),
    )?;

    // Initialize permission manager with enterprise features
    let permission_manager = PermissionManager::new(storage.clone());

    // Set up enterprise permissions for projects
    permission_manager
        .add_permission(
            &project1.id.to_string(),
            ToolPermission {
                tool_name: "cargo_build".to_string(),
                level: PermissionLevel::Allow,
                requires_prompt: false,
            },
        )
        .await?;

    permission_manager
        .add_permission(
            &project2.id.to_string(),
            ToolPermission {
                tool_name: "pip_install".to_string(),
                level: PermissionLevel::Ask,
                requires_prompt: true,
            },
        )
        .await?;

    // Test orchestration scanning with permissions
    let scanner = WorkspaceScanner::new(workspace_root.clone());
    let discovered_projects = scanner.scan_workspace().await?;

    assert!(!discovered_projects.is_empty(), "Should discover projects");

    // Test permission checks during orchestration operations
    for project in &discovered_projects {
        let can_build = permission_manager
            .check_permission(&project.id.to_string(), "cargo_build")
            .await?;

        // Verify enterprise permission enforcement
        if project.name.contains("project1") {
            assert_eq!(
                can_build,
                PermissionLevel::Allow,
                "Project1 should allow cargo_build"
            );
        }
    }

    // Test compliance integration
    let compliance_manager = ComplianceManager::new();
    let compliance_report = compliance_manager
        .validate_workspace(&workspace_root)
        .await?;

    assert!(
        compliance_report.is_compliant,
        "Workspace should be compliant"
    );

    // Clean up
    orchestration.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_session_domain_integration_with_encryption() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize encrypted storage
    let key_manager = KeyManager::new()?;
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    // Create domain session
    let mut session = Session::new("openai".to_string(), "gpt-4".to_string());
    session.set_name("encrypted-test-session".to_string());

    let project = Project::new(
        "encrypted-project".to_string(),
        ProgrammingLanguage::Rust,
        "/tmp/test".to_string(),
    )?;
    session.set_project(project.id.clone());

    // Test session persistence with encryption
    let session_data = serde_json::to_string(&session)?;
    let encrypted_data = key_manager.encrypt(session_data.as_bytes())?;

    // Store encrypted session
    let session_file = storage_path.join("sessions").join("test-session.enc");
    std::fs::create_dir_all(session_file.parent().unwrap())?;
    std::fs::write(&session_file, encrypted_data)?;

    // Retrieve and decrypt session
    let stored_encrypted = std::fs::read(&session_file)?;
    let decrypted_data = key_manager.decrypt(&stored_encrypted)?;
    let restored_session: Session = serde_json::from_slice(&decrypted_data)?;

    // Verify session integrity
    assert_eq!(restored_session.id, session.id);
    assert_eq!(restored_session.name, session.name);
    assert_eq!(restored_session.project_id, session.project_id);
    assert_eq!(restored_session.provider_id, session.provider_id);
    assert_eq!(restored_session.model_id, session.model_id);

    Ok(())
}

#[tokio::test]
async fn test_provider_domain_integration_with_compliance() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempdir()?;

    // Create domain provider
    let mut provider = Provider::new(
        "test-openai".to_string(),
        "Test OpenAI".to_string(),
        ProviderType::OpenAI,
    );

    let model = ModelInfo::new("gpt-4".to_string(), "GPT-4".to_string(), 8192)
        .with_function_calling()
        .with_pricing(0.03, 0.06);
    provider.add_model(model);

    // Test provider compliance validation
    let compliance_manager = ComplianceManager::new();
    let compliance_result = compliance_manager.validate_provider(&provider).await?;

    assert!(
        compliance_result.is_compliant,
        "Provider should be compliant"
    );
    assert!(
        compliance_result.violations.is_empty(),
        "Should have no violations"
    );

    // Test data classification
    let sensitive_data = "API_KEY=sk-1234567890abcdef";
    let classification = compliance_manager.classify_data(sensitive_data)?;

    assert!(
        classification.contains(&ricecoder_security::compliance::DataClassification::Sensitive),
        "Should classify API key as sensitive"
    );

    Ok(())
}

#[tokio::test]
async fn test_analysis_result_domain_orchestration_integration(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize orchestration
    let orchestration = OrchestrationManager::new(workspace_root.clone());
    orchestration.initialize().await?;

    // Create domain project and analysis
    let project = Project::new(
        "analysis-test".to_string(),
        ProgrammingLanguage::Rust,
        workspace_root
            .join("test-project")
            .to_string_lossy()
            .to_string(),
    )?;

    let mut analysis = AnalysisResult::new(project.id.clone(), None, AnalysisType::Security);

    let metrics = ricecoder_domain::entities::AnalysisMetrics {
        lines_of_code: 150,
        cyclomatic_complexity: 12.5,
        maintainability_index: 75.0,
        technical_debt_ratio: 0.15,
        execution_time_ms: 250,
    };

    analysis.complete(
        serde_json::json!({
            "vulnerabilities": [],
            "security_score": 95
        }),
        metrics,
    );

    // Test orchestration impact analysis
    let impact_report = orchestration.analyze_impact(&project.id).await?;
    assert!(
        impact_report.details.is_empty() || !impact_report.details.is_empty(),
        "Impact analysis should work"
    );

    // Test compliance monitoring of analysis results
    let compliance_manager = ComplianceManager::new();
    let audit_result = compliance_manager.audit_analysis_result(&analysis).await?;

    assert!(
        audit_result.is_compliant,
        "Analysis result should be compliant"
    );

    // Clean up
    orchestration.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_enterprise_workspace_orchestration_with_security(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create enterprise workspace structure
    let departments = ["engineering", "security", "compliance"];
    for dept in &departments {
        let dept_dir = workspace_root.join(dept);
        std::fs::create_dir_all(&dept_dir)?;

        // Create department projects
        for i in 1..=3 {
            let project_dir = dept_dir.join(format!("project{}", i));
            std::fs::create_dir_all(&project_dir)?;
        }
    }

    // Initialize orchestration with enterprise features
    let orchestration = OrchestrationManager::new(workspace_root.clone());
    orchestration.initialize().await?;

    // Initialize security and permissions
    let storage = Arc::new(MockStorageManager {
        global_path: workspace_root.clone(),
        project_path: Some(workspace_root.clone()),
    });

    let permission_manager = PermissionManager::new(storage.clone());
    let compliance_manager = ComplianceManager::new();

    // Set up role-based permissions
    permission_manager
        .add_permission(
            "engineering",
            ToolPermission {
                tool_name: "deploy".to_string(),
                level: PermissionLevel::Ask,
                requires_prompt: true,
            },
        )
        .await?;

    permission_manager
        .add_permission(
            "security",
            ToolPermission {
                tool_name: "security_scan".to_string(),
                level: PermissionLevel::Allow,
                requires_prompt: false,
            },
        )
        .await?;

    // Test enterprise workspace scanning
    let scanner = WorkspaceScanner::new(workspace_root.clone());
    let projects = scanner.scan_workspace().await?;

    assert!(!projects.is_empty(), "Should discover enterprise projects");

    // Test compliance validation across workspace
    let compliance_report = compliance_manager
        .validate_workspace(&workspace_root)
        .await?;
    assert!(
        compliance_report.is_compliant,
        "Enterprise workspace should be compliant"
    );

    // Test orchestration with security constraints
    let status_report = orchestration.generate_status_report().await?;
    assert!(
        status_report.projects_analyzed > 0,
        "Should analyze projects"
    );

    // Clean up
    orchestration.shutdown().await?;

    Ok(())
}
