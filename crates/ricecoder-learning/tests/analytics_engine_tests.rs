/// Integration tests for AnalyticsEngine and rule export/import functionality
use ricecoder_learning::{
    AnalyticsEngine, AnalyticsInsights, RuleMetrics, RuleExporter, RuleImporter, RuleExport,
    Rule, RuleScope, RuleSource, LearningManager,
};
use std::path::PathBuf;

fn create_test_rule(id: &str) -> Rule {
    let mut rule = Rule::new(
        RuleScope::Global,
        "test_pattern".to_string(),
        "test_action".to_string(),
        RuleSource::Learned,
    );
    rule.id = id.to_string();
    rule
}

fn create_test_rule_with_scope(id: &str, scope: RuleScope) -> Rule {
    let mut rule = Rule::new(
        scope,
        "test_pattern".to_string(),
        "test_action".to_string(),
        RuleSource::Learned,
    );
    rule.id = id.to_string();
    rule
}

#[tokio::test]
async fn test_analytics_engine_record_single_success() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    let metrics = engine.get_rule_metrics("rule_1").await.unwrap().unwrap();
    assert_eq!(metrics.usage_count, 1);
    assert_eq!(metrics.success_count, 1);
    assert_eq!(metrics.failure_count, 0);
    assert_eq!(metrics.success_rate, 1.0);
}

#[tokio::test]
async fn test_analytics_engine_record_multiple_applications() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    engine
        .record_application("rule_1".to_string(), true, 12.0)
        .await
        .unwrap();
    engine
        .record_application("rule_1".to_string(), false, 8.0)
        .await
        .unwrap();

    let metrics = engine.get_rule_metrics("rule_1").await.unwrap().unwrap();
    assert_eq!(metrics.usage_count, 3);
    assert_eq!(metrics.success_count, 2);
    assert_eq!(metrics.failure_count, 1);
    assert!((metrics.success_rate - 2.0 / 3.0).abs() < 0.01);
}

#[tokio::test]
async fn test_analytics_engine_get_all_metrics() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    engine
        .record_application("rule_2".to_string(), false, 5.0)
        .await
        .unwrap();
    engine
        .record_application("rule_3".to_string(), true, 15.0)
        .await
        .unwrap();

    let all_metrics = engine.get_all_metrics().await.unwrap();
    assert_eq!(all_metrics.len(), 3);
}

#[tokio::test]
async fn test_analytics_engine_update_confidence() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    engine
        .update_confidence("rule_1", 0.95)
        .await
        .unwrap();

    let metrics = engine.get_rule_metrics("rule_1").await.unwrap().unwrap();
    assert_eq!(metrics.confidence, 0.95);
}

#[tokio::test]
async fn test_analytics_engine_invalid_confidence() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    let result = engine.update_confidence("rule_1", 1.5).await;
    assert!(result.is_err());

    let result = engine.update_confidence("rule_1", -0.1).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_analytics_engine_generate_insights_empty() {
    let engine = AnalyticsEngine::new();
    
    let insights = engine.generate_insights().await.unwrap();
    assert_eq!(insights.total_rules, 0);
    assert_eq!(insights.total_applications, 0);
    assert_eq!(insights.avg_success_rate, 0.0);
    assert_eq!(insights.avg_confidence, 0.0);
    assert!(insights.most_used_rule.is_none());
    assert!(insights.least_used_rule.is_none());
}

#[tokio::test]
async fn test_analytics_engine_generate_insights_with_data() {
    let engine = AnalyticsEngine::new();
    
    // Record applications for multiple rules
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    engine
        .record_application("rule_1".to_string(), true, 12.0)
        .await
        .unwrap();
    engine
        .record_application("rule_2".to_string(), false, 5.0)
        .await
        .unwrap();
    engine
        .record_application("rule_3".to_string(), true, 15.0)
        .await
        .unwrap();

    let insights = engine.generate_insights().await.unwrap();
    assert_eq!(insights.total_rules, 3);
    assert_eq!(insights.total_applications, 4);
    assert!(insights.most_used_rule.is_some());
    assert_eq!(insights.most_used_rule.unwrap(), "rule_1");
}

#[tokio::test]
async fn test_analytics_engine_clear_metrics() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    let metrics_before = engine.get_all_metrics().await.unwrap();
    assert_eq!(metrics_before.len(), 1);

    engine.clear_metrics().await.unwrap();

    let metrics_after = engine.get_all_metrics().await.unwrap();
    assert_eq!(metrics_after.len(), 0);
}

#[tokio::test]
async fn test_analytics_engine_get_metrics_by_scope() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    engine
        .record_application("rule_2".to_string(), true, 5.0)
        .await
        .unwrap();

    let mut rules = vec![create_test_rule("rule_1"), create_test_rule("rule_2")];
    rules[0].scope = RuleScope::Global;
    rules[1].scope = RuleScope::Project;

    let global_metrics = engine
        .get_metrics_by_scope(&rules, RuleScope::Global)
        .await
        .unwrap();
    assert_eq!(global_metrics.len(), 1);
    assert_eq!(global_metrics[0].rule_id, "rule_1");
}

