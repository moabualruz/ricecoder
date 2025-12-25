# Suggested Commands

## Development Commands
- `cargo run`: Start the API server in development mode (listens on 127.0.0.1:3000)
- `cargo build`: Build the project in debug mode
- `cargo build --release`: Build the project in release mode for production

## Testing
- `cargo test`: Run all tests
- `cargo test --lib`: Run only library tests (excluding integration tests)
- `cargo test --doc`: Run documentation tests

## Code Quality
- `cargo fmt`: Format the code according to Rust standards
- `cargo clippy`: Run the linter to check for common mistakes and style issues
- `cargo check`: Check the code for compilation errors without building

## Utility Commands (Windows)
- `dir`: List files in current directory (equivalent to `ls` on Unix)
- `cd <path>`: Change directory
- `type <file>`: Display file contents (equivalent to `cat` on Unix)
- `findstr <pattern> <file>`: Search for patterns in files (equivalent to `grep` on Unix)
- `git status`: Check git status
- `git add .`: Stage all changes
- `git commit -m "message"`: Commit changes
- `git push`: Push to remote repository

## OpenAPI Documentation
The API provides Swagger UI documentation at `/swagger-ui` when the server is running.

## Health Check
Once running, you can check the server health at `http://127.0.0.1:3000/health`