//! Unit tests for data models

use ricecoder_local_models::PullProgress;

#[test]
fn test_pull_progress_percentage() {
    let progress = PullProgress {
        model: "mistral".to_string(),
        status: "downloading".to_string(),
        digest: "abc123".to_string(),
        total: 1000,
        completed: 500,
    };

    assert_eq!(progress.percentage(), 50.0);
}

#[test]
fn test_pull_progress_percentage_at_25() {
    let progress = PullProgress {
        model: "llama2".to_string(),
        status: "downloading".to_string(),
        digest: "def456".to_string(),
        total: 1000,
        completed: 250,
    };

    assert_eq!(progress.percentage(), 25.0);
}

#[test]
fn test_pull_progress_complete() {
    let progress = PullProgress {
        model: "mistral".to_string(),
        status: "complete".to_string(),
        digest: "abc123".to_string(),
        total: 1000,
        completed: 1000,
    };

    assert!(progress.is_complete());
}

#[test]
fn test_pull_progress_not_complete() {
    let progress = PullProgress {
        model: "mistral".to_string(),
        status: "downloading".to_string(),
        digest: "abc123".to_string(),
        total: 1000,
        completed: 500,
    };

    assert!(!progress.is_complete());
}

#[test]
fn test_pull_progress_zero_total() {
    let progress = PullProgress {
        model: "mistral".to_string(),
        status: "downloading".to_string(),
        digest: "abc123".to_string(),
        total: 0,
        completed: 0,
    };

    assert_eq!(progress.percentage(), 0.0);
    assert!(!progress.is_complete());
}

#[test]
fn test_pull_progress_over_100_percent() {
    // Edge case: completed > total
    let progress = PullProgress {
        model: "mistral".to_string(),
        status: "complete".to_string(),
        digest: "abc123".to_string(),
        total: 1000,
        completed: 1500,
    };

    // Should be 150% (no clamping)
    assert_eq!(progress.percentage(), 150.0);
    assert!(progress.is_complete());
}
