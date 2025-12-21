use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_component_creation() {
        let config = BannerComponentConfig::default();
        let component = BannerComponent::new(config);
        assert!(component.is_enabled());
        assert_eq!(component.height(), 7);
    }

    #[test]
    fn test_banner_component_disabled() {
        let config = BannerComponentConfig {
            enabled: false,
            ..Default::default()
        };
        let component = BannerComponent::new(config);
        assert!(!component.is_enabled());
        assert_eq!(component.height(), 0);
    }

    #[test]
    fn test_banner_component_config_update() {
        let config = BannerComponentConfig::default();
        let mut component = BannerComponent::new(config);

        let new_config = BannerComponentConfig {
            enabled: false,
            height: 10,
            fallback_text: "Test".to_string(),
            ..Default::default()
        };

        component.update_config(new_config);
        assert!(!component.is_enabled());
        assert_eq!(component.height(), 0);
    }

    #[test]
    fn test_banner_component_theme_colors() {
        let config = BannerComponentConfig::default();
        let mut component = BannerComponent::new(config);

        let colors = ImageThemeColors {
            primary: (255, 0, 0),
            secondary: (0, 255, 0),
            accent: (0, 0, 255),
        };

        component.set_theme_colors(colors);
        // Should clear cache when theme changes
        assert!(component.cached_output.is_none());
    }

    #[test]
    fn test_banner_component_render_disabled() {
        let config = BannerComponentConfig {
            enabled: false,
            ..Default::default()
        };
        let mut component = BannerComponent::new(config);

        let output = component.render_banner();
        assert!(output.is_empty());
    }

    #[test]
    fn test_banner_component_render_fallback() {
        let config = BannerComponentConfig {
            enabled: true,
            fallback_text: "TestBanner".to_string(),
            ..Default::default()
        };
        let mut component = BannerComponent::new(config);

        let output = component.render_banner();
        assert!(!output.is_empty());
        // Should contain fallback text since no SVG is provided
        assert!(output.contains("TestBanner") || output.contains("==="));
    }

    #[test]
    fn test_banner_area_calculation() {
        let config = BannerComponentConfig {
            enabled: true,
            height: 5,
            ..Default::default()
        };
        let banner_area = BannerArea::new(config);

        let terminal_area = LayoutRect::new(0, 0, 80, 24);
        let calculated = banner_area.calculate_area(terminal_area);

        assert!(calculated.is_some());
        let area = calculated.unwrap();
        assert_eq!(area.height, 5);
        assert_eq!(area.width, 80);
    }

    #[test]
    fn test_banner_area_disabled() {
        let config = BannerComponentConfig {
            enabled: false,
            ..Default::default()
        };
        let banner_area = BannerArea::new(config);

        let terminal_area = LayoutRect::new(0, 0, 80, 24);
        let calculated = banner_area.calculate_area(terminal_area);

        assert!(calculated.is_none());
    }

    #[test]
    fn test_banner_area_remaining() {
        let config = BannerComponentConfig {
            enabled: true,
            height: 5,
            ..Default::default()
        };
        let banner_area = BannerArea::new(config);

        let terminal_area = LayoutRect::new(0, 0, 80, 24);
        let remaining = banner_area.remaining_area(terminal_area);

        assert_eq!(remaining.y, 5);
        assert_eq!(remaining.height, 19);
        assert_eq!(remaining.width, 80);
    }

    #[test]
    fn test_banner_area_remaining_disabled() {
        let config = BannerComponentConfig {
            enabled: false,
            ..Default::default()
        };
        let banner_area = BannerArea::new(config);

        let terminal_area = LayoutRect::new(0, 0, 80, 24);
        let remaining = banner_area.remaining_area(terminal_area);

        // Should return the full terminal area when banner is disabled
        assert_eq!(remaining, terminal_area);
    }

    #[test]
    fn test_terminal_capabilities_detection() {
        let config = BannerComponentConfig::default();
        let component = BannerComponent::new(config);
        let caps = component.detect_terminal_capabilities();

        // Should use conservative defaults
        assert!(!caps.supports_sixel);
        assert!(caps.supports_unicode);
        assert!(caps.supports_color);
    }
}
