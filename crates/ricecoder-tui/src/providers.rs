//! Provider management UI components
//!
//! This module provides UI components for managing AI providers in the TUI,
//! including provider selection, status displays, and performance monitoring.

use std::sync::Arc;

use ratatui::{layout::Rect, Frame};

use crate::{
    components::{Component, ComponentId, EventResult},
    event::Event,
    model::{AppMessage, ProviderInfo, ProviderMetrics, ProviderViewMode},
};

/// Provider manager component for handling provider operations
pub struct ProviderManager {
    id: ComponentId,
    providers: Vec<ProviderInfo>,
    selected_index: usize,
    view_mode: ProviderViewMode,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            providers: Vec::new(),
            selected_index: 0,
            view_mode: ProviderViewMode::List,
        }
    }

    /// Update the list of available providers
    pub fn update_providers(&mut self, providers: Vec<ProviderInfo>) {
        self.providers = providers;
        if self.selected_index >= self.providers.len() && !self.providers.is_empty() {
            self.selected_index = self.providers.len() - 1;
        }
    }

    /// Set the view mode
    pub fn set_view_mode(&mut self, mode: ProviderViewMode) {
        self.view_mode = mode;
    }

    /// Get the currently selected provider
    pub fn selected_provider(&self) -> Option<&ProviderInfo> {
        self.providers.get(self.selected_index)
    }

    /// Select the next provider
    pub fn select_next(&mut self) {
        if !self.providers.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.providers.len();
        }
    }

    /// Select the previous provider
    pub fn select_previous(&mut self) {
        if !self.providers.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.providers.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}

impl Component for ProviderManager {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, _model: &crate::model::AppModel) {
        // This component delegates rendering to the main view functions
        // The actual rendering is handled in view.rs render_provider_mode
    }

    fn update(
        &mut self,
        _message: &crate::model::AppMessage,
        _model: &crate::model::AppModel,
    ) -> bool {
        false
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn set_focused(&mut self, _focused: bool) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {}

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    fn bounds(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }

    fn set_bounds(&mut self, _bounds: ratatui::layout::Rect) {}

    fn handle_focus(
        &mut self,
        _direction: crate::components::FocusDirection,
    ) -> crate::components::FocusResult {
        crate::components::FocusResult::Boundary
    }

    fn children(&self) -> Vec<&dyn Component> {
        vec![]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Component> {
        vec![]
    }

    fn find_child(&self, _id: &ComponentId) -> Option<&dyn Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &ComponentId) -> Option<&mut dyn Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn Component>) {}

    fn remove_child(&mut self, _id: &ComponentId) -> Option<Box<dyn Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        0
    }

    fn set_z_index(&mut self, _z_index: i32) {}

    fn can_focus(&self) -> bool {
        false
    }

    fn tab_order(&self) -> Option<usize> {
        None
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {}

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(ProviderManager {
            id: self.id.clone(),
            providers: self.providers.clone(),
            selected_index: self.selected_index,
            view_mode: self.view_mode.clone(),
        })
    }
}

/// Provider status widget for displaying connection status
pub struct ProviderStatusWidget {
    id: ComponentId,
    provider: Option<ProviderInfo>,
}

impl ProviderStatusWidget {
    pub fn new(id: ComponentId) -> Self {
        Self { id, provider: None }
    }

    pub fn update_provider(&mut self, provider: ProviderInfo) {
        self.provider = Some(provider);
    }
}

impl Component for ProviderStatusWidget {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, _model: &crate::model::AppModel) {
        // Status rendering is handled in the main view functions
    }

    fn update(
        &mut self,
        _message: &crate::model::AppMessage,
        _model: &crate::model::AppModel,
    ) -> bool {
        false
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn set_focused(&mut self, _focused: bool) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {}

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    fn bounds(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }

    fn set_bounds(&mut self, _bounds: ratatui::layout::Rect) {}

    fn handle_focus(
        &mut self,
        _direction: crate::components::FocusDirection,
    ) -> crate::components::FocusResult {
        crate::components::FocusResult::Boundary
    }

    fn children(&self) -> Vec<&dyn Component> {
        vec![]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Component> {
        vec![]
    }

    fn find_child(&self, _id: &ComponentId) -> Option<&dyn Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &ComponentId) -> Option<&mut dyn Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn Component>) {}

    fn remove_child(&mut self, _id: &ComponentId) -> Option<Box<dyn Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        0
    }

    fn set_z_index(&mut self, _z_index: i32) {}

    fn can_focus(&self) -> bool {
        false
    }

    fn tab_order(&self) -> Option<usize> {
        None
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {}

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(ProviderStatusWidget {
            id: self.id.clone(),
            provider: self.provider.clone(),
        })
    }
}

/// Provider performance widget for displaying metrics
pub struct ProviderPerformanceWidget {
    id: ComponentId,
    metrics: Option<ProviderMetrics>,
}

impl ProviderPerformanceWidget {
    pub fn new(id: ComponentId) -> Self {
        Self { id, metrics: None }
    }

    pub fn update_metrics(&mut self, metrics: ProviderMetrics) {
        self.metrics = Some(metrics);
    }
}

impl Component for ProviderPerformanceWidget {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, _model: &crate::model::AppModel) {
        // Performance rendering is handled in the main view functions
    }

    fn update(
        &mut self,
        _message: &crate::model::AppMessage,
        _model: &crate::model::AppModel,
    ) -> bool {
        false
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn set_focused(&mut self, _focused: bool) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {}

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    fn bounds(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }

    fn set_bounds(&mut self, _bounds: ratatui::layout::Rect) {}

    fn handle_focus(
        &mut self,
        _direction: crate::components::FocusDirection,
    ) -> crate::components::FocusResult {
        crate::components::FocusResult::Boundary
    }

    fn children(&self) -> Vec<&dyn Component> {
        vec![]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Component> {
        vec![]
    }

    fn find_child(&self, _id: &ComponentId) -> Option<&dyn Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &ComponentId) -> Option<&mut dyn Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn Component>) {}

    fn remove_child(&mut self, _id: &ComponentId) -> Option<Box<dyn Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        0
    }

    fn set_z_index(&mut self, _z_index: i32) {}

    fn can_focus(&self) -> bool {
        false
    }

    fn tab_order(&self) -> Option<usize> {
        None
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {}

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(ProviderPerformanceWidget {
            id: self.id.clone(),
            metrics: self.metrics.clone(),
        })
    }
}

/// Provider factory for creating provider-related UI components
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new() -> Self {
        Self
    }

    /// Create a provider manager component
    pub fn provider_manager(&self, id: ComponentId) -> ProviderManager {
        ProviderManager::new(id)
    }

    /// Create a provider status widget
    pub fn provider_status_widget(&self, id: ComponentId) -> ProviderStatusWidget {
        ProviderStatusWidget::new(id)
    }

    /// Create a provider performance widget
    pub fn provider_performance_widget(&self, id: ComponentId) -> ProviderPerformanceWidget {
        ProviderPerformanceWidget::new(id)
    }
}

impl Default for ProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}
