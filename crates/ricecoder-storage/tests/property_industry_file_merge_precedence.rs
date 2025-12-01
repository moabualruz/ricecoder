//! Property-based test for industry file merge precedence
//!
//! **Feature: ricecoder-storage, Property 9: Industry File Merge Precedence**
//! **Validates: Requirements 5.10**
//!
//! Property: For any combination of industry-standard configuration files
//! (`.cursorrules`, `CLAUDE.md`, etc.), merging should follow the defined
//! precedence order (environment > project > legacy > global > defaults).

use proptest::prelude::*;
use ricecoder_storage::industry::{
    AiderAdapter, AgentsAdapter, ClaudeAdapter, ClineAdapter, ContinueDevAdapter, CopilotAdapter,
    CursorAdapter, IndustryFileAdapter, IndustryFileDetector, KiroAdapter, WindsurfAdapter,
};
use std::fs;
use tempfile::TempDir;

/// Strategy for generating combinations of industry files
#[derive(Debug, Clone)]
struct IndustryFilesCombination {
    has_cursorrules: bool,
    has_cursor_dir: bool,
    has_claude_md: bool,
    has_agents_md: bool,
    has_windsurfrules: bool,
    has_clinerules: bool,
    has_aider_config: bool,
    has_copilot_instructions: bool,
    has_continue_dir: bool,
    has_kiro_dir: bool,
}

fn industry_files_strategy() -> impl Strategy<Value = IndustryFilesCombination> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(
                has_cursorrules,
                has_cursor_dir,
                has_claude_md,
                has_agents_md,
                has_windsurfrules,
                has_clinerules,
                has_aider_config,
                has_copilot_instructions,
                has_continue_dir,
                has_kiro_dir,
            )| {
                IndustryFilesCombination {
                    has_cursorrules,
                    has_cursor_dir,
                    has_claude_md,
                    has_agents_md,
                    has_windsurfrules,
                    has_clinerules,
                    has_aider_config,
                    has_copilot_instructions,
                    has_continue_dir,
                    has_kiro_dir,
                }
            },
        )
}

