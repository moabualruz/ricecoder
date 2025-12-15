//! Comprehensive enterprise features unit tests
//! **Feature: ricecoder-sessions, Unit Tests: Enterprise Features**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2**

use chrono::Duration;
use ricecoder_sessions::{
    DataClassification, EnterpriseShareMetrics, EnterpriseSharingPolicy, Session, SessionContext,
    SessionMode, SharePermissions, ShareService,
};
use ricecoder_security::audit::{AuditEventType, AuditLogger, MemoryAuditStorage};
use std::sync::Arc;

fn create_test_session(name: &str) -> Session {
    let context = SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat);
    Session::new(name.to_string(), context)
}

fn create_enterprise_policy() -> EnterpriseSharingPolicy {
    EnterpriseSharingPolicy {
        max_expiration_days: Some(30),
        requires_approval: true,
        allowed_domains: vec!["company.com".to_string(), "trusted.org".to_string()],
        compliance_logging: true,
        data_classification: DataClassification::Confidential,
    }
}

fn create_audit_logger() -> Arc<AuditLogger> {
    let storage = Arc::new(MemoryAuditStorage::new());
    Arc::new(AuditLogger::new(storage))
}

#[test]
fn test_enterprise_sharing_policy_validation() {
    let share_service = ShareService::new();

    // Valid policy
    let valid_policy = create_enterprise_policy();
    assert!(share_service.validate_enterprise_policy(&valid_policy).is_ok());

    // Invalid policy - excessive expiration
    let mut invalid_policy = valid_policy.clone();
    invalid_policy.max_expiration_days = Some(400); // Over 365 limit
    assert!(share_service.validate_enterprise_policy(&invalid_policy).is_err());
}

#[test]
fn test_enterprise_sharing_policy_enforcement() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger);

    let session = create_test_session("Enterprise Session");
    let policy = create_enterprise_policy();

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Create share with policy that should cap expiration
    let share = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions,
            Some(Duration::days(60)), // Try to exceed policy limit
            Some(policy),
            Some("user@company.com".to_string()),
        )
        .unwrap();

    // Verify expiration was capped at policy limit
    let expected_max = share.created_at + Duration::days(30);
    assert_eq!(share.expires_at, Some(expected_max));
    assert!(share.policy.is_some());
    assert_eq!(share.creator_user_id, Some("user@company.com".to_string()));
}

#[test]
fn test_enterprise_data_classification_levels() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger);

    let session = create_test_session("Classified Session");

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Test different classification levels
    let classifications = vec![
        DataClassification::Public,
        DataClassification::Internal,
        DataClassification::Confidential,
        DataClassification::Restricted,
    ];

    for classification in classifications {
        let policy = EnterpriseSharingPolicy {
            max_expiration_days: Some(7),
            requires_approval: false,
            allowed_domains: vec!["company.com".to_string()],
            compliance_logging: true,
            data_classification: classification,
        };

        let share = share_service
            .generate_share_link_with_policy(
                &session.id,
                permissions.clone(),
                None,
                Some(policy),
                Some("user@company.com".to_string()),
            )
            .unwrap();

        assert_eq!(share.policy.as_ref().unwrap().data_classification, classification);
    }
}

#[test]
fn test_enterprise_compliance_logging() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger.clone());

    let session = create_test_session("Compliance Session");
    let policy = create_enterprise_policy();

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Generate share (should trigger compliance logging)
    let share = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions,
            None,
            Some(policy),
            Some("user@company.com".to_string()),
        )
        .unwrap();

    // Access the share (should trigger more compliance logging)
    let _retrieved = share_service.get_share(&share.id).unwrap();

    // Check that audit events were logged
    let events = audit_logger.get_storage().get_events().unwrap();
    assert!(!events.is_empty());

    // Should have events for share creation and access
    let share_events: Vec<_> = events.iter()
        .filter(|e| e.resource.contains("share") || e.resource.contains("session"))
        .collect();
    assert!(!share_events.is_empty());
}

#[test]
fn test_enterprise_domain_restrictions() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger);

    let session = create_test_session("Domain Restricted Session");

    let policy = EnterpriseSharingPolicy {
        max_expiration_days: Some(30),
        requires_approval: false,
        allowed_domains: vec!["company.com".to_string(), "partner.org".to_string()],
        compliance_logging: true,
        data_classification: DataClassification::Internal,
    };

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // This test validates that the policy includes domain restrictions
    // In a real implementation, domain validation would happen during access
    let share = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions,
            None,
            Some(policy.clone()),
            Some("user@company.com".to_string()),
        )
        .unwrap();

    assert_eq!(share.policy.as_ref().unwrap().allowed_domains, vec!["company.com".to_string(), "partner.org".to_string()]);
}

