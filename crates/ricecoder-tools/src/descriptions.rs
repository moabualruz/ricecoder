//! Tool description helper
//!
//! Provides a helper function to get tool descriptions from external config files,
//! falling back to hardcoded defaults.

use ricecoder_storage::loaders::tools::global_tool_descriptions;

/// Get the description for a tool, preferring external config over hardcoded default
///
/// # Arguments
/// * `tool_id` - The tool identifier (e.g., "bash", "glob", "grep")
/// * `fallback` - The hardcoded fallback description
///
/// # Returns
/// The external description if available, otherwise the fallback
pub fn get_description(tool_id: &str, fallback: &str) -> String {
    global_tool_descriptions()
        .get_description(tool_id)
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_description() {
        // Non-existent tool should return fallback
        let desc = get_description("nonexistent_tool_xyz", "Fallback description");
        assert_eq!(desc, "Fallback description");
    }
}
