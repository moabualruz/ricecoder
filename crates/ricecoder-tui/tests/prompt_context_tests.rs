use ricecoder_tui::*;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_context_creation() {
        let context = PromptContext::new();
        assert_eq!(context.text, "");
        assert_eq!(context.images.len(), 0);
        assert!(!context.ready);
    }

    #[test]
    fn test_prompt_context_with_text() {
        let context = PromptContext::with_text("Hello, world!");
        assert_eq!(context.text, "Hello, world!");
        assert_eq!(context.images.len(), 0);
    }

    #[test]
    fn test_prompt_context_with_text_and_images() {
        let images = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ];
        let context = PromptContext::with_text_and_images("Hello", images.clone());

        assert_eq!(context.text, "Hello");
        assert_eq!(context.images.len(), 2);
        assert_eq!(context.images, images);
    }

    #[test]
    fn test_set_text() {
        let mut context = PromptContext::new();
        context.set_text("New text");
        assert_eq!(context.get_text(), "New text");
    }

    #[test]
    fn test_add_image() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        context.add_image(path.clone());
        assert_eq!(context.image_count(), 1);
        assert!(context.has_images());
        assert_eq!(context.get_images()[0], path);
    }

    #[test]
    fn test_add_duplicate_image() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        context.add_image(path.clone());
        context.add_image(path.clone());

        // Should not add duplicate
        assert_eq!(context.image_count(), 1);
    }

    #[test]
    fn test_add_multiple_images() {
        let mut context = PromptContext::new();
        let paths = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ];

        context.add_images(paths.clone());
        assert_eq!(context.image_count(), 2);
    }

    #[test]
    fn test_remove_image() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        context.add_image(path.clone());
        assert_eq!(context.image_count(), 1);

        let removed = context.remove_image(&path);
        assert!(removed);
        assert_eq!(context.image_count(), 0);
    }

    #[test]
    fn test_remove_image_not_found() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        let removed = context.remove_image(&path);
        assert!(!removed);
    }

    #[test]
    fn test_clear_images() {
        let mut context = PromptContext::new();
        context.add_images(vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ]);

        assert_eq!(context.image_count(), 2);
        context.clear_images();
        assert_eq!(context.image_count(), 0);
    }

    #[test]
    fn test_has_text() {
        let mut context = PromptContext::new();
        assert!(!context.has_text());

        context.set_text("Some text");
        assert!(context.has_text());
    }

    #[test]
    fn test_has_images() {
        let mut context = PromptContext::new();
        assert!(!context.has_images());

        context.add_image(PathBuf::from("/path/to/image.png"));
        assert!(context.has_images());
    }

    #[test]
    fn test_is_complete() {
        let mut context = PromptContext::new();
        assert!(!context.is_complete());

        context.set_text("Some text");
        assert!(context.is_complete());

        context.clear();
        assert!(!context.is_complete());

        context.add_image(PathBuf::from("/path/to/image.png"));
        assert!(context.is_complete());
    }

    #[test]
    fn test_ready_state() {
        let mut context = PromptContext::new();
        assert!(!context.is_ready());

        context.mark_ready();
        assert!(!context.is_ready()); // Still not ready because context is empty

        context.set_text("Some text");
        assert!(context.is_ready());

        context.mark_not_ready();
        assert!(!context.is_ready());
    }

    #[test]
    fn test_clear() {
        let mut context = PromptContext::new();
        context.set_text("Some text");
        context.add_image(PathBuf::from("/path/to/image.png"));
        context.mark_ready();

        assert!(context.has_text());
        assert!(context.has_images());
        assert!(context.ready);

        context.clear();

        assert!(!context.has_text());
        assert!(!context.has_images());
        assert!(!context.ready);
    }

    #[test]
    fn test_summary() {
        let mut context = PromptContext::new();
        assert_eq!(context.summary(), "Empty context");

        context.set_text("Hello, world!");
        assert!(context.summary().contains("Text"));

        context.add_image(PathBuf::from("/path/to/image.png"));
        let summary = context.summary();
        assert!(summary.contains("Text"));
        assert!(summary.contains("Images"));
    }

    #[test]
    fn test_image_count() {
        let mut context = PromptContext::new();
        assert_eq!(context.image_count(), 0);

        context.add_images(vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
            PathBuf::from("/path/to/image3.gif"),
        ]);

        assert_eq!(context.image_count(), 3);
    }

    #[test]
    fn test_get_images() {
        let mut context = PromptContext::new();
        let paths = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ];

        context.add_images(paths.clone());
        assert_eq!(context.get_images(), paths.as_slice());
    }
}
