use ricecoder_storage::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_default() {
        let args = CliArgs::default();
        assert!(!args.has_overrides());
        assert!(!args.verbose);
        assert!(!args.no_telemetry);
    }

    #[test]
    fn test_cli_args_with_overrides() {
        let mut args = CliArgs::default();
        args.provider = Some("openai".to_string());
        assert!(args.has_overrides());
    }
}