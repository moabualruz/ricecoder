// Progress indicators and spinners

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a spinner for long-running operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template("{spinner:.cyan} {msg}")
        .unwrap_or_else(|e| {
            eprintln!("Failed to create spinner template: {}", e);
            ProgressStyle::default_spinner()
        });
    spinner.set_style(style);
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

/// Create a progress bar for operations with known length
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    let style = ProgressStyle::default_bar()
        .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
        .unwrap_or_else(|e| {
            eprintln!("Failed to create progress bar template: {}", e);
            ProgressStyle::default_bar()
        })
        .progress_chars("=>-");
    pb.set_style(style);
    pb.set_message(message.to_string());
    pb
}
