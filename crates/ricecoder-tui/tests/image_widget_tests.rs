use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("gif"), Some(ImageFormat::Gif));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("svg"), Some(ImageFormat::Svg));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_image_format_name() {
        assert_eq!(ImageFormat::Png.name(), "PNG");
        assert_eq!(ImageFormat::Jpeg.name(), "JPEG");
        assert_eq!(ImageFormat::Gif.name(), "GIF");
    }

    #[test]
    fn test_image_widget_creation() {
        let capabilities = TerminalCapabilities::detect();
        let widget = ImageWidget::new(&capabilities);
        assert!(!widget.is_loaded());
        assert_eq!(widget.width(), 0);
        assert_eq!(widget.height(), 0);
    }

    #[test]
    fn test_image_widget_set_dimensions() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        widget.set_dimensions(800, 600);

        assert_eq!(widget.width(), 800);
        assert_eq!(widget.height(), 600);
    }

    #[test]
    fn test_image_widget_render_mode() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        // Render mode is now selected based on capabilities
        // Just test that we can set it
        widget.set_render_mode(RenderMode::Sixel);
        assert_eq!(widget.render_mode(), RenderMode::Sixel);
    }

    #[test]
    fn test_image_widget_aspect_ratio() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        assert!(widget.maintain_aspect_ratio());

        widget.set_maintain_aspect_ratio(false);
        assert!(!widget.maintain_aspect_ratio());
    }

    #[test]
    fn test_image_widget_title() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        assert_eq!(widget.title(), "Image");

        widget.set_title("My Image");
        assert_eq!(widget.title(), "My Image");
    }

    #[test]
    fn test_image_widget_borders() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        assert!(widget.show_borders());

        widget.set_show_borders(false);
        assert!(!widget.show_borders());
    }

    #[test]
    fn test_image_widget_load_from_data() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header

        let result = widget.load_from_data(data, ImageFormat::Png);
        assert!(result.is_ok());
        assert!(widget.is_loaded());
        assert_eq!(widget.format(), Some(ImageFormat::Png));
    }

    #[test]
    fn test_image_widget_clear() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        widget.set_dimensions(800, 600);
        let _ = widget.load_from_data(vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png);

        widget.clear();
        assert!(!widget.is_loaded());
        assert_eq!(widget.width(), 0);
        assert_eq!(widget.height(), 0);
    }

    #[test]
    fn test_image_widget_calculate_scaled_dimensions() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        widget.set_dimensions(1600, 1200);

        // Test scaling to fit in 800x600
        let (width, height) = widget.calculate_scaled_dimensions(800, 600);
        assert_eq!(width, 800);
        assert_eq!(height, 600);

        // Test with different aspect ratio
        widget.set_dimensions(800, 1200);
        let (width, height) = widget.calculate_scaled_dimensions(800, 600);
        assert_eq!(width, 400);
        assert_eq!(height, 600);
    }

    #[test]
    fn test_image_widget_display_text() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        assert_eq!(widget.get_display_text(), "[No image loaded]");

        widget.set_dimensions(800, 600);
        let _ = widget.load_from_data(vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png);
        let text = widget.get_display_text();
        assert!(text.contains("PNG"));
        assert!(text.contains("800"));
        assert!(text.contains("600"));
    }

    #[test]
    fn test_image_widget_rendered_output() {
        let capabilities = TerminalCapabilities::detect();
        let mut widget = ImageWidget::new(&capabilities);
        widget.set_dimensions(800, 600);
        let _ = widget.load_from_data(vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png);

        widget.set_render_mode(RenderMode::UnicodeBlocks);
        assert!(widget.get_rendered_output().is_some());

        widget.set_render_mode(RenderMode::Sixel);
        assert!(widget.get_rendered_output().is_some());

        widget.set_render_mode(RenderMode::Ascii);
        assert!(widget.get_rendered_output().is_some());
    }
}
