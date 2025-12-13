# ricecoder-learning

**Purpose**: Learning system that captures user decisions and converts them into reusable rules with scope-based application and promotion for RiceCoder

## Overview

`ricecoder-learning` implements an intelligent learning system that captures user decisions, patterns, and preferences to create reusable rules. It supports multiple learning scopes (global, project, session) with automatic rule promotion, conflict resolution, and drift detection to ensure rules remain relevant and effective.

## Features

- **Decision Capture**: Automatically captures user decisions and patterns
- **Rule Generation**: Converts decisions into reusable, validated rules
- **Scope Management**: Global, project, and session-level rule application
- **Rule Promotion**: Automatic promotion of rules across scopes based on confidence
- **Conflict Resolution**: Intelligent conflict detection and resolution
- **Drift Detection**: Monitors rule effectiveness and detects when rules become outdated
- **Analytics Engine**: Provides insights into rule usage and effectiveness
- **Pattern Validation**: Ensures learned patterns are safe and effective
- **Intent Tracking**: Tracks architectural decisions and evolution

## Architecture

### Responsibilities
- User decision capture and analysis
- Pattern extraction and rule generation
- Rule validation and safety checking
- Scope-based rule application and inheritance
- Rule promotion and conflict resolution
- Drift detection and rule lifecycle management
- Analytics and effectiveness tracking

### Dependencies
- **Storage**: `ricecoder-storage` for rule persistence
- **Async Runtime**: `tokio` for concurrent learning operations
- **Serialization**: `serde` for rule storage and exchange
- **Analytics**: Statistical analysis for confidence scoring

### Integration Points
- **All Crates**: Applies learned rules across RiceCoder functionality
- **Storage**: Persists learned rules and decision data
- **Sessions**: Learns from session interactions and applies rules
- **Commands**: Learns from command usage patterns
- **TUI**: Adapts interface based on learned preferences

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ricecoder-learning = "0.1"
```

## Usage

### Basic Usage

```rust
use ricecoder_learning::{LearningManager, Decision, RuleScope};

// Create learning manager
let manager = LearningManager::new().await?;

// Capture a user decision
let decision = Decision {
    context: DecisionContext::CodeReview,
    action: "approved".to_string(),
    confidence: 0.9,
    ..Default::default()
};

manager.capture_decision(decision).await?;
```

### Rule Application

```rust
use ricecoder_learning::{Rule, RuleScope};

// Create a learned rule
let rule = Rule {
    pattern: "use_async_for_io".to_string(),
    action: "suggest_async".to_string(),
    scope: RuleScope::Project,
    confidence: 0.85,
    ..Default::default()
};

// Apply rule to new context
let applicable_rules = manager.get_applicable_rules(&context).await?;
for rule in applicable_rules {
    if rule.matches(&current_context) {
        manager.apply_rule(&rule).await?;
    }
}
```

### Analytics and Insights

```rust
use ricecoder_learning::AnalyticsEngine;

// Get learning insights
let analytics = AnalyticsEngine::new();
let insights = analytics.generate_insights().await?;

println!("Total rules: {}", insights.total_rules);
println!("High-confidence rules: {}", insights.high_confidence_rules);
println!("Rule effectiveness: {:.2}%", insights.average_effectiveness * 100.0);
```

## Configuration

Learning system configuration via YAML:

```yaml
learning:
  # Learning settings
  enabled: true
  auto_promotion: true
  confidence_threshold: 0.8

  # Scope settings
  scopes:
    global:
      max_rules: 1000
      retention_days: 365
    project:
      max_rules: 500
      retention_days: 180
    session:
      max_rules: 100
      retention_days: 30

  # Rule validation
  validation:
    require_testing: true
    max_conflicts: 5
    drift_detection_enabled: true

  # Analytics
  analytics:
    report_interval_days: 7
    effectiveness_tracking: true
    pattern_analysis: true
```

## API Reference

### Key Types

- **`LearningManager`**: Central learning system coordinator
- **`Decision`**: Captured user decision with context
- **`Rule`**: Learned rule with pattern and action
- **`AnalyticsEngine`**: Learning analytics and insights
- **`ConflictResolver`**: Rule conflict detection and resolution

### Key Functions

- **`capture_decision()`**: Record user decision for learning
- **`generate_rules()`**: Create rules from captured decisions
- **`get_applicable_rules()`**: Retrieve rules for current context
- **`promote_rule()`**: Promote rule to higher scope
- **`detect_drift()`**: Check if rules are still effective

## Error Handling

```rust
use ricecoder_learning::LearningError;

match manager.capture_decision(decision).await {
    Ok(()) => println!("Decision captured successfully"),
    Err(LearningError::ValidationError(msg)) => eprintln!("Invalid decision: {}", msg),
    Err(LearningError::StorageError(msg)) => eprintln!("Storage error: {}", msg),
    Err(LearningError::ConflictError(conflicts)) => eprintln!("Rule conflicts detected: {}", conflicts.len()),
}
```

## Testing

Run comprehensive learning system tests:

```bash
# Run all tests
cargo test -p ricecoder-learning

# Run property tests for learning behavior
cargo test -p ricecoder-learning property

# Test rule promotion logic
cargo test -p ricecoder-learning promotion

# Test conflict resolution
cargo test -p ricecoder-learning conflicts
```

Key test areas:
- Decision capture accuracy
- Rule generation correctness
- Scope isolation and inheritance
- Conflict resolution effectiveness
- Drift detection reliability

## Performance

- **Decision Capture**: < 10ms per decision
- **Rule Application**: < 5ms for rule matching
- **Rule Generation**: < 100ms for pattern analysis
- **Analytics Generation**: < 200ms for comprehensive reports
- **Scope Queries**: < 20ms for rule retrieval

## Contributing

When working with `ricecoder-learning`:

1. **Safety First**: Ensure learned rules don't introduce security vulnerabilities
2. **Validation**: Implement comprehensive rule validation before application
3. **Privacy**: Respect user privacy in decision capture and analytics
4. **Performance**: Optimize for real-time rule application
5. **Testing**: Test learning scenarios thoroughly with property-based testing

## License

MIT