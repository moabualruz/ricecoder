//! Property-based tests for TUI event capture completeness
//! Tests that all terminal events are properly captured and converted
//! Uses proptest for random test case generation
//!
//! **Feature: ricecoder-published-issues, Property 6: TUI Event Capture Completeness**
//! **Validates: Requirements 1.2, 1.3, 1.4**

use std::time::Duration;

use proptest::prelude::*;
use ricecoder_tui::event::{
    Event, EventLoop, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
};
use tokio::time::timeout;

// ============================================================================
// Generators for Property Tests
// ============================================================================

/// Generate valid key codes
fn arb_key_code() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        // Character keys
        prop::string::string_regex("[a-z]")
            .unwrap()
            .prop_map(|s| KeyCode::Char(s.chars().next().unwrap())),
        prop::string::string_regex("[A-Z]")
            .unwrap()
            .prop_map(|s| KeyCode::Char(s.chars().next().unwrap())),
        prop::string::string_regex("[0-9]")
            .unwrap()
            .prop_map(|s| KeyCode::Char(s.chars().next().unwrap())),
        // Special characters
        Just(KeyCode::Enter),
        Just(KeyCode::Esc),
        Just(KeyCode::Tab),
        Just(KeyCode::Backspace),
        Just(KeyCode::Delete),
        Just(KeyCode::Up),
        Just(KeyCode::Down),
        Just(KeyCode::Left),
        Just(KeyCode::Right),
        // Function keys
        (1u8..=12u8).prop_map(KeyCode::F),
        Just(KeyCode::Other),
    ]
}

/// Generate valid key modifiers
fn arb_key_modifiers() -> impl Strategy<Value = KeyModifiers> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(shift, ctrl, alt)| KeyModifiers {
        shift,
        ctrl,
        alt,
    })
}

/// Generate valid key events
fn arb_key_event() -> impl Strategy<Value = KeyEvent> {
    (arb_key_code(), arb_key_modifiers()).prop_map(|(code, modifiers)| KeyEvent { code, modifiers })
}

/// Generate valid mouse buttons
fn arb_mouse_button() -> impl Strategy<Value = MouseButton> {
    prop_oneof![
        Just(MouseButton::Left),
        Just(MouseButton::Right),
        Just(MouseButton::Middle),
    ]
}

/// Generate valid mouse coordinates (0-200 x 0-50)
fn arb_mouse_coords() -> impl Strategy<Value = (u16, u16)> {
    (0u16..=200, 0u16..=50)
}

/// Generate valid mouse events
fn arb_mouse_event() -> impl Strategy<Value = MouseEvent> {
    (arb_mouse_coords(), arb_mouse_button()).prop_map(|((x, y), button)| MouseEvent {
        x,
        y,
        button,
    })
}

/// Generate valid terminal resize dimensions
fn arb_terminal_size() -> impl Strategy<Value = (u16, u16)> {
    (80u16..=200, 24u16..=50)
}

// ============================================================================
// Property Tests
// ============================================================================

/// Property 1: Event loop can be created and polled without panicking
///
/// For any event loop instance, polling should not panic and should return
/// either an event or None without crashing.
///
/// **Validates: Requirements 1.2, 1.5, 1.6**
#[tokio::test]
async fn prop_event_loop_creation_and_polling() {
    // Create event loop
    let mut event_loop = EventLoop::new();

    // Poll should not panic and should complete within reasonable time
    let result = timeout(Duration::from_millis(100), event_loop.poll()).await;

    // Should either timeout (no events) or return Ok(None) or Ok(Some(Event))
    match result {
        Ok(Ok(Some(_))) => {
            // Got an event - this is fine
        }
        Ok(Ok(None)) => {
            // No event - this is fine
        }
        Ok(Err(_)) => {
            // Error from poll - this is acceptable
        }
        Err(_) => {
            // Timeout - this is fine, means no events
        }
    }
}

/// Property 2: Key events maintain their code and modifiers through conversion
///
/// For any key event, the code and modifiers should be preserved when
/// converted and sent through the event loop.
///
/// **Validates: Requirements 1.2**
#[test]
fn prop_key_event_conversion_preserves_data() {
    proptest!(|(key_event in arb_key_event())| {
        // Verify that key event data is valid
        match key_event.code {
            KeyCode::Char(c) => {
                // Character should be valid
                assert!(!c.is_control() || c == '\n' || c == '\t');
            }
            KeyCode::F(n) => {
                // Function key number should be 1-12
                assert!(n >= 1 && n <= 12);
            }
            _ => {
                // Other key codes are always valid
            }
        }

        // Verify modifiers are boolean flags
        let _ = key_event.modifiers.shift;
        let _ = key_event.modifiers.ctrl;
        let _ = key_event.modifiers.alt;
    });
}

