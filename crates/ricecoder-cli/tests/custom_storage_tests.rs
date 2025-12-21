use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_creation() {
        let storage = CustomCommandsStorage::new();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_load_empty_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CustomCommandsStorage {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        };

        let registry = storage.load_all().unwrap();
        assert_eq!(registry.list_all().len(), 0);
    }

    #[test]
    fn test_save_and_load_command() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CustomCommandsStorage {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        };

        // Create a command
        let cmd = CommandDefinition::new("test-cmd", "Test Command", "echo hello")
            .with_description("A test command");

        // Save it
        let saved_path = storage.save_command(&cmd).unwrap();
        assert!(saved_path.exists());

        // Load it back
        let registry = storage.load_all().unwrap();
        let commands = registry.list_all();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].id, "test-cmd");
    }

    #[test]
    fn test_delete_command() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CustomCommandsStorage {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        };

        // Create and save a command
        let cmd = CommandDefinition::new("test-cmd", "Test Command", "echo hello");
        storage.save_command(&cmd).unwrap();

        // Verify it exists
        let registry = storage.load_all().unwrap();
        assert_eq!(registry.list_all().len(), 1);

        // Delete it
        storage.delete_command("test-cmd").unwrap();

        // Verify it's gone
        let registry = storage.load_all().unwrap();
        assert_eq!(registry.list_all().len(), 0);
    }

    #[test]
    fn test_delete_nonexistent_command() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CustomCommandsStorage {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        };

        let result = storage.delete_command("nonexistent");
        assert!(result.is_err());
    }
}
