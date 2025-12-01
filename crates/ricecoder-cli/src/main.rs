// RiceCoder CLI Entry Point

use ricecoder_cli::router::CommandRouter;
use ricecoder_cli::output;
use std::path::Path;

fn main() {
    // Check for multi-call binary pattern
    let binary_name = std::env::args()
        .next()
        .and_then(|s| Path::new(&s).file_stem().map(|s| s.to_string_lossy().to_string()));

    match binary_name.as_deref() {
        Some("rice-gen") | Some("ricecoder-gen") => {
            // Handle as gen command
            std::env::set_var("RICECODER_COMMAND", "gen");
        }
        Some("rice-chat") | Some("ricecoder-chat") => {
            // Handle as chat command
            std::env::set_var("RICECODER_COMMAND", "chat");
        }
        Some("rice-init") | Some("ricecoder-init") => {
            // Handle as init command
            std::env::set_var("RICECODER_COMMAND", "init");
        }
        _ => {}
    }

    // Route and execute command
    if let Err(e) = CommandRouter::route() {
        output::print_error(&e.user_message());
        std::process::exit(1);
    }
}
