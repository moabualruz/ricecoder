//! Spinner animations for RiceCoder TUI
//!
//! Animated spinners for indicating activity:
//! - Multiple spinner styles (dots, blocks, braille)
//! - Color transitions
//! - Agent-colored spinners
//!
//! # DDD Layer: Presentation
//! Animated spinner widgets.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{StatefulWidget, Widget},
};
use std::time::{Duration, Instant};

/// Spinner style presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerStyle {
    #[default]
    Dots,
    Blocks,
    Braille,
    Line,
    Arrow,
    Clock,
}

impl SpinnerStyle {
    /// Get frames for this style
    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            Self::Dots => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Blocks => &["▖", "▘", "▝", "▗"],
            Self::Braille => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            Self::Line => &["-", "\\", "|", "/"],
            Self::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            Self::Clock => &["🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚"],
        }
    }
    
    /// Get default interval for this style
    pub fn interval(&self) -> Duration {
        match self {
            Self::Dots => Duration::from_millis(80),
            Self::Blocks => Duration::from_millis(100),
            Self::Braille => Duration::from_millis(80),
            Self::Line => Duration::from_millis(130),
            Self::Arrow => Duration::from_millis(100),
            Self::Clock => Duration::from_millis(100),
        }
    }
}

/// Generate frames with color variations
pub fn create_frames(color: Color, style: SpinnerStyle, inactive_factor: f32, min_alpha: f32) -> Vec<ColoredFrame> {
    let base_frames = style.frames();
    let len = base_frames.len();
    
    base_frames.iter().enumerate().map(|(i, &frame)| {
        // Calculate brightness based on position (front of wave is brightest)
        let brightness = 1.0 - (i as f32 / len as f32) * inactive_factor;
        let brightness = brightness.max(min_alpha);
        
        ColoredFrame {
            text: frame.to_string(),
            color: adjust_color_brightness(color, brightness),
        }
    }).collect()
}

/// Generate color sequence for frames
pub fn create_colors(color: Color, style: SpinnerStyle, inactive_factor: f32, min_alpha: f32) -> Vec<Color> {
    let len = style.frames().len();
    
    (0..len).map(|i| {
        let brightness = 1.0 - (i as f32 / len as f32) * inactive_factor;
        let brightness = brightness.max(min_alpha);
        adjust_color_brightness(color, brightness)
    }).collect()
}

/// Adjust color brightness
fn adjust_color_brightness(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            Color::Rgb(
                (r as f32 * factor) as u8,
                (g as f32 * factor) as u8,
                (b as f32 * factor) as u8,
            )
        }
        Color::Indexed(idx) => {
            // For indexed colors, just return the original
            // A more sophisticated implementation would convert to RGB first
            Color::Indexed(idx)
        }
        // For named colors, convert to approximate RGB and adjust
        Color::Red => adjust_color_brightness(Color::Rgb(255, 0, 0), factor),
        Color::Green => adjust_color_brightness(Color::Rgb(0, 255, 0), factor),
        Color::Blue => adjust_color_brightness(Color::Rgb(0, 0, 255), factor),
        Color::Yellow => adjust_color_brightness(Color::Rgb(255, 255, 0), factor),
        Color::Magenta => adjust_color_brightness(Color::Rgb(255, 0, 255), factor),
        Color::Cyan => adjust_color_brightness(Color::Rgb(0, 255, 255), factor),
        Color::White => adjust_color_brightness(Color::Rgb(255, 255, 255), factor),
        Color::Gray => adjust_color_brightness(Color::Rgb(128, 128, 128), factor),
        Color::DarkGray => adjust_color_brightness(Color::Rgb(64, 64, 64), factor),
        _ => color,
    }
}

/// A frame with its color
#[derive(Debug, Clone)]
pub struct ColoredFrame {
    pub text: String,
    pub color: Color,
}

