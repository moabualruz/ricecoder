//! Event handling for the TUI

use std::path::PathBuf;
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
    /// Drag-and-drop event with file paths
    /// Requirements: 1.1 - Detect drag-and-drop event via crossterm
    DragDrop { paths: Vec<PathBuf> },
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
    /// Sender for events (used for drag-and-drop and other programmatic events)
    tx: Option<tokio::sync::mpsc::UnboundedSender<Event>>,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let tx_clone = tx.clone();

        // Spawn a thread to handle terminal events
        thread::spawn(move || {
            // Tick event every 250ms
            let tick_interval = Duration::from_millis(250);
            let mut last_tick = std::time::Instant::now();

            loop {
                if last_tick.elapsed() >= tick_interval {
                    let _ = tx_clone.send(Event::Tick);
                    last_tick = std::time::Instant::now();
                }

                // TODO: Poll for actual terminal events with crossterm
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self { rx: Some(rx), tx: Some(tx) }
    }

    /// Poll for the next event
    pub async fn poll(&mut self) -> anyhow::Result<Option<Event>> {
        if let Some(rx) = &mut self.rx {
            Ok(rx.recv().await)
        } else {
            Ok(None)
        }
    }

    /// Send a drag-and-drop event with file paths
    /// 
    /// # Arguments
    /// 
    /// * `paths` - File paths from the drag-and-drop event
    /// 
    /// # Requirements
    /// 
    /// - Req 1.1: Create interface for receiving drag-and-drop events from ricecoder-tui
    /// - Req 1.1: Implement file path extraction from events
    pub fn send_drag_drop_event(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        if let Some(tx) = self.tx.as_ref() {
            tx.send(Event::DragDrop { paths })?;
        }
        Ok(())
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}
