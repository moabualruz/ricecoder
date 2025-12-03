//! Property-based tests for data model serialization
//! **Feature: ricecoder-research, Property 1: Serialization Round-Trip**
//! **Validates: Requirements 1.1**

use proptest::prelude::*;
use ricecoder_research::models::*;

// ============================================================================
// Generators for property testing
// ============================================================================

fn arb_project_type() -> impl Strategy<Value = ProjectType> {
    prop_oneof![
        Just(ProjectType::Library),
        Just(ProjectType::Application),
        Just(ProjectType::Service),
        Just(ProjectType::Monorepo),
        Just(ProjectType::Unknown),
    ]
}

fn arb_language() -> impl Strategy<Value = Language> {
    prop_oneof![
        Just(Language::Rust),
        Just(Language::TypeScript),
        Just(Language::Python),
        Just(Language::Go),
        Just(Language::Java),
        Just(Language::Kotlin),
        Just(Language::CSharp),
        Just(Language::Php),
        Just(Language::Ruby),
        Just(Language::Swift),
        Just(Language::Dart),
    ]
}

fn arb_case_style() -> impl Strategy<Value = CaseStyle> {
    prop_oneof![
        Just(CaseStyle::CamelCase),
        Just(CaseStyle::SnakeCase),
        Just(CaseStyle::PascalCase),
        Just(CaseStyle::KebabCase),
        Just(CaseStyle::UpperCase),
        Just(CaseStyle::Mixed),
    ]
}

fn arb_symbol_kind() -> impl Strategy<Value = SymbolKind> {
    prop_oneof![
        Just(SymbolKind::Function),
        Just(SymbolKind::Class),
        Just(SymbolKind::Type),
        Just(SymbolKind::Constant),
        Just(SymbolKind::Variable),
        Just(SymbolKind::Module),
        Just(SymbolKind::Trait),
        Just(SymbolKind::Enum),
    ]
}

fn arb_reference_kind() -> impl Strategy<Value = ReferenceKind> {
    prop_oneof![
        Just(ReferenceKind::Definition),
        Just(ReferenceKind::Usage),
        Just(ReferenceKind::Import),
        Just(ReferenceKind::Export),
    ]
}

fn arb_indent_type() -> impl Strategy<Value = IndentType> {
    prop_oneof![Just(IndentType::Spaces), Just(IndentType::Tabs),]
}

fn arb_import_group() -> impl Strategy<Value = ImportGroup> {
    prop_oneof![
        Just(ImportGroup::Standard),
        Just(ImportGroup::External),
        Just(ImportGroup::Internal),
        Just(ImportGroup::Relative),
    ]
}

fn arb_doc_format() -> impl Strategy<Value = DocFormat> {
    prop_oneof![
        Just(DocFormat::JavaDoc),
        Just(DocFormat::RustDoc),
        Just(DocFormat::JSDoc),
        Just(DocFormat::PythonDoc),
    ]
}

fn arb_architectural_style() -> impl Strategy<Value = ArchitecturalStyle> {
    prop_oneof![
        Just(ArchitecturalStyle::Layered),
        Just(ArchitecturalStyle::Microservices),
        Just(ArchitecturalStyle::EventDriven),
        Just(ArchitecturalStyle::Monolithic),
        Just(ArchitecturalStyle::Serverless),
        Just(ArchitecturalStyle::Unknown),
    ]
}