/// Spinner state
#[derive(Debug, Clone)]
pub struct SpinnerState {
    /// Current frame index
    frame: usize,
    /// Last update time
    last_update: Instant,
    /// Animation interval
    interval: Duration,
    /// Whether spinning
    spinning: bool,
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinnerState {
    pub fn new() -> Self {
        Self {
            frame: 0,
            last_update: Instant::now(),
            interval: Duration::from_millis(80),
            spinning: true,
        }
    }
    
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
    
    /// Start spinning
    pub fn start(&mut self) {
        self.spinning = true;
        self.last_update = Instant::now();
    }
    
    /// Stop spinning
    pub fn stop(&mut self) {
        self.spinning = false;
    }
    
    /// Check if should advance frame
    pub fn should_advance(&self) -> bool {
        self.spinning && self.last_update.elapsed() >= self.interval
    }
    
    /// Advance to next frame
    pub fn advance(&mut self, frame_count: usize) {
        if self.spinning {
            self.frame = (self.frame + 1) % frame_count;
            self.last_update = Instant::now();
        }
    }
    
    /// Get current frame index
    pub fn frame(&self) -> usize {
        self.frame
    }
    
    /// Tick - advance if needed
    pub fn tick(&mut self, frame_count: usize) -> bool {
        if self.should_advance() {
            self.advance(frame_count);
            true
        } else {
            false
        }
    }
}

/// Spinner widget configuration
#[derive(Debug, Clone)]
pub struct SpinnerConfig {
    pub style: SpinnerStyle,
    pub color: Color,
    pub inactive_factor: f32,
    pub min_alpha: f32,
}

impl Default for SpinnerConfig {
    fn default() -> Self {
        Self {
            style: SpinnerStyle::Dots,
            color: Color::Cyan,
            inactive_factor: 0.6,
            min_alpha: 0.3,
        }
    }
}

/// Spinner widget
pub struct Spinner {
    config: SpinnerConfig,
    frames: Vec<ColoredFrame>,
}

impl Spinner {
    pub fn new(config: SpinnerConfig) -> Self {
        let frames = create_frames(
            config.color,
            config.style,
            config.inactive_factor,
            config.min_alpha,
        );
        Self { config, frames }
    }
    
    pub fn dots(color: Color) -> Self {
        Self::new(SpinnerConfig {
            style: SpinnerStyle::Dots,
            color,
            ..Default::default()
        })
    }
    
    pub fn blocks(color: Color) -> Self {
        Self::new(SpinnerConfig {
            style: SpinnerStyle::Blocks,
            color,
            ..Default::default()
        })
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new(SpinnerConfig::default())
    }
}

impl StatefulWidget for Spinner {
    type State = SpinnerState;
    
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        
        // Tick the state
        state.tick(self.frames.len());
        
        // Get current frame
        if let Some(frame) = self.frames.get(state.frame()) {
            let style = Style::default().fg(frame.color);
            buf.set_string(area.x, area.y, &frame.text, style);
        }
    }
}

/// Simple spinner that doesn't need state (uses time-based frame selection)
pub struct SimpleSpinner {
    frames: &'static [&'static str],
    color: Color,
    interval_ms: u64,
}

impl SimpleSpinner {
    pub fn new(style: SpinnerStyle, color: Color) -> Self {
        Self {
            frames: style.frames(),
            color,
            interval_ms: style.interval().as_millis() as u64,
        }
    }
    
    pub fn dots(color: Color) -> Self {
        Self::new(SpinnerStyle::Dots, color)
    }
}

impl Widget for SimpleSpinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 || self.frames.is_empty() {
            return;
        }
        
        // Time-based frame selection
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let frame_idx = ((now / self.interval_ms) % self.frames.len() as u64) as usize;
        let frame = self.frames[frame_idx];
        
        let style = Style::default().fg(self.color);
        buf.set_string(area.x, area.y, frame, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spinner_style_frames() {
        assert_eq!(SpinnerStyle::Dots.frames().len(), 10);
        assert_eq!(SpinnerStyle::Blocks.frames().len(), 4);
        assert_eq!(SpinnerStyle::Braille.frames().len(), 8);
    }
    
    #[test]
    fn test_spinner_state() {
        let mut state = SpinnerState::new();
        assert_eq!(state.frame(), 0);
        
        state.advance(10);
        assert_eq!(state.frame(), 1);
        
        // Wrap around
        for _ in 0..9 {
            state.advance(10);
        }
        assert_eq!(state.frame(), 0);
    }
    
    #[test]
    fn test_spinner_state_stop() {
        let mut state = SpinnerState::new();
        state.stop();
        
        let old_frame = state.frame();
        state.advance(10);
        assert_eq!(state.frame(), old_frame); // Doesn't advance when stopped
    }
    
    #[test]
    fn test_create_frames() {
        let frames = create_frames(Color::Cyan, SpinnerStyle::Dots, 0.6, 0.3);
        assert_eq!(frames.len(), 10);
    }
    
    #[test]
    fn test_adjust_color_brightness() {
        let color = Color::Rgb(100, 100, 100);
        let dimmed = adjust_color_brightness(color, 0.5);
        
        if let Color::Rgb(r, g, b) = dimmed {
            assert_eq!(r, 50);
            assert_eq!(g, 50);
            assert_eq!(b, 50);
        } else {
            panic!("Expected RGB color");
        }
    }
    
    #[test]
    fn test_spinner_new() {
        let spinner = Spinner::dots(Color::Green);
        assert!(!spinner.frames.is_empty());
    }
}
