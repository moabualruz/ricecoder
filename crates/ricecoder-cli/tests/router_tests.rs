use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_similar_command() {
        assert_eq!(CommandRouter::find_similar("i"), Some("init".to_string()));
        assert_eq!(CommandRouter::find_similar("g"), Some("gen".to_string()));
        assert_eq!(CommandRouter::find_similar("c"), Some("chat".to_string()));
    }
}