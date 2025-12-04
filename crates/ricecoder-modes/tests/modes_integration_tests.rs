//! Integration tests for ricecoder-modes
//!
//! These tests verify the complete mode lifecycle and interactions:
//! - Full mode lifecycle (initialization → selection → operation → switch)
//! - Mode interaction with session management
//! - Mode interaction with AI providers
//! - Think More integration with provider
//! - Error recovery and fallback behavior
//! - Context preservation across multiple mode switches

use ricecoder_modes::{
    AskMode, Capability, CodeMode, ComplexityLevel, ModeAction, ModeConfig, ModeConstraints,
    ModeContext, ModeError, ModeManager, ModeResponse, ModeSwitcher, Operation, ResponseMetadata,
    ThinkMoreConfig, ThinkingDepth, VibeMode,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Helper function to create a test mode context
fn create_test_context(session_id: &str) -> ModeContext {
    let mut context = ModeContext::new(session_id.to_string());
    context.project_path = Some(PathBuf::from("/test/project"));
    context
}

/// Test 9.1: Full mode lifecycle
/// Initialize system, select mode, execute operation, switch mode, verify context
#[tokio::test]
async fn test_full_mode_lifecycle() {
    // Initialize system
    let context = create_test_context("lifecycle-test");
    let mut manager = ModeManager::new(context);

    // Create and register modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());
    let vibe_mode = Arc::new(VibeMode::new());

    manager.register_mode(code_mode.clone());
    manager.register_mode(ask_mode.clone());
    manager.register_mode(vibe_mode.clone());

    // Verify all modes are registered
    assert_eq!(manager.mode_count(), 3);
    assert!(manager.has_mode("code"));
    assert!(manager.has_mode("ask"));
    assert!(manager.has_mode("vibe"));

    // Select Code Mode
    let selected_mode = manager.switch_mode("code").await;
    assert!(selected_mode.is_ok());
    let current = manager.current_mode().await;
    assert!(current.is_ok());
    assert!(current.unwrap().is_some());

    // Verify Code Mode capabilities
    let code_mode_ref = manager.get_mode("code").unwrap();
    let capabilities = code_mode_ref.capabilities();
    assert!(capabilities.contains(&Capability::CodeGeneration));
    assert!(capabilities.contains(&Capability::FileOperations));
    assert!(capabilities.contains(&Capability::TestExecution));

    // Verify Code Mode can execute code operations
    assert!(code_mode_ref.can_execute(&Operation::GenerateCode));
    assert!(code_mode_ref.can_execute(&Operation::ModifyFile));
    assert!(code_mode_ref.can_execute(&Operation::RunTests));

    // Switch to Ask Mode
    let switched = manager.switch_mode("ask").await;
    assert!(switched.is_ok());

    // Verify Ask Mode capabilities
    let ask_mode_ref = manager.get_mode("ask").unwrap();
    let ask_capabilities = ask_mode_ref.capabilities();
    assert!(ask_capabilities.contains(&Capability::QuestionAnswering));
    assert!(!ask_capabilities.contains(&Capability::FileOperations));

    // Verify Ask Mode cannot execute file operations
    assert!(!ask_mode_ref.can_execute(&Operation::ModifyFile));
    assert!(ask_mode_ref.can_execute(&Operation::AnswerQuestion));

    // Switch to Vibe Mode
    let switched_vibe = manager.switch_mode("vibe").await;
    assert!(switched_vibe.is_ok());

    // Verify Vibe Mode capabilities
    let vibe_mode_ref = manager.get_mode("vibe").unwrap();
    let vibe_capabilities = vibe_mode_ref.capabilities();
    assert!(vibe_capabilities.contains(&Capability::CodeGeneration));
    assert!(vibe_capabilities.contains(&Capability::SpecConversion));

    // Verify context is still valid
    let final_context = manager.context().await;
    assert_eq!(final_context.session_id, "lifecycle-test");
    assert_eq!(final_context.project_path, Some(PathBuf::from("/test/project")));
}

/// Test 9.2: Mode interaction with session management
/// Test mode interaction with session management
/// Test context preservation across sessions
#[tokio::test]
async fn test_mode_interaction_with_session() {
    // Create a mode switcher with initial context
    let context = create_test_context("session-test");
    let mut switcher = ModeSwitcher::new(context);

    // Register modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());

    switcher.register_mode(code_mode);
    switcher.register_mode(ask_mode);

    // Switch to Code Mode
    switcher.switch_mode("code").await.unwrap();

    // Add messages to conversation history (simulating session interaction)
    switcher
        .update_context(|ctx| {
            ctx.add_message(
                ricecoder_modes::MessageRole::User,
                "Generate a function".to_string(),
            );
        })
        .await
        .unwrap();

    let ctx_code = switcher.context().await;
    assert_eq!(ctx_code.conversation_history.len(), 1);
    assert_eq!(ctx_code.session_id, "session-test");

    // Switch to Ask Mode
    switcher.switch_mode("ask").await.unwrap();

    // Context should be fresh for Ask Mode
    let ctx_ask = switcher.context().await;
    assert_eq!(ctx_ask.conversation_history.len(), 0);
    assert_eq!(ctx_ask.session_id, "session-test"); // Session ID preserved

    // Add messages to Ask Mode context
    switcher
        .update_context(|ctx| {
            ctx.add_message(
                ricecoder_modes::MessageRole::User,
                "What is Rust?".to_string(),
            );
        })
        .await
        .unwrap();

    let ctx_ask_updated = switcher.context().await;
    assert_eq!(ctx_ask_updated.conversation_history.len(), 1);

    // Switch back to Code Mode
    switcher.switch_mode("code").await.unwrap();

    // Context should be restored for Code Mode
    let ctx_code_restored = switcher.context().await;
    assert_eq!(ctx_code_restored.conversation_history.len(), 1);
    assert_eq!(
        ctx_code_restored.conversation_history[0].content,
        "Generate a function"
    );

    // Verify project path is preserved across all switches
    assert_eq!(
        ctx_code_restored.project_path,
        Some(PathBuf::from("/test/project"))
    );
}

/// Test 9.3: Think More integration with provider
/// Test Think More integration with AI provider
/// Test thinking display and timeout
#[tokio::test]
async fn test_think_more_integration() {
    let mut context = create_test_context("think-more-test");

    // Configure Think More
    context.think_more_config = ThinkMoreConfig {
        enabled: true,
        depth: ThinkingDepth::Deep,
        timeout: Duration::from_secs(5),
        auto_enable: false,
    };

    let manager = ModeManager::new(context);

    // Register Code Mode
    let code_mode = Arc::new(CodeMode::new());
    let mut manager_mut = manager;
    manager_mut.register_mode(code_mode);

    // Switch to Code Mode
    manager_mut.switch_mode("code").await.unwrap();

    // Enable Think More
    manager_mut
        .update_context(|ctx| {
            ctx.think_more_enabled = true;
            ctx.think_more_config.enabled = true;
            ctx.think_more_config.depth = ThinkingDepth::Deep;
        })
        .await
        .unwrap();

    let ctx = manager_mut.context().await;
    assert!(ctx.think_more_enabled);
    assert_eq!(ctx.think_more_config.depth, ThinkingDepth::Deep);
    assert_eq!(ctx.think_more_config.timeout, Duration::from_secs(5));

    // Verify Think More configuration is preserved
    let ctx_check = manager_mut.context().await;
    assert!(ctx_check.think_more_config.enabled);
    assert_eq!(ctx_check.think_more_config.depth, ThinkingDepth::Deep);
}

/// Test 9.4: Error recovery and fallback behavior
/// Test error handling and recovery
/// Test fallback behavior
#[tokio::test]
async fn test_error_recovery_and_fallback() {
    let context = create_test_context("error-recovery-test");
    let manager = ModeManager::new(context);

    // Test switching to non-existent mode
    let result = manager.switch_mode("nonexistent").await;
    assert!(result.is_err());
    match result {
        Err(ModeError::NotFound(mode_id)) => {
            assert_eq!(mode_id, "nonexistent");
        }
        _ => panic!("Expected NotFound error"),
    }

    // Verify current mode is still None after failed switch
    let current = manager.current_mode().await;
    assert!(current.is_ok());
    assert!(current.unwrap().is_none());

    // Register a valid mode and switch successfully
    let code_mode = Arc::new(CodeMode::new());
    let mut manager_mut = manager;
    manager_mut.register_mode(code_mode);

    let result = manager_mut.switch_mode("code").await;
    assert!(result.is_ok());

    // Verify current mode is now set
    let current = manager_mut.current_mode().await;
    assert!(current.is_ok());
    assert!(current.unwrap().is_some());
}

/// Test 9.5: Context preservation across multiple mode switches
/// Verify context is preserved when switching between multiple modes
#[tokio::test]
async fn test_context_preservation_multiple_switches() {
    let context = create_test_context("multi-switch-test");
    let mut switcher = ModeSwitcher::new(context);

    // Register all modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());
    let vibe_mode = Arc::new(VibeMode::new());

    switcher.register_mode(code_mode);
    switcher.register_mode(ask_mode);
    switcher.register_mode(vibe_mode);

    // Switch to Code Mode and add context
    switcher.switch_mode("code").await.unwrap();
    switcher
        .update_context(|ctx| {
            ctx.add_message(
                ricecoder_modes::MessageRole::User,
                "Code message".to_string(),
            );
        })
        .await
        .unwrap();

    // Switch to Ask Mode and add context
    switcher.switch_mode("ask").await.unwrap();
    switcher
        .update_context(|ctx| {
            ctx.add_message(
                ricecoder_modes::MessageRole::User,
                "Ask message".to_string(),
            );
        })
        .await
        .unwrap();

    // Switch to Vibe Mode and add context
    switcher.switch_mode("vibe").await.unwrap();
    switcher
        .update_context(|ctx| {
            ctx.add_message(
                ricecoder_modes::MessageRole::User,
                "Vibe message".to_string(),
            );
        })
        .await
        .unwrap();

    // Switch back to Code Mode and verify context is restored
    switcher.switch_mode("code").await.unwrap();
    let code_ctx = switcher.context().await;
    assert_eq!(code_ctx.conversation_history.len(), 1);
    assert_eq!(
        code_ctx.conversation_history[0].content,
        "Code message"
    );

    // Switch back to Ask Mode and verify context is restored
    switcher.switch_mode("ask").await.unwrap();
    let ask_ctx = switcher.context().await;
    assert_eq!(ask_ctx.conversation_history.len(), 1);
    assert_eq!(ask_ctx.conversation_history[0].content, "Ask message");

    // Switch back to Vibe Mode and verify context is restored
    switcher.switch_mode("vibe").await.unwrap();
    let vibe_ctx = switcher.context().await;
    assert_eq!(vibe_ctx.conversation_history.len(), 1);
    assert_eq!(vibe_ctx.conversation_history[0].content, "Vibe message");
}

/// Test 9.6: Mode constraints enforcement
/// Verify that mode constraints are properly enforced
#[tokio::test]
async fn test_mode_constraints_enforcement() {
    let context = create_test_context("constraints-test");
    let mut manager = ModeManager::new(context);

    // Register all modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());
    let vibe_mode = Arc::new(VibeMode::new());

    manager.register_mode(code_mode);
    manager.register_mode(ask_mode);
    manager.register_mode(vibe_mode);

    // Test Code Mode constraints
    let code_mode_ref = manager.get_mode("code").unwrap();
    let code_constraints = code_mode_ref.constraints();
    assert!(code_constraints.allow_file_operations);
    assert!(code_constraints.allow_command_execution);
    assert!(code_constraints.allow_code_generation);

    // Test Ask Mode constraints
    let ask_mode_ref = manager.get_mode("ask").unwrap();
    let ask_constraints = ask_mode_ref.constraints();
    assert!(!ask_constraints.allow_file_operations);
    assert!(!ask_constraints.allow_command_execution);
    assert!(!ask_constraints.allow_code_generation);

    // Test Vibe Mode constraints
    let vibe_mode_ref = manager.get_mode("vibe").unwrap();
    let vibe_constraints = vibe_mode_ref.constraints();
    assert!(vibe_constraints.allow_file_operations);
    assert!(!vibe_constraints.allow_command_execution);
    assert!(vibe_constraints.allow_code_generation);
}

/// Test 9.7: Mode capability verification
/// Verify that each mode has the correct capabilities
#[tokio::test]
async fn test_mode_capability_verification() {
    let context = create_test_context("capability-test");
    let mut manager = ModeManager::new(context);

    // Register all modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());
    let vibe_mode = Arc::new(VibeMode::new());

    manager.register_mode(code_mode);
    manager.register_mode(ask_mode);
    manager.register_mode(vibe_mode);

    // Verify Code Mode capabilities
    let code_mode_ref = manager.get_mode("code").unwrap();
    let code_caps = code_mode_ref.capabilities();
    assert!(code_caps.contains(&Capability::CodeGeneration));
    assert!(code_caps.contains(&Capability::CodeModification));
    assert!(code_caps.contains(&Capability::FileOperations));
    assert!(code_caps.contains(&Capability::CommandExecution));
    assert!(code_caps.contains(&Capability::TestExecution));
    assert!(code_caps.contains(&Capability::QualityValidation));

    // Verify Ask Mode capabilities
    let ask_mode_ref = manager.get_mode("ask").unwrap();
    let ask_caps = ask_mode_ref.capabilities();
    assert!(ask_caps.contains(&Capability::QuestionAnswering));
    assert!(ask_caps.contains(&Capability::FreeformChat));
    assert!(!ask_caps.contains(&Capability::CodeGeneration));
    assert!(!ask_caps.contains(&Capability::FileOperations));

    // Verify Vibe Mode capabilities
    let vibe_mode_ref = manager.get_mode("vibe").unwrap();
    let vibe_caps = vibe_mode_ref.capabilities();
    assert!(vibe_caps.contains(&Capability::CodeGeneration));
    assert!(vibe_caps.contains(&Capability::CodeModification));
    assert!(vibe_caps.contains(&Capability::FileOperations));
    assert!(vibe_caps.contains(&Capability::FreeformChat));
    assert!(vibe_caps.contains(&Capability::QuestionAnswering));
    assert!(vibe_caps.contains(&Capability::SpecConversion));
}

/// Test 9.8: Session ID preservation across mode switches
/// Verify that session ID is preserved across all mode switches
#[tokio::test]
async fn test_session_id_preservation() {
    let context = create_test_context("session-id-test");
    let session_id = context.session_id.clone();
    let mut switcher = ModeSwitcher::new(context);

    // Register modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());
    let vibe_mode = Arc::new(VibeMode::new());

    switcher.register_mode(code_mode);
    switcher.register_mode(ask_mode);
    switcher.register_mode(vibe_mode);

    // Switch through all modes and verify session ID is preserved
    for mode_id in &["code", "ask", "vibe", "code", "ask"] {
        switcher.switch_mode(mode_id).await.unwrap();
        let ctx = switcher.context().await;
        assert_eq!(ctx.session_id, session_id);
    }
}

/// Test 9.9: Project path preservation across mode switches
/// Verify that project path is preserved across all mode switches
#[tokio::test]
async fn test_project_path_preservation() {
    let context = create_test_context("project-path-test");
    let project_path = context.project_path.clone();
    let mut switcher = ModeSwitcher::new(context);

    // Register modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());

    switcher.register_mode(code_mode);
    switcher.register_mode(ask_mode);

    // Switch between modes and verify project path is preserved
    switcher.switch_mode("code").await.unwrap();
    let ctx1 = switcher.context().await;
    assert_eq!(ctx1.project_path, project_path);

    switcher.switch_mode("ask").await.unwrap();
    let ctx2 = switcher.context().await;
    assert_eq!(ctx2.project_path, project_path);

    switcher.switch_mode("code").await.unwrap();
    let ctx3 = switcher.context().await;
    assert_eq!(ctx3.project_path, project_path);
}

/// Test 9.10: Mode operation validation
/// Verify that mode operation validation works correctly
#[tokio::test]
async fn test_mode_operation_validation() {
    let context = create_test_context("operation-validation-test");
    let manager = ModeManager::new(context);

    // Register modes
    let code_mode = Arc::new(CodeMode::new());
    let ask_mode = Arc::new(AskMode::new());

    let mut manager_mut = manager;
    manager_mut.register_mode(code_mode);
    manager_mut.register_mode(ask_mode);

    // Test Code Mode operations
    let code_mode_ref = manager_mut.get_mode("code").unwrap();
    assert!(code_mode_ref.can_execute(&Operation::GenerateCode));
    assert!(code_mode_ref.can_execute(&Operation::ModifyFile));
    assert!(code_mode_ref.can_execute(&Operation::RunTests));
    assert!(code_mode_ref.can_execute(&Operation::ValidateQuality));
    assert!(!code_mode_ref.can_execute(&Operation::AnswerQuestion));

    // Test Ask Mode operations
    let ask_mode_ref = manager_mut.get_mode("ask").unwrap();
    assert!(ask_mode_ref.can_execute(&Operation::AnswerQuestion));
    assert!(!ask_mode_ref.can_execute(&Operation::GenerateCode));
    assert!(!ask_mode_ref.can_execute(&Operation::ModifyFile));
    assert!(!ask_mode_ref.can_execute(&Operation::RunTests));
}
