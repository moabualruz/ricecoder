//! Property-based tests for value objects
//!
//! REQ-ARCH-002.15 through REQ-ARCH-002.17: LSP validation via property testing
//!
//! These tests verify that value objects maintain their invariants across
//! all possible inputs and operations.

use proptest::prelude::*;
use ricecoder_domain::value_objects::*;

// ============================================================================
// ProjectId Property Tests
// ============================================================================

proptest! {
    /// ProjectId roundtrip: serialize -> deserialize == original
    #[test]
    fn test_project_id_roundtrip(_dummy in 0u8..1) {
        let id = ProjectId::new();
        let serialized = id.to_string();
        let deserialized = ProjectId::from_string(&serialized).unwrap();
        prop_assert_eq!(id, deserialized);
    }

    /// ProjectId serialization is deterministic
    #[test]
    fn test_project_id_serialization_deterministic(_dummy in 0u8..1) {
        let id = ProjectId::new();
        let s1 = id.to_string();
        let s2 = id.to_string();
        prop_assert_eq!(s1, s2);
    }

    /// ProjectId JSON roundtrip
    #[test]
    fn test_project_id_json_roundtrip(_dummy in 0u8..1) {
        let id = ProjectId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ProjectId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, deserialized);
    }
}

// ============================================================================
// SessionId Property Tests
// ============================================================================

proptest! {
    /// SessionId roundtrip: serialize -> deserialize == original
    #[test]
    fn test_session_id_roundtrip(_dummy in 0u8..1) {
        let id = SessionId::new();
        let serialized = id.to_string();
        let deserialized = SessionId::from_string(&serialized).unwrap();
        prop_assert_eq!(id, deserialized);
    }

    /// SessionId serialization is deterministic
    #[test]
    fn test_session_id_serialization_deterministic(_dummy in 0u8..1) {
        let id = SessionId::new();
        let s1 = id.to_string();
        let s2 = id.to_string();
        prop_assert_eq!(s1, s2);
    }

    /// SessionId JSON roundtrip
    #[test]
    fn test_session_id_json_roundtrip(_dummy in 0u8..1) {
        let id = SessionId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: SessionId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, deserialized);
    }
}

// ============================================================================
// FileId Property Tests
// ============================================================================

proptest! {
    /// FileId preserves path through operations
    #[test]
    fn test_file_id_preserves_path(path in "[a-zA-Z0-9/_.-]{1,256}") {
        let id = FileId::from_path(&path);
        prop_assert_eq!(id.as_path(), path);
    }

    /// FileId JSON roundtrip
    #[test]
    fn test_file_id_json_roundtrip(path in "[a-zA-Z0-9/_.-]{1,256}") {
        let id = FileId::from_path(&path);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: FileId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, deserialized);
    }

    /// FileId equality is reflexive
    #[test]
    fn test_file_id_equality_reflexive(path in "[a-zA-Z0-9/_.-]{1,256}") {
        let id = FileId::from_path(&path);
        prop_assert_eq!(&id, &id);
    }

    /// FileId equality is symmetric
    #[test]
    fn test_file_id_equality_symmetric(path in "[a-zA-Z0-9/_.-]{1,256}") {
        let id1 = FileId::from_path(&path);
        let id2 = FileId::from_path(&path);
        prop_assert_eq!(&id1, &id2);
        prop_assert_eq!(&id2, &id1);
    }
}

// ============================================================================
// ProgrammingLanguage Property Tests
// ============================================================================

