# Suggested Commands

## Development Commands
- **Build project**: `cargo build -p ricecoder-completion`
- **Run tests**: `cargo test -p ricecoder-completion`
- **Run specific test**: `cargo test -p ricecoder-completion <test_name>`
- **Run property tests**: `cargo test -p ricecoder-completion property`
- **Run LSP tests**: `cargo test -p ricecoder-completion lsp`
- **Run ghost text tests**: `cargo test -p ricecoder-completion ghost_text`

## Code Quality Commands
- **Format code**: `cargo fmt -p ricecoder-completion`
- **Lint code**: `cargo clippy -p ricecoder-completion`
- **Check compilation**: `cargo check -p ricecoder-completion`

## Utility Commands (Windows)
- **List files**: `dir` (or `ls` if using bash)
- **Change directory**: `cd <path>`
- **Search text**: `findstr <pattern> <file>` (or `grep` if using bash)
- **Find files**: `dir /s /b <pattern>` (or `find` if using bash)
- **Git status**: `git status`
- **Git commit**: `git add . && git commit -m "<message>"`
- **Git push**: `git push`

## Workspace Commands
Since this is part of a Cargo workspace, use `-p ricecoder-completion` for package-specific operations.

## Common Workflows
1. **After code changes**: `cargo fmt -p ricecoder-completion && cargo clippy -p ricecoder-completion && cargo test -p ricecoder-completion`
2. **Before commit**: `cargo fmt --check -p ricecoder-completion && cargo clippy -p ricecoder-completion && cargo test -p ricecoder-completion`
3. **Full check**: `cargo build -p ricecoder-completion && cargo test -p ricecoder-completion && cargo clippy -p ricecoder-completion`