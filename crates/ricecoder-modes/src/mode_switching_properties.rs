//! Property-based tests for mode switching and context preservation
//!
//! **Feature: ricecoder-modes, Property 3: Mode Context Preservation**
//! **Validates: Requirements 2.5**

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::error::Result;
    use crate::mode::Mode;
    use crate::models::{
        Capability, ModeConfig, ModeConstraints, ModeContext, ModeResponse, Operation,
    };

    struct TestMode {
        id: String,
        config: ModeConfig,
    }

    #[async_trait::async_trait]
    impl Mode for TestMode {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            "Test Mode"
        }

        fn description(&self) -> &str {
            "A test mode"
        }

        fn system_prompt(&self) -> &str {
            "You are a test mode"
        }

        async fn process(&self, _input: &str, _context: &ModeContext) -> Result<ModeResponse> {
            Ok(ModeResponse::new(
                "Test response".to_string(),
                self.id.clone(),
            ))
        }

        fn capabilities(&self) -> Vec<Capability> {
            vec![Capability::QuestionAnswering]
        }

        fn config(&self) -> &ModeConfig {
            &self.config
        }

        fn can_execute(&self, _operation: &Operation) -> bool {
            true
        }

        fn constraints(&self) -> ModeConstraints {
            ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            }
        }
    }

    /// Property 3: Mode Context Preservation
    ///
    /// For any mode switch, all session context (conversation history, project state, user preferences)
    /// SHALL be preserved and accessible after the switch.
    ///
    /// **Validates: Requirements 2.5**
    #[tokio::test]
    async fn prop_mode_context_preserved_on_switch() {
        // Test with multiple iterations to simulate property-based testing
        for i in 0..10 {
            let session_id = format!("session-{}", i);
            let mode1_id = "mode1";
            let mode2_id = "mode2";
            let message_content = format!("Test message {}", i);
            // Create a mode switcher with initial context
            let context = ModeContext::new(session_id.clone());
            let mut switcher = crate::ModeSwitcher::new(context);

            // Create two test modes
            let mode1 = Arc::new(TestMode {
                id: mode1_id.to_string(),
                config: ModeConfig {
                    temperature: 0.7,
                    max_tokens: 1000,
                    system_prompt: "Mode 1".to_string(),
                    capabilities: vec![Capability::QuestionAnswering],
                    constraints: ModeConstraints {
                        allow_file_operations: false,
                        allow_command_execution: false,
                        allow_code_generation: false,
                        require_specs: false,
                        auto_think_more_threshold: None,
                    },
                },
            });

            let mode2 = Arc::new(TestMode {
                id: mode2_id.to_string(),
                config: ModeConfig {
                    temperature: 0.7,
                    max_tokens: 1000,
                    system_prompt: "Mode 2".to_string(),
                    capabilities: vec![Capability::QuestionAnswering],
                    constraints: ModeConstraints {
                        allow_file_operations: false,
                        allow_command_execution: false,
                        allow_code_generation: false,
                        require_specs: false,
                        auto_think_more_threshold: None,
                    },
                },
            });

            switcher.register_mode(mode1);
            switcher.register_mode(mode2);

            // Switch to mode1
            switcher.switch_mode(mode1_id).await.unwrap();

            // Add a message to the context
            switcher
                .update_context(|ctx| {
                    ctx.add_message(crate::models::MessageRole::User, message_content.clone());
                })
                .await
                .unwrap();

            // Verify the message was added
            let ctx1 = switcher.context().await;
            assert_eq!(ctx1.conversation_history.len(), 1);
            assert_eq!(ctx1.conversation_history[0].content, message_content);
            assert_eq!(ctx1.session_id, session_id);

            // Switch to mode2
            switcher.switch_mode(mode2_id).await.unwrap();

            // Context should be empty for mode2 (fresh context)
            let ctx2 = switcher.context().await;
            assert_eq!(ctx2.conversation_history.len(), 0);
            assert_eq!(ctx2.session_id, session_id);

            // Switch back to mode1
            switcher.switch_mode(mode1_id).await.unwrap();

            // Context should be restored with the original message
            let ctx1_restored = switcher.context().await;
            assert_eq!(ctx1_restored.conversation_history.len(), 1);
            assert_eq!(
                ctx1_restored.conversation_history[0].content,
                message_content
            );
            assert_eq!(ctx1_restored.session_id, session_id);

            // Session ID should be preserved throughout
            assert_eq!(ctx1_restored.session_id, session_id);
        }
    }

    /// Property: Multiple mode switches preserve context
    ///
    /// For any sequence of mode switches, each mode's context should be preserved
    /// and restored correctly when switching back to that mode.
    #[tokio::test]
    async fn prop_multiple_mode_switches_preserve_context() {
        for i in 0..5 {
            let session_id = format!("session-{}", i);
            let mode1_id = "mode1";
            let mode2_id = "mode2";
            let mode3_id = "mode3";
            let msg1 = format!("Message 1 - {}", i);
            let msg2 = format!("Message 2 - {}", i);
            let msg3 = format!("Message 3 - {}", i);

            let context = ModeContext::new(session_id.clone());
            let mut switcher = crate::ModeSwitcher::new(context);

            // Create three test modes
            for mode_id in &[mode1_id, mode2_id, mode3_id] {
                let mode = Arc::new(TestMode {
                    id: mode_id.to_string(),
                    config: ModeConfig {
                        temperature: 0.7,
                        max_tokens: 1000,
                        system_prompt: format!("Mode {}", mode_id),
                        capabilities: vec![Capability::QuestionAnswering],
                        constraints: ModeConstraints {
                            allow_file_operations: false,
                            allow_command_execution: false,
                            allow_code_generation: false,
                            require_specs: false,
                            auto_think_more_threshold: None,
                        },
                    },
                });
                switcher.register_mode(mode);
            }

            // Switch to mode1 and add message
            switcher.switch_mode(mode1_id).await.unwrap();
            switcher
                .update_context(|ctx| {
                    ctx.add_message(crate::models::MessageRole::User, msg1.clone());
                })
                .await
                .unwrap();

            // Switch to mode2 and add message
            switcher.switch_mode(mode2_id).await.unwrap();
            switcher
                .update_context(|ctx| {
                    ctx.add_message(crate::models::MessageRole::User, msg2.clone());
                })
                .await
                .unwrap();

            // Switch to mode3 and add message
            switcher.switch_mode(mode3_id).await.unwrap();
            switcher
                .update_context(|ctx| {
                    ctx.add_message(crate::models::MessageRole::User, msg3.clone());
                })
                .await
                .unwrap();

            // Verify mode3 context
            let ctx3 = switcher.context().await;
            assert_eq!(ctx3.conversation_history.len(), 1);
            assert_eq!(ctx3.conversation_history[0].content, msg3);

            // Switch back to mode1
            switcher.switch_mode(mode1_id).await.unwrap();
            let ctx1 = switcher.context().await;
            assert_eq!(ctx1.conversation_history.len(), 1);
            assert_eq!(ctx1.conversation_history[0].content, msg1);

            // Switch back to mode2
            switcher.switch_mode(mode2_id).await.unwrap();
            let ctx2 = switcher.context().await;
            assert_eq!(ctx2.conversation_history.len(), 1);
            assert_eq!(ctx2.conversation_history[0].content, msg2);

            // Switch back to mode3
            switcher.switch_mode(mode3_id).await.unwrap();
            let ctx3_restored = switcher.context().await;
            assert_eq!(ctx3_restored.conversation_history.len(), 1);
            assert_eq!(ctx3_restored.conversation_history[0].content, msg3);
        }
    }

    /// Property: Context is independent between modes
    ///
    /// For any two modes, changes to context in one mode should not affect
    /// the context of the other mode.
    #[tokio::test]
    async fn prop_context_independent_between_modes() {
        for i in 0..10 {
            let session_id = format!("session-{}", i);
            let mode1_id = "mode1";
            let mode2_id = "mode2";
            let msg1 = format!("Message 1 - {}", i);
            let msg2 = format!("Message 2 - {}", i);

            let context = ModeContext::new(session_id.clone());
            let mut switcher = crate::ModeSwitcher::new(context);

            let mode1 = Arc::new(TestMode {
                id: mode1_id.to_string(),
                config: ModeConfig {
                    temperature: 0.7,
                    max_tokens: 1000,
                    system_prompt: "Mode 1".to_string(),
                    capabilities: vec![Capability::QuestionAnswering],
                    constraints: ModeConstraints {
                        allow_file_operations: false,
                        allow_command_execution: false,
                        allow_code_generation: false,
                        require_specs: false,
                        auto_think_more_threshold: None,
                    },
                },
            });

            let mode2 = Arc::new(TestMode {
                id: mode2_id.to_string(),
                config: ModeConfig {
                    temperature: 0.7,
                    max_tokens: 1000,
                    system_prompt: "Mode 2".to_string(),
                    capabilities: vec![Capability::QuestionAnswering],
                    constraints: ModeConstraints {
                        allow_file_operations: false,
                        allow_command_execution: false,
                        allow_code_generation: false,
                        require_specs: false,
                        auto_think_more_threshold: None,
                    },
                },
            });

            switcher.register_mode(mode1);
            switcher.register_mode(mode2);

            // Switch to mode1 and add message
            switcher.switch_mode(mode1_id).await.unwrap();
            switcher
                .update_context(|ctx| {
                    ctx.add_message(crate::models::MessageRole::User, msg1.clone());
                })
                .await
                .unwrap();

            // Switch to mode2 and add different message
            switcher.switch_mode(mode2_id).await.unwrap();
            switcher
                .update_context(|ctx| {
                    ctx.add_message(crate::models::MessageRole::User, msg2.clone());
                })
                .await
                .unwrap();

            // Verify mode2 has only its message
            let ctx2 = switcher.context().await;
            assert_eq!(ctx2.conversation_history.len(), 1);
            assert_eq!(ctx2.conversation_history[0].content, msg2);

            // Switch back to mode1
            switcher.switch_mode(mode1_id).await.unwrap();

            // Verify mode1 still has only its original message
            let ctx1 = switcher.context().await;
            assert_eq!(ctx1.conversation_history.len(), 1);
            assert_eq!(ctx1.conversation_history[0].content, msg1);
        }
    }
}