#[tokio::test]
async fn test_rule_exporter_export_rules() {
    let rules = vec![create_test_rule("rule_1"), create_test_rule("rule_2")];
    
    let export = RuleExporter::export_rules(rules, Some("Test export".to_string())).unwrap();
    assert_eq!(export.rules.len(), 2);
    assert_eq!(export.metadata.rule_count, 2);
    assert_eq!(export.metadata.version, "1.0");
}

#[tokio::test]
async fn test_rule_exporter_export_to_json() {
    let rules = vec![create_test_rule("rule_1")];
    
    let json = RuleExporter::export_to_json(rules, None).unwrap();
    assert!(json.contains("version") && json.contains("1.0"));
    assert!(json.contains("rule_count") && json.contains("1"));
}

#[tokio::test]
async fn test_rule_exporter_export_to_file() {
    let rules = vec![create_test_rule("rule_1")];
    let temp_file = std::env::temp_dir().join("test_export_analytics.json");

    RuleExporter::export_to_file(rules, &temp_file, None).unwrap();
    assert!(temp_file.exists());

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
}

#[tokio::test]
async fn test_rule_importer_import_from_json() {
    let rules = vec![create_test_rule("rule_1"), create_test_rule("rule_2")];
    let export = RuleExport::new(rules, None);
    let json = export.to_json().unwrap();

    let imported = RuleImporter::import_from_json(&json).unwrap();
    assert_eq!(imported.len(), 2);
}

#[tokio::test]
async fn test_rule_importer_import_from_file() {
    let rules = vec![create_test_rule("rule_1")];
    let export = RuleExport::new(rules, None);
    let temp_file = std::env::temp_dir().join("test_import_analytics.json");

    export.write_to_file(&temp_file).unwrap();

    let imported = RuleImporter::import_from_file(&temp_file).unwrap();
    assert_eq!(imported.len(), 1);

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
}

#[tokio::test]
async fn test_rule_importer_import_and_validate() {
    let mut rules = vec![create_test_rule("rule_1")];
    let mut invalid_rule = create_test_rule("rule_2");
    invalid_rule.pattern = String::new(); // Invalid: empty pattern
    rules.push(invalid_rule);

    let export = RuleExport::new(rules, None);
    let json = export.to_json().unwrap();

    let (valid, errors) = RuleImporter::import_and_validate(&json).unwrap();
    assert_eq!(valid.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("pattern cannot be empty"));
}

#[tokio::test]
async fn test_rule_importer_validate_invalid_confidence() {
    let mut rule = create_test_rule("rule_1");
    rule.confidence = 1.5; // Invalid: > 1.0

    let export = RuleExport::new(vec![rule], None);
    let json = export.to_json().unwrap();

    let (valid, errors) = RuleImporter::import_and_validate(&json).unwrap();
    assert_eq!(valid.len(), 0);
    assert_eq!(errors.len(), 1);
}

#[tokio::test]
async fn test_rule_importer_validate_invalid_success_rate() {
    let mut rule = create_test_rule("rule_1");
    rule.success_rate = -0.1; // Invalid: < 0.0

    let export = RuleExport::new(vec![rule], None);
    let json = export.to_json().unwrap();

    let (valid, errors) = RuleImporter::import_and_validate(&json).unwrap();
    assert_eq!(valid.len(), 0);
    assert_eq!(errors.len(), 1);
}

#[tokio::test]
async fn test_learning_manager_record_rule_application() {
    let manager = LearningManager::new(RuleScope::Global);
    
    manager
        .record_rule_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    let metrics = manager.get_rule_metrics("rule_1").await.unwrap();
    assert!(metrics.is_some());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.usage_count, 1);
    assert_eq!(metrics.success_count, 1);
}

#[tokio::test]
async fn test_learning_manager_get_all_rule_metrics() {
    let manager = LearningManager::new(RuleScope::Global);
    
    manager
        .record_rule_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    manager
        .record_rule_application("rule_2".to_string(), false, 5.0)
        .await
        .unwrap();

    let all_metrics = manager.get_all_rule_metrics().await.unwrap();
    assert_eq!(all_metrics.len(), 2);
}

#[tokio::test]
async fn test_learning_manager_update_rule_confidence() {
    let manager = LearningManager::new(RuleScope::Global);
    
    manager
        .record_rule_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    manager
        .update_rule_confidence("rule_1", 0.85)
        .await
        .unwrap();

    let metrics = manager.get_rule_metrics("rule_1").await.unwrap().unwrap();
    assert_eq!(metrics.confidence, 0.85);
}

