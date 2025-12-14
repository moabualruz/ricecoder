//! Providers command - Manage AI providers

use crate::commands::Command;
use crate::error::{CliError, CliResult};
use ricecoder_agents::use_cases::{
    ProviderSwitchingUseCase, ProviderPerformanceUseCase, ProviderFailoverUseCase,
    ProviderModelUseCase, ProviderHealthUseCase, ProviderCommunityUseCase,
};
use std::sync::Arc;

/// Providers command action
#[derive(Debug, Clone)]
pub enum ProvidersAction {
    /// List all available providers
    List,
    /// Switch to a specific provider
    Switch { provider_id: String },
    /// Show current provider status
    Status { provider_id: Option<String> },
    /// Show provider performance metrics
    Performance { provider_id: Option<String> },
    /// Check provider health
    Health { provider_id: Option<String> },
    /// List available models for a provider
    Models { provider_id: Option<String>, filter: Option<String> },
    /// Get failover provider for a failing provider
    Failover { provider_id: String },
    /// Show community provider analytics
    Community { provider_id: Option<String> },
}

/// Providers command handler
pub struct ProvidersCommand {
    action: ProvidersAction,
}

impl ProvidersCommand {
    /// Create a new providers command
    pub fn new(action: ProvidersAction) -> Self {
        Self { action }
    }

    /// Get provider use cases (create them if needed)
    fn get_use_cases(&self) -> CliResult<(
        Arc<ProviderSwitchingUseCase>,
        Arc<ProviderPerformanceUseCase>,
        Arc<ProviderFailoverUseCase>,
        Arc<ProviderModelUseCase>,
        Arc<ProviderHealthUseCase>,
        Arc<ProviderCommunityUseCase>,
    )> {
        // For now, create the infrastructure components here
        // In a real application, these would be injected from a DI container
        let registry = ricecoder_providers::provider::ProviderRegistry::new();
        let provider_manager = Arc::new(ricecoder_providers::provider::manager::ProviderManager::new(
            registry,
            "openai".to_string(), // default provider
        ));

        let switching = Arc::new(ProviderSwitchingUseCase::new(provider_manager.clone()));
        let performance = Arc::new(ProviderPerformanceUseCase::new(provider_manager.clone()));
        let failover = Arc::new(ProviderFailoverUseCase::new(provider_manager.clone()));
        let models = Arc::new(ProviderModelUseCase::new(provider_manager.clone()));
        let health = Arc::new(ProviderHealthUseCase::new(provider_manager.clone()));
        let community = Arc::new(ProviderCommunityUseCase::new(provider_manager));

        Ok((switching, performance, failover, models, health, community))
    }
}

impl Command for ProvidersCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            ProvidersAction::List => self.list_providers(),
            ProvidersAction::Switch { provider_id } => self.switch_provider(provider_id),
            ProvidersAction::Status { provider_id } => self.show_provider_status(provider_id.as_deref()),
            ProvidersAction::Performance { provider_id } => self.show_provider_performance(provider_id.as_deref()),
            ProvidersAction::Health { provider_id } => self.check_provider_health(provider_id.as_deref()),
            ProvidersAction::Models { provider_id, filter } => self.list_provider_models(provider_id.as_deref(), filter.as_deref()),
            ProvidersAction::Failover { provider_id } => self.show_failover_provider(provider_id),
            ProvidersAction::Community { provider_id } => self.show_community_analytics(provider_id.as_deref()),
        }
    }
}

impl ProvidersCommand {
    /// List all available providers
    fn list_providers(&self) -> CliResult<()> {
        let (switching, _, _, _, _, _) = self.get_use_cases()?;
        let providers = switching.list_available_providers();

        if providers.is_empty() {
            println!("No providers configured. Configure providers first.");
            return Ok(());
        }

        println!("Available Providers:");
        println!();

        for provider in providers {
            let status_icon = match provider.state {
                ricecoder_providers::provider::manager::ConnectionState::Connected => "🟢",
                ricecoder_providers::provider::manager::ConnectionState::Disconnected => "🟡",
                ricecoder_providers::provider::manager::ConnectionState::Error => "🔴",
                ricecoder_providers::provider::manager::ConnectionState::Disabled => "⚪",
            };

            println!("{} {} - {}", status_icon, provider.name, provider.id);
            println!("  Status: {:?}", provider.state);
            println!("  Models: {}", provider.models.len());

            if let Some(error) = &provider.error_message {
                println!("  Error: {}", error);
            }

            if let Some(last_checked) = provider.last_checked {
                println!("  Last checked: {}", last_checked.format("%Y-%m-%d %H:%M:%S"));
            }

            println!();
        }

        Ok(())
    }

