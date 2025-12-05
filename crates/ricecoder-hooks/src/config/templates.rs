//! Hook templates and built-in patterns
//!
//! Provides built-in hook templates for common patterns like file save hooks,
//! git hooks, and build hooks. Templates can be instantiated with parameters
//! to create concrete hooks.

use crate::error::{HooksError, Result};
use crate::types::{Action, CommandAction, Hook};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hook template for creating hooks with parameters
///
/// Templates define reusable hook patterns that can be instantiated with
/// specific parameters. This allows users to create hooks without writing
/// full hook configurations.
///
/// # Examples
///
/// ```ignore
/// let template = HookTemplate {
///     name: "Format on Save".to_string(),
///     description: Some("Format code when file is saved".to_string()),
///     event: "file_saved".to_string(),
///     action: Action::Command(CommandAction {
///         command: "prettier".to_string(),
///         args: vec!["--write".to_string(), "{{file_path}}".to_string()],
///         timeout_ms: Some(5000),
///         capture_output: true,
///     }),
///     parameters: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookTemplate {
    /// Template name
    pub name: String,

    /// Template description
    pub description: Option<String>,

    /// Event that triggers hooks from this template
    pub event: String,

    /// Action template (may contain parameter placeholders)
    pub action: Action,

    /// Template parameters
    pub parameters: Vec<TemplateParameter>,
}

/// Template parameter definition
///
/// Defines a parameter that can be customized when instantiating a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,

    /// Parameter description
    pub description: String,

    /// Whether the parameter is required
    pub required: bool,

    /// Default value if not provided
    pub default_value: Option<String>,
}

/// Template manager for creating and managing hook templates
pub struct TemplateManager;

impl TemplateManager {
    /// Get all built-in templates
    ///
    /// Returns a map of template names to template definitions.
    pub fn get_builtin_templates() -> HashMap<String, HookTemplate> {
        let mut templates = HashMap::new();

        // File save template
        templates.insert("file_save".to_string(), Self::create_file_save_template());

        // Git hooks template
        templates.insert("git_hooks".to_string(), Self::create_git_hooks_template());

        // Build hooks template
        templates.insert(
            "build_hooks".to_string(),
            Self::create_build_hooks_template(),
        );

        templates
    }

    /// Create file save template
    ///
    /// Template for running actions when files are saved.
    fn create_file_save_template() -> HookTemplate {
        HookTemplate {
            name: "File Save Hook".to_string(),
            description: Some("Run actions when files are saved".to_string()),
            event: "file_saved".to_string(),
            action: Action::Command(CommandAction {
                command: "{{command}}".to_string(),
                args: vec!["{{file_path}}".to_string()],
                timeout_ms: Some(5000),
                capture_output: true,
            }),
            parameters: vec![TemplateParameter {
                name: "command".to_string(),
                description: "Command to run on file save".to_string(),
                required: true,
                default_value: None,
            }],
        }
    }

    /// Create git hooks template
    ///
    /// Template for running actions on git events.
    fn create_git_hooks_template() -> HookTemplate {
        HookTemplate {
            name: "Git Hooks".to_string(),
            description: Some(
                "Run actions on git events (pre-commit, post-commit, etc.)".to_string(),
            ),
            event: "git_event".to_string(),
            action: Action::Command(CommandAction {
                command: "{{command}}".to_string(),
                args: vec![],
                timeout_ms: Some(10000),
                capture_output: true,
            }),
            parameters: vec![TemplateParameter {
                name: "command".to_string(),
                description: "Command to run on git event".to_string(),
                required: true,
                default_value: None,
            }],
        }
    }

    /// Create build hooks template
    ///
    /// Template for running actions on build events.
    fn create_build_hooks_template() -> HookTemplate {
        HookTemplate {
            name: "Build Hooks".to_string(),
            description: Some(
                "Run actions on build events (pre-build, post-build, etc.)".to_string(),
            ),
            event: "build_event".to_string(),
            action: Action::Command(CommandAction {
                command: "{{command}}".to_string(),
                args: vec![],
                timeout_ms: Some(30000),
                capture_output: true,
            }),
            parameters: vec![TemplateParameter {
                name: "command".to_string(),
                description: "Command to run on build event".to_string(),
                required: true,
                default_value: None,
            }],
        }
    }

