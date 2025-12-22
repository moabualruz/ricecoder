//! Integration tests for the updates crate

pub mod analytics;
pub mod checker;
pub mod error;
pub mod models;
pub mod policy;
pub mod rollback;
pub mod updater;

#[cfg(test)]
mod integration_tests {
    use std::str::FromStr;

    use ricecoder_updates::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_complete_update_workflow() {
        // Create temporary directories
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().join("install");
        let backup_dir = temp_dir.path().join("backups");
        let staging_dir = temp_dir.path().join("staging");

        // Create install directory with a version file
        std::fs::create_dir_all(&install_dir).unwrap();
        std::fs::write(install_dir.join("version.txt"), "1.0.0").unwrap();

        // Create policy
        let policy = UpdatePolicy::default();

        // Create update checker
        let current_version = semver::Version::from_str("1.0.0").unwrap();
        let checker = UpdateChecker::new(
            policy.clone(),
            "https://updates.example.com".to_string(),
            current_version,
        );

        // Create binary updater
        let updater = BinaryUpdater::new(policy.clone(), install_dir.clone());

        // Create rollback manager
        let rollback_manager = RollbackManager::new(backup_dir.clone(), install_dir.clone(), 5);

        // Create analytics collector
        let analytics = AnalyticsCollector::new(
            "https://analytics.example.com".to_string(),
            current_version,
            "linux-x86_64".to_string(),
        );

        // Create staged release manager
        let staged_manager = StagedReleaseManager::new(staging_dir.clone());

        // Test policy evaluation
        let release_info = models::ReleaseInfo {
            version: semver::Version::from_str("1.1.0").unwrap(),
            channel: models::ReleaseChannel::Stable,
            release_date: chrono::Utc::now(),
            minimum_version: None,
            notes: "Test release".to_string(),
            downloads: std::collections::HashMap::new(),
            security_advisories: vec![],
            compliance: models::ComplianceInfo {
                soc2_compliant: true,
                gdpr_compliant: true,
                hipaa_compliant: false,
                security_audited: true,
                last_review: chrono::Utc::now(),
            },
        };

        // Test policy evaluation
        match policy.evaluate_update(&release_info.channel, 50, &[]) {
            policy::PolicyResult::Allowed => {}
            _ => panic!("Policy should allow this update"),
        }

        // Test rollback manager backup creation
        let backup_path = rollback_manager
            .create_backup(&current_version)
            .await
            .unwrap();
        assert!(backup_path.exists());

        // Test backup listing
        let backups = rollback_manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].version, current_version);

        // Test staged release staging
        staged_manager
            .stage_release(&release_info, "stable")
            .await
            .unwrap();

        let staged = staged_manager
            .get_staged_release("stable")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(staged.release_info.version, release_info.version);

        // Test analytics recording
        analytics
            .record_usage(
                300,
                vec!["lsp".to_string()],
                vec!["lsp".to_string()],
                0,
                std::collections::HashMap::new(),
            )
            .await
            .unwrap();

        // Verify analytics event was buffered
        let buffer = analytics.event_buffer.read().await;
        assert_eq!(buffer.len(), 1);

        println!("✅ Complete update workflow test passed");
    }

    #[tokio::test]
    async fn test_enterprise_features() {
        // Create enterprise policy
        let enterprise_config = models::UpdatePolicyConfig {
            auto_update_enabled: false,
            check_interval_hours: 12,
            allowed_channels: vec![models::ReleaseChannel::Stable, models::ReleaseChannel::Beta],
            require_approval: true,
            max_download_size_mb: 50,
            security_requirements: models::SecurityRequirements {
                require_signature: true,
                require_checksum: true,
                allowed_cas: vec!["TestCA".to_string()],
                minimum_security_level: models::SecuritySeverity::High,
            },
            enterprise_settings: Some(models::EnterpriseSettings {
                organization_id: "test-org".to_string(),
                compliance_requirements: vec!["SOC2".to_string(), "GDPR".to_string()],
                custom_update_server: Some("https://enterprise-updates.example.com".to_string()),
                proxy_settings: Some(models::ProxySettings {
                    url: "http://proxy.example.com:8080".to_string(),
                    auth: Some(models::ProxyAuth {
                        username: "user".to_string(),
                        password: "pass".to_string(),
                    }),
                }),
                audit_level: "detailed".to_string(),
            }),
        };

        let policy = UpdatePolicy::new(enterprise_config);

        // Test enterprise features
        assert!(!policy.auto_updates_allowed());
        assert!(policy.channel_allowed(&models::ReleaseChannel::Beta));
        assert!(policy.enterprise_enabled());
        assert_eq!(policy.organization_id(), Some("test-org"));
        assert!(policy.compliance_requirements_met(&["SOC2".to_string(), "GDPR".to_string()]));

        // Create analytics collector
        let current_version = semver::Version::from_str("1.0.0").unwrap();
        let collector = AnalyticsCollector::new(
            "https://analytics.example.com".to_string(),
            current_version,
            "linux-x86_64".to_string(),
        );

        // Create enterprise dashboard
        let dashboard = EnterpriseDashboard::new(collector, "test-org".to_string());

        // Test enterprise reporting
        let report = dashboard.get_usage_report(30).await.unwrap();
        assert_eq!(report.organization_id, "test-org");

        // Test compliance reporting
        let reporter = ComplianceReporter::new(dashboard);
        let soc2_report = reporter.generate_soc2_report(30).await.unwrap();
        assert!(soc2_report.contains("test-org"));

        println!("✅ Enterprise features test passed");
    }

    #[tokio::test]
    async fn test_rollback_validation() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().join("install");
        let backup_dir = temp_dir.path().join("backups");

        std::fs::create_dir_all(&install_dir).unwrap();
        std::fs::write(install_dir.join("version.txt"), "1.0.0").unwrap();

        let rollback_manager = RollbackManager::new(backup_dir, install_dir, 5);

        // Create a backup
        let version = semver::Version::from_str("1.0.0").unwrap();
        rollback_manager.create_backup(&version).await.unwrap();

        // Test rollback validation
        let validation = rollback_manager.validate_rollback(&version).await.unwrap();
        assert!(validation.can_rollback);
        assert!(validation.issues.is_empty());

        // Test rollback plan
        let plan = rollback_manager.get_rollback_plan(&version).await.unwrap();
        assert_eq!(plan.current_version, version);
        assert_eq!(plan.target_version, version);
        assert!(plan.backup_path.is_some());
        assert!(!plan.steps.is_empty());

        println!("✅ Rollback validation test passed");
    }
}