fn arb_pattern_category() -> impl Strategy<Value = PatternCategory> {
    prop_oneof![
        Just(PatternCategory::Architectural),
        Just(PatternCategory::Design),
        Just(PatternCategory::Coding),
        Just(PatternCategory::Testing),
        Just(PatternCategory::Configuration),
    ]
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: ProjectType serialization round-trip
    /// For any ProjectType, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_project_type_round_trip(project_type in arb_project_type()) {
        let json = serde_json::to_string(&project_type).expect("serialization failed");
        let deserialized: ProjectType = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(project_type, deserialized);
    }

    /// Property: Language serialization round-trip
    /// For any Language, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_language_round_trip(language in arb_language()) {
        let json = serde_json::to_string(&language).expect("serialization failed");
        let deserialized: Language = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(language, deserialized);
    }

    /// Property: CaseStyle serialization round-trip
    /// For any CaseStyle, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_case_style_round_trip(style in arb_case_style()) {
        let json = serde_json::to_string(&style).expect("serialization failed");
        let deserialized: CaseStyle = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(style, deserialized);
    }

    /// Property: SymbolKind serialization round-trip
    /// For any SymbolKind, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_symbol_kind_round_trip(kind in arb_symbol_kind()) {
        let json = serde_json::to_string(&kind).expect("serialization failed");
        let deserialized: SymbolKind = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(kind, deserialized);
    }

    /// Property: ReferenceKind serialization round-trip
    /// For any ReferenceKind, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_reference_kind_round_trip(kind in arb_reference_kind()) {
        let json = serde_json::to_string(&kind).expect("serialization failed");
        let deserialized: ReferenceKind = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(kind, deserialized);
    }

    /// Property: IndentType serialization round-trip
    /// For any IndentType, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_indent_type_round_trip(indent_type in arb_indent_type()) {
        let json = serde_json::to_string(&indent_type).expect("serialization failed");
        let deserialized: IndentType = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(indent_type, deserialized);
    }

    /// Property: ImportGroup serialization round-trip
    /// For any ImportGroup, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_import_group_round_trip(group in arb_import_group()) {
        let json = serde_json::to_string(&group).expect("serialization failed");
        let deserialized: ImportGroup = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(group, deserialized);
    }

    /// Property: DocFormat serialization round-trip
    /// For any DocFormat, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_doc_format_round_trip(format in arb_doc_format()) {
        let json = serde_json::to_string(&format).expect("serialization failed");
        let deserialized: DocFormat = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(format, deserialized);
    }

    /// Property: ArchitecturalStyle serialization round-trip
    /// For any ArchitecturalStyle, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_architectural_style_round_trip(style in arb_architectural_style()) {
        let json = serde_json::to_string(&style).expect("serialization failed");
        let deserialized: ArchitecturalStyle = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(style, deserialized);
    }

    /// Property: PatternCategory serialization round-trip
    /// For any PatternCategory, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_pattern_category_round_trip(category in arb_pattern_category()) {
        let json = serde_json::to_string(&category).expect("serialization failed");
        let deserialized: PatternCategory = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(category, deserialized);
    }

    /// Property: NamingConventions serialization round-trip
    /// For any NamingConventions, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_naming_conventions_round_trip(
        func_case in arb_case_style(),
        var_case in arb_case_style(),
        class_case in arb_case_style(),
        const_case in arb_case_style(),
    ) {
        let conventions = NamingConventions {
            function_case: func_case,
            variable_case: var_case,
            class_case,
            constant_case: const_case,
        };

        let json = serde_json::to_string(&conventions).expect("serialization failed");
        let deserialized: NamingConventions = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(conventions, deserialized);
    }

    /// Property: FormattingStyle serialization round-trip
    /// For any FormattingStyle, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_formatting_style_round_trip(
        indent_size in 1usize..16,
        indent_type in arb_indent_type(),
        line_length in 40usize..200,
    ) {
        let style = FormattingStyle {
            indent_size,
            indent_type,
            line_length,
        };

        let json = serde_json::to_string(&style).expect("serialization failed");
        let deserialized: FormattingStyle = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(style, deserialized);
    }

    /// Property: ImportOrganization serialization round-trip
    /// For any ImportOrganization, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_import_organization_round_trip(
        sort_within_group in any::<bool>(),
    ) {
        let org = ImportOrganization {
            order: vec![ImportGroup::Standard, ImportGroup::External, ImportGroup::Internal],
            sort_within_group,
        };

        let json = serde_json::to_string(&org).expect("serialization failed");
        let deserialized: ImportOrganization = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(org, deserialized);
    }

    /// Property: DocumentationStyle serialization round-trip
    /// For any DocumentationStyle, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_documentation_style_round_trip(
        format in arb_doc_format(),
        required in any::<bool>(),
    ) {
        let style = DocumentationStyle {
            format,
            required_for_public: required,
        };

        let json = serde_json::to_string(&style).expect("serialization failed");
        let deserialized: DocumentationStyle = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(style, deserialized);
    }

    /// Property: StandardsProfile serialization round-trip
    /// For any StandardsProfile, serializing to JSON and deserializing should produce the same value
    #[test]
    fn prop_standards_profile_round_trip(
        func_case in arb_case_style(),
        var_case in arb_case_style(),
        class_case in arb_case_style(),
        const_case in arb_case_style(),
        indent_size in 1usize..16,
        indent_type in arb_indent_type(),
        line_length in 40usize..200,
        sort_within_group in any::<bool>(),
        doc_format in arb_doc_format(),
        required_docs in any::<bool>(),
    ) {
        let profile = StandardsProfile {
            naming_conventions: NamingConventions {
                function_case: func_case,
                variable_case: var_case,
                class_case,
                constant_case: const_case,
            },
            formatting_style: FormattingStyle {
                indent_size,
                indent_type,
                line_length,
            },
            import_organization: ImportOrganization {
                order: vec![ImportGroup::Standard, ImportGroup::External, ImportGroup::Internal],
                sort_within_group,
            },
            documentation_style: DocumentationStyle {
                format: doc_format,
                required_for_public: required_docs,
            },
        };

        let json = serde_json::to_string(&profile).expect("serialization failed");
        let deserialized: StandardsProfile = serde_json::from_str(&json).expect("deserialization failed");
        prop_assert_eq!(profile, deserialized);
    }
}
