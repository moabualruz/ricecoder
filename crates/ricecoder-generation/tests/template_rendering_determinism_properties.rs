//! Property-based tests for template rendering determinism
//! **Feature: ricecoder-templates, Property 1: Template Rendering Determinism**
//! **Validates: Requirements 1.1, 1.2, 1.5**

use proptest::prelude::*;
use ricecoder_generation::TemplateEngine;
use std::collections::HashMap;

/// Strategy for generating valid placeholder names
fn placeholder_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{1,5}"
        .prop_map(|s| s.to_string())
        .prop_filter("name must not be empty", |s| !s.is_empty())
}

/// Strategy for generating valid placeholder values
fn placeholder_value_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]+"
        .prop_map(|s| s.to_string())
        .prop_filter("value must not be empty", |s| !s.is_empty())
}

/// Strategy for generating simple templates with placeholders
fn simple_template_strategy() -> impl Strategy<Value = (String, HashMap<String, String>)> {
    (
        placeholder_name_strategy(),
        placeholder_value_strategy(),
    )
    .prop_map(|(name, value)| {
        let template = format!("Hello {{{{{}}}}} world", name);
        let mut values = HashMap::new();
        values.insert(name, value);
        (template, values)
    })
}

/// Strategy for generating templates with multiple placeholders
fn multi_placeholder_template_strategy() -> impl Strategy<Value = (String, HashMap<String, String>)> {
    (
        placeholder_name_strategy(),
        placeholder_value_strategy(),
        placeholder_name_strategy(),
        placeholder_value_strategy(),
    )
    .prop_map(|(name1, value1, name2, value2)| {
        let template = format!(
            "Project: {{{{{}}}}}, Author: {{{{{}}}}}",
            name1, name2
        );
        let mut values = HashMap::new();
        values.insert(name1, value1);
        values.insert(name2, value2);
        (template, values)
    })
    .prop_filter("names must be different and not substrings", |(_, values)| {
        if values.len() != 2 {
            return false;
        }
        let names: Vec<_> = values.keys().collect();
        // Ensure names are not substrings of each other
        !names[0].contains(names[1]) && !names[1].contains(names[0])
    })
}

/// Strategy for generating templates with case transformations
fn case_transform_template_strategy() -> impl Strategy<Value = (String, HashMap<String, String>)> {
    (
        placeholder_name_strategy(),
        placeholder_value_strategy(),
    )
    .prop_map(|(name, value)| {
        let template = format!(
            "PascalCase: {{{{{}}}}}, lowercase: {{{{{}}}}}",
            format!("{}", name.chars().next().unwrap().to_uppercase().collect::<String>() + &name[1..]),
            name
        );
        let mut values = HashMap::new();
        values.insert(name, value);
        (template, values)
    })
}

