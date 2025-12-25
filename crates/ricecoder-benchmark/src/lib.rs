//! Benchmarking framework for RiceCoder

pub mod cli;
pub mod error;
pub mod evaluator;
pub mod exercise;
pub mod results;
pub mod runner;

pub use cli::*;
pub use error::*;
pub use evaluator::*;
pub use exercise::*;
pub use results::*;
pub use runner::*;

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // Placeholder test
        assert!(true);
    }
}
