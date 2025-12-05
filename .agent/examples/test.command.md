---
name: test
description: Run tests for the current project
template: "cargo test {{test_filter}}"
keybinding: "ctrl+shift+t"
parameters:
  - name: test_filter
    description: Optional filter for specific tests (e.g., "test_parser")
    required: false
    default: ""
---

# Test Command

The test command runs tests for the current project using the appropriate test runner.

## Features

- **Automatic Detection**: Detects project type and uses appropriate test runner
- **Filtering**: Run specific tests by name or pattern
- **Parallel Execution**: Runs tests in parallel for faster feedback
- **Detailed Output**: Shows test results with pass/fail status
- **Coverage**: Optional code coverage reporting

## Supported Project Types

- **Rust**: Uses `cargo test`
- **TypeScript/JavaScript**: Uses `npm test` or `yarn test`
- **Python**: Uses `pytest`
- **Go**: Uses `go test`

## Usage

### Run All Tests

```
ricecoder test
```

### Run Specific Test

```
ricecoder test parser
```

This runs all tests matching "parser" in their name.

### Run Tests with Options

```
ricecoder test --verbose
ricecoder test --coverage
ricecoder test --watch
```

## Configuration

The test command is configured in `~/.ricecoder/commands/test.command.md`:

```yaml
name: test
description: Run tests for the current project
template: "cargo test {{test_filter}}"
keybinding: "ctrl+shift+t"
parameters:
  - name: test_filter
    description: Optional filter for specific tests
    required: false
    default: ""
```

### Customization

To customize the test command for your project:

```yaml
# For TypeScript projects
template: "npm test -- {{test_filter}}"

# For Python projects
template: "pytest {{test_filter}}"

# With additional options
template: "cargo test {{test_filter}} -- --nocapture"
```

## Examples

### Example 1: Run All Tests (Rust)

```
$ ricecoder test
   Compiling ricecoder v0.4.0
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src/lib.rs

running 42 tests
test parser::tests::test_parse_markdown ... ok
test parser::tests::test_parse_yaml ... ok
...
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Example 2: Run Specific Tests

```
$ ricecoder test parser
   Compiling ricecoder v0.4.0
    Finished test [unoptimized + debuginfo] target(s) in 1.23s
     Running unittests src/lib.rs

running 8 tests
test parser::tests::test_parse_markdown ... ok
test parser::tests::test_parse_yaml ... ok
test parser::tests::test_parse_frontmatter ... ok
...
test result: ok. 8 passed; 0 failed
```

### Example 3: Run Tests with Coverage

```
$ ricecoder test --coverage
   Compiling ricecoder v0.4.0
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src/lib.rs

running 42 tests
...
test result: ok. 42 passed; 0 failed

Coverage Report:
  parser.rs: 95%
  loader.rs: 87%
  registry.rs: 92%
  Overall: 91%
```

## Tips

1. **Use keybinding**: Press `Ctrl+Shift+T` to quickly run tests
2. **Filter tests**: Use test filters to run specific tests during development
3. **Watch mode**: Use `--watch` to re-run tests on file changes
4. **Coverage**: Check coverage regularly to identify untested code

## Troubleshooting

### Tests Not Found

If tests aren't found, ensure:
- Test files are in the correct location (`tests/` or `*_test.rs`)
- Test functions are marked with `#[test]` (Rust) or `test` prefix (other languages)
- Project is properly configured

### Tests Failing

If tests fail:
1. Check the error message for details
2. Run with `--verbose` for more information
3. Run specific test to isolate the issue
4. Check recent code changes

## See Also

- [Testing Guide](https://github.com/moabualruz/ricecoder/wiki/Testing.md)
- [CLI Commands Reference](https://github.com/moabualruz/ricecoder/wiki/CLI-Commands.md)
- [Configuration Guide](https://github.com/moabualruz/ricecoder/wiki/Configuration.md)
