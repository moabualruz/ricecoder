// Help and tutorial command

use super::Command;
use crate::error::CliResult;
use crate::output::OutputStyle;

/// Help and tutorial command
pub struct HelpCommand {
    pub topic: Option<String>,
}

impl HelpCommand {
    pub fn new(topic: Option<String>) -> Self {
        Self { topic }
    }

    fn show_main_help(&self) {
        let style = OutputStyle::default();
        println!("{}", style.section("RiceCoder - Spec-Driven Code Generation"));
        println!();
        println!("RiceCoder is a terminal-first, spec-driven coding assistant that helps you");
        println!("write better code through research-first project analysis and AI-powered generation.");
        println!();

        println!("{}", style.section("Quick Start"));
        println!();
        println!("{}", style.numbered_item(1, "Initialize a new project"));
        println!("   rice init");
        println!();
        println!("{}", style.numbered_item(2, "Create a specification"));
        println!("   Create a file with your requirements and design");
        println!();
        println!("{}", style.numbered_item(3, "Generate code"));
        println!("   rice gen --spec my-spec.md");
        println!();

        println!("{}", style.section("Available Commands"));
        println!();
        println!("{}", style.key_value("init", "Initialize a new ricecoder project"));
        println!("{}", style.key_value("gen", "Generate code from specifications"));
        println!("{}", style.key_value("chat", "Interactive chat mode"));
        println!("{}", style.key_value("config", "Manage configuration"));
        println!("{}", style.key_value("lsp", "Start LSP server"));
        println!("{}", style.key_value("help", "Show this help message"));
        println!();

        println!("{}", style.section("Getting Help"));
        println!();
        println!("For help on a specific command:");
        println!("   rice help <command>");
        println!();
        println!("For tutorials:");
        println!("   rice help tutorial");
        println!();
        println!("For common issues:");
        println!("   rice help troubleshooting");
        println!();
        println!("For keyboard shortcuts:");
        println!("   rice help shortcuts");
        println!();
        println!("For accessibility features:");
        println!("   rice help accessibility");
        println!();

        println!("{}", style.section("Resources"));
        println!();
        println!("{}", style.link("Documentation", "https://ricecoder.dev/docs"));
        println!("{}", style.link("Examples", "https://ricecoder.dev/examples"));
        println!("{}", style.link("GitHub", "https://github.com/ricecoder/ricecoder"));
        println!();
    }

    fn show_command_help(&self, command: &str) {
        use crate::accessibility::{AccessibilityFeatures, KeyboardShortcuts};
        
        let style = OutputStyle::default();
        match command {
            "shortcuts" => {
                KeyboardShortcuts::print_all();
                return;
            }
            "accessibility" => {
                AccessibilityFeatures::print_guide();
                return;
            }
            "init" => {
                println!("{}", style.section("rice init - Initialize a Project"));
                println!();
                println!("Initialize a new RiceCoder project with interactive setup.");
                println!();
                println!("{}", style.header("Usage"));
                println!("   rice init [PATH]");
                println!();
                println!("{}", style.header("Arguments"));
                println!("{}", style.key_value("PATH", "Project directory (default: current)"));
                println!();
                println!("{}", style.header("What it does"));
                println!("{}", style.list_item("Creates .agent/ directory"));
                println!("{}", style.list_item("Generates ricecoder.toml configuration"));
                println!("{}", style.list_item("Creates example specification"));
                println!("{}", style.list_item("Creates README.md"));
                println!();
                println!("{}", style.header("Example"));
                println!("   rice init my-project");
                println!();
            }
            "gen" => {
                println!("{}", style.section("rice gen - Generate Code"));
                println!();
                println!("Generate code from specifications using AI.");
                println!();
                println!("{}", style.header("Usage"));
                println!("   rice gen --spec <FILE>");
                println!();
                println!("{}", style.header("Options"));
                println!("{}", style.key_value("--spec FILE", "Specification file to use"));
                println!("{}", style.key_value("--provider", "AI provider to use"));
                println!("{}", style.key_value("--output", "Output directory"));
                println!();
                println!("{}", style.header("Example"));
                println!("   rice gen --spec my-spec.md");
                println!();
            }
            "chat" => {
                println!("{}", style.section("rice chat - Interactive Chat"));
                println!();
                println!("Start an interactive chat session with RiceCoder.");
                println!();
                println!("{}", style.header("Usage"));
                println!("   rice chat [MESSAGE]");
                println!();
                println!("{}", style.header("Arguments"));
                println!("{}", style.key_value("MESSAGE", "Initial message (optional)"));
                println!();
                println!("{}", style.header("Commands in chat"));
                println!("{}", style.key_value("/exit", "Exit chat mode"));
                println!("{}", style.key_value("/help", "Show chat help"));
                println!("{}", style.key_value("/clear", "Clear chat history"));
                println!();
                println!("{}", style.header("Example"));
                println!("   rice chat");
                println!("   rice chat \"How do I create a REST API?\"");
                println!();
            }
            "config" => {
                println!("{}", style.section("rice config - Manage Configuration"));
                println!();
                println!("View and manage RiceCoder configuration.");
                println!();
                println!("{}", style.header("Usage"));
                println!("   rice config [ACTION]");
                println!();
                println!("{}", style.header("Actions"));
                println!("{}", style.key_value("show", "Show current configuration"));
                println!("{}", style.key_value("set", "Set a configuration value"));
                println!("{}", style.key_value("get", "Get a configuration value"));
                println!();
                println!("{}", style.header("Example"));
                println!("   rice config show");
                println!("   rice config set providers.default anthropic");
                println!();
            }
            _ => {
                println!("{}", style.error(&format!("Unknown command: {}", command)));
                println!();
                println!("Run 'rice help' for available commands.");
            }
        }
    }

