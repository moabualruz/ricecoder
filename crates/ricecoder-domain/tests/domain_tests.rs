//! Unit tests for ricecoder-domain

use proptest::prelude::*;
use ricecoder_domain::*;

#[cfg(test)]
mod tests {
    use super::*;

    mod project_tests {
        use super::*;

        #[test]
        fn test_project_creation_valid() {
            let result = Project::new(
                "test-project".to_string(),
                ProgrammingLanguage::Rust,
                "/path/to/project".to_string(),
            );

            assert!(result.is_ok());
            let project = result.unwrap();
            assert_eq!(project.name, "test-project");
            assert_eq!(project.language, ProgrammingLanguage::Rust);
            assert_eq!(project.root_path, "/path/to/project");
        }

        #[test]
        fn test_project_creation_empty_name() {
            let result = Project::new(
                "".to_string(),
                ProgrammingLanguage::Rust,
                "/path/to/project".to_string(),
            );

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                DomainError::InvalidProjectName { .. }
            ));
        }

        #[test]
        fn test_project_creation_name_too_long() {
            let long_name = "a".repeat(101);
            let result = Project::new(
                long_name,
                ProgrammingLanguage::Rust,
                "/path/to/project".to_string(),
            );

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                DomainError::InvalidProjectName { .. }
            ));
        }

        #[test]
        fn test_project_creation_invalid_characters() {
            let result = Project::new(
                "test project".to_string(), // space not allowed
                ProgrammingLanguage::Rust,
                "/path/to/project".to_string(),
            );

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                DomainError::InvalidProjectName { .. }
            ));
        }

        #[test]
        fn test_project_update_name() {
            let mut project = Project::new(
                "old-name".to_string(),
                ProgrammingLanguage::Rust,
                "/path/to/project".to_string(),
            )
            .unwrap();

            let old_updated = project.updated_at;
            std::thread::sleep(std::time::Duration::from_millis(1)); // ensure time difference

            let result = project.update_name("new-name".to_string());
            assert!(result.is_ok());
            assert_eq!(project.name, "new-name");
            assert!(project.updated_at > old_updated);
        }

        #[test]
        fn test_project_update_description() {
            let mut project = Project::new(
                "test-project".to_string(),
                ProgrammingLanguage::Rust,
                "/path/to/project".to_string(),
            )
            .unwrap();

            let old_updated = project.updated_at;
            std::thread::sleep(std::time::Duration::from_millis(1));

            project.update_description(Some("Test description".to_string()));
            assert_eq!(project.description, Some("Test description".to_string()));
            assert!(project.updated_at > old_updated);
        }
    }

    mod file_tests {
        use super::*;

        #[test]
        fn test_file_creation() {
            let project_id = ProjectId::new();
            let result = CodeFile::new(
                project_id,
                "src/main.rs".to_string(),
                "fn main() {}".to_string(),
                ProgrammingLanguage::Rust,
            );

            assert!(result.is_ok());
            let file = result.unwrap();
            assert_eq!(file.relative_path, "src/main.rs");
            assert_eq!(file.language, ProgrammingLanguage::Rust);
            assert_eq!(file.content, "fn main() {}");
            assert_eq!(file.size_bytes, 12); // "fn main() {}" is 12 characters
        }

        #[test]
        fn test_file_update_content() {
            let project_id = ProjectId::new();
            let mut file = CodeFile::new(
                project_id,
                "src/main.rs".to_string(),
                "fn main() {}".to_string(),
                ProgrammingLanguage::Rust,
            )
            .unwrap();

            let old_modified = file.last_modified;
            std::thread::sleep(std::time::Duration::from_millis(1));

            file.update_content("fn main() { println!(\"Hello\"); }".to_string());

            assert_eq!(file.content, "fn main() { println!(\"Hello\"); }");
            assert_eq!(file.size_bytes, 32);
            assert!(file.last_modified > old_modified);
        }

        #[test]
        fn test_file_extension() {
            let project_id = ProjectId::new();
            let file = CodeFile::new(
                project_id,
                "src/main.rs".to_string(),
                "content".to_string(),
                ProgrammingLanguage::Rust,
            )
            .unwrap();

            assert_eq!(file.extension(), Some("rs"));
        }

        #[test]
        fn test_file_is_empty() {
            let project_id = ProjectId::new();
            let empty_file = CodeFile::new(
                project_id,
                "empty.txt".to_string(),
                "".to_string(),
                ProgrammingLanguage::Other,
            )
            .unwrap();

            let non_empty_file = CodeFile::new(
                project_id,
                "main.rs".to_string(),
                "fn main() {}".to_string(),
                ProgrammingLanguage::Rust,
            )
            .unwrap();

            assert!(empty_file.is_empty());
            assert!(!non_empty_file.is_empty());
        }
    }

    mod session_tests {
        use super::*;

        #[test]
        fn test_session_creation() {
            let session = Session::new("openai".to_string(), "gpt-4".to_string());

            assert_eq!(session.provider_id, "openai");
            assert_eq!(session.model_id, "gpt-4");
            assert_eq!(session.status, SessionStatus::Active);
            assert!(session.is_active());
        }

        #[test]
        fn test_session_set_project() {
            let mut session = Session::new("openai".to_string(), "gpt-4".to_string());
            let project_id = ProjectId::new();

            let old_updated = session.updated_at;
            std::thread::sleep(std::time::Duration::from_millis(1));

            session.set_project(project_id);

            assert_eq!(session.project_id, Some(project_id));
            assert!(session.updated_at > old_updated);
        }

        #[test]
        fn test_session_lifecycle() {
            let mut session = Session::new("openai".to_string(), "gpt-4".to_string());

            // Initially active
            assert!(session.is_active());

            // Pause
            session.pause();
            assert_eq!(session.status, SessionStatus::Paused);
            assert!(!session.is_active());

            // Resume
            session.resume();
            assert_eq!(session.status, SessionStatus::Active);
            assert!(session.is_active());

            // End
            session.end();
            assert_eq!(session.status, SessionStatus::Ended);
            assert!(!session.is_active());
        }

        #[test]
        fn test_session_set_name() {
            let mut session = Session::new("openai".to_string(), "gpt-4".to_string());

            session.set_name("My Session".to_string());
            assert_eq!(session.name, Some("My Session".to_string()));
        }
    }

    mod analysis_tests {
        use super::*;

        #[test]
        fn test_analysis_result_creation() {
            let project_id = ProjectId::new();
            let result = AnalysisResult::new(project_id, None, AnalysisType::Syntax);

            assert_eq!(result.project_id, project_id);
            assert_eq!(result.analysis_type, AnalysisType::Syntax);
            assert_eq!(result.status, AnalysisStatus::Pending);
            assert!(!result.is_complete());
        }

        #[test]
        fn test_analysis_result_complete() {
            let project_id = ProjectId::new();
            let mut result = AnalysisResult::new(project_id, None, AnalysisType::Complexity);

            let metrics = AnalysisMetrics {
                lines_of_code: 100,
                cyclomatic_complexity: 5.0,
                maintainability_index: 85.0,
                technical_debt_ratio: 0.1,
                execution_time_ms: 150,
            };

            let results_data = serde_json::json!({
                "complexity_score": 5.0,
                "maintainability": 85.0
            });

            result.complete(results_data.clone(), metrics.clone());

            assert_eq!(result.status, AnalysisStatus::Completed);
            assert_eq!(result.results, results_data);
            assert_eq!(result.metrics, metrics);
            assert!(result.completed_at.is_some());
            assert!(result.is_complete());
        }

        #[test]
        fn test_analysis_result_fail() {
            let project_id = ProjectId::new();
            let mut result = AnalysisResult::new(project_id, None, AnalysisType::Security);

            result.fail("Analysis failed due to timeout".to_string());

            assert_eq!(result.status, AnalysisStatus::Failed);
            assert_eq!(
                result.results,
                serde_json::Value::String("Analysis failed due to timeout".to_string())
            );
            assert!(result.completed_at.is_some());
            assert!(result.is_complete());
        }
    }

    mod provider_tests {
        use super::*;

        #[test]
        fn test_provider_creation() {
            let provider = Provider::new(
                "openai".to_string(),
                "OpenAI".to_string(),
                ProviderType::OpenAI,
            );

            assert_eq!(provider.id, "openai");
            assert_eq!(provider.name, "OpenAI");
            assert_eq!(provider.provider_type, ProviderType::OpenAI);
            assert!(provider.is_active);
            assert!(provider.models.is_empty());
        }

        #[test]
        fn test_provider_add_model() {
            let mut provider = Provider::new(
                "anthropic".to_string(),
                "Anthropic".to_string(),
                ProviderType::Anthropic,
            );

            let model = ModelInfo::new("claude-3".to_string(), "Claude 3".to_string(), 200000)
                .with_function_calling();

            provider.add_model(model.clone());

            assert_eq!(provider.models.len(), 1);
            assert!(provider.supports_model("claude-3"));
            assert_eq!(provider.get_model("claude-3"), Some(&model));
        }

        #[test]
        fn test_model_info_creation() {
            let model = ModelInfo::new("gpt-4".to_string(), "GPT-4".to_string(), 128000)
                .with_function_calling()
                .with_vision()
                .with_pricing(0.03, 0.06);

            assert_eq!(model.id, "gpt-4");
            assert_eq!(model.name, "GPT-4");
            assert_eq!(model.context_window, 128000);
            assert!(model.supports_function_calling);
            assert!(model.supports_vision);
            assert_eq!(model.cost_per_1m_input, Some(0.03));
            assert_eq!(model.cost_per_1m_output, Some(0.06));
        }
    }

    mod value_object_tests {
        use super::*;

        #[test]
        fn test_project_id_generation() {
            let id1 = ProjectId::new();
            let id2 = ProjectId::new();

            assert_ne!(id1, id2); // Should be unique
        }

        #[test]
        fn test_project_id_from_string() {
            let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
            let id = ProjectId::from_string(uuid_str).unwrap();
            assert_eq!(id.to_string(), uuid_str);
        }

        #[test]
        fn test_programming_language_extensions() {
            assert_eq!(ProgrammingLanguage::Rust.extensions(), &["rs"]);
            assert_eq!(ProgrammingLanguage::Python.extensions(), &["py", "pyw"]);
            assert_eq!(ProgrammingLanguage::TypeScript.extensions(), &["ts"]);
        }

        #[test]
        fn test_language_from_extension() {
            assert_eq!(
                ProgrammingLanguage::from_extension("rs"),
                Some(ProgrammingLanguage::Rust)
            );
            assert_eq!(
                ProgrammingLanguage::from_extension("py"),
                Some(ProgrammingLanguage::Python)
            );
            assert_eq!(
                ProgrammingLanguage::from_extension("ts"),
                Some(ProgrammingLanguage::TypeScript)
            );
            assert_eq!(ProgrammingLanguage::from_extension("xyz"), None);
        }

        #[test]
        fn test_semantic_version() {
            let version = SemanticVersion::new(1, 2, 3);
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
            assert_eq!(version.to_string(), "1.2.3");

            let parsed = SemanticVersion::parse("2.0.1").unwrap();
            assert_eq!(parsed.major, 2);
            assert_eq!(parsed.minor, 0);
            assert_eq!(parsed.patch, 1);
        }

        #[test]
        fn test_valid_url() {
            let url = ValidUrl::parse("https://api.openai.com/v1").unwrap();
            assert_eq!(url.as_url().host_str(), Some("api.openai.com"));
        }

        #[test]
        fn test_mime_type() {
            let mime = MimeType::from_path("main.rs");
            assert_eq!(mime.as_str(), "text/x-rust"); // rust files are detected as text/x-rust

            let json_mime = MimeType::from_path("config.json");
            assert_eq!(json_mime.as_str(), "application/json");
        }
    }

    mod business_rules_tests {
        use super::*;

        #[test]
        fn test_project_creation_validation() {
            // Valid project
            let result = BusinessRulesValidator::validate_project_creation(
                "my-rust-project",
                ProgrammingLanguage::Rust,
            );
            assert!(result.is_valid);

            // Short name warning
            let result =
                BusinessRulesValidator::validate_project_creation("x", ProgrammingLanguage::Rust);
            assert!(result.is_valid);
            assert!(!result.warnings.is_empty());

            // Rust naming convention warning
            let result = BusinessRulesValidator::validate_project_creation(
                "MY_PROJECT",
                ProgrammingLanguage::Rust,
            );
            assert!(result.is_valid);
            assert!(result.warnings.iter().any(|w| w.contains("lowercase")));
        }

        #[test]
        fn test_session_operation_validation() {
            let session = Session::new("openai".to_string(), "gpt-4".to_string());

            // Valid end operation
            let result =
                BusinessRulesValidator::validate_session_operation(&session, SessionOperation::End);
            assert!(result.is_valid);

            // Invalid resume on active session
            let result = BusinessRulesValidator::validate_session_operation(
                &session,
                SessionOperation::Resume,
            );
            assert!(!result.is_valid);
        }

        #[test]
        fn test_analysis_operation_validation() {
            let project_id = ProjectId::new();
            let small_file = CodeFile::new(
                project_id,
                "small.rs".to_string(),
                "fn main() {}".to_string(),
                ProgrammingLanguage::Rust,
            )
            .unwrap();

            // Complexity analysis on small file
            let result = BusinessRulesValidator::validate_analysis_operation(
                &small_file,
                AnalysisType::Complexity,
            );
            assert!(result.is_valid);
            assert!(!result.warnings.is_empty()); // Should warn about small file
        }
    }
}