    /// Instantiate a template with parameters
    ///
    /// Creates a concrete hook from a template by substituting parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if required parameters are missing or if the template
    /// cannot be instantiated.
    pub fn instantiate_template(
        template: &HookTemplate,
        hook_id: &str,
        hook_name: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<Hook> {
        // Validate required parameters
        for param in &template.parameters {
            if param.required
                && !parameters.contains_key(&param.name)
                && param.default_value.is_none()
            {
                return Err(HooksError::InvalidConfiguration(format!(
                    "Required parameter '{}' not provided for template '{}'",
                    param.name, template.name
                )));
            }
        }

        // Substitute parameters in action
        let action = Self::substitute_action(&template.action, parameters)?;

        Ok(Hook {
            id: hook_id.to_string(),
            name: hook_name.to_string(),
            description: template.description.clone(),
            event: template.event.clone(),
            action,
            enabled: true,
            tags: vec!["template".to_string()],
            metadata: serde_json::json!({
                "template": template.name,
            }),
            condition: None,
        })
    }

    /// Substitute parameters in an action
    fn substitute_action(action: &Action, parameters: &HashMap<String, String>) -> Result<Action> {
        match action {
            Action::Command(cmd) => {
                let command = Self::substitute_string(&cmd.command, parameters);
                let args = cmd
                    .args
                    .iter()
                    .map(|arg| Self::substitute_string(arg, parameters))
                    .collect();

                Ok(Action::Command(CommandAction {
                    command,
                    args,
                    timeout_ms: cmd.timeout_ms,
                    capture_output: cmd.capture_output,
                }))
            }
            // For other action types, return as-is for now
            other => Ok(other.clone()),
        }
    }

    /// Substitute parameters in a string
    ///
    /// Replaces `{{parameter_name}}` with the corresponding parameter value.
    fn substitute_string(template: &str, parameters: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (name, value) in parameters {
            let placeholder = format!("{{{{{}}}}}", name);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_templates() {
        let templates = TemplateManager::get_builtin_templates();
        assert!(templates.contains_key("file_save"));
        assert!(templates.contains_key("git_hooks"));
        assert!(templates.contains_key("build_hooks"));
    }

    #[test]
    fn test_file_save_template() {
        let templates = TemplateManager::get_builtin_templates();
        let template = templates.get("file_save").expect("Should find template");
        assert_eq!(template.name, "File Save Hook");
        assert_eq!(template.event, "file_saved");
        assert_eq!(template.parameters.len(), 1);
        assert_eq!(template.parameters[0].name, "command");
        assert!(template.parameters[0].required);
    }

    #[test]
    fn test_git_hooks_template() {
        let templates = TemplateManager::get_builtin_templates();
        let template = templates.get("git_hooks").expect("Should find template");
        assert_eq!(template.name, "Git Hooks");
        assert_eq!(template.event, "git_event");
    }

    #[test]
    fn test_build_hooks_template() {
        let templates = TemplateManager::get_builtin_templates();
        let template = templates.get("build_hooks").expect("Should find template");
        assert_eq!(template.name, "Build Hooks");
        assert_eq!(template.event, "build_event");
    }

    #[test]
    fn test_instantiate_template_valid() {
        let templates = TemplateManager::get_builtin_templates();
        let template = templates.get("file_save").expect("Should find template");

        let mut params = HashMap::new();
        params.insert("command".to_string(), "prettier".to_string());

        let hook = TemplateManager::instantiate_template(
            template,
            "format-on-save",
            "Format on Save",
            &params,
        )
        .expect("Should instantiate template");

        assert_eq!(hook.id, "format-on-save");
        assert_eq!(hook.name, "Format on Save");
        assert_eq!(hook.event, "file_saved");
        assert!(hook.enabled);
    }

    #[test]
    fn test_instantiate_template_missing_required_parameter() {
        let templates = TemplateManager::get_builtin_templates();
        let template = templates.get("file_save").expect("Should find template");

        let params = HashMap::new();

        let result = TemplateManager::instantiate_template(
            template,
            "format-on-save",
            "Format on Save",
            &params,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_substitute_string() {
        let template = "prettier --write {{file_path}}";
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "/path/to/file.js".to_string());

        let result = TemplateManager::substitute_string(template, &params);
        assert_eq!(result, "prettier --write /path/to/file.js");
    }

    #[test]
    fn test_substitute_string_multiple_parameters() {
        let template = "{{command}} {{file_path}} {{format}}";
        let mut params = HashMap::new();
        params.insert("command".to_string(), "prettier".to_string());
        params.insert("file_path".to_string(), "/path/to/file.js".to_string());
        params.insert("format".to_string(), "json".to_string());

        let result = TemplateManager::substitute_string(template, &params);
        assert_eq!(result, "prettier /path/to/file.js json");
    }

    #[test]
    fn test_substitute_string_no_parameters() {
        let template = "echo hello";
        let params = HashMap::new();

        let result = TemplateManager::substitute_string(template, &params);
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_substitute_action_command() {
        let action = Action::Command(CommandAction {
            command: "{{command}}".to_string(),
            args: vec!["{{file_path}}".to_string()],
            timeout_ms: Some(5000),
            capture_output: true,
        });

        let mut params = HashMap::new();
        params.insert("command".to_string(), "prettier".to_string());
        params.insert("file_path".to_string(), "/path/to/file.js".to_string());

        let result =
            TemplateManager::substitute_action(&action, &params).expect("Should substitute");

        match result {
            Action::Command(cmd) => {
                assert_eq!(cmd.command, "prettier");
                assert_eq!(cmd.args[0], "/path/to/file.js");
            }
            _ => panic!("Expected command action"),
        }
    }
}
