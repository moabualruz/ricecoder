//! Provider management UI components
//!
//! This module provides UI components for managing AI providers in the TUI,
//! including provider selection, status displays, and performance monitoring.

use crate::components::{Component, ComponentId, Event, EventResult};
use crate::model::{AppMessage, ProviderInfo, ProviderMetrics, ProviderViewMode};
use ratatui::layout::Rect;
use ratatui::Frame;
use std::sync::Arc;

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
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn handle_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(key) => match key.code {
                crossterm::event::KeyCode::Up => {
                    self.select_previous();
                    EventResult::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    self.select_next();
                    EventResult::Consumed
                }
                crossterm::event::KeyCode::Enter => {
                    if let Some(provider) = self.selected_provider() {
                        EventResult::Message(AppMessage::ProviderSelected(provider.id.clone()))
                    } else {
                        EventResult::Consumed
                    }
                }
                crossterm::event::KeyCode::Char('l') => {
                    self.view_mode = ProviderViewMode::List;
                    EventResult::Message(AppMessage::ProviderViewModeChanged(ProviderViewMode::List))
                }
                crossterm::event::KeyCode::Char('s') => {
                    self.view_mode = ProviderViewMode::Status;
                    EventResult::Message(AppMessage::ProviderViewModeChanged(ProviderViewMode::Status))
                }
                crossterm::event::KeyCode::Char('p') => {
                    self.view_mode = ProviderViewMode::Performance;
                    EventResult::Message(AppMessage::ProviderViewModeChanged(ProviderViewMode::Performance))
                }
                crossterm::event::KeyCode::Char('a') => {
                    self.view_mode = ProviderViewMode::Analytics;
                    EventResult::Message(AppMessage::ProviderViewModeChanged(ProviderViewMode::Analytics))
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // This component delegates rendering to the main view functions
        // The actual rendering is handled in view.rs render_provider_mode
    }
}

/// Provider status widget for displaying connection status
pub struct ProviderStatusWidget {
    id: ComponentId,
    provider: Option<ProviderInfo>,
}

impl ProviderStatusWidget {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            provider: None,
        }
    }

    pub fn update_provider(&mut self, provider: ProviderInfo) {
        self.provider = Some(provider);
    }
}

impl Component for ProviderStatusWidget {
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn handle_event(&mut self, _event: Event) -> EventResult {
        EventResult::Ignored
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Status rendering is handled in the main view functions
    }
}

/// Provider performance widget for displaying metrics
pub struct ProviderPerformanceWidget {
    id: ComponentId,
    metrics: Option<ProviderMetrics>,
}

impl ProviderPerformanceWidget {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            metrics: None,
        }
    }

    pub fn update_metrics(&mut self, metrics: ProviderMetrics) {
        self.metrics = Some(metrics);
    }
}

impl Component for ProviderPerformanceWidget {
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn handle_event(&mut self, _event: Event) -> EventResult {
        EventResult::Ignored
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Performance rendering is handled in the main view functions
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