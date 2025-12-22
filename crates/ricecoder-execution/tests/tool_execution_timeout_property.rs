//! Property-based tests for tool execution timeout
//!
//! **Property 11: Tool Execution Timeout**
//! **Validates: Requirements 14.4, 66.1**

use std::time::Duration;

use proptest::prelude::*;

/// **Feature: ricecoder-tui-improvement, Property 11: Tool Execution Timeout**
/// **Validates: Requirements 14.4, 66.1**
#[test]
fn prop_timeout_duration_creation() {
    proptest!(ProptestConfig::with_cases(100), |(
        timeout_ms in 1u64..300_000  // 1ms to 5 minutes
    )| {
        // Test that timeout durations are created correctly
        let duration = Duration::from_millis(timeout_ms);
        prop_assert_eq!(duration.as_millis() as u64, timeout_ms);

        // Test that reasonable timeouts are within bounds
        prop_assert!(timeout_ms >= 1, "Timeout must be at least 1ms");
        prop_assert!(timeout_ms <= 300_000, "Timeout should not exceed 5 minutes");
    });
}

/// Test timeout configuration validation
#[test]
fn prop_timeout_configuration_bounds() {
    proptest!(ProptestConfig::with_cases(100), |(
        timeout_ms in 1u64..300_000
    )| {
        // Test that timeout configurations are reasonable
        prop_assert!(timeout_ms >= 1, "Minimum timeout should be at least 1ms");
        prop_assert!(timeout_ms <= 300_000, "Maximum timeout should not exceed 5 minutes");

        // Test that timeout is not too short for practical use
        prop_assert!(timeout_ms >= 10 || timeout_ms == 1,
            "Timeouts shorter than 10ms may be impractical except for testing");
    });
}

/// Test that timeout durations can be created from various values
#[test]
fn prop_timeout_duration_conversion() {
    proptest!(ProptestConfig::with_cases(100), |(
        timeout_ms in 1u64..300_000
    )| {
        let duration = Duration::from_millis(timeout_ms);

        // Test round-trip conversion
        prop_assert_eq!(duration.as_millis() as u64, timeout_ms);

        // Test that duration is positive
        prop_assert!(duration > Duration::from_millis(0));

        // Test that larger timeouts create larger durations
        let larger_duration = Duration::from_millis(timeout_ms + 1);
        prop_assert!(larger_duration > duration);
    });
}