#[tokio::test]
async fn test_learning_manager_generate_analytics_insights() {
    let manager = LearningManager::new(RuleScope::Global);
    
    manager
        .record_rule_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    manager
        .record_rule_application("rule_1".to_string(), true, 12.0)
        .await
        .unwrap();
    manager
        .record_rule_application("rule_2".to_string(), false, 5.0)
        .await
        .unwrap();

    let insights = manager.generate_analytics_insights().await.unwrap();
    assert_eq!(insights.total_rules, 2);
    assert_eq!(insights.total_applications, 3);
    assert!(insights.most_used_rule.is_some());
}

#[tokio::test]
async fn test_learning_manager_clear_analytics_metrics() {
    let manager = LearningManager::new(RuleScope::Global);
    
    manager
        .record_rule_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();

    let metrics_before = manager.get_all_rule_metrics().await.unwrap();
    assert_eq!(metrics_before.len(), 1);

    manager.clear_analytics_metrics().await.unwrap();

    let metrics_after = manager.get_all_rule_metrics().await.unwrap();
    assert_eq!(metrics_after.len(), 0);
}

#[tokio::test]
async fn test_learning_manager_export_rules_with_metrics() {
    let manager = LearningManager::new(RuleScope::Session);
    
    let rule = create_test_rule_with_scope("rule_export_1", RuleScope::Session);
    manager.store_rule(rule).await.unwrap();

    let json = manager
        .export_rules_with_metrics(Some("Test export".to_string()))
        .await
        .unwrap();

    assert!(json.contains("version") && json.contains("1.0"));
    assert!(json.contains("rule_count") && json.contains("1"));
}

#[tokio::test]
async fn test_learning_manager_export_rules_to_file() {
    let manager = LearningManager::new(RuleScope::Session);
    
    let rule = create_test_rule_with_scope("export_file_rule_1", RuleScope::Session);
    manager.store_rule(rule).await.unwrap();

    let temp_file = std::env::temp_dir().join("test_manager_export.json");
    manager
        .export_rules_to_file(&temp_file, None)
        .await
        .unwrap();

    assert!(temp_file.exists());

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
}

#[tokio::test]
async fn test_learning_manager_import_rules_from_json() {
    let manager = LearningManager::new(RuleScope::Global);
    
    let rules = vec![create_test_rule("rule_1"), create_test_rule("rule_2")];
    let export = RuleExport::new(rules, None);
    let json = export.to_json().unwrap();

    let imported = manager.import_rules_from_json(&json).await.unwrap();
    assert_eq!(imported.len(), 2);
}

#[tokio::test]
async fn test_learning_manager_import_and_validate_rules() {
    let manager = LearningManager::new(RuleScope::Global);
    
    let mut rules = vec![create_test_rule("rule_1")];
    let mut invalid_rule = create_test_rule("rule_2");
    invalid_rule.action = String::new(); // Invalid: empty action
    rules.push(invalid_rule);

    let export = RuleExport::new(rules, None);
    let json = export.to_json().unwrap();

    let (valid, errors) = manager.import_and_validate_rules(&json).await.unwrap();
    assert_eq!(valid.len(), 1);
    assert_eq!(errors.len(), 1);
}

#[tokio::test]
async fn test_learning_manager_store_imported_rules() {
    let manager = LearningManager::new(RuleScope::Session);
    
    // Create rules with unique IDs
    let mut rules = vec![
        create_test_rule_with_scope("unique_store_import_1", RuleScope::Session),
        create_test_rule_with_scope("unique_store_import_2", RuleScope::Session),
    ];
    
    // Change the pattern to make them unique
    rules[0].pattern = "pattern_1".to_string();
    rules[1].pattern = "pattern_2".to_string();
    
    let stored_ids = manager.store_imported_rules(rules).await.unwrap();

    assert_eq!(stored_ids.len(), 2);
}

#[tokio::test]
async fn test_analytics_engine_average_application_time() {
    let engine = AnalyticsEngine::new();
    
    engine
        .record_application("rule_1".to_string(), true, 10.0)
        .await
        .unwrap();
    engine
        .record_application("rule_1".to_string(), true, 20.0)
        .await
        .unwrap();
    engine
        .record_application("rule_1".to_string(), true, 30.0)
        .await
        .unwrap();

    let metrics = engine.get_rule_metrics("rule_1").await.unwrap().unwrap();
    assert!((metrics.avg_application_time_ms - 20.0).abs() < 0.01);
}

#[tokio::test]
async fn test_analytics_engine_top_and_bottom_performing_rules() {
    let engine = AnalyticsEngine::new();
    
    // Create rules with different success rates
    for i in 1..=5 {
        for _ in 0..i {
            engine
                .record_application(format!("rule_{}", i), true, 10.0)
                .await
                .unwrap();
        }
        // Add one failure to rule_1 to lower its success rate
        if i == 1 {
            engine
                .record_application("rule_1".to_string(), false, 10.0)
                .await
                .unwrap();
        }
    }

    let insights = engine.generate_insights().await.unwrap();
    assert!(!insights.top_performing_rules.is_empty());
    assert!(!insights.bottom_performing_rules.is_empty());
}
