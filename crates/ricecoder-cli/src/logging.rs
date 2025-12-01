// Logging and verbosity control
// Adapted from automation/src/utils/logging.rs

use std::sync::atomic::{AtomicU8, Ordering};

/// Global verbosity level
static VERBOSITY: AtomicU8 = AtomicU8::new(0);

/// Verbosity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    /// Quiet mode - minimal output
    Quiet = 0,
    /// Normal mode - standard output
    Normal = 1,
    /// Verbose mode - detailed output
    Verbose = 2,
    /// Very verbose mode - debug output
    VeryVerbose = 3,
}

impl VerbosityLevel {
    /// Get the current verbosity level
    pub fn current() -> Self {
        match VERBOSITY.load(Ordering::Relaxed) {
            0 => VerbosityLevel::Quiet,
            1 => VerbosityLevel::Normal,
            2 => VerbosityLevel::Verbose,
            _ => VerbosityLevel::VeryVerbose,
        }
    }

    /// Set the verbosity level
    pub fn set(level: Self) {
        VERBOSITY.store(level as u8, Ordering::Relaxed);
    }

    /// Check if we should output at this level
    pub fn should_output(&self) -> bool {
        self <= &Self::current()
    }
}

/// Initialize logging based on CLI flags
pub fn init_logging(verbose: bool, quiet: bool) {
    let level = if quiet {
        VerbosityLevel::Quiet
    } else if verbose {
        VerbosityLevel::Verbose
    } else {
        VerbosityLevel::Normal
    };

    VerbosityLevel::set(level);
}

/// Log a message at the given verbosity level
pub fn log_at_level(level: VerbosityLevel, message: &str) {
    if level.should_output() {
        eprintln!("{}", message);
    }
}

/// Log a debug message (only in verbose mode)
pub fn debug(message: &str) {
    log_at_level(VerbosityLevel::Verbose, message);
}

/// Log an info message (in normal and verbose modes)
pub fn info(message: &str) {
    log_at_level(VerbosityLevel::Normal, message);
}

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
