//! Terminal layout management

/// Layout constraints
#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    /// Percentage of available space
    pub percentage: u16,
}

impl Constraint {
    /// Create a constraint with a percentage
    pub fn percentage(percentage: u16) -> Self {
        Self { percentage }
    }

    /// Create a constraint for fixed size
    pub fn fixed(size: u16) -> Self {
        Self { percentage: size }
    }

    /// Create a constraint for minimum size
    pub fn min(size: u16) -> Self {
        Self { percentage: size }
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
        Self { x, y, width, height }
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

/// Layout manager
pub struct Layout {
    /// Terminal width
    pub width: u16,
    /// Terminal height
    pub height: u16,
}

impl Layout {
    /// Create a new layout
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Check if terminal size is valid (minimum 80x24)
    pub fn is_valid(&self) -> bool {
        self.width >= 80 && self.height >= 24
    }

    /// Get the main content area
    pub fn content_area(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height.saturating_sub(3))
    }

    /// Get the input area (bottom 3 lines)
    pub fn input_area(&self) -> Rect {
        let input_height = 3;
        let y = self.height.saturating_sub(input_height);
        Rect::new(0, y, self.width, input_height)
    }

    /// Split a rect vertically
    pub fn split_vertical(&self, rect: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return vec![rect];
        }

        let mut rects = Vec::new();
        let mut y = rect.y;

        for constraint in constraints {
            let height = (rect.height as u32 * constraint.percentage as u32 / 100) as u16;
            rects.push(Rect::new(rect.x, y, rect.width, height));
            y = y.saturating_add(height);
        }

        rects
    }

    /// Split a rect horizontally
    pub fn split_horizontal(&self, rect: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return vec![rect];
        }

        let mut rects = Vec::new();
        let mut x = rect.x;

        for constraint in constraints {
            let width = (rect.width as u32 * constraint.percentage as u32 / 100) as u16;
            rects.push(Rect::new(x, rect.y, width, rect.height));
            x = x.saturating_add(width);
        }

        rects
    }
}

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
}