proptest! {
    /// ProgrammingLanguage roundtrip: extension -> language -> extension
    #[test]
    fn test_programming_language_extension_roundtrip(
        lang in prop_oneof![
            Just(ProgrammingLanguage::Rust),
            Just(ProgrammingLanguage::Python),
            Just(ProgrammingLanguage::JavaScript),
            Just(ProgrammingLanguage::TypeScript),
            Just(ProgrammingLanguage::Java),
            Just(ProgrammingLanguage::CSharp),
            Just(ProgrammingLanguage::Go),
            Just(ProgrammingLanguage::C),
            Just(ProgrammingLanguage::Cpp),
        ]
    ) {
        // Get first extension for this language
        if let Some(ext) = lang.extensions().first() {
            // Detect language from extension
            let detected = ProgrammingLanguage::from_extension(ext);
            prop_assert_eq!(detected, Some(lang));
        }
    }

    /// ProgrammingLanguage extensions are non-empty (except Other)
    #[test]
    fn test_programming_language_extensions_non_empty(
        lang in prop_oneof![
            Just(ProgrammingLanguage::Rust),
            Just(ProgrammingLanguage::Python),
            Just(ProgrammingLanguage::JavaScript),
            Just(ProgrammingLanguage::TypeScript),
            Just(ProgrammingLanguage::Java),
            Just(ProgrammingLanguage::CSharp),
            Just(ProgrammingLanguage::Go),
            Just(ProgrammingLanguage::C),
            Just(ProgrammingLanguage::Cpp),
        ]
    ) {
        prop_assert!(!lang.extensions().is_empty());
    }

    /// ProgrammingLanguage JSON roundtrip
    #[test]
    fn test_programming_language_json_roundtrip(
        lang in prop_oneof![
            Just(ProgrammingLanguage::Rust),
            Just(ProgrammingLanguage::Python),
            Just(ProgrammingLanguage::JavaScript),
            Just(ProgrammingLanguage::TypeScript),
            Just(ProgrammingLanguage::Java),
            Just(ProgrammingLanguage::Go),
        ]
    ) {
        let json = serde_json::to_string(&lang).unwrap();
        let deserialized: ProgrammingLanguage = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(lang, deserialized);
    }
}

// ============================================================================
// SemanticVersion Property Tests
// ============================================================================

proptest! {
    /// SemanticVersion roundtrip: format -> parse == original
    #[test]
    fn test_semantic_version_roundtrip(
        major in 0u32..1000,
        minor in 0u32..1000,
        patch in 0u32..1000
    ) {
        let version = SemanticVersion::new(major, minor, patch);
        let formatted = version.to_string();
        let parsed = SemanticVersion::parse(&formatted).unwrap();
        prop_assert_eq!(version, parsed);
    }

    /// SemanticVersion parse rejects invalid formats
    #[test]
    fn test_semantic_version_parse_rejects_invalid(
        s in "[a-zA-Z]{1,10}"
    ) {
        let result = SemanticVersion::parse(&s);
        prop_assert!(result.is_none());
    }

    /// SemanticVersion JSON roundtrip
    #[test]
    fn test_semantic_version_json_roundtrip(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100
    ) {
        let version = SemanticVersion::new(major, minor, patch);
        let json = serde_json::to_string(&version).unwrap();
        let deserialized: SemanticVersion = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(version, deserialized);
    }

    /// SemanticVersion format is consistent
    #[test]
    fn test_semantic_version_format_consistent(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100
    ) {
        let version = SemanticVersion::new(major, minor, patch);
        let expected = format!("{}.{}.{}", major, minor, patch);
        prop_assert_eq!(version.to_string(), expected);
    }
}

// ============================================================================
// ValidUrl Property Tests
// ============================================================================

proptest! {
    /// ValidUrl rejects invalid URLs
    #[test]
    fn test_valid_url_rejects_invalid(
        s in "[a-zA-Z0-9]{1,20}"  // Not a valid URL
    ) {
        let result = ValidUrl::parse(&s);
        prop_assert!(result.is_err());
    }

    /// ValidUrl accepts valid HTTP URLs
    #[test]
    fn test_valid_url_accepts_http(
        domain in "[a-z]{3,10}",
        tld in "[a-z]{2,3}"
    ) {
        let url_str = format!("http://{}.{}", domain, tld);
        let result = ValidUrl::parse(&url_str);
        prop_assert!(result.is_ok());
    }

    /// ValidUrl accepts valid HTTPS URLs
    #[test]
    fn test_valid_url_accepts_https(
        domain in "[a-z]{3,10}",
        tld in "[a-z]{2,3}"
    ) {
        let url_str = format!("https://{}.{}", domain, tld);
        let result = ValidUrl::parse(&url_str);
        prop_assert!(result.is_ok());
    }

    /// ValidUrl JSON roundtrip
    #[test]
    fn test_valid_url_json_roundtrip(
        domain in "[a-z]{3,10}",
        tld in "[a-z]{2,3}"
    ) {
        let url_str = format!("https://{}.{}", domain, tld);
        let url = ValidUrl::parse(&url_str).unwrap();
        let json = serde_json::to_string(&url).unwrap();
        let deserialized: ValidUrl = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(url, deserialized);
    }
}

// ============================================================================
// MimeType Property Tests
// ============================================================================