// Property-based tests
proptest! {
    #[test]
    fn test_project_name_validation_prop(name in "[a-zA-Z0-9_-]{1,100}") {
        let result = Project::new(
            name.clone(),
            ProgrammingLanguage::Rust,
            "/test/path".to_string(),
        );

        // Valid names should succeed
        if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') && !name.is_empty() {
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn test_semantic_version_parsing_prop(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100
    ) {
        let version_str = format!("{}.{}.{}", major, minor, patch);
        let parsed = SemanticVersion::parse(&version_str).unwrap();
        prop_assert_eq!(parsed.major, major);
        prop_assert_eq!(parsed.minor, minor);
        prop_assert_eq!(parsed.patch, patch);
    }

    #[test]
    fn test_file_content_update_prop(content in ".{0,1000}") {
        let project_id = ProjectId::new();
        let mut file = CodeFile::new(
            project_id,
            "test.rs".to_string(),
            "initial".to_string(),
            ProgrammingLanguage::Rust,
        ).unwrap();

        file.update_content(content.clone());
        prop_assert_eq!(file.content, content.clone());
        prop_assert_eq!(file.size_bytes, content.len());
    }

    #[test]
    fn test_session_metadata_prop(
        key in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
        value in r#"[a-zA-Z0-9\s]{0,100}"#
    ) {
        let mut session = Session::new("test".to_string(), "model".to_string());

        session.metadata.insert(key.clone(), serde_json::Value::String(value.clone()));

        prop_assert_eq!(
            session.metadata.get(&key).unwrap(),
            &serde_json::Value::String(value)
        );
    }
}