    fn show_tutorial(&self) {
        let style = OutputStyle::default();
        println!("{}", style.section("RiceCoder Tutorial"));
        println!();

        println!("{}", style.header("1. Getting Started"));
        println!();
        println!("First, initialize a new project:");
        println!("   rice init my-project");
        println!();
        println!("This creates:");
        println!("{}", style.list_item(".agent/ricecoder.toml - Configuration"));
        println!("{}", style.list_item(".agent/example-spec.md - Example specification"));
        println!("{}", style.list_item("README.md - Project documentation"));
        println!();

        println!("{}", style.header("2. Create a Specification"));
        println!();
        println!("Create a file with your requirements and design:");
        println!();
        println!("   # my-feature.md");
        println!("   ## Requirements");
        println!("   - User can create tasks");
        println!("   - Tasks are persisted to storage");
        println!();
        println!("   ## Design");
        println!("   - Use SQLite for storage");
        println!("   - REST API for task management");
        println!();

        println!("{}", style.header("3. Generate Code"));
        println!();
        println!("Generate code from your specification:");
        println!("   rice gen --spec my-feature.md");
        println!();
        println!("RiceCoder will:");
        println!("{}", style.list_item("Analyze your specification"));
        println!("{}", style.list_item("Generate code based on requirements"));
        println!("{}", style.list_item("Create tests"));
        println!("{}", style.list_item("Generate documentation"));
        println!();

        println!("{}", style.header("4. Review and Refine"));
        println!();
        println!("Review the generated code and refine as needed:");
        println!("   rice chat \"How can I improve this code?\"");
        println!();

        println!("{}", style.header("5. Deploy"));
        println!();
        println!("Once satisfied, deploy your code:");
        println!("   git add .");
        println!("   git commit -m \"Add generated feature\"");
        println!("   git push");
        println!();

        println!("{}", style.section("Tips & Tricks"));
        println!();
        println!("{}", style.tip("Use detailed specifications for better results"));
        println!("{}", style.tip("Include examples in your requirements"));
        println!("{}", style.tip("Review generated code before using"));
        println!("{}", style.tip("Use chat mode for interactive refinement"));
        println!();

        println!("{}", style.section("Learn More"));
        println!();
        println!("{}", style.link("Full Documentation", "https://ricecoder.dev/docs"));
        println!("{}", style.link("Examples", "https://ricecoder.dev/examples"));
        println!("{}", style.link("Best Practices", "https://ricecoder.dev/docs/best-practices"));
        println!();
    }

    fn show_troubleshooting(&self) {
        let style = OutputStyle::default();
        println!("{}", style.section("Troubleshooting"));
        println!();

        println!("{}", style.header("Common Issues"));
        println!();

        println!("{}", style.header("Q: \"Provider error: Invalid API key\""));
        println!();
        println!("A: Check your API key configuration:");
        println!("   1. Run: rice config show");
        println!("   2. Verify your API key is set correctly");
        println!("   3. Check: https://ricecoder.dev/docs/providers");
        println!();

        println!("{}", style.header("Q: \"Configuration error: File not found\""));
        println!();
        println!("A: Initialize your project first:");
        println!("   rice init");
        println!();

        println!("{}", style.header("Q: \"Generation failed: Invalid specification\""));
        println!();
        println!("A: Check your specification format:");
        println!("   1. Review the example: .agent/example-spec.md");
        println!("   2. Ensure all required sections are present");
        println!("   3. Check: https://ricecoder.dev/docs/specifications");
        println!();

        println!("{}", style.header("Q: \"Network error: Connection refused\""));
        println!();
        println!("A: Check your network connection:");
        println!("   1. Verify internet connectivity");
        println!("   2. Check firewall settings");
        println!("   3. Try again later if provider is down");
        println!();

        println!("{}", style.section("Getting Help"));
        println!();
        println!("If you can't find the answer:");
        println!();
        println!("{}", style.list_item("Check the documentation: https://ricecoder.dev/docs"));
        println!("{}", style.list_item("Search GitHub issues: https://github.com/ricecoder/ricecoder/issues"));
        println!("{}", style.list_item("Ask in discussions: https://github.com/ricecoder/ricecoder/discussions"));
        println!();
    }
}

impl Command for HelpCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.topic {
            None => self.show_main_help(),
            Some(topic) => match topic.as_str() {
                "tutorial" => self.show_tutorial(),
                "troubleshooting" => self.show_troubleshooting(),
                _ => self.show_command_help(topic),
            },
        }
        Ok(())
    }
}
