use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(0, 0, 80, 24);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 80);
        assert_eq!(rect.height, 24);
    }

    #[test]
    fn test_rect_edges() {
        let rect = Rect::new(10, 5, 20, 15);
        assert_eq!(rect.right(), 30);
        assert_eq!(rect.bottom(), 20);
    }

    #[test]
    fn test_layout_valid() {
        let layout = Layout::new(80, 24);
        assert!(layout.is_valid());

        let layout = Layout::new(79, 24);
        assert!(!layout.is_valid());

        let layout = Layout::new(80, 23);
        assert!(!layout.is_valid());
    }

    #[test]
    fn test_layout_areas() {
        let layout = Layout::new(80, 24);
        let content = layout.content_area();
        assert_eq!(content.height, 21);

        let input = layout.input_area();
        assert_eq!(input.height, 3);
        assert_eq!(input.y, 21);
    }

    #[test]
    fn test_split_vertical() {
        let layout = Layout::new(80, 24);
        let rect = Rect::new(0, 0, 80, 20);
        let constraints = vec![Constraint::percentage(50), Constraint::percentage(50)];
        let rects = layout.split_vertical(rect, &constraints);

        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 10);
        assert_eq!(rects[1].height, 10);
    }

    #[test]
    fn test_split_horizontal() {
        let layout = Layout::new(80, 24);
        let rect = Rect::new(0, 0, 80, 20);
        let constraints = vec![Constraint::percentage(30), Constraint::percentage(70)];
        let rects = layout.split_horizontal(rect, &constraints);

        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 24);
        assert_eq!(rects[1].width, 56);
    }

    #[test]
    fn test_constraint_types() {
        // Test different constraint types
        let fixed = Constraint::fixed(10);
        assert_eq!(fixed.constraint_type, ConstraintType::Fixed);
        assert_eq!(fixed.value, 10);

        let percentage = Constraint::percentage(50);
        assert_eq!(percentage.constraint_type, ConstraintType::Percentage);
        assert_eq!(percentage.value, 50);

        let min = Constraint::min(5);
        assert_eq!(min.constraint_type, ConstraintType::Min);
        assert_eq!(min.value, 5);

        let max = Constraint::max(20);
        assert_eq!(max.constraint_type, ConstraintType::Max);
        assert_eq!(max.value, 20);

        let fill = Constraint::fill(2);
        assert_eq!(fill.constraint_type, ConstraintType::Fill);
        assert_eq!(fill.value, 2);
    }

    #[test]
    fn test_enhanced_split_vertical() {
        let layout = Layout::new(80, 24);
        let rect = Rect::new(0, 0, 80, 20);

        // Test mixed constraints
        let constraints = vec![
            Constraint::fixed(5),
            Constraint::fill(1),
            Constraint::fixed(3),
        ];
        let rects = layout.split_vertical(rect, &constraints);

        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].height, 5); // Fixed
        assert_eq!(rects[2].height, 3); // Fixed
        assert_eq!(rects[1].height, 12); // Fill (20 - 5 - 3)
    }

    #[test]
    fn test_enhanced_split_horizontal() {
        let layout = Layout::new(80, 24);
        let rect = Rect::new(0, 0, 80, 20);

        // Test mixed constraints
        let constraints = vec![
            Constraint::fixed(20),
            Constraint::fill(1),
            Constraint::min(10),
        ];
        let rects = layout.split_horizontal(rect, &constraints);

        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].width, 20); // Fixed
        assert_eq!(rects[2].width, 10); // Min
        assert_eq!(rects[1].width, 50); // Fill (80 - 20 - 10)
    }

    #[test]
    fn test_layout_config_default() {
        let config = LayoutConfig::default();
        assert_eq!(config.banner_height, 7);
        assert_eq!(config.sidebar_width, 25);
        assert!(config.sidebar_enabled);
        assert_eq!(config.input_height, 3);
        assert_eq!(config.min_width, 80);
        assert_eq!(config.min_height, 24);
        assert_eq!(config.min_chat_width, 20);
        assert!(config.auto_hide_sidebar);
        assert!(config.auto_reduce_banner);
    }

    #[test]
    fn test_calculate_areas_full_layout() {
        let layout = Layout::new(100, 30);
        let config = LayoutConfig::default();
        let areas = layout.calculate_areas(&config);

        // Banner should be present
        assert!(areas.banner.is_some());
        let banner = areas.banner.unwrap();
        assert_eq!(banner.height, 7);
        assert_eq!(banner.width, 100);

        // Sidebar should be present
        assert!(areas.sidebar.is_some());
        let sidebar = areas.sidebar.unwrap();
        assert_eq!(sidebar.width, 25);

        // Chat area should use remaining space
        assert_eq!(areas.chat.x, 25); // After sidebar
        assert_eq!(areas.chat.y, 7); // After banner
        assert_eq!(areas.chat.width, 75); // Remaining width

        // Input and status should be at bottom
        assert!(areas.input.y > areas.chat.y);
        assert!(areas.status.y > areas.input.y);
    }

    #[test]
    fn test_calculate_areas_no_banner() {
        let layout = Layout::new(100, 30);
        let config = LayoutConfig {
            banner_height: 0,
            ..Default::default()
        };
        let areas = layout.calculate_areas(&config);

        // Banner should not be present
        assert!(areas.banner.is_none());

        // Chat area should start at top
        assert_eq!(areas.chat.y, 0);
    }

    #[test]
    fn test_calculate_areas_no_sidebar() {
        let layout = Layout::new(100, 30);
        let config = LayoutConfig {
            sidebar_enabled: false,
            ..Default::default()
        };
        let areas = layout.calculate_areas(&config);

        // Sidebar should not be present
        assert!(areas.sidebar.is_none());

        // Chat area should start at left edge
        assert_eq!(areas.chat.x, 0);
        assert_eq!(areas.chat.width, 100);
    }

    #[test]
    fn test_calculate_areas_small_terminal() {
        let layout = Layout::new(80, 24);
        let config = LayoutConfig::default();
        let areas = layout.calculate_areas(&config);

        // All areas should fit within terminal bounds
        if let Some(banner) = areas.banner {
            assert!(banner.right() <= 80);
            assert!(banner.bottom() <= 24);
        }
        if let Some(sidebar) = areas.sidebar {
            assert!(sidebar.right() <= 80);
            assert!(sidebar.bottom() <= 24);
        }
        assert!(areas.chat.right() <= 80);
        assert!(areas.chat.bottom() <= 24);
        assert!(areas.input.right() <= 80);
        assert!(areas.input.bottom() <= 24);
        assert!(areas.status.right() <= 80);
        assert!(areas.status.bottom() <= 24);
    }

    #[test]
    fn test_meets_minimum_requirements() {
        let layout = Layout::new(80, 24);
        let config = LayoutConfig::default();
        assert!(layout.meets_minimum(&config));

        let small_layout = Layout::new(70, 20);
        assert!(!small_layout.meets_minimum(&config));
    }

    #[test]
    fn test_degradation_levels() {
        let config = LayoutConfig::default();

        // Full layout
        let layout = Layout::new(100, 40);
        assert_eq!(layout.degradation_level(&config), DegradationLevel::Full);

        // Hide sidebar (narrow width)
        let layout = Layout::new(70, 40);
        assert_eq!(
            layout.degradation_level(&config),
            DegradationLevel::HideSidebar
        );

        // Reduce banner (short height)
        let layout = Layout::new(100, 25);
        assert_eq!(
            layout.degradation_level(&config),
            DegradationLevel::ReduceBanner
        );

        // Minimal layout
        let layout = Layout::new(60, 15);
        assert_eq!(layout.degradation_level(&config), DegradationLevel::Minimal);

        // Too small
        let layout = Layout::new(30, 8);
        assert_eq!(
            layout.degradation_level(&config),
            DegradationLevel::TooSmall
        );
    }

    #[test]
    fn test_graceful_degradation() {
        let config = LayoutConfig::default();

        // Test sidebar hiding on narrow terminals
        let layout = Layout::new(70, 30);
        let areas = layout.calculate_areas(&config);
        assert!(areas.sidebar.is_none()); // Sidebar should be hidden
        assert_eq!(areas.chat.x, 0); // Chat should start at left edge

        // Test banner reduction on short terminals
        let layout = Layout::new(100, 25);
        let areas = layout.calculate_areas(&config);
        if let Some(banner) = areas.banner {
            assert_eq!(banner.height, 3); // Reduced banner height
        }

        // Test minimal layout
        let layout = Layout::new(50, 15);
        let areas = layout.calculate_areas(&config);
        assert!(areas.banner.is_none()); // No banner
        assert!(areas.sidebar.is_none()); // No sidebar
        assert_eq!(areas.input.height, 2); // Minimal input height
    }

    #[test]
    fn test_resize_handling() {
        let mut layout = Layout::new(80, 24);
        let config = LayoutConfig::default();

        // Initial calculation and store as previous areas
        let initial_areas = layout.calculate_areas(&config);
        layout.previous_areas = Some(initial_areas);

        // Resize to larger
        let (new_areas, scroll_adjustment) = layout.handle_resize(120, 40, &config);

        // Should have scroll adjustment due to height change
        assert!(scroll_adjustment.is_some());
        if let Some(adj) = scroll_adjustment {
            assert!(adj.height_delta > 0); // Taller
            assert!(adj.preserve_bottom);
        }

        // New areas should be larger
        assert!(new_areas.chat.width > initial_areas.chat.width);
        assert!(new_areas.chat.height > initial_areas.chat.height);
    }

    #[test]
    fn test_is_usable() {
        // Usable terminals
        assert!(Layout::new(80, 24).is_usable());
        assert!(Layout::new(40, 10).is_usable());

        // Unusable terminals
        assert!(!Layout::new(30, 8).is_usable());
        assert!(!Layout::new(39, 10).is_usable());
        assert!(!Layout::new(40, 9).is_usable());
    }

    #[test]
    fn test_degradation_warnings() {
        let config = LayoutConfig::default();

        // No warning for good size
        let layout = Layout::new(100, 40);
        assert!(layout.get_degradation_warning(&config).is_none());

        // Warning for too small
        let layout = Layout::new(30, 8);
        let warning = layout.get_degradation_warning(&config);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("too small"));

        // Warning for minimal
        let layout = Layout::new(50, 15);
        let warning = layout.get_degradation_warning(&config);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("Minimal layout"));
    }

    #[test]
    fn test_minimum_chat_width_enforcement() {
        let config = LayoutConfig {
            sidebar_width: 30,
            min_chat_width: 25,
            ..Default::default()
        };

        // Terminal too narrow for sidebar + minimum chat width
        let layout = Layout::new(50, 30); // 50 < 30 + 25
        let areas = layout.calculate_areas(&config);

        // Sidebar should be hidden to preserve minimum chat width
        assert!(areas.sidebar.is_none());
        assert_eq!(areas.chat.width, 50);
    }

    #[test]
    fn test_layout_validation() {
        let layout = Layout::new(80, 24);
        let config = LayoutConfig::default();
        let areas = layout.calculate_areas(&config);

        // Valid areas should pass validation
        assert!(layout.validate_areas(&areas).is_ok());

        // Test invalid area (exceeds bounds)
        let invalid_areas = LayoutAreas {
            banner: Some(Rect::new(0, 0, 100, 10)), // Width exceeds terminal
            sidebar: areas.sidebar,
            chat: areas.chat,
            input: areas.input,
            status: areas.status,
        };
        assert!(layout.validate_areas(&invalid_areas).is_err());
    }

    #[test]
    fn test_resize_performance_tracking() {
        let mut layout = Layout::new(80, 24);
        let config = LayoutConfig::default();

        // Initial state should have no performance data
        assert!(layout.get_resize_performance().is_none());

        // Perform resize
        let (_areas, _scroll) = layout.handle_resize(100, 30, &config);

        // Should now have performance data
        assert!(layout.get_resize_performance().is_some());

        // Performance should meet requirement (≤16ms)
        assert!(layout.meets_resize_performance_requirement());
    }

    #[test]
    fn test_rect_overlap_detection() {
        let layout = Layout::new(80, 24);

        // Non-overlapping rects
        let rect1 = Rect::new(0, 0, 10, 10);
        let rect2 = Rect::new(10, 0, 10, 10);
        assert!(!layout.rects_overlap(&rect1, &rect2));

        // Overlapping rects
        let rect3 = Rect::new(5, 5, 10, 10);
        assert!(layout.rects_overlap(&rect1, &rect3));

        // Adjacent rects (should not overlap)
        let rect4 = Rect::new(0, 10, 10, 10);
        assert!(!layout.rects_overlap(&rect1, &rect4));
    }

    #[test]
    fn test_rect_fits_within() {
        let layout = Layout::new(80, 24);
        let outer = Rect::new(0, 0, 80, 24);

        // Rect that fits
        let inner1 = Rect::new(10, 10, 20, 10);
        assert!(layout.rect_fits_within(&inner1, &outer));

        // Rect that doesn't fit (exceeds width)
        let inner2 = Rect::new(70, 10, 20, 10);
        assert!(!layout.rect_fits_within(&inner2, &outer));

        // Rect that doesn't fit (exceeds height)
        let inner3 = Rect::new(10, 20, 20, 10);
        assert!(!layout.rect_fits_within(&inner3, &outer));

        // Rect that exactly fits
        let inner4 = Rect::new(0, 0, 80, 24);
        assert!(layout.rect_fits_within(&inner4, &outer));
    }

    #[test]
    fn test_constraint_percentage_clamping() {
        // Percentage should be clamped to 100
        let constraint = Constraint::percentage(150);
        assert_eq!(constraint.value, 100);

        let constraint = Constraint::percentage(50);
        assert_eq!(constraint.value, 50);
    }

    #[test]
    fn test_fill_constraint_minimum() {
        // Fill ratio should be at least 1
        let constraint = Constraint::fill(0);
        assert_eq!(constraint.value, 1);

        let constraint = Constraint::fill(3);
        assert_eq!(constraint.value, 3);
    }
}
