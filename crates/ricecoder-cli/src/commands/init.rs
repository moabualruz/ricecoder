// Initialize a new ricecoder project with interactive setup wizard

use std::io::{self, Write};

use async_trait::async_trait;

use super::Command;
use crate::{
    error::{CliError, CliResult},
    output::OutputStyle,
};

/// Initialize a new ricecoder project
pub struct InitCommand {
    pub project_path: Option<String>,
    pub interactive: bool,
    pub provider: String,
    pub model: Option<String>,
    pub force: bool,
}

impl InitCommand {
    pub fn new(project_path: Option<String>) -> Self {
        Self {
            project_path,
            interactive: false,
            provider: "zen".to_string(),
            model: None,
            force: false,
        }
    }

    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_model(mut self, model: Option<String>) -> Self {
        self.model = model;
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Prompt user for input
    fn prompt(&self, question: &str) -> CliResult<String> {
        let style = OutputStyle::default();
        print!("{}", style.prompt(question));
        io::stdout().flush().map_err(CliError::Io)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(CliError::Io)?;

        Ok(input.trim().to_string())
    }

    /// Prompt user for yes/no
    #[allow(dead_code)]
    fn prompt_yes_no(&self, question: &str) -> CliResult<bool> {
        loop {
            let response = self.prompt(&format!("{} (y/n): ", question))?;
            match response.to_lowercase().as_str() {
                "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                _ => println!("Please enter 'y' or 'n'"),
            }
        }
    }

    /// Show welcome message
    fn show_welcome(&self) {
        let style = OutputStyle::default();
        println!("{}", style.section("Welcome to RiceCoder"));
        println!();
        println!("This wizard will help you set up a new RiceCoder project.");
        println!();
        println!("{}", style.list_item("Create project configuration"));
        println!("{}", style.list_item("Set up AI provider"));
        println!("{}", style.list_item("Configure storage"));
        println!();
    }

    /// Show getting started guide
    fn show_getting_started(&self, project_path: &str) {
        let style = OutputStyle::default();
        println!();
        println!("{}", style.section("Getting Started"));
        println!();
        println!("Your project is ready! Here are the next steps:");
        println!();
        println!("{}", style.numbered_item(1, "Navigate to your project"));
        println!("   cd {}", project_path);
        println!();
        println!("{}", style.numbered_item(2, "Start the interactive chat"));
        println!("   rice chat");
        println!();
        println!(
            "{}",
            style.numbered_item(3, "Generate code from specifications")
        );
        println!("   rice gen --spec my-spec.md");
        println!();
        println!("{}", style.section("Learn More"));
        println!();
        println!(
            "{}",
            style.link(
                "Documentation",
                "https://github.com/moabualruz/ricecoder/wiki/docs"
            )
        );
        println!(
            "{}",
            style.link(
                "Examples",
                "https://github.com/moabualruz/ricecoder/wiki/examples"
            )
        );
        println!(
            "{}",
            style.link(
                "Troubleshooting",
                "https://github.com/moabualruz/ricecoder/wiki"
            )
        );
        println!();
    }

    /// Interactive setup wizard
    /// Only runs when explicitly requested with -i flag AND in a TTY environment
    fn run_wizard(&self, _path: &str) -> CliResult<(String, String, String)> {
        self.show_welcome();

        // Project name
        let project_name = self.prompt("Project name")?;
        let project_name = if project_name.is_empty() {
            "My Project".to_string()
        } else {
            project_name
        };

        // Project description
        let project_description = self.prompt("Project description (optional)")?;

        // Provider selection
        println!();
        println!("Available AI providers:");
        println!("  1. Zen (OpenCode.ai - https://opencode.ai/zen/v1)");
        println!("  2. OpenAI (GPT-4, GPT-3.5)");
        println!("  3. Anthropic (Claude)");
        println!("  4. Local (Ollama)");
        println!("  5. Other");
        println!();

        let provider = loop {
            let choice = self.prompt("Select provider (1-5)")?;
            match choice.as_str() {
                "1" => break "zen".to_string(),
                "2" => break "openai".to_string(),
                "3" => break "anthropic".to_string(),
                "4" => break "ollama".to_string(),
                "5" => break "other".to_string(),
                _ => println!("Please enter 1, 2, 3, 4, or 5"),
            }
        };

        Ok((project_name, project_description, provider))
    }
}

#[async_trait::async_trait]
impl Command for InitCommand {
    async fn execute(&self) -> CliResult<()> {
        let path = self.project_path.as_deref().unwrap_or(".");
        let style = OutputStyle::default();

        // Check if configuration already exists
        let config_path = format!("{}/.agent/ricecoder.toml", path);
        let config_exists = std::path::Path::new(&config_path).exists();

        // If config exists and force flag is not set, skip creation but don't error
        // This makes init idempotent - running it multiple times is safe
        if config_exists && !self.force {
            // Configuration already exists, nothing to do
            println!(
                "{}",
                style.info(&format!("Configuration already exists at {}", config_path))
            );
            return Ok(());
        }

        // Auto-detect TTY for CI/CD environments
        // Only run interactive wizard if:
        // 1. User explicitly requested it with -i flag, AND
        // 2. We're in a TTY environment (not CI/CD)
        let is_tty = atty::is(atty::Stream::Stdin);
        let should_run_wizard = self.interactive && is_tty;

        // Run interactive wizard if enabled and in TTY
        let (project_name, project_description, provider) = if should_run_wizard {
            self.run_wizard(path)?
        } else {
            // Use non-interactive defaults with command-line overrides
            let provider = self.provider.clone();
            ("My Project".to_string(), String::new(), provider)
        };

        // Create .agent/ directory structure
        std::fs::create_dir_all(format!("{}/.agent", path)).map_err(CliError::Io)?;

        // Create default configuration
        let model_line = if let Some(model) = &self.model {
            format!("model = \"{}\"\n", model)
        } else {
            String::new()
        };

        let config_content = format!(
            r#"# RiceCoder Project Configuration
# This file configures ricecoder for your project

[project]
name = "{}"
description = "{}"

[providers]
default = "{}"
{}
[storage]
mode = "merged"

# For more configuration options, see:
# https://github.com/moabualruz/ricecoder/wiki
"#,
            project_name, project_description, provider, model_line
        );

        std::fs::write(format!("{}/.agent/ricecoder.toml", path), config_content)
            .map_err(CliError::Io)?;

        // Create example spec file
        let example_spec = r#"# Example Specification

## Overview

This is an example specification for RiceCoder. You can use this as a template
for your own specifications.

## Requirements

### Requirement 1

**User Story:** As a user, I want to do something, so that I can achieve a goal.

#### Acceptance Criteria

1. WHEN I do something THEN the system SHALL do something else
2. WHEN I do another thing THEN the system SHALL respond appropriately

## Design

### Architecture

Describe your architecture here.

### Data Models

Describe your data models here.

## Tasks

- [ ] Task 1: Implement feature
- [ ] Task 2: Write tests
- [ ] Task 3: Document

For more information, see: https://github.com/moabualruz/ricecoder/wiki
"#;

        std::fs::write(format!("{}/.agent/example-spec.md", path), example_spec)
            .map_err(CliError::Io)?;

        // Create README
        let readme = format!(
            r#"# {}

{}

## Getting Started

1. Configure your AI provider in `.agent/ricecoder.toml`
2. Create a specification file (see `example-spec.md`)
3. Run `rice gen --spec your-spec.md` to generate code

## Documentation

- [RiceCoder Documentation](https://github.com/moabualruz/ricecoder/wiki/docs)
- [Configuration Guide](https://github.com/moabualruz/ricecoder/wiki
- [Specification Guide](https://github.com/moabualruz/ricecoder/wiki

## Support

- [GitHub Issues](https://github.com/ricecoder/ricecoder/issues)
- [Discussions](https://github.com/ricecoder/ricecoder/discussions)
- [Troubleshooting](https://github.com/moabualruz/ricecoder/wiki
"#,
            project_name, project_description
        );

        std::fs::write(format!("{}/README.md", path), readme).map_err(CliError::Io)?;

        // Print success message
        println!();
        println!(
            "{}",
            style.success(&format!("Initialized ricecoder project at {}", path))
        );
        println!();
        println!("Created files:");
        println!(
            "{}",
            style.list_item(".agent/ricecoder.toml - Project configuration")
        );
        println!(
            "{}",
            style.list_item(".agent/example-spec.md - Example specification")
        );
        println!("{}", style.list_item("README.md - Project documentation"));
        println!();

        // Show getting started guide
        self.show_getting_started(path);

        Ok(())
    }
}