#[test]
fn test_enterprise_analytics_tracking() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger);

    let session1 = create_test_session("Analytics Session 1");
    let session2 = create_test_session("Analytics Session 2");

    let policy = create_enterprise_policy();

    let permissions1 = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let permissions2 = SharePermissions {
        read_only: false,
        include_history: false,
        include_context: true,
    };

    // Create shares with different classifications
    let _share1 = share_service
        .generate_share_link_with_policy(
            &session1.id,
            permissions1,
            None,
            Some(policy.clone()),
            Some("user1@company.com".to_string()),
        )
        .unwrap();

    let policy2 = EnterpriseSharingPolicy {
        max_expiration_days: Some(14),
        requires_approval: false,
        allowed_domains: vec!["company.com".to_string()],
        compliance_logging: true,
        data_classification: DataClassification::Restricted,
    };

    let _share2 = share_service
        .generate_share_link_with_policy(
            &session2.id,
            permissions2,
            None,
            Some(policy2),
            Some("user2@company.com".to_string()),
        )
        .unwrap();

    // Check analytics
    let analytics = share_service.get_analytics();
    assert_eq!(analytics.total_shares_created, 2);
    assert!(analytics.enterprise_metrics.is_some());

    let enterprise_metrics = analytics.enterprise_metrics.as_ref().unwrap();
    // Should track different classifications
    assert!(enterprise_metrics.shares_by_classification.len() >= 2);
}

#[test]
fn test_enterprise_share_revocation_with_audit() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger.clone());

    let session = create_test_session("Revocation Session");
    let policy = create_enterprise_policy();

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let share = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions,
            None,
            Some(policy),
            Some("user@company.com".to_string()),
        )
        .unwrap();

    // Revoke the share
    share_service.revoke_share(&share.id, Some("admin@company.com".to_string())).unwrap();

    // Verify share is gone
    assert!(share_service.get_share(&share.id).is_err());

    // Check audit logging for revocation
    let events = audit_logger.get_storage().get_events().unwrap();
    let revoke_events: Vec<_> = events.iter()
        .filter(|e| e.action == "share_revoked")
        .collect();
    assert!(!revoke_events.is_empty());
}

#[test]
fn test_enterprise_features_flag() {
    // Service without enterprise features
    let basic_service = ShareService::new();
    assert!(!basic_service.has_enterprise_features());

    // Service with enterprise features
    let audit_logger = create_audit_logger();
    let enterprise_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger);
    assert!(enterprise_service.has_enterprise_features());
}

#[test]
fn test_enterprise_policy_edge_cases() {
    let share_service = ShareService::new();

    // Policy with no expiration limit
    let policy_no_limit = EnterpriseSharingPolicy {
        max_expiration_days: None,
        requires_approval: false,
        allowed_domains: vec![],
        compliance_logging: false,
        data_classification: DataClassification::Public,
    };
    assert!(share_service.validate_enterprise_policy(&policy_no_limit).is_ok());

    // Policy with maximum allowed expiration
    let policy_max_expiration = EnterpriseSharingPolicy {
        max_expiration_days: Some(365),
        requires_approval: true,
        allowed_domains: vec!["example.com".to_string()],
        compliance_logging: true,
        data_classification: DataClassification::Restricted,
    };
    assert!(share_service.validate_enterprise_policy(&policy_max_expiration).is_ok());

    // Policy with empty domains (should be valid)
    let policy_empty_domains = EnterpriseSharingPolicy {
        max_expiration_days: Some(30),
        requires_approval: false,
        allowed_domains: vec![],
        compliance_logging: false,
        data_classification: DataClassification::Internal,
    };
    assert!(share_service.validate_enterprise_policy(&policy_empty_domains).is_ok());
}

#[test]
fn test_enterprise_compliance_event_types() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger.clone());

    let session = create_test_session("Compliance Event Session");
    let policy = create_enterprise_policy();

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Create share
    let share = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions,
            None,
            Some(policy),
            Some("user@company.com".to_string()),
        )
        .unwrap();

    // Access share
    let _retrieved = share_service.get_share(&share.id).unwrap();

    // Import shared session
    let shared_session = share_service.create_shared_session_view(&session, &permissions);
    let _imported = share_service.import_shared_session(&share.id, &shared_session, Some("importer@company.com".to_string())).unwrap();

    // Check for various compliance events
    let events = audit_logger.get_storage().get_events().unwrap();

    let event_types: std::collections::HashSet<_> = events.iter()
        .map(|e| e.event_type)
        .collect();

    // Should include data access events
    assert!(event_types.contains(&AuditEventType::DataAccess));
}

#[test]
fn test_enterprise_session_invalidation_cascade() {
    let audit_logger = create_audit_logger();
    let share_service = ShareService::with_audit_logging("https://enterprise.ricecoder.com".to_string(), audit_logger);

    let session = create_test_session("Cascade Session");
    let policy = create_enterprise_policy();

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Create multiple shares for the same session
    let share1 = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions.clone(),
            None,
            Some(policy.clone()),
            Some("user1@company.com".to_string()),
        )
        .unwrap();

    let share2 = share_service
        .generate_share_link_with_policy(
            &session.id,
            permissions,
            None,
            Some(policy),
            Some("user2@company.com".to_string()),
        )
        .unwrap();

    // Verify both shares exist
    assert!(share_service.get_share(&share1.id).is_ok());
    assert!(share_service.get_share(&share2.id).is_ok());

    // Invalidate all shares for the session
    let removed = share_service.invalidate_session_shares(&session.id).unwrap();
    assert_eq!(removed, 2);

    // Verify both shares are gone
    assert!(share_service.get_share(&share1.id).is_err());
    assert!(share_service.get_share(&share2.id).is_err());
}</content>
<parameter name="filePath">projects/ricecoder/crates/ricecoder-sessions/tests/enterprise_features_unit_tests.rs