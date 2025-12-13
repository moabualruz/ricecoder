use ricecoder_storage::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_dir_names() {
        assert_eq!(ResourceType::Template.dir_name(), "templates");
        assert_eq!(ResourceType::Standard.dir_name(), "standards");
        assert_eq!(ResourceType::Spec.dir_name(), "specs");
        assert_eq!(ResourceType::Steering.dir_name(), "steering");
        assert_eq!(ResourceType::Boilerplate.dir_name(), "boilerplates");
        assert_eq!(ResourceType::Rule.dir_name(), "rules");
        assert_eq!(ResourceType::CustomCommand.dir_name(), "commands");
        assert_eq!(ResourceType::HooksConfig.dir_name(), "hooks");
        assert_eq!(
            ResourceType::RefactoringLanguageConfig.dir_name(),
            "refactoring/languages"
        );
    }

    #[test]
    fn test_config_format_extensions() {
        assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
        assert_eq!(ConfigFormat::Toml.extension(), "toml");
        assert_eq!(ConfigFormat::Json.extension(), "json");
        assert_eq!(ConfigFormat::Jsonc.extension(), "jsonc");
    }

    #[test]
    fn test_config_format_detection() {
        assert_eq!(
            ConfigFormat::from_extension("yaml"),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_extension("yml"),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_extension("toml"),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_extension("json"),
            Some(ConfigFormat::Json)
        );
        assert_eq!(
            ConfigFormat::from_extension("jsonc"),
            Some(ConfigFormat::Jsonc)
        );
        assert_eq!(ConfigFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_document_format_detection() {
        assert_eq!(
            DocumentFormat::from_extension("yaml"),
            Some(DocumentFormat::Yaml)
        );
        assert_eq!(
            DocumentFormat::from_extension("md"),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(
            DocumentFormat::from_extension("markdown"),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(DocumentFormat::from_extension("txt"), None);
    }
}