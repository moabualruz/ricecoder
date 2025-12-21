//! Regression detection utilities

use crate::baseline::PerformanceBaseline;
use crate::monitor::PerformanceMetrics;
use crate::regression::{RegressionAlert, RegressionConfig, RegressionDetector};

/// Simple regression detector wrapper
pub struct PerformanceRegressionDetector {
    detector: RegressionDetector,
}

impl PerformanceRegressionDetector {
    /// Create a new detector
    pub fn new(baseline: PerformanceBaseline) -> Self {
        let config = RegressionConfig::default();
        Self {
            detector: RegressionDetector::new(baseline, config),
        }
    }

    /// Detect regressions
    pub fn detect(&mut self, metrics: &[PerformanceMetrics]) -> Vec<RegressionAlert> {
        self.detector.detect_regressions(metrics)
    }

    /// Update baseline
    pub fn update_baseline(&mut self, metrics: &[PerformanceMetrics]) {
        self.detector.update_baseline(metrics)
    }
}
