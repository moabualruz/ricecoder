use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_levels() {
        VerbosityLevel::set(VerbosityLevel::Normal);
        assert_eq!(VerbosityLevel::current(), VerbosityLevel::Normal);

        VerbosityLevel::set(VerbosityLevel::Verbose);
        assert_eq!(VerbosityLevel::current(), VerbosityLevel::Verbose);
    }

    #[test]
    fn test_should_output() {
        VerbosityLevel::set(VerbosityLevel::Normal);
        assert!(VerbosityLevel::Normal.should_output());
        assert!(!VerbosityLevel::Verbose.should_output());
        assert!(VerbosityLevel::Quiet.should_output());
    }

    #[test]
    fn test_init_logging_quiet() {
        init_logging(false, true);
        assert_eq!(VerbosityLevel::current(), VerbosityLevel::Quiet);
    }

    #[test]
    fn test_init_logging_verbose() {
        init_logging(true, false);
        assert_eq!(VerbosityLevel::current(), VerbosityLevel::Verbose);
    }

    #[test]
    fn test_init_logging_normal() {
        init_logging(false, false);
        assert_eq!(VerbosityLevel::current(), VerbosityLevel::Normal);
    }
}