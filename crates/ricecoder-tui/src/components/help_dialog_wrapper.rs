//! Component wrapper for HelpDialog

use crate::components::traits::{Component, ComponentId};
use crate::components::{FocusDirection, FocusResult};
use crate::model::{AppModel, AppMessage};
use ratatui::prelude::*;
use ricecoder_help::HelpDialog;

/// Wrapper that implements Component for HelpDialog
pub struct HelpDialogComponent {
    inner: HelpDialog,
    id: ComponentId,
    bounds: Rect,
    focused: bool,
    z_index: i32,
}

impl HelpDialogComponent {
    pub fn new() -> Self {
        Self {
            inner: HelpDialog::default_ricecoder(),
            id: "help-dialog".to_string(),
            bounds: Rect::default(),
            focused: false,
            z_index: 100,
        }
    }

    pub fn inner(&self) -> &HelpDialog {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut HelpDialog {
        &mut self.inner
    }
}

impl Default for HelpDialogComponent {
    fn default() -> Self {
        Self::new()
    }
}

// Note: We don't implement Clone because HelpDialog doesn't implement Clone

#[allow(deprecated)]
impl Component for HelpDialogComponent {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, _model: &AppModel) {
        // Delegate to inner HelpDialog's render method
        // SAFETY: We cast away const since HelpDialog::render requires &mut self
        // This is safe because render() only mutates internal layout state
        let inner_mut = unsafe { &mut *((&self.inner) as *const HelpDialog as *mut HelpDialog) };
        inner_mut.render(frame, area);
    }

    fn clone_box(&self) -> Box<dyn Component> {
        // Create a new instance instead of cloning
        Box::new(HelpDialogComponent::new())
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        if visible {
            self.inner.show();
        } else {
            self.inner.hide();
        }
    }

    fn z_index(&self) -> i32 {
        self.z_index
    }

    fn set_z_index(&mut self, z_index: i32) {
        self.z_index = z_index;
    }

    fn update(&mut self, message: &AppMessage, _model: &AppModel) -> bool {
        // Convert AppMessage key events to HelpDialog handle_key calls
        if let AppMessage::KeyPress(key) = message {
            // HelpDialog::handle_key returns Result<bool>
            // We handle the result and return whether we handled the event
            match self.inner.handle_key(*key) {
                Ok(handled) => return handled,
                Err(_) => return false,
            }
        }
        false
    }
}
