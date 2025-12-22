//! Context management for sessions

use crate::{
    error::{SessionError, SessionResult},
    models::SessionContext,
};

/// Manages session context with isolation between sessions
///
/// Each ContextManager instance maintains its own isolated context.
/// Modifications to one context do not affect other contexts.
#[derive(Debug, Clone)]
pub struct ContextManager {
    /// The current session context
    context: Option<SessionContext>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self { context: None }
    }

    /// Create a context manager with an initial context
    pub fn with_context(context: SessionContext) -> Self {
        Self {
            context: Some(context),
        }
    }

    /// Set the session context
    ///
    /// This replaces the entire context. Each ContextManager maintains
    /// its own isolated context, so changes here do not affect other
    /// ContextManager instances.
    pub fn set_context(&mut self, context: SessionContext) {
        self.context = Some(context);
    }

    /// Get the current context
    ///
    /// Returns a clone of the context, ensuring isolation between
    /// different parts of the system.
    pub fn get_context(&self) -> SessionResult<SessionContext> {
        self.context
            .clone()
            .ok_or_else(|| SessionError::Invalid("No context set".to_string()))
    }

    /// Get a mutable reference to the context for modifications
    ///
    /// This is used internally for operations that need to modify the context.
    fn get_context_mut(&mut self) -> SessionResult<&mut SessionContext> {
        self.context
            .as_mut()
            .ok_or_else(|| SessionError::Invalid("No context set".to_string()))
    }

    /// Add a file to the context
    ///
    /// Files are stored as paths in the context. Adding a file to one
    /// context does not affect other contexts.
    pub fn add_file(&mut self, file_path: String) -> SessionResult<()> {
        let context = self.get_context_mut()?;

        // Avoid duplicates
        if !context.files.contains(&file_path) {
            context.files.push(file_path);
        }

        Ok(())
    }

    /// Remove a file from the context
    ///
    /// Removes the specified file path from the context. If the file
    /// is not in the context, this is a no-op.
    pub fn remove_file(&mut self, file_path: &str) -> SessionResult<()> {
        let context = self.get_context_mut()?;
        context.files.retain(|f| f != file_path);
        Ok(())
    }

    /// Get all files in the context
    pub fn get_files(&self) -> SessionResult<Vec<String>> {
        self.get_context().map(|ctx| ctx.files)
    }

    /// Clear all files from the context
    pub fn clear_files(&mut self) -> SessionResult<()> {
        let context = self.get_context_mut()?;
        context.files.clear();
        Ok(())
    }

    /// Set the project path in the context
    pub fn set_project_path(&mut self, path: Option<String>) -> SessionResult<()> {
        let context = self.get_context_mut()?;
        context.project_path = path;
        Ok(())
    }

    /// Get the project path from the context
    pub fn get_project_path(&self) -> SessionResult<Option<String>> {
        self.get_context().map(|ctx| ctx.project_path)
    }

    /// Check if a file is in the context
    pub fn has_file(&self, file_path: &str) -> SessionResult<bool> {
        self.get_context()
            .map(|ctx| ctx.files.contains(&file_path.to_string()))
    }

    /// Check if context is set
    pub fn is_set(&self) -> bool {
        self.context.is_some()
    }

    /// Clear the entire context
    pub fn clear(&mut self) {
        self.context = None;
    }

    /// Switch to a different project
    ///
    /// This updates the project path in the context and clears the file list
    /// to reflect the new project context.
    pub fn switch_project(&mut self, project_path: String) -> SessionResult<()> {
        let context = self.get_context_mut()?;
        context.project_path = Some(project_path);
        // Clear files when switching projects
        context.files.clear();
        Ok(())
    }

    /// Get the context for persistence
    ///
    /// Returns the current context for saving to disk. This is used by
    /// the SessionStore to persist context with the session.
    pub fn get_context_for_persistence(&self) -> SessionResult<SessionContext> {
        self.get_context()
    }

    /// Restore context from persistence
    ///
    /// Restores the context from a previously saved state. This is used
    /// by the SessionStore when loading a session from disk.
    pub fn restore_from_persistence(&mut self, context: SessionContext) {
        self.context = Some(context);
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionMode;

    fn create_test_context() -> SessionContext {
        SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
    }

    #[test]
    fn test_context_manager_new() {
        let manager = ContextManager::new();
        assert!(!manager.is_set());
    }

    #[test]
    fn test_set_and_get_context() {
        let mut manager = ContextManager::new();
        let context = create_test_context();

        manager.set_context(context.clone());

        let retrieved = manager.get_context().unwrap();
        assert_eq!(retrieved.provider, context.provider);
        assert_eq!(retrieved.model, context.model);
    }

    #[test]
    fn test_add_file() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager.add_file("file1.rs".to_string()).unwrap();
        manager.add_file("file2.rs".to_string()).unwrap();

        let files = manager.get_files().unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.rs".to_string()));
        assert!(files.contains(&"file2.rs".to_string()));
    }

    #[test]
    fn test_add_duplicate_file() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager.add_file("file1.rs".to_string()).unwrap();
        manager.add_file("file1.rs".to_string()).unwrap();

        let files = manager.get_files().unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_remove_file() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager.add_file("file1.rs".to_string()).unwrap();
        manager.add_file("file2.rs".to_string()).unwrap();

        manager.remove_file("file1.rs").unwrap();

        let files = manager.get_files().unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.contains(&"file2.rs".to_string()));
    }

    #[test]
    fn test_context_isolation() {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        manager1.set_context(create_test_context());
        manager2.set_context(create_test_context());

        manager1.add_file("file1.rs".to_string()).unwrap();
        manager2.add_file("file2.rs".to_string()).unwrap();

        let files1 = manager1.get_files().unwrap();
        let files2 = manager2.get_files().unwrap();

        assert_eq!(files1.len(), 1);
        assert_eq!(files2.len(), 1);
        assert!(files1.contains(&"file1.rs".to_string()));
        assert!(files2.contains(&"file2.rs".to_string()));
    }

    #[test]
    fn test_clear_files() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager.add_file("file1.rs".to_string()).unwrap();
        manager.add_file("file2.rs".to_string()).unwrap();

        manager.clear_files().unwrap();

        let files = manager.get_files().unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_project_path() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager
            .set_project_path(Some("/path/to/project".to_string()))
            .unwrap();

        let path = manager.get_project_path().unwrap();
        assert_eq!(path, Some("/path/to/project".to_string()));
    }

    #[test]
    fn test_has_file() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager.add_file("file1.rs".to_string()).unwrap();

        assert!(manager.has_file("file1.rs").unwrap());
        assert!(!manager.has_file("file2.rs").unwrap());
    }

    #[test]
    fn test_clear_context() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        assert!(manager.is_set());

        manager.clear();

        assert!(!manager.is_set());
        assert!(manager.get_context().is_err());
    }

    #[test]
    fn test_operations_without_context() {
        let mut manager = ContextManager::new();

        assert!(manager.get_context().is_err());
        assert!(manager.add_file("file.rs".to_string()).is_err());
        assert!(manager.remove_file("file.rs").is_err());
    }

    #[test]
    fn test_switch_project() {
        let mut manager = ContextManager::new();
        manager.set_context(create_test_context());

        manager.add_file("file1.rs".to_string()).unwrap();
        manager.add_file("file2.rs".to_string()).unwrap();

        manager.switch_project("/new/project".to_string()).unwrap();

        let path = manager.get_project_path().unwrap();
        assert_eq!(path, Some("/new/project".to_string()));

        // Files should be cleared when switching projects
        let files = manager.get_files().unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let mut manager1 = ContextManager::new();
        let context = create_test_context();
        manager1.set_context(context);

        manager1.add_file("file1.rs".to_string()).unwrap();
        manager1.add_file("file2.rs".to_string()).unwrap();
        manager1
            .set_project_path(Some("/project".to_string()))
            .unwrap();

        // Get context for persistence
        let persisted_context = manager1.get_context_for_persistence().unwrap();

        // Create new manager and restore
        let mut manager2 = ContextManager::new();
        manager2.restore_from_persistence(persisted_context);

        // Verify context is restored
        let restored_context = manager2.get_context().unwrap();
        assert_eq!(restored_context.provider, "openai");
        assert_eq!(restored_context.model, "gpt-4");
        assert_eq!(restored_context.project_path, Some("/project".to_string()));

        let files = manager2.get_files().unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.rs".to_string()));
        assert!(files.contains(&"file2.rs".to_string()));
    }

    #[test]
    fn test_context_isolation_with_operations() {
        let mut manager1 = ContextManager::new();
        let mut manager2 = ContextManager::new();

        manager1.set_context(create_test_context());
        manager2.set_context(create_test_context());

        // Perform different operations on each manager
        manager1.add_file("file1.rs".to_string()).unwrap();
        manager1
            .set_project_path(Some("/project1".to_string()))
            .unwrap();

        manager2.add_file("file2.rs".to_string()).unwrap();
        manager2
            .set_project_path(Some("/project2".to_string()))
            .unwrap();

        // Verify isolation
        let context1 = manager1.get_context().unwrap();
        let context2 = manager2.get_context().unwrap();

        assert_eq!(context1.project_path, Some("/project1".to_string()));
        assert_eq!(context2.project_path, Some("/project2".to_string()));

        assert_eq!(context1.files.len(), 1);
        assert_eq!(context2.files.len(), 1);
        assert!(context1.files.contains(&"file1.rs".to_string()));
        assert!(context2.files.contains(&"file2.rs".to_string()));
    }
}
