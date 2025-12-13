use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = create_spinner("Testing...");
        assert!(!spinner.is_finished());
        spinner.finish_with_message("Done!");
    }

    #[test]
    fn test_progress_bar_creation() {
        let pb = create_progress_bar(100, "Processing...");
        assert_eq!(pb.length(), Some(100));
        pb.finish_with_message("Complete!");
    }
}