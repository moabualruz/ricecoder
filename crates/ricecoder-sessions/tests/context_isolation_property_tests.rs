//! Property-based tests for context isolation
//! **Feature: ricecoder-sessions, Property 3: Context Isolation**
//! **Validates: Requirements 1.4**

use proptest::prelude::*;
use ricecoder_sessions::{ContextManager, SessionContext, SessionMode};

/// Property: For any two distinct ContextManager instances, modifying the context
/// of one SHALL NOT affect the context of the other.
///
/// This property tests that:
/// 1. Two context managers can be created independently
/// 2. Modifications to one manager's context don't affect the other
/// 3. Each manager maintains its own isolated state
#[test]
fn prop_context_isolation_independent_managers() {
    proptest!(|(
        provider1 in "openai|anthropic|local",
        model1 in "gpt-4|claude-3|llama",
        provider2 in "openai|anthropic|local",
        model2 in "gpt-4|claude-3|llama",
        files1 in prop::collection::vec("[a-z0-9_]+\\.rs", 0..5),
        files2 in prop::collection::vec("[a-z0-9_]+\\.rs", 0..5),
    )| {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        // Set different contexts
        let context1 = SessionContext::new(
            provider1.to_string(),
            model1.to_string(),
            SessionMode::Chat,
        );
        let context2 = SessionContext::new(
            provider2.to_string(),
            model2.to_string(),
            SessionMode::Code,
        );

        manager1.set_context(context1);
        manager2.set_context(context2);

        // Add files to each manager
        for file in &files1 {
            manager1.add_file(file.clone()).ok();
        }
        for file in &files2 {
            manager2.add_file(file.clone()).ok();
        }

        // Verify contexts are independent
        let ctx1 = manager1.get_context().unwrap();
        let ctx2 = manager2.get_context().unwrap();

        prop_assert_eq!(ctx1.provider, provider1);
        prop_assert_eq!(ctx1.model, model1);
        prop_assert_eq!(ctx2.provider, provider2);
        prop_assert_eq!(ctx2.model, model2);

        // Verify files are isolated - each manager should have its own files
        // Note: files1 and files2 might contain duplicates, which are deduplicated
        // when added to the context
        let unique_files1: std::collections::HashSet<_> = files1.iter().cloned().collect();
        let unique_files2: std::collections::HashSet<_> = files2.iter().cloned().collect();

        prop_assert_eq!(ctx1.files.len(), unique_files1.len());
        prop_assert_eq!(ctx2.files.len(), unique_files2.len());

        // Verify that all files in ctx1 came from files1
        for file in &ctx1.files {
            prop_assert!(files1.contains(file));
        }

        // Verify that all files in ctx2 came from files2
        for file in &ctx2.files {
            prop_assert!(files2.contains(file));
        }
    });
}

/// Property: Modifying files in one context manager SHALL NOT affect files
/// in another context manager.
///
/// This property tests that:
/// 1. Adding files to one manager doesn't affect the other
/// 2. Removing files from one manager doesn't affect the other
/// 3. Clearing files in one manager doesn't affect the other
#[test]
fn prop_context_isolation_file_operations() {
    proptest!(|(
        files1 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..5),
        files2 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..5),
        remove_idx1 in 0usize..5,
    )| {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        let context = SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        );

        manager1.set_context(context.clone());
        manager2.set_context(context);

        // Add files to each manager
        for file in &files1 {
            manager1.add_file(file.clone()).ok();
        }
        for file in &files2 {
            manager2.add_file(file.clone()).ok();
        }

        // Remove a file from manager1
        if !files1.is_empty() {
            let idx = remove_idx1 % files1.len();
            manager1.remove_file(&files1[idx]).ok();
        }

        // Clear files from manager2
        manager2.clear_files().ok();

        // Verify isolation
        let ctx1_files = manager1.get_files().unwrap();
        let ctx2_files = manager2.get_files().unwrap();

        // Manager1 should have files1 minus the removed file
        prop_assert!(ctx1_files.len() <= files1.len());

        // Manager2 should have no files
        prop_assert_eq!(ctx2_files.len(), 0);

        // Manager1's files should not include manager2's files
        for file in &files2 {
            prop_assert!(!ctx1_files.contains(file));
        }
    });
}

/// Property: Switching projects in one context manager SHALL NOT affect
/// the project path in another context manager.
///
/// This property tests that:
/// 1. Switching projects in one manager doesn't affect the other
/// 2. Each manager maintains its own project path
/// 3. Project switching clears files only in the affected manager
#[test]
fn prop_context_isolation_project_switching() {
    proptest!(|(
        project1 in "/[a-z0-9_/]+",
        project2 in "/[a-z0-9_/]+",
        files1 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..3),
        files2 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..3),
    )| {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        let context = SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        );

        manager1.set_context(context.clone());
        manager2.set_context(context);

        // Add files to each manager
        for file in &files1 {
            manager1.add_file(file.clone()).ok();
        }
        for file in &files2 {
            manager2.add_file(file.clone()).ok();
        }

        // Switch projects
        manager1.switch_project(project1.to_string()).ok();
        manager2.switch_project(project2.to_string()).ok();

        // Verify project paths are independent
        let path1 = manager1.get_project_path().unwrap();
        let path2 = manager2.get_project_path().unwrap();

        prop_assert_eq!(path1, Some(project1.to_string()));
        prop_assert_eq!(path2, Some(project2.to_string()));

        // Verify files were cleared in both (due to project switch)
        let ctx1_files = manager1.get_files().unwrap();
        let ctx2_files = manager2.get_files().unwrap();

        prop_assert_eq!(ctx1_files.len(), 0);
        prop_assert_eq!(ctx2_files.len(), 0);
    });
}

/// Property: Restoring context from persistence in one manager SHALL NOT
/// affect the context in another manager.
///
/// This property tests that:
/// 1. Restoring context in one manager doesn't affect the other
/// 2. Each manager maintains its own persisted state
/// 3. Persistence operations are isolated
#[test]
fn prop_context_isolation_persistence() {
    proptest!(|(
        provider1 in "openai|anthropic",
        model1 in "gpt-4|claude-3",
        provider2 in "openai|anthropic",
        model2 in "gpt-4|claude-3",
        files1 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..3),
        files2 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..3),
    )| {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        // Create and configure first context
        let mut context1 = SessionContext::new(
            provider1.to_string(),
            model1.to_string(),
            SessionMode::Chat,
        );
        for file in &files1 {
            context1.files.push(file.clone());
        }

        // Create and configure second context
        let mut context2 = SessionContext::new(
            provider2.to_string(),
            model2.to_string(),
            SessionMode::Code,
        );
        for file in &files2 {
            context2.files.push(file.clone());
        }

        // Restore contexts
        manager1.restore_from_persistence(context1.clone());
        manager2.restore_from_persistence(context2.clone());

        // Verify restored contexts are independent
        let restored1 = manager1.get_context().unwrap();
        let restored2 = manager2.get_context().unwrap();

        prop_assert_eq!(restored1.provider, provider1);
        prop_assert_eq!(restored1.model, model1);
        prop_assert_eq!(restored2.provider, provider2);
        prop_assert_eq!(restored2.model, model2);

        // Verify files are isolated
        prop_assert_eq!(restored1.files.len(), files1.len());
        prop_assert_eq!(restored2.files.len(), files2.len());

        for file in &files1 {
            prop_assert!(restored1.files.contains(file));
            prop_assert!(!restored2.files.contains(file));
        }

        for file in &files2 {
            prop_assert!(restored2.files.contains(file));
            prop_assert!(!restored1.files.contains(file));
        }
    });
}

/// Property: Clearing context in one manager SHALL NOT affect the context
/// in another manager.
///
/// This property tests that:
/// 1. Clearing one manager's context doesn't affect the other
/// 2. Each manager can be independently cleared
/// 3. Clearing is an isolated operation
#[test]
fn prop_context_isolation_clearing() {
    proptest!(|(
        files1 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..3),
        files2 in prop::collection::vec("[a-z0-9_]+\\.rs", 1..3),
    )| {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        let context = SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        );

        manager1.set_context(context.clone());
        manager2.set_context(context);

        // Add files
        for file in &files1 {
            manager1.add_file(file.clone()).ok();
        }
        for file in &files2 {
            manager2.add_file(file.clone()).ok();
        }

        // Clear only manager1
        manager1.clear();

        // Verify manager1 is cleared
        prop_assert!(!manager1.is_set());
        prop_assert!(manager1.get_context().is_err());

        // Verify manager2 is unaffected
        prop_assert!(manager2.is_set());
        let ctx2 = manager2.get_context().unwrap();
        prop_assert_eq!(ctx2.files.len(), files2.len());
    });
}