proptest! {
    /// MimeType from_path is deterministic
    #[test]
    fn test_mime_type_deterministic(
        filename in "[a-z]{1,10}",
        ext in prop_oneof![
            Just("txt"),
            Just("json"),
            Just("rs"),
            Just("py"),
            Just("js"),
        ]
    ) {
        let path = format!("{}.{}", filename, ext);
        let mime1 = MimeType::from_path(&path);
        let mime2 = MimeType::from_path(&path);
        prop_assert_eq!(mime1, mime2);
    }

    /// MimeType JSON roundtrip
    #[test]
    fn test_mime_type_json_roundtrip(
        filename in "[a-z]{1,10}",
        ext in prop_oneof![
            Just("txt"),
            Just("json"),
            Just("rs"),
        ]
    ) {
        let path = format!("{}.{}", filename, ext);
        let mime = MimeType::from_path(&path);
        let json = serde_json::to_string(&mime).unwrap();
        let deserialized: MimeType = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(mime, deserialized);
    }
}

// ============================================================================
// UserRole Property Tests
// ============================================================================

proptest! {
    /// UserRole default_permissions is non-empty
    #[test]
    fn test_user_role_permissions_non_empty(
        role in prop_oneof![
            Just(UserRole::Admin),
            Just(UserRole::Developer),
            Just(UserRole::Analyst),
            Just(UserRole::Viewer),
            Just(UserRole::Guest),
        ]
    ) {
        let permissions = role.default_permissions();
        prop_assert!(!permissions.is_empty());
    }

    /// UserRole Admin has all permissions
    #[test]
    fn test_user_role_admin_has_all_permissions(_dummy in any::<u8>()) {
        let permissions = UserRole::Admin.default_permissions();
        prop_assert!(permissions.contains(&Permission::Read));
        prop_assert!(permissions.contains(&Permission::Write));
        prop_assert!(permissions.contains(&Permission::Delete));
        prop_assert!(permissions.contains(&Permission::Admin));
        prop_assert!(permissions.contains(&Permission::Audit));
    }

    /// UserRole JSON roundtrip
    #[test]
    fn test_user_role_json_roundtrip(
        role in prop_oneof![
            Just(UserRole::Admin),
            Just(UserRole::Developer),
            Just(UserRole::Analyst),
            Just(UserRole::Viewer),
            Just(UserRole::Guest),
        ]
    ) {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: UserRole = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(role, deserialized);
    }
}

// ============================================================================
// Permission Property Tests
// ============================================================================

proptest! {
    /// Permission implication is reflexive (P implies P)
    #[test]
    fn test_permission_implication_reflexive(
        perm in prop_oneof![
            Just(Permission::Read),
            Just(Permission::Write),
            Just(Permission::Delete),
            Just(Permission::Execute),
            Just(Permission::Analyze),
            Just(Permission::Admin),
            Just(Permission::Audit),
        ]
    ) {
        prop_assert!(perm.implies(&perm));
    }

    /// Permission Admin implies all permissions
    #[test]
    fn test_permission_admin_implies_all(
        perm in prop_oneof![
            Just(Permission::Read),
            Just(Permission::Write),
            Just(Permission::Delete),
            Just(Permission::Execute),
            Just(Permission::Analyze),
            Just(Permission::Audit),
        ]
    ) {
        prop_assert!(Permission::Admin.implies(&perm));
    }

    /// Permission Write implies Read
    #[test]
    fn test_permission_write_implies_read(_dummy in any::<u8>()) {
        prop_assert!(Permission::Write.implies(&Permission::Read));
    }

    /// Permission Delete implies Read and Write
    #[test]
    fn test_permission_delete_implies_read_write(_dummy in any::<u8>()) {
        prop_assert!(Permission::Delete.implies(&Permission::Read));
        prop_assert!(Permission::Delete.implies(&Permission::Write));
    }

    /// Permission JSON roundtrip
    #[test]
    fn test_permission_json_roundtrip(
        perm in prop_oneof![
            Just(Permission::Read),
            Just(Permission::Write),
            Just(Permission::Delete),
            Just(Permission::Execute),
            Just(Permission::Analyze),
            Just(Permission::Admin),
            Just(Permission::Audit),
        ]
    ) {
        let json = serde_json::to_string(&perm).unwrap();
        let deserialized: Permission = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(perm, deserialized);
    }
}