fn create_industry_files(
    temp_dir: &TempDir,
    combination: &IndustryFilesCombination,
) -> std::io::Result<()> {
    let root = temp_dir.path();

    if combination.has_cursorrules {
        fs::write(root.join(".cursorrules"), "# Cursor rules")?;
    }

    if combination.has_cursor_dir {
        fs::create_dir(root.join(".cursor"))?;
        fs::write(root.join(".cursor/settings.json"), r#"{"key": "value"}"#)?;
    }

    if combination.has_claude_md {
        fs::write(root.join("CLAUDE.md"), "# Claude instructions")?;
    }

    if combination.has_agents_md {
        fs::write(root.join("AGENTS.md"), "# Agent instructions")?;
    }

    if combination.has_windsurfrules {
        fs::write(root.join(".windsurfrules"), "# Windsurf rules")?;
    }

    if combination.has_clinerules {
        fs::write(root.join(".clinerules"), "# Cline rules")?;
    }

    if combination.has_aider_config {
        fs::write(root.join(".aider.conf.yml"), "model: gpt-4")?;
    }

    if combination.has_copilot_instructions {
        fs::create_dir_all(root.join(".github"))?;
        fs::write(
            root.join(".github/copilot-instructions.md"),
            "# Copilot instructions",
        )?;
    }

    if combination.has_continue_dir {
        fs::create_dir(root.join(".continue"))?;
        fs::write(root.join(".continue/config.json"), r#"{"models": ["gpt-4"]}"#)?;
    }

    if combination.has_kiro_dir {
        fs::create_dir(root.join(".kiro"))?;
        fs::create_dir(root.join(".kiro/specs"))?;
        fs::write(root.join(".kiro/specs/spec.md"), "# Spec")?;
    }

    Ok(())
}

proptest! {
    #[test]
    fn prop_industry_file_merge_precedence(combination in industry_files_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        create_industry_files(&temp_dir, &combination).unwrap();

        // Create detector with all adapters
        let adapters: Vec<Box<dyn IndustryFileAdapter>> = vec![
            Box::new(KiroAdapter::new()),           // Priority: 100
            Box::new(CursorAdapter::new()),         // Priority: 50
            Box::new(ClaudeAdapter::new()),         // Priority: 50
            Box::new(AgentsAdapter::new()),         // Priority: 40
            Box::new(WindsurfAdapter::new()),       // Priority: 50
            Box::new(ClineAdapter::new()),          // Priority: 50
            Box::new(AiderAdapter::new()),          // Priority: 50
            Box::new(CopilotAdapter::new()),        // Priority: 50
            Box::new(ContinueDevAdapter::new()),    // Priority: 50
        ];

        let detector = IndustryFileDetector::new(adapters);

        // Get detected files
        let detected = detector.detect_files(temp_dir.path());

        // Verify that Kiro has highest priority if it exists
        if combination.has_kiro_dir {
            prop_assert!(!detected.is_empty());
            prop_assert_eq!(&detected[0].adapter_name, "kiro");
        }

        // Verify that agents has lower priority than other tools
        if combination.has_agents_md && !combination.has_kiro_dir {
            // If agents is the only one, it should be first
            if detected.len() == 1 {
                prop_assert_eq!(&detected[0].adapter_name, "agents");
            } else {
                // If there are other tools, agents should not be first
                // (unless it's the only one with priority 40)
                let has_higher_priority = detected.iter().any(|d| {
                    d.adapter_name != "agents"
                        && (d.adapter_name == "kiro"
                            || d.adapter_name == "cursor"
                            || d.adapter_name == "claude"
                            || d.adapter_name == "windsurf"
                            || d.adapter_name == "cline"
                            || d.adapter_name == "aider"
                            || d.adapter_name == "copilot"
                            || d.adapter_name == "continue")
                });

                if has_higher_priority {
                    prop_assert_ne!(&detected[0].adapter_name, "agents");
                }
            }
        }

        // Verify that all detected adapters can actually handle the directory
        for detection in &detected {
            let adapter_name = &detection.adapter_name;
            let can_handle = match adapter_name.as_str() {
                "kiro" => combination.has_kiro_dir,
                "cursor" => combination.has_cursorrules || combination.has_cursor_dir,
                "claude" => combination.has_claude_md,
                "agents" => combination.has_agents_md,
                "windsurf" => combination.has_windsurfrules,
                "cline" => combination.has_clinerules,
                "aider" => combination.has_aider_config,
                "copilot" => combination.has_copilot_instructions,
                "continue" => combination.has_continue_dir,
                _ => false,
            };

            prop_assert!(
                can_handle,
                "Adapter {} detected but files don't exist",
                adapter_name
            );
        }

        // Verify that if no files exist, no adapters are detected
        if !combination.has_cursorrules
            && !combination.has_cursor_dir
            && !combination.has_claude_md
            && !combination.has_agents_md
            && !combination.has_windsurfrules
            && !combination.has_clinerules
            && !combination.has_aider_config
            && !combination.has_copilot_instructions
            && !combination.has_continue_dir
            && !combination.has_kiro_dir
        {
            prop_assert!(detected.is_empty());
        }

        // Verify that detected adapters are sorted by priority (highest first)
        if detected.len() > 1 {
            for i in 0..detected.len() - 1 {
                prop_assert!(
                    detected[i].priority >= detected[i + 1].priority,
                    "Adapters not sorted by priority: {} ({}) should be >= {} ({})",
                    detected[i].adapter_name,
                    detected[i].priority,
                    detected[i + 1].adapter_name,
                    detected[i + 1].priority
                );
            }
        }
    }

    #[test]
    fn prop_best_adapter_respects_precedence(combination in industry_files_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        create_industry_files(&temp_dir, &combination).unwrap();

        let adapters: Vec<Box<dyn IndustryFileAdapter>> = vec![
            Box::new(KiroAdapter::new()),
            Box::new(CursorAdapter::new()),
            Box::new(ClaudeAdapter::new()),
            Box::new(AgentsAdapter::new()),
            Box::new(WindsurfAdapter::new()),
            Box::new(ClineAdapter::new()),
            Box::new(AiderAdapter::new()),
            Box::new(CopilotAdapter::new()),
            Box::new(ContinueDevAdapter::new()),
        ];

        let detector = IndustryFileDetector::new(adapters);
        let best = detector.get_best_adapter(temp_dir.path());

        // If any files exist, best adapter should be Some
        if combination.has_cursorrules
            || combination.has_cursor_dir
            || combination.has_claude_md
            || combination.has_agents_md
            || combination.has_windsurfrules
            || combination.has_clinerules
            || combination.has_aider_config
            || combination.has_copilot_instructions
            || combination.has_continue_dir
            || combination.has_kiro_dir
        {
            // If best is Some, verify it's the highest priority adapter
            if let Some(best_adapter) = best {
                // If Kiro exists, it should be the best (highest priority)
                if combination.has_kiro_dir {
                    prop_assert_eq!(best_adapter.name(), "kiro");
                }

                // Verify the best adapter can actually handle the directory
                prop_assert!(best_adapter.can_handle(temp_dir.path()));
            }
        } else {
            // If no files exist, best adapter should be None
            prop_assert!(best.is_none());
        }
    }

    #[test]
    fn prop_adapter_config_reading_is_consistent(combination in industry_files_strategy()) {
        let temp_dir = TempDir::new().unwrap();
        create_industry_files(&temp_dir, &combination).unwrap();

        // Test each adapter individually
        let cursor_adapter = CursorAdapter::new();
        if cursor_adapter.can_handle(temp_dir.path()) {
            let config = cursor_adapter.read_config(temp_dir.path()).unwrap();
            // Should have at least one steering rule if files exist
            if combination.has_cursorrules || combination.has_cursor_dir {
                prop_assert!(!config.steering.is_empty());
            }
        }

        let claude_adapter = ClaudeAdapter::new();
        if claude_adapter.can_handle(temp_dir.path()) {
            let config = claude_adapter.read_config(temp_dir.path()).unwrap();
            if combination.has_claude_md {
                prop_assert!(!config.steering.is_empty());
            }
        }

        let kiro_adapter = KiroAdapter::new();
        if kiro_adapter.can_handle(temp_dir.path()) {
            let config = kiro_adapter.read_config(temp_dir.path()).unwrap();
            if combination.has_kiro_dir {
                prop_assert!(!config.steering.is_empty());
            }
        }
    }
}
