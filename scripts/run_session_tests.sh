#!/usr/bin/env bash

# Run the new session unit tests
cd projects/ricecoder

echo "Running session lifecycle unit tests..."
cargo test -p ricecoder-sessions --test session_lifecycle_unit_tests -- --nocapture

echo "Running session state management unit tests..."
cargo test -p ricecoder-sessions --test session_state_management_unit_tests -- --nocapture

echo "Running enterprise features unit tests..."
cargo test -p ricecoder-sessions --test enterprise_features_unit_tests -- --nocapture

echo "Running validation, error handling, and compliance tests..."
cargo test -p ricecoder-sessions --test validation_error_handling_compliance_tests -- --nocapture

echo "Running performance, memory, and security regression tests..."
cargo test -p ricecoder-sessions --test performance_memory_security_regression_tests -- --nocapture

echo "Running persistence, serialization, and encryption validation tests..."
cargo test -p ricecoder-sessions --test persistence_serialization_encryption_validation_tests -- --nocapture

echo "All session unit tests completed!"</content>
<parameter name="filePath">run_session_tests.sh