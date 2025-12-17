#!/usr/bin/env bash

# Run the new session enterprise tests
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd "$SCRIPT_DIR/../.."

echo "Testing session sharing in team environments with enterprise access controls..."
echo "Testing session persistence, recovery, and enterprise backup strategies..."
echo "Testing multi-user session access controls and compliance auditing..."
echo "Testing session compliance, audit features, and GDPR/HIPAA requirements..."

echo "Checking if ricecoder-sessions compiles..."
if cargo check -p ricecoder-sessions; then
    echo "✅ ricecoder-sessions compiles successfully"
else
    echo "❌ ricecoder-sessions compilation failed"
    exit 1
fi

echo "Running enterprise features unit tests..."
if cargo test -p ricecoder-sessions --test enterprise_features_unit_tests -- --nocapture; then
    echo "✅ Enterprise features test passed"
else
    echo "❌ Enterprise features test failed"
    exit 1
fi

echo "Running validation, error handling, and compliance tests..."
if cargo test -p ricecoder-sessions --test validation_error_handling_compliance_tests -- --nocapture; then
    echo "✅ Validation and compliance tests passed"
else
    echo "❌ Validation and compliance tests failed"
    exit 1
fi

echo "Running persistence, serialization, and encryption validation tests..."
if cargo test -p ricecoder-sessions --test persistence_serialization_encryption_validation_tests -- --nocapture; then
    echo "✅ Persistence tests passed"
else
    echo "❌ Persistence tests failed"
    exit 1
fi

echo "Running session concurrency and sharing property tests..."
if cargo test -p ricecoder-sessions --test session_concurrency_sharing_property_tests -- --nocapture --test-threads=1; then
    echo "✅ Concurrency tests passed"
else
    echo "❌ Concurrency tests failed"
    exit 1
fi

echo "All session enterprise tests completed successfully!"
echo ""
echo "Task 26.2 completed: Session management enterprise features tested"
echo "- ✅ Session sharing in team environments with enterprise access controls"
echo "- ✅ Session persistence, recovery, and enterprise backup strategies"
echo "- ✅ Multi-user session access controls and compliance auditing"
echo "- ✅ Session compliance, audit features, and GDPR/HIPAA requirements"