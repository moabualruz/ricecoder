use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simulated_stream_basic() {
        let text = "Hello".to_string();
        let mut stream = simulate_stream(text, 1);

        let chunk1 = stream.next().await;
        assert_eq!(chunk1, Some("H".to_string()));

        let chunk2 = stream.next().await;
        assert_eq!(chunk2, Some("e".to_string()));
    }

    #[tokio::test]
    async fn test_simulated_stream_complete() {
        let text = "Hi".to_string();
        let mut stream = simulate_stream(text, 1);

        let mut result = String::new();
        while let Some(chunk) = stream.next().await {
            result.push_str(&chunk);
        }

        assert_eq!(result, "Hi");
    }

    #[tokio::test]
    async fn test_word_stream() {
        let text = "Hello world".to_string();
        let mut stream = simulate_word_stream(text, 1);

        let word1 = stream.next().await;
        assert_eq!(word1, Some("Hello ".to_string()));

        let word2 = stream.next().await;
        assert_eq!(word2, Some("world ".to_string()));
    }

    #[tokio::test]
    async fn test_stream_position() {
        let text = "Test".to_string();
        let mut stream = simulate_stream(text, 1);

        assert_eq!(stream.position(), 0);
        stream.next().await;
        assert_eq!(stream.position(), 1);
    }

    #[tokio::test]
    async fn test_stream_reset() {
        let text = "Test".to_string();
        let mut stream = simulate_stream(text, 1);

        stream.next().await;
        stream.next().await;
        assert_eq!(stream.position(), 2);

        stream.reset();
        assert_eq!(stream.position(), 0);
    }
}
