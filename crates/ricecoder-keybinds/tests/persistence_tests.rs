use ricecoder_keybinds::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Keybind;
    use crate::profile::Profile;

    #[test]
    fn test_save_and_load_profile() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path()).unwrap();

        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let profile = Profile::new("default", keybinds);

        assert!(persistence.save_profile(&profile).is_ok());

        let loaded = persistence.load_profile("default").unwrap();
        assert_eq!(loaded.name, "default");
        assert_eq!(loaded.keybinds.len(), 1);
    }

    #[test]
    fn test_delete_profile() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path()).unwrap();

        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let profile = Profile::new("default", keybinds);

        persistence.save_profile(&profile).unwrap();
        assert!(persistence.delete_profile("default").is_ok());
        assert!(persistence.load_profile("default").is_err());
    }

    #[test]
    fn test_list_profiles() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path()).unwrap();

        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        let profile1 = Profile::new("default", keybinds.clone());
        let profile2 = Profile::new("vim", keybinds);

        persistence.save_profile(&profile1).unwrap();
        persistence.save_profile(&profile2).unwrap();

        let profiles = persistence.list_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"vim".to_string()));
    }

    #[test]
    fn test_save_active_profile() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path()).unwrap();

        assert!(persistence.save_active_profile("default").is_ok());

        let loaded = persistence.load_active_profile().unwrap();
        assert_eq!(loaded, Some("default".to_string()));
    }

    #[test]
    fn test_with_default_location() {
        // This test verifies that with_default_location can find or create the default storage location
        let result = FileSystemPersistence::with_default_location();
        assert!(result.is_ok());

        let persistence = result.unwrap();

        // Verify the config directory exists
        assert!(persistence.config_dir().exists());
    }

    #[test]
    fn test_with_default_location_creates_directory() {
        // This test verifies that with_default_location creates the directory if needed
        let persistence = FileSystemPersistence::with_default_location().unwrap();

        // Verify we can save and load profiles with the default location
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let profile = Profile::new("test_profile", keybinds);

        assert!(persistence.save_profile(&profile).is_ok());
        assert!(persistence.load_profile("test_profile").is_ok());

        // Clean up
        let _ = persistence.delete_profile("test_profile");
    }

    #[test]
    fn test_config_dir_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path()).unwrap();

        assert_eq!(persistence.config_dir(), temp_dir.path());
    }
}
