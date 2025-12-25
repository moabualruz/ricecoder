//! Animation configuration for accessibility
//!
//! Provides controls for animation behavior including speed, reduced motion,
//! and duration calculations for WCAG 2.1 compliance.

use serde::{Deserialize, Serialize};

/// Animation configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Enable animations
    pub enabled: bool,
    /// Animation speed (0.1 to 2.0, where 1.0 is normal)
    pub speed: f32,
    /// Reduce motion for accessibility
    pub reduce_motion: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            speed: 1.0,
            reduce_motion: false,
        }
    }
}

impl AnimationConfig {
    /// Get the effective animation duration in milliseconds
    pub fn duration_ms(&self, base_ms: u32) -> u32 {
        if !self.enabled || self.reduce_motion {
            return 0;
        }
        ((base_ms as f32) / self.speed) as u32
    }

    /// Check if animations should be shown
    pub fn should_animate(&self) -> bool {
        self.enabled && !self.reduce_motion
    }
}
