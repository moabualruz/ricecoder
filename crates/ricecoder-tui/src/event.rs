//! Event handling for the TUI

use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

/// Event types for the TUI
#[derive(Debug, Clone)]
pub enum Event {
    /// Keyboard input event
    Key(KeyEvent),
    /// Mouse input event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize { width: u16, height: u16 },
    /// Tick event for periodic updates
    Tick,
}

/// Keyboard event
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// Key code
    pub code: KeyCode,
    /// Modifier keys
    pub modifiers: KeyModifiers,
}

/// Key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Character key
    Char(char),
    /// Enter key
    Enter,
    /// Escape key
    Esc,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    /// Function keys
    F(u8),
    /// Other keys
    Other,
}

/// Key modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    /// Shift key
    pub shift: bool,
    /// Control key
    pub ctrl: bool,
    /// Alt key
    pub alt: bool,
}

/// Mouse event
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    /// X coordinate
    pub x: u16,
    /// Y coordinate
    pub y: u16,
    /// Mouse button
    pub button: MouseButton,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left button
    Left,
    /// Right button
    Right,
    /// Middle button
    Middle,
}

/// Event loop for handling terminal events
pub struct EventLoop {
    /// Receiver for events
    rx: Option<UnboundedReceiver<Event>>,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Spawn a thread to handle terminal events
        thread::spawn(move || {
            // Tick event every 250ms
            let tick_interval = Duration::from_millis(250);
            let mut last_tick = std::time::Instant::now();

            loop {
                if last_tick.elapsed() >= tick_interval {
                    let _ = tx.send(Event::Tick);
                    last_tick = std::time::Instant::now();
                }

                // TODO: Poll for actual terminal events with crossterm
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self { rx: Some(rx) }
    }

    /// Poll for the next event
    pub async fn poll(&mut self) -> anyhow::Result<Option<Event>> {
        if let Some(rx) = &mut self.rx {
            Ok(rx.recv().await)
        } else {
            Ok(None)
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}