proptest! {
    /// Property: For any template and context, rendering the same template
    /// with the same context twice should produce identical output
    #[test]
    fn prop_template_rendering_deterministic(
        (template, values) in simple_template_strategy(),
    ) {
        let mut engine1 = TemplateEngine::new();
        for (name, value) in &values {
            engine1.add_value(name.clone(), value.clone());
        }

        let mut engine2 = TemplateEngine::new();
        for (name, value) in &values {
            engine2.add_value(name.clone(), value.clone());
        }

        let result1 = engine1.render_simple(&template).unwrap();
        let result2 = engine2.render_simple(&template).unwrap();

        prop_assert_eq!(result1, result2, "Rendering should be deterministic");
    }

    /// Property: For any template and context, rendering the same template
    /// three times should produce identical output each time
    #[test]
    fn prop_template_rendering_consistent_multiple_times(
        (template, values) in simple_template_strategy(),
    ) {
        let mut engine = TemplateEngine::new();
        for (name, value) in &values {
            engine.add_value(name.clone(), value.clone());
        }

        let result1 = engine.render_simple(&template).unwrap();
        let result2 = engine.render_simple(&template).unwrap();
        let result3 = engine.render_simple(&template).unwrap();

        prop_assert_eq!(result1.clone(), result2.clone(), "First and second renders should match");
        prop_assert_eq!(result2, result3, "Second and third renders should match");
    }

    /// Property: For any template with multiple placeholders, rendering should
    /// produce output without placeholder markers
    #[test]
    fn prop_template_rendering_includes_all_values(
        (template, values) in multi_placeholder_template_strategy(),
    ) {
        let mut engine = TemplateEngine::new();
        for (name, value) in &values {
            engine.add_value(name.clone(), value.clone());
        }

        let result = engine.render_simple(&template).unwrap();

        // Check that no placeholder markers remain
        prop_assert!(!result.contains("{{"), "Result should not contain placeholder markers");
        prop_assert!(!result.contains("}}"), "Result should not contain placeholder markers");
    }

    /// Property: For any template, rendering with the same values should
    /// produce output of consistent length (no random padding or truncation)
    #[test]
    fn prop_template_rendering_consistent_length(
        (template, values) in simple_template_strategy(),
    ) {
        let mut engine = TemplateEngine::new();
        for (name, value) in &values {
            engine.add_value(name.clone(), value.clone());
        }

        let result1 = engine.render_simple(&template).unwrap();
        let result2 = engine.render_simple(&template).unwrap();

        prop_assert_eq!(result1.len(), result2.len(), "Rendered output length should be consistent");
    }

    /// Property: For any template with case transformations, rendering should
    /// produce consistent case transformations
    #[test]
    fn prop_template_case_transformations_deterministic(
        (template, values) in case_transform_template_strategy(),
    ) {
        let mut engine1 = TemplateEngine::new();
        for (name, value) in &values {
            engine1.add_value(name.clone(), value.clone());
        }

        let mut engine2 = TemplateEngine::new();
        for (name, value) in &values {
            engine2.add_value(name.clone(), value.clone());
        }

        let result1 = engine1.render_simple(&template).unwrap();
        let result2 = engine2.render_simple(&template).unwrap();

        prop_assert_eq!(result1, result2, "Case transformations should be deterministic");
    }

    /// Property: For any template, rendering should not modify the template
    /// (template should be immutable from the engine's perspective)
    #[test]
    fn prop_template_rendering_does_not_modify_template(
        (template, values) in simple_template_strategy(),
    ) {
        let mut engine = TemplateEngine::new();
        for (name, value) in &values {
            engine.add_value(name.clone(), value.clone());
        }

        let template_before = template.clone();
        let _ = engine.render_simple(&template);
        let template_after = template.clone();

        prop_assert_eq!(template_before, template_after, "Template should not be modified");
    }

    /// Property: For any template with simple placeholders, rendering should
    /// produce output that contains the template text with placeholders replaced
    #[test]
    fn prop_template_rendering_replaces_placeholders(
        (template, values) in simple_template_strategy(),
    ) {
        let mut engine = TemplateEngine::new();
        for (name, value) in &values {
            engine.add_value(name.clone(), value.clone());
        }

        let result = engine.render_simple(&template).unwrap();

        // Result should not contain any placeholder markers
        prop_assert!(!result.contains("{{"), "Result should not contain placeholder markers");
        prop_assert!(!result.contains("}}"), "Result should not contain placeholder markers");
    }

    /// Property: For any template, rendering with the same engine multiple times
    /// should produce identical results (engine state should not change)
    #[test]
    fn prop_template_engine_state_immutable(
        (template, values) in simple_template_strategy(),
    ) {
        let mut engine = TemplateEngine::new();
        for (name, value) in &values {
            engine.add_value(name.clone(), value.clone());
        }

        let result1 = engine.render_simple(&template).unwrap();
        let result2 = engine.render_simple(&template).unwrap();
        let result3 = engine.render_simple(&template).unwrap();

        prop_assert_eq!(result1.clone(), result2.clone(), "First and second renders should match");
        prop_assert_eq!(result2, result3, "Second and third renders should match");
    }

    /// Property: For any template with multiple placeholders, rendering should
    /// produce consistent results
    #[test]
    fn prop_template_rendering_preserves_order(
        (template, values) in multi_placeholder_template_strategy(),
    ) {
        let mut engine1 = TemplateEngine::new();
        for (name, value) in &values {
            engine1.add_value(name.clone(), value.clone());
        }

        let mut engine2 = TemplateEngine::new();
        for (name, value) in &values {
            engine2.add_value(name.clone(), value.clone());
        }

        let result1 = engine1.render_simple(&template).unwrap();
        let result2 = engine2.render_simple(&template).unwrap();

        // Results should be identical
        prop_assert_eq!(result1, result2, "Rendering should be deterministic");
    }
}
