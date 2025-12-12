//! Event handling for the TUI

use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use crossterm::event as crossterm_event;
use crate::model::AppMessage;

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
    /// 
    /// Spawns a thread that polls for terminal events using crossterm with a 10ms timeout.
    /// Converts crossterm events to RiceCoder Event types and sends them through an mpsc channel.
    /// Also sends periodic Tick events every 250ms for UI updates.
    /// 
    /// Requirements: 1.2, 1.5, 1.6
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let tx_clone = tx.clone();

        // Spawn a thread to handle terminal events
        thread::spawn(move || {
            // Tick event every 250ms
            let tick_interval = Duration::from_millis(250);
            let poll_timeout = Duration::from_millis(10);
            let mut last_tick = std::time::Instant::now();

            loop {
                // Poll for terminal events using crossterm with 10ms timeout
                // Requirements: 1.5 - Use crossterm::event::poll() with 10ms timeout
                if crossterm_event::poll(poll_timeout).unwrap_or(false) {
                    // Read the event from crossterm
                    // Requirements: 1.5 - Use crossterm::event::read() for event capture
                    if let Ok(event) = crossterm_event::read() {
                        let rice_event = Self::convert_crossterm_event(event);
                        if let Some(e) = rice_event {
                            if tx_clone.send(e).is_err() {
                                // Channel closed, exit loop
                                break;
                            }
                        }
                    }
                }

                // Send tick event at regular intervals
                if last_tick.elapsed() >= tick_interval {
                    if tx_clone.send(Event::Tick).is_err() {
                        // Channel closed, exit loop
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self { rx: Some(rx), tx: Some(tx) }
    }

    /// Convert a crossterm event to a RiceCoder Event
    /// 
    /// Requirements: 1.2, 1.3, 1.4, 1.6
    fn convert_crossterm_event(event: crossterm_event::Event) -> Option<Event> {
        match event {
            // Keyboard events
            // Requirements: 1.2 - Convert crossterm::event::KeyEvent to Event::Key
            crossterm_event::Event::Key(key) => {
                Some(Event::Key(Self::convert_key_event(key)))
            }
            // Mouse events
            // Requirements: 1.3 - Convert crossterm::event::MouseEvent to Event::Mouse
            crossterm_event::Event::Mouse(mouse) => {
                Some(Event::Mouse(Self::convert_mouse_event(mouse)))
            }
            // Resize events
            // Requirements: 1.4 - Convert resize events to Event::Resize
            crossterm_event::Event::Resize(width, height) => {
                Some(Event::Resize { width, height })
            }
            // Ignore focus events
            crossterm_event::Event::FocusGained | crossterm_event::Event::FocusLost => None,
            // Ignore paste events (handle separately if needed)
            crossterm_event::Event::Paste(_) => None,
        }
    }

    /// Convert a crossterm KeyEvent to a RiceCoder KeyEvent
    /// 
    /// Requirements: 1.2 - Verify key codes and modifiers are mapped correctly
    fn convert_key_event(key: crossterm_event::KeyEvent) -> KeyEvent {
        let code = match key.code {
            crossterm_event::KeyCode::Char(c) => KeyCode::Char(c),
            crossterm_event::KeyCode::Enter => KeyCode::Enter,
            crossterm_event::KeyCode::Esc => KeyCode::Esc,
            crossterm_event::KeyCode::Tab => KeyCode::Tab,
            crossterm_event::KeyCode::Backspace => KeyCode::Backspace,
            crossterm_event::KeyCode::Delete => KeyCode::Delete,
            crossterm_event::KeyCode::Up => KeyCode::Up,
            crossterm_event::KeyCode::Down => KeyCode::Down,
            crossterm_event::KeyCode::Left => KeyCode::Left,
            crossterm_event::KeyCode::Right => KeyCode::Right,
            crossterm_event::KeyCode::F(n) => KeyCode::F(n),
            _ => KeyCode::Other,
        };

        let modifiers = KeyModifiers {
            shift: key.modifiers.contains(crossterm_event::KeyModifiers::SHIFT),
            ctrl: key.modifiers.contains(crossterm_event::KeyModifiers::CONTROL),
            alt: key.modifiers.contains(crossterm_event::KeyModifiers::ALT),
        };

        KeyEvent { code, modifiers }
    }

    /// Convert a crossterm MouseEvent to a RiceCoder MouseEvent
    /// 
    /// Requirements: 1.3 - Verify mouse events are mapped correctly
    fn convert_mouse_event(mouse: crossterm_event::MouseEvent) -> MouseEvent {
        let button = match mouse.kind {
            crossterm_event::MouseEventKind::Down(btn)
            | crossterm_event::MouseEventKind::Up(btn)
            | crossterm_event::MouseEventKind::Drag(btn) => match btn {
                crossterm_event::MouseButton::Left => MouseButton::Left,
                crossterm_event::MouseButton::Right => MouseButton::Right,
                crossterm_event::MouseButton::Middle => MouseButton::Middle,
            },
            _ => MouseButton::Left,
        };

        MouseEvent {
            x: mouse.column,
            y: mouse.row,
            button,
        }
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

/// Convert RiceCoder Event to TEA AppMessage
pub fn event_to_message(event: Event) -> AppMessage {
    match event {
        Event::Key(key) => AppMessage::KeyPress(crossterm::event::KeyEvent::new(
            match key.code {
                KeyCode::Char(c) => crossterm::event::KeyCode::Char(c),
                KeyCode::Enter => crossterm::event::KeyCode::Enter,
                KeyCode::Esc => crossterm::event::KeyCode::Esc,
                KeyCode::Tab => crossterm::event::KeyCode::Tab,
                KeyCode::Backspace => crossterm::event::KeyCode::Backspace,
                KeyCode::Delete => crossterm::event::KeyCode::Delete,
                KeyCode::Up => crossterm::event::KeyCode::Up,
                KeyCode::Down => crossterm::event::KeyCode::Down,
                KeyCode::Left => crossterm::event::KeyCode::Left,
                KeyCode::Right => crossterm::event::KeyCode::Right,
                KeyCode::Home => crossterm::event::KeyCode::Home,
                KeyCode::End => crossterm::event::KeyCode::End,
                KeyCode::PageUp => crossterm::event::KeyCode::PageUp,
                KeyCode::PageDown => crossterm::event::KeyCode::PageDown,
                KeyCode::F1 => crossterm::event::KeyCode::F(1),
                KeyCode::F2 => crossterm::event::KeyCode::F(2),
                KeyCode::F3 => crossterm::event::KeyCode::F(3),
                KeyCode::F4 => crossterm::event::KeyCode::F(4),
                KeyCode::F5 => crossterm::event::KeyCode::F(5),
                KeyCode::F6 => crossterm::event::KeyCode::F(6),
                KeyCode::F7 => crossterm::event::KeyCode::F(7),
                KeyCode::F8 => crossterm::event::KeyCode::F(8),
                KeyCode::F9 => crossterm::event::KeyCode::F(9),
                KeyCode::F10 => crossterm::event::KeyCode::F(10),
                KeyCode::F11 => crossterm::event::KeyCode::F(11),
                KeyCode::F12 => crossterm::event::KeyCode::F(12),
                _ => crossterm::event::KeyCode::Null,
            },
            match key.modifiers {
                KeyModifiers { ctrl: true, alt: false, shift: false } => crossterm::event::KeyModifiers::CONTROL,
                KeyModifiers { ctrl: false, alt: true, shift: false } => crossterm::event::KeyModifiers::ALT,
                KeyModifiers { ctrl: false, alt: false, shift: true } => crossterm::event::KeyModifiers::SHIFT,
                _ => crossterm::event::KeyModifiers::empty(),
            },
        )),
        Event::Mouse(mouse) => AppMessage::MouseEvent(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Moved,
            column: mouse.x,
            row: mouse.y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        }),
        Event::Resize { width, height } => AppMessage::Resize { width, height },
        Event::Tick => AppMessage::Tick,
        Event::DragDrop { .. } => {
            // For now, ignore drag-drop events in TEA
            // Could be handled in future by adding a new AppMessage variant
            AppMessage::Tick
        }
    }
}
