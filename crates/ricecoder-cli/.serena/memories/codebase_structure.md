The codebase is organized as follows:

- `src/main.rs`: Main entry point, handles initialization and command routing
- `src/lib.rs`: Library crate with core functionality
- `src/commands/`: Submodules for different CLI commands (chat, lsp, gen, etc.)
- `src/`: Various modules for features like accessibility, async optimization, branding, chat, completion, DI, error handling, lifecycle, logging, memory optimization, output, profiling, progress, router, sync utils
- `tests/`: Unit and integration tests
- `Cargo.toml`: Package configuration and dependencies
- `.serena/`: Serena-specific configuration and memories