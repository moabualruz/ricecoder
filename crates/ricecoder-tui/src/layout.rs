//! Terminal layout management with responsive design and graceful degradation

use std::time::Instant;

/// Layout constraints compatible with ratatui
#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Value for the constraint
    pub value: u16,
}

/// Types of layout constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Percentage of available space
    Percentage,
    /// Fixed size in characters
    Fixed,
    /// Minimum size in characters
    Min,
    /// Maximum size in characters
    Max,
    /// Fill remaining space
    Fill,
}

impl Constraint {
    /// Create a constraint with a percentage
    pub fn percentage(percentage: u16) -> Self {
        Self { 
            constraint_type: ConstraintType::Percentage,
            value: percentage.min(100),
        }
    }

    /// Create a constraint for fixed size
    pub fn fixed(size: u16) -> Self {
        Self { 
            constraint_type: ConstraintType::Fixed,
            value: size,
        }
    }

    /// Create a constraint for minimum size
    pub fn min(size: u16) -> Self {
        Self { 
            constraint_type: ConstraintType::Min,
            value: size,
        }
    }

    /// Create a constraint for maximum size
    pub fn max(size: u16) -> Self {
        Self { 
            constraint_type: ConstraintType::Max,
            value: size,
        }
    }

    /// Create a constraint to fill remaining space
    pub fn fill(ratio: u16) -> Self {
        Self { 
            constraint_type: ConstraintType::Fill,
            value: ratio.max(1),
        }
    }
}

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Horizontal layout
    Horizontal,
    /// Vertical layout
    Vertical,
}

/// Rect represents a rectangular area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// X coordinate
    pub x: u16,
    /// Y coordinate
    pub y: u16,
    /// Width
    pub width: u16,
    /// Height
    pub height: u16,
}

impl Rect {
    /// Create a new rect
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the right edge
    pub const fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Get the bottom edge
    pub const fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Check if rect is empty
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Layout manager with enhanced responsive capabilities
pub struct Layout {
    /// Terminal width
    pub width: u16,
    /// Terminal height
    pub height: u16,
    /// Previous layout areas for resize handling
    pub previous_areas: Option<LayoutAreas>,
    /// Last resize timestamp for performance tracking
    pub last_resize: Option<Instant>,
    /// Resize performance metrics
    pub resize_duration_ms: Option<u64>,
}

/// Layout areas for different UI components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutAreas {
    /// Banner area (optional)
    pub banner: Option<Rect>,
    /// Sidebar area (optional)
    pub sidebar: Option<Rect>,
    /// Main chat area
    pub chat: Rect,
    /// Input area
    pub input: Rect,
    /// Status bar area
    pub status: Rect,
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Banner height (0 to disable)
    pub banner_height: u16,
    /// Sidebar width (0 to disable)
    pub sidebar_width: u16,
    /// Whether sidebar is enabled
    pub sidebar_enabled: bool,
    /// Input area height
    pub input_height: u16,
    /// Minimum terminal width
    pub min_width: u16,
    /// Minimum terminal height
    pub min_height: u16,
    /// Minimum chat area width when sidebar is enabled
    pub min_chat_width: u16,
    /// Whether to auto-hide sidebar on narrow terminals
    pub auto_hide_sidebar: bool,
    /// Whether to reduce banner height on short terminals
    pub auto_reduce_banner: bool,
}

/// Layout degradation level for small terminals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationLevel {
    /// Full layout with all areas
    Full,
    /// Hide sidebar (width < 80)
    HideSidebar,
    /// Reduce banner height (height < 30)
    ReduceBanner,
    /// Minimum viable layout (height < 20)
    Minimal,
    /// Terminal too small to be usable
    TooSmall,
}

/// Scroll position adjustment for resize handling
#[derive(Debug, Clone, Copy)]
pub struct ScrollAdjustment {
    /// Height difference (positive = taller, negative = shorter)
    pub height_delta: i32,
    /// Whether to preserve bottom position (auto-scroll behavior)
    pub preserve_bottom: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            banner_height: 7,
            sidebar_width: 25,
            sidebar_enabled: true,
            input_height: 3,
            min_width: 80,
            min_height: 24,
            min_chat_width: 20,
            auto_hide_sidebar: true,
            auto_reduce_banner: true,
        }
    }
}

impl Layout {
    /// Create a new layout
    pub fn new(width: u16, height: u16) -> Self {
        Self { 
            width, 
            height,
            previous_areas: None,
            last_resize: None,
            resize_duration_ms: None,
        }
    }

    /// Update layout dimensions and preserve previous areas for resize handling
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.last_resize = Some(Instant::now());
    }

    /// Get the last resize performance metrics
    pub fn get_resize_performance(&self) -> Option<u64> {
        self.resize_duration_ms
    }

    /// Check if resize performance meets the 16ms requirement
    pub fn meets_resize_performance_requirement(&self) -> bool {
        self.resize_duration_ms.map_or(true, |duration| duration <= 16)
    }

    /// Get the current degradation level based on terminal size
    pub fn degradation_level(&self, _config: &LayoutConfig) -> DegradationLevel {
        if self.width < 40 || self.height < 10 {
            DegradationLevel::TooSmall
        } else if self.height < 20 {
            DegradationLevel::Minimal
        } else if self.height < 30 {
            DegradationLevel::ReduceBanner
        } else if self.width < 80 {
            DegradationLevel::HideSidebar
        } else {
            DegradationLevel::Full
        }
    }

    /// Check if terminal size is valid (minimum 80x24)
    pub fn is_valid(&self) -> bool {
        self.width >= 80 && self.height >= 24
    }

    /// Check if terminal size meets minimum requirements
    pub fn meets_minimum(&self, config: &LayoutConfig) -> bool {
        self.width >= config.min_width && self.height >= config.min_height
    }

    /// Check if terminal is usable (not too small)
    pub fn is_usable(&self) -> bool {
        self.width >= 40 && self.height >= 10
    }

    /// Handle resize event and return updated areas with scroll position preservation
    /// Requirement 2.2: Recalculate layout within 16ms
    pub fn handle_resize(&mut self, new_width: u16, new_height: u16, config: &LayoutConfig) -> (LayoutAreas, Option<ScrollAdjustment>) {
        let start_time = Instant::now();
        let old_areas = self.previous_areas;
        
        // Update dimensions
        self.resize(new_width, new_height);
        
        // Calculate new areas
        let new_areas = self.calculate_areas(config);
        
        // Calculate scroll adjustment if we had previous areas
        let scroll_adjustment = if let Some(old) = old_areas {
            self.calculate_scroll_adjustment(&old, &new_areas)
        } else {
            None
        };

        // Store new areas as previous for next resize
        self.previous_areas = Some(new_areas);

        // Track resize performance (Requirement 2.2: within 16ms)
        let duration = start_time.elapsed();
        self.resize_duration_ms = Some(duration.as_millis() as u64);

        // Log performance warning if resize takes too long
        if duration.as_millis() > 16 {
            eprintln!("Warning: Layout resize took {}ms (requirement: ≤16ms)", duration.as_millis());
        }

        (new_areas, scroll_adjustment)
    }

    /// Calculate scroll position adjustment for resize
    fn calculate_scroll_adjustment(&self, old_areas: &LayoutAreas, new_areas: &LayoutAreas) -> Option<ScrollAdjustment> {
        // Calculate height difference in chat area
        let old_chat_height = old_areas.chat.height;
        let new_chat_height = new_areas.chat.height;
        
        if old_chat_height != new_chat_height {
            Some(ScrollAdjustment {
                height_delta: new_chat_height as i32 - old_chat_height as i32,
                preserve_bottom: true, // Keep scroll at bottom if user was at bottom
            })
        } else {
            None
        }
    }

    /// Get warning message for degraded layout
    pub fn get_degradation_warning(&self, config: &LayoutConfig) -> Option<String> {
        match self.degradation_level(config) {
            DegradationLevel::TooSmall => {
                Some(format!("Terminal too small ({}x{}). Minimum: 40x10", self.width, self.height))
            }
            DegradationLevel::Minimal => {
                Some(format!("Minimal layout active ({}x{}). Recommended: 80x24+", self.width, self.height))
            }
            DegradationLevel::ReduceBanner => {
                Some("Banner height reduced due to small terminal height".to_string())
            }
            DegradationLevel::HideSidebar => {
                Some("Sidebar hidden due to narrow terminal width".to_string())
            }
            DegradationLevel::Full => None,
        }
    }

    /// Calculate layout areas based on configuration
    pub fn calculate_areas(&self, config: &LayoutConfig) -> LayoutAreas {
        let degradation = self.degradation_level(config);
        self.calculate_areas_with_degradation(config, degradation)
    }

    /// Calculate layout areas with specific degradation level
    pub fn calculate_areas_with_degradation(&self, config: &LayoutConfig, degradation: DegradationLevel) -> LayoutAreas {
        let mut current_y = 0;
        let mut current_x = 0;
        let mut remaining_width = self.width;
        let mut remaining_height = self.height;

        // Apply degradation-specific adjustments
        let (banner_height, sidebar_enabled, input_height) = match degradation {
            DegradationLevel::TooSmall => {
                // Minimal layout: no banner, no sidebar, minimal input
                (0, false, 1)
            }
            DegradationLevel::Minimal => {
                // Minimal viable: no banner, no sidebar, small input
                (0, false, 2)
            }
            DegradationLevel::ReduceBanner => {
                // Reduce banner height, keep sidebar if width allows
                let reduced_banner_height = if config.banner_height > 0 {
                    3.min(config.banner_height)
                } else {
                    0
                };
                (reduced_banner_height, config.sidebar_enabled && self.width >= 80, config.input_height)
            }
            DegradationLevel::HideSidebar => {
                // Hide sidebar but keep banner
                (config.banner_height, false, config.input_height)
            }
            DegradationLevel::Full => {
                // Full layout
                (config.banner_height, config.sidebar_enabled, config.input_height)
            }
        };

        // Banner area (top)
        let banner = if banner_height > 0 && remaining_height > banner_height {
            let area = Rect::new(0, current_y, self.width, banner_height);
            current_y += banner_height;
            remaining_height = remaining_height.saturating_sub(banner_height);
            Some(area)
        } else {
            None
        };

        // Status bar area (bottom, reserve 1 line)
        let status_height = 1;
        remaining_height = remaining_height.saturating_sub(status_height);
        let status = Rect::new(
            0,
            current_y + remaining_height,
            self.width,
            status_height,
        );

        // Input area (bottom, above status bar)
        let actual_input_height = input_height.min(remaining_height / 2).max(1);
        remaining_height = remaining_height.saturating_sub(actual_input_height);
        let input = Rect::new(
            0,
            current_y + remaining_height,
            self.width,
            actual_input_height,
        );

        // Sidebar area (left side of remaining area)
        let sidebar = if sidebar_enabled 
            && config.sidebar_width > 0 
            && remaining_width > config.sidebar_width + config.min_chat_width // Ensure minimum chat width
        {
            let area = Rect::new(current_x, current_y, config.sidebar_width, remaining_height);
            current_x += config.sidebar_width;
            remaining_width = remaining_width.saturating_sub(config.sidebar_width);
            Some(area)
        } else {
            None
        };

        // Chat area (remaining space)
        let chat = Rect::new(current_x, current_y, remaining_width, remaining_height);

        LayoutAreas {
            banner,
            sidebar,
            chat,
            input,
            status,
        }
    }

    /// Get the main content area (legacy method)
    pub fn content_area(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height.saturating_sub(3))
    }

    /// Get the input area (legacy method)
    pub fn input_area(&self) -> Rect {
        let input_height = 3;
        let y = self.height.saturating_sub(input_height);
        Rect::new(0, y, self.width, input_height)
    }

    /// Split a rect vertically with enhanced constraint handling
    pub fn split_vertical(&self, rect: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return vec![rect];
        }

        let mut rects = Vec::new();
        let mut y = rect.y;
        let mut remaining_height = rect.height;

        // First pass: handle fixed and minimum constraints
        let mut flexible_constraints = Vec::new();
        let mut total_flexible_ratio = 0u32;

        for (i, constraint) in constraints.iter().enumerate() {
            match constraint.constraint_type {
                ConstraintType::Fixed => {
                    let height = constraint.value.min(remaining_height);
                    rects.push(Rect::new(rect.x, y, rect.width, height));
                    y = y.saturating_add(height);
                    remaining_height = remaining_height.saturating_sub(height);
                }
                ConstraintType::Min => {
                    let height = constraint.value.min(remaining_height);
                    rects.push(Rect::new(rect.x, y, rect.width, height));
                    y = y.saturating_add(height);
                    remaining_height = remaining_height.saturating_sub(height);
                }
                ConstraintType::Percentage => {
                    let height = (rect.height as u32 * constraint.value as u32 / 100) as u16;
                    let height = height.min(remaining_height);
                    rects.push(Rect::new(rect.x, y, rect.width, height));
                    y = y.saturating_add(height);
                    remaining_height = remaining_height.saturating_sub(height);
                }
                ConstraintType::Max => {
                    // Handle max constraints in flexible pass
                    flexible_constraints.push((i, constraint));
                    total_flexible_ratio += constraint.value as u32;
                    rects.push(Rect::new(0, 0, 0, 0)); // Placeholder
                }
                ConstraintType::Fill => {
                    flexible_constraints.push((i, constraint));
                    total_flexible_ratio += constraint.value as u32;
                    rects.push(Rect::new(0, 0, 0, 0)); // Placeholder
                }
            }
        }

        // Second pass: handle flexible constraints
        for (i, constraint) in flexible_constraints {
            let height = if total_flexible_ratio > 0 {
                (remaining_height as u32 * constraint.value as u32 / total_flexible_ratio) as u16
            } else {
                0
            };

            let final_height = match constraint.constraint_type {
                ConstraintType::Max => height.min(constraint.value),
                _ => height,
            };

            rects[i] = Rect::new(rect.x, y, rect.width, final_height);
            y = y.saturating_add(final_height);
        }

        rects
    }

    /// Split a rect horizontally with enhanced constraint handling
    pub fn split_horizontal(&self, rect: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return vec![rect];
        }

        let mut rects = Vec::new();
        let mut x = rect.x;
        let mut remaining_width = rect.width;

        // First pass: handle fixed and minimum constraints
        let mut flexible_constraints = Vec::new();
        let mut total_flexible_ratio = 0u32;

        for (i, constraint) in constraints.iter().enumerate() {
            match constraint.constraint_type {
                ConstraintType::Fixed => {
                    let width = constraint.value.min(remaining_width);
                    rects.push(Rect::new(x, rect.y, width, rect.height));
                    x = x.saturating_add(width);
                    remaining_width = remaining_width.saturating_sub(width);
                }
                ConstraintType::Min => {
                    let width = constraint.value.min(remaining_width);
                    rects.push(Rect::new(x, rect.y, width, rect.height));
                    x = x.saturating_add(width);
                    remaining_width = remaining_width.saturating_sub(width);
                }
                ConstraintType::Percentage => {
                    let width = (rect.width as u32 * constraint.value as u32 / 100) as u16;
                    let width = width.min(remaining_width);
                    rects.push(Rect::new(x, rect.y, width, rect.height));
                    x = x.saturating_add(width);
                    remaining_width = remaining_width.saturating_sub(width);
                }
                ConstraintType::Max => {
                    // Handle max constraints in flexible pass
                    flexible_constraints.push((i, constraint));
                    total_flexible_ratio += constraint.value as u32;
                    rects.push(Rect::new(0, 0, 0, 0)); // Placeholder
                }
                ConstraintType::Fill => {
                    flexible_constraints.push((i, constraint));
                    total_flexible_ratio += constraint.value as u32;
                    rects.push(Rect::new(0, 0, 0, 0)); // Placeholder
                }
            }
        }

        // Second pass: handle flexible constraints
        for (i, constraint) in flexible_constraints {
            let width = if total_flexible_ratio > 0 {
                (remaining_width as u32 * constraint.value as u32 / total_flexible_ratio) as u16
            } else {
                0
            };

            let final_width = match constraint.constraint_type {
                ConstraintType::Max => width.min(constraint.value),
                _ => width,
            };

            rects[i] = Rect::new(x, rect.y, final_width, rect.height);
            x = x.saturating_add(final_width);
        }

        rects
    }

    /// Validate that layout areas don't overlap and fit within terminal bounds
    pub fn validate_areas(&self, areas: &LayoutAreas) -> Result<(), String> {
        let terminal_rect = Rect::new(0, 0, self.width, self.height);

        // Check each area fits within terminal bounds
        if let Some(banner) = areas.banner {
            if !self.rect_fits_within(&banner, &terminal_rect) {
                return Err(format!("Banner area {:?} exceeds terminal bounds {:?}", banner, terminal_rect));
            }
        }

        if let Some(sidebar) = areas.sidebar {
            if !self.rect_fits_within(&sidebar, &terminal_rect) {
                return Err(format!("Sidebar area {:?} exceeds terminal bounds {:?}", sidebar, terminal_rect));
            }
        }

        if !self.rect_fits_within(&areas.chat, &terminal_rect) {
            return Err(format!("Chat area {:?} exceeds terminal bounds {:?}", areas.chat, terminal_rect));
        }

        if !self.rect_fits_within(&areas.input, &terminal_rect) {
            return Err(format!("Input area {:?} exceeds terminal bounds {:?}", areas.input, terminal_rect));
        }

        if !self.rect_fits_within(&areas.status, &terminal_rect) {
            return Err(format!("Status area {:?} exceeds terminal bounds {:?}", areas.status, terminal_rect));
        }

        // Check for overlaps between areas
        let mut all_areas = Vec::new();
        if let Some(banner) = areas.banner {
            all_areas.push(("banner", banner));
        }
        if let Some(sidebar) = areas.sidebar {
            all_areas.push(("sidebar", sidebar));
        }
        all_areas.push(("chat", areas.chat));
        all_areas.push(("input", areas.input));
        all_areas.push(("status", areas.status));

        for i in 0..all_areas.len() {
            for j in (i + 1)..all_areas.len() {
                let (name1, rect1) = all_areas[i];
                let (name2, rect2) = all_areas[j];
                if self.rects_overlap(&rect1, &rect2) {
                    return Err(format!("Areas {} and {} overlap: {:?} and {:?}", name1, name2, rect1, rect2));
                }
            }
        }

        Ok(())
    }

    /// Check if a rect fits within another rect
    fn rect_fits_within(&self, inner: &Rect, outer: &Rect) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.right() <= outer.right()
            && inner.bottom() <= outer.bottom()
    }

    /// Check if two rects overlap
    fn rects_overlap(&self, rect1: &Rect, rect2: &Rect) -> bool {
        !(rect1.right() <= rect2.x
            || rect2.right() <= rect1.x
            || rect1.bottom() <= rect2.y
            || rect2.bottom() <= rect1.y)
    }
}