    /// Switch to a specific provider
    fn switch_provider(&self, provider_id: &str) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let (switching, _, _, _, _, _) = self.get_use_cases()?;

        rt.block_on(switching.switch_provider(provider_id))
            .map_err(|e| CliError::Internal(format!("Failed to switch provider: {}", e)))?;

        println!("Successfully switched to provider: {}", provider_id);
        Ok(())
    }

    /// Show provider status
    fn show_provider_status(&self, provider_id: Option<&str>) -> CliResult<()> {
        let (switching, _, _, _, _, _) = self.get_use_cases()?;

        if let Some(provider_id) = provider_id {
            if let Some(status) = switching.get_provider_status(provider_id) {
                println!("Provider: {}", status.name);
                println!("  ID: {}", status.id);
                println!("  Status: {:?}", status.state);
                println!("  Models: {}", status.models.len());

                if let Some(error) = &status.error_message {
                    println!("  Error: {}", error);
                }

                if let Some(last_checked) = status.last_checked {
                    println!("  Last checked: {}", last_checked.format("%Y-%m-%d %H:%M:%S"));
                }
            } else {
                println!("Provider '{}' not found", provider_id);
            }
        } else {
            // Show current provider
            let current = switching.get_current_provider();
            println!("Current provider: {}", current);
        }

        Ok(())
    }

    /// Show provider performance metrics
    fn show_provider_performance(&self, provider_id: Option<&str>) -> CliResult<()> {
        let (_, performance, _, _, _, _) = self.get_use_cases()?;

        if let Some(provider_id) = provider_id {
            if let Some(metrics) = performance.get_provider_performance(provider_id) {
                println!("Performance Metrics for {}:", provider_id);
                println!("  Total requests: {}", metrics.total_requests);
                println!("  Successful requests: {}", metrics.successful_requests);
                println!("  Failed requests: {}", metrics.failed_requests);
                println!("  Average response time: {:.2}ms", metrics.avg_response_time_ms);
                println!("  Error rate: {:.2}%", metrics.error_rate * 100.0);
                println!("  Total tokens: {}", metrics.total_tokens);
                println!("  Total cost: ${:.4}", metrics.total_cost);
                println!("  Requests/second: {:.2}", metrics.requests_per_second);
                println!("  Tokens/second: {:.2}", metrics.tokens_per_second);
            } else {
                println!("No performance data available for provider '{}'", provider_id);
            }
        } else {
            // Show all providers performance
            let summary = performance.get_all_provider_performance();
            println!("Overall Performance Summary:");
            println!("  Total providers: {}", summary.total_providers);
            println!("  Total requests: {}", summary.total_requests);
            println!("  Total errors: {}", summary.total_errors);
            println!("  Average response time: {:.2}ms", summary.avg_response_time_ms);
            println!("  Overall error rate: {:.2}%", summary.overall_error_rate * 100.0);
            println!("  Performing providers: {}", summary.performing_providers);

            println!();
            println!("Providers by performance (response time):");
            let sorted = performance.get_providers_by_performance();
            for (id, response_time) in sorted {
                println!("  {}: {:.2}ms", id, response_time);
            }
        }

        Ok(())
    }

    /// Check provider health
    fn check_provider_health(&self, provider_id: Option<&str>) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
        let (_, _, _, _, health, _) = self.get_use_cases()?;

        if let Some(provider_id) = provider_id {
            let is_healthy = rt.block_on(health.check_provider_health(provider_id))
                .map_err(|e| CliError::Internal(format!("Health check failed: {}", e)))?;

            println!("Provider '{}' health: {}", provider_id, if is_healthy { "Healthy" } else { "Unhealthy" });
        } else {
            // Check all providers
            let results = rt.block_on(health.check_all_provider_health());
            println!("Provider Health Status:");
            println!();

            for (id, result) in results {
                let status = match result {
                    Ok(true) => "🟢 Healthy",
                    Ok(false) => "🔴 Unhealthy",
                    Err(_) => "❓ Error",
                };
                println!("  {}: {}", id, status);
            }
        }

        Ok(())
    }

    /// List provider models
    fn list_provider_models(&self, provider_id: Option<&str>, filter: Option<&str>) -> CliResult<()> {
        let (_, _, _, models, _, _) = self.get_use_cases()?;

        let filter_criteria = filter.map(|f| match f.to_lowercase().as_str() {
            "free" => ricecoder_providers::provider::manager::ModelFilterCriteria::FreeOnly,
            "chat" => ricecoder_providers::provider::manager::ModelFilterCriteria::Capability(ricecoder_providers::models::Capability::Chat),
            "completion" => ricecoder_providers::provider::manager::ModelFilterCriteria::Capability(ricecoder_providers::models::Capability::Completion),
            _ => {
                println!("Unknown filter: {}. Available filters: free, chat, completion", f);
                return Ok(());
            }
        });

        let mut model_filter = ricecoder_providers::provider::manager::ModelFilter::new();
        if let Some(criteria) = filter_criteria {
            model_filter = model_filter.with_criterion(criteria);
        }

        let available_models = models.get_available_models(Some(model_filter));

        if available_models.is_empty() {
            println!("No models available matching the criteria.");
            return Ok(());
        }

        if let Some(provider_id) = provider_id {
            // Filter by provider
            let provider_models: Vec<_> = available_models.into_iter()
                .filter(|m| m.provider == provider_id)
                .collect();

            if provider_models.is_empty() {
                println!("No models available for provider '{}'", provider_id);
                return Ok(());
            }

            println!("Models for provider '{}':", provider_id);
            for model in provider_models {
                self.display_model_info(&model);
            }
        } else {
            // Show all models grouped by provider
            let mut by_provider: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();

            for model in available_models {
                by_provider.entry(model.provider.clone()).or_default().push(model);
            }

            for (provider, models) in by_provider {
                println!("Provider: {}", provider);
                for model in models {
                    print!("  ");
                    self.display_model_info(&model);
                }
                println!();
            }
        }

        Ok(())
    }

    /// Display model information
    fn display_model_info(&self, model: &ricecoder_providers::models::ModelInfo) {
        println!("{} - {} tokens", model.id, model.context_window);
        if model.is_free {
            print!(" (Free)");
        }
        if let Some(ref pricing) = model.pricing {
            print!(" (${:.4}/1K input, ${:.4}/1K output)",
                pricing.input_per_1k_tokens, pricing.output_per_1k_tokens);
        }
        println!();
    }

    /// Show failover provider
    fn show_failover_provider(&self, provider_id: &str) -> CliResult<()> {
        let (_, _, failover, _, _, _) = self.get_use_cases()?;

        if let Some(failover_id) = failover.get_failover_provider(provider_id) {
            println!("Failover provider for '{}': {}", provider_id, failover_id);
        } else {
            println!("No failover provider available for '{}'", provider_id);
        }

        Ok(())
    }

    /// Show community analytics
    fn show_community_analytics(&self, provider_id: Option<&str>) -> CliResult<()> {
        let (_, _, _, _, _, community) = self.get_use_cases()?;

        if let Some(provider_id) = provider_id {
            if let Some(analytics) = community.get_provider_analytics(provider_id) {
                println!("Community Analytics for {}:", provider_id);
                println!("  Total requests: {}", analytics.total_requests);
                println!("  Successful requests: {}", analytics.successful_requests);
                println!("  Failed requests: {}", analytics.failed_requests);
                println!("  Average response time: {:.2}ms", analytics.avg_response_time_ms);
                println!("  Community rating: {:.2}/5.0", analytics.community_rating);
                println!("  Reliability score: {:.2}%", analytics.reliability_score * 100.0);
            } else {
                println!("No community analytics available for provider '{}'", provider_id);
            }
        } else {
            // Show popular providers
            let popular = community.get_popular_providers(10);
            if popular.is_empty() {
                println!("No community data available");
            } else {
                println!("Most Popular Providers:");
                for (id, requests) in popular {
                    println!("  {}: {} requests", id, requests);
                }
            }

            println!();
            let quality = community.get_providers_by_community_quality(10);
            if !quality.is_empty() {
                println!("Highest Quality Providers:");
                for (id, score) in quality {
                    println!("  {}: {:.2}/5.0", id, score);
                }
            }
        }

        Ok(())
    }
}