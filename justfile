# Repository gate: the same checks .github/workflows/ci.yml runs, in the same order.
ci: fmt clippy test audit coverage

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy -- -D warnings

test:
    cargo test --workspace --all-features

audit:
    cargo audit

# Ratchet at the current floor; raising it to 80 is tracked in moabualruz/ricecoder#17.
coverage:
    cargo tarpaulin --engine llvm --timeout 600 --workspace --all-features --fail-under 45
