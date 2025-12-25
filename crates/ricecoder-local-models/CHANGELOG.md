# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.72] - 2025-12-25

### Added

- `health_check()` method for Ollama server connectivity checks
- `health_check_with_retry()` method with exponential backoff (3 retries)
- `with_timeout()` constructor for custom timeout configuration
- `timeout()` accessor for configured timeout duration
- `client()` accessor for advanced HTTP client usage
- Connection pooling with TCP keep-alive (60s interval)
- Pool idle timeout configuration (90s default)

### Changed

- Improved HTTP client configuration with best practices
- README updated with DDD layer documentation and accurate code examples
- Test organization moved to `tests/` directory per project policy
- Enhanced documentation with usage examples

### Fixed

- README.md incorrectly referenced `OllamaManager` instead of `LocalModelManager`

## [0.1.0] - 2025-12-09

### Added

- Initial release with `LocalModelManager` for Ollama integration
- Model operations: `pull_model`, `remove_model`, `update_model`
- Model queries: `list_models`, `get_model_info`, `model_exists`
- `LocalModel` and `ModelMetadata` data types
- `PullProgress` for download tracking with `percentage()` and `is_complete()`
- Comprehensive error types via `LocalModelError`