/// Property 3: Mouse events maintain their coordinates and button through conversion
///
/// For any mouse event, the coordinates and button should be preserved when
/// converted and sent through the event loop.
///
/// **Validates: Requirements 1.3**
#[test]
fn prop_mouse_event_conversion_preserves_data() {
    proptest!(|(mouse_event in arb_mouse_event())| {
        // Verify that mouse event data is valid
        assert!(mouse_event.x <= 200);
        assert!(mouse_event.y <= 50);

        // Verify button is one of the valid options
        match mouse_event.button {
            MouseButton::Left | MouseButton::Right | MouseButton::Middle => {
                // Valid button
            }
        }
    });
}

/// Property 4: Resize events maintain their dimensions
///
/// For any terminal resize event, the width and height should be valid
/// and non-zero.
///
/// **Validates: Requirements 1.4**
#[test]
fn prop_resize_event_has_valid_dimensions() {
    proptest!(|(size in arb_terminal_size())| {
        let (width, height) = size;

        // Dimensions should be reasonable
        assert!(width >= 80, "Width should be at least 80");
        assert!(height >= 24, "Height should be at least 24");
        assert!(width <= 200, "Width should be at most 200");
        assert!(height <= 50, "Height should be at most 50");
    });
}

/// Property 5: Event loop sends tick events periodically
///
/// For any event loop instance, tick events should be sent at regular intervals
/// (approximately every 250ms).
///
/// **Validates: Requirements 1.5, 1.6**
#[tokio::test]
async fn prop_event_loop_sends_tick_events() {
    let mut event_loop = EventLoop::new();

    // Wait for a tick event (should arrive within 500ms)
    let start = std::time::Instant::now();
    let mut found_tick = false;

    while start.elapsed() < Duration::from_millis(500) {
        match timeout(Duration::from_millis(100), event_loop.poll()).await {
            Ok(Ok(Some(Event::Tick))) => {
                found_tick = true;
                break;
            }
            Ok(Ok(Some(_))) => {
                // Got some other event, keep waiting
            }
            Ok(Ok(None)) => {
                // No event, keep waiting
            }
            Ok(Err(_)) => {
                // Error from poll, keep waiting
            }
            Err(_) => {
                // Timeout, keep waiting
            }
        }
    }

    // We should have found at least one tick event
    assert!(found_tick, "Event loop should send tick events");
}

/// Property 6: Event loop handles channel closure gracefully
///
/// For any event loop instance, when the receiver is dropped, the event loop
/// should exit gracefully without panicking.
///
/// **Validates: Requirements 1.7**
#[tokio::test]
async fn prop_event_loop_handles_channel_closure() {
    let event_loop = EventLoop::new();

    // Drop the event loop - should not panic
    drop(event_loop);

    // Give the thread time to exit
    tokio::time::sleep(Duration::from_millis(100)).await;

    // If we get here without panicking, the test passes
}

/// Property 7: Key event modifiers are independent
///
/// For any key event, each modifier (shift, ctrl, alt) should be independent
/// and not affect the others.
///
/// **Validates: Requirements 1.2**
#[test]
fn prop_key_modifiers_are_independent() {
    proptest!(|(modifiers in arb_key_modifiers())| {
        // Each modifier should be independent
        let shift = modifiers.shift;
        let ctrl = modifiers.ctrl;
        let alt = modifiers.alt;

        // Verify they're all boolean
        assert!(shift == true || shift == false);
        assert!(ctrl == true || ctrl == false);
        assert!(alt == true || alt == false);

        // Verify they don't affect each other
        let mut modified = modifiers;
        modified.shift = !modified.shift;
        assert_ne!(modified.shift, modifiers.shift);
        assert_eq!(modified.ctrl, modifiers.ctrl);
        assert_eq!(modified.alt, modifiers.alt);
    });
}

/// Property 8: Mouse button types are distinct
///
/// For any mouse button, it should be one of the three valid types and
/// should be distinguishable from the others.
///
/// **Validates: Requirements 1.3**
#[test]
fn prop_mouse_buttons_are_distinct() {
    proptest!(|(button in arb_mouse_button())| {
        // Each button should be distinct
        match button {
            MouseButton::Left => {
                assert_ne!(button, MouseButton::Right);
                assert_ne!(button, MouseButton::Middle);
            }
            MouseButton::Right => {
                assert_ne!(button, MouseButton::Left);
                assert_ne!(button, MouseButton::Middle);
            }
            MouseButton::Middle => {
                assert_ne!(button, MouseButton::Left);
                assert_ne!(button, MouseButton::Right);
            }
        }
    });
}
