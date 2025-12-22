use std::path::PathBuf;

use ricecoder_tui::*;
use tempfile::NamedTempFile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_integration_creation() {
        let integration = ImageIntegration::new();
        assert!(integration.enabled);
        assert_eq!(integration.max_images_per_prompt, 10);
        assert_eq!(integration.current_images.len(), 0);
    }

    #[test]
    fn test_handle_drag_drop_event_single_file() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let (added, errors) = integration.handle_drag_drop_event(vec![path.clone()]);

        assert_eq!(added.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(integration.current_images.len(), 1);
        assert_eq!(integration.current_images[0], path);
    }

    #[test]
    fn test_handle_drag_drop_event_multiple_files() {
        let mut integration = ImageIntegration::new();
        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let path1 = temp_file1.path().to_path_buf();
        let path2 = temp_file2.path().to_path_buf();

        let (added, errors) =
            integration.handle_drag_drop_event(vec![path1.clone(), path2.clone()]);

        assert_eq!(added.len(), 2);
        assert_eq!(errors.len(), 0);
        assert_eq!(integration.current_images.len(), 2);
    }

    #[test]
    fn test_handle_drag_drop_event_nonexistent_file() {
        let mut integration = ImageIntegration::new();
        let path = PathBuf::from("/nonexistent/image.png");

        let (added, errors) = integration.handle_drag_drop_event(vec![path]);

        assert_eq!(added.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("does not exist"));
    }

    #[test]
    fn test_handle_drag_drop_event_directory() {
        let mut integration = ImageIntegration::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let (added, errors) = integration.handle_drag_drop_event(vec![path]);

        assert_eq!(added.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not a file"));
    }

    #[test]
    fn test_handle_drag_drop_event_duplicate() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Add first time
        let (added1, errors1) = integration.handle_drag_drop_event(vec![path.clone()]);
        assert_eq!(added1.len(), 1);
        assert_eq!(errors1.len(), 0);

        // Try to add again
        let (added2, errors2) = integration.handle_drag_drop_event(vec![path]);
        assert_eq!(added2.len(), 0);
        assert_eq!(errors2.len(), 1);
        assert!(errors2[0].contains("already in context"));
    }

    #[test]
    fn test_handle_drag_drop_event_max_images() {
        let mut integration = ImageIntegration::new();
        integration.set_max_images(2);

        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let temp_file3 = NamedTempFile::new().unwrap();

        let path1 = temp_file1.path().to_path_buf();
        let path2 = temp_file2.path().to_path_buf();
        let path3 = temp_file3.path().to_path_buf();

        // Add first two
        let (added1, errors1) = integration.handle_drag_drop_event(vec![path1, path2]);
        assert_eq!(added1.len(), 2);
        assert_eq!(errors1.len(), 0);

        // Try to add third
        let (added2, errors2) = integration.handle_drag_drop_event(vec![path3]);
        assert_eq!(added2.len(), 0);
        assert_eq!(errors2.len(), 1);
        assert!(errors2[0].contains("Maximum number of images"));
    }

    #[test]
    fn test_remove_image() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        integration.handle_drag_drop_event(vec![path.clone()]);
        assert_eq!(integration.current_images.len(), 1);

        let removed = integration.remove_image(&path);
        assert!(removed);
        assert_eq!(integration.current_images.len(), 0);
    }

    #[test]
    fn test_remove_image_not_found() {
        let mut integration = ImageIntegration::new();
        let path = PathBuf::from("/nonexistent/image.png");

        let removed = integration.remove_image(&path);
        assert!(!removed);
    }

    #[test]
    fn test_clear_images() {
        let mut integration = ImageIntegration::new();
        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();

        integration.handle_drag_drop_event(vec![
            temp_file1.path().to_path_buf(),
            temp_file2.path().to_path_buf(),
        ]);
        assert_eq!(integration.current_images.len(), 2);

        integration.clear_images();
        assert_eq!(integration.current_images.len(), 0);
    }

    #[test]
    fn test_get_images() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        integration.handle_drag_drop_event(vec![path.clone()]);

        let images = integration.get_images();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0], path);
    }

    #[test]
    fn test_has_images() {
        let mut integration = ImageIntegration::new();
        assert!(!integration.has_images());

        let temp_file = NamedTempFile::new().unwrap();
        integration.handle_drag_drop_event(vec![temp_file.path().to_path_buf()]);
        assert!(integration.has_images());
    }

    #[test]
    fn test_image_count() {
        let mut integration = ImageIntegration::new();
        assert_eq!(integration.image_count(), 0);

        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();

        integration.handle_drag_drop_event(vec![
            temp_file1.path().to_path_buf(),
            temp_file2.path().to_path_buf(),
        ]);
        assert_eq!(integration.image_count(), 2);
    }

    #[test]
    fn test_enable_disable() {
        let mut integration = ImageIntegration::new();
        assert!(integration.enabled);

        integration.disable();
        assert!(!integration.enabled);

        integration.enable();
        assert!(integration.enabled);
    }

    #[test]
    fn test_set_max_images() {
        let mut integration = ImageIntegration::new();
        assert_eq!(integration.max_images_per_prompt, 10);

        integration.set_max_images(5);
        assert_eq!(integration.max_images_per_prompt, 5);
    }
}
