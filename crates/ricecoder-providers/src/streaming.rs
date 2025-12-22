/// Streaming response simulation for non-streaming providers
///
/// This module provides utilities to simulate streaming responses by typing
/// out non-stream responses character-by-character, creating a typing effect.
use std::time::Duration;

use tokio::time::sleep;

/// Simulates a streaming response by yielding characters one at a time
/// with a typing effect.
///
/// # Arguments
///
/// * `text` - The complete response text to stream
/// * `char_delay_ms` - Delay in milliseconds between each character (default: 10ms)
///
/// # Example
///
/// ```ignore
/// let response = "Hello, world!";
/// let mut stream = simulate_stream(response, 10);
/// while let Some(chunk) = stream.next().await {
///     print!("{}", chunk);
/// }
/// ```
pub struct SimulatedStream {
    text: String,
    position: usize,
    char_delay: Duration,
}

impl SimulatedStream {
    /// Creates a new simulated stream
    pub fn new(text: String, char_delay_ms: u64) -> Self {
        Self {
            text,
            position: 0,
            char_delay: Duration::from_millis(char_delay_ms),
        }
    }

    /// Gets the next character chunk from the stream
    pub async fn next(&mut self) -> Option<String> {
        if self.position >= self.text.len() {
            return None;
        }

        // Get the next character
        let ch = self.text.chars().nth(self.position)?;
        self.position += 1;

        // Add delay for typing effect
        sleep(self.char_delay).await;

        Some(ch.to_string())
    }

    /// Gets all remaining text as a stream of characters
    pub async fn collect_all(&mut self) -> Vec<String> {
        let mut chunks = Vec::new();
        while let Some(chunk) = self.next().await {
            chunks.push(chunk);
        }
        chunks
    }

    /// Gets the complete text without streaming
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Gets the current position in the stream
    pub fn position(&self) -> usize {
        self.position
    }

    /// Resets the stream to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

/// Creates a simulated stream from a response text
pub fn simulate_stream(text: String, char_delay_ms: u64) -> SimulatedStream {
    SimulatedStream::new(text, char_delay_ms)
}

/// Simulates streaming with word-by-word output (faster than character-by-character)
pub struct WordStream {
    words: Vec<String>,
    position: usize,
    word_delay: Duration,
}

impl WordStream {
    /// Creates a new word-based stream
    pub fn new(text: String, word_delay_ms: u64) -> Self {
        let words = text
            .split_whitespace()
            .map(|w| w.to_string() + " ")
            .collect();

        Self {
            words,
            position: 0,
            word_delay: Duration::from_millis(word_delay_ms),
        }
    }

    /// Gets the next word from the stream
    pub async fn next(&mut self) -> Option<String> {
        if self.position >= self.words.len() {
            return None;
        }

        let word = self.words[self.position].clone();
        self.position += 1;

        sleep(self.word_delay).await;

        Some(word)
    }

    /// Gets all remaining words as a stream
    pub async fn collect_all(&mut self) -> Vec<String> {
        let mut chunks = Vec::new();
        while let Some(chunk) = self.next().await {
            chunks.push(chunk);
        }
        chunks
    }
}

/// Creates a word-based simulated stream
pub fn simulate_word_stream(text: String, word_delay_ms: u64) -> WordStream {
    WordStream::new(text, word_delay_ms)
}
