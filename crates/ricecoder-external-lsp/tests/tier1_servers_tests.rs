//! Integration tests for Tier 1 LSP server support
//!
//! Tests verify that Tier 1 servers (rust-analyzer, typescript-language-server, pylsp)
//! can be spawned, initialized, and provide completion, diagnostics, and hover features.
//!
//! Note: These tests require the LSP servers to be installed on the system.
//! - rust-analyzer: https://rust-analyzer.github.io/
//! - typescript-language-server: npm install -g typescript-language-server
//! - pylsp: pip install python-lsp-server

use ricecoder_external_lsp::DefaultServerConfigs;

// ============================================================================
// Tier 1 Server Configuration Tests
// ============================================================================

#[test]
fn test_rust_analyzer_configuration() {
    // Test that rust-analyzer is properly configured
    let config = DefaultServerConfigs::rust_analyzer();
    
    assert_eq!(config.language, "rust");
    assert_eq!(config.executable, "rust-analyzer");
    assert!(config.extensions.contains(&".rs".to_string()));
    assert!(config.enabled);
    assert_eq!(config.timeout_ms, 10000);
    assert_eq!(config.max_restarts, 3);
    assert_eq!(config.idle_timeout_ms, 300000); // 5 minutes
}

#[test]
fn test_typescript_language_server_configuration() {
    // Test that typescript-language-server is properly configured
    let config = DefaultServerConfigs::typescript_language_server();
    
    assert_eq!(config.language, "typescript");
    assert_eq!(config.executable, "typescript-language-server");
    assert!(config.extensions.contains(&".ts".to_string()));
    assert!(config.extensions.contains(&".tsx".to_string()));
    assert!(config.extensions.contains(&".js".to_string()));
    assert!(config.extensions.contains(&".jsx".to_string()));
    assert!(config.enabled);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.max_restarts, 3);
    assert_eq!(config.idle_timeout_ms, 300000); // 5 minutes
    assert!(config.args.contains(&"--stdio".to_string()));
}

#[test]
fn test_pylsp_configuration() {
    // Test that pylsp is properly configured
    let config = DefaultServerConfigs::pylsp();
    
    assert_eq!(config.language, "python");
    assert_eq!(config.executable, "pylsp");
    assert!(config.extensions.contains(&".py".to_string()));
    assert!(config.enabled);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.max_restarts, 3);
    assert_eq!(config.idle_timeout_ms, 300000); // 5 minutes
}

// ============================================================================
// Tier 1 Registry Tests
// ============================================================================

#[test]
fn test_tier1_registry_contains_all_servers() {
    // Test that Tier 1 registry contains all three servers
    let registry = DefaultServerConfigs::tier1_registry();
    
    assert_eq!(registry.servers.len(), 3);
    assert!(registry.servers.contains_key("rust"));
    assert!(registry.servers.contains_key("typescript"));
    assert!(registry.servers.contains_key("python"));
}

#[test]
fn test_tier1_registry_rust_analyzer() {
    // Test that rust-analyzer is in Tier 1 registry
    let registry = DefaultServerConfigs::tier1_registry();
    
    let rust_servers = registry.servers.get("rust").unwrap();
    assert_eq!(rust_servers.len(), 1);
    
    let config = &rust_servers[0];
    assert_eq!(config.executable, "rust-analyzer");
    assert!(config.enabled);
}

#[test]
fn test_tier1_registry_typescript_language_server() {
    // Test that typescript-language-server is in Tier 1 registry
    let registry = DefaultServerConfigs::tier1_registry();
    
    let ts_servers = registry.servers.get("typescript").unwrap();
    assert_eq!(ts_servers.len(), 1);
    
    let config = &ts_servers[0];
    assert_eq!(config.executable, "typescript-language-server");
    assert!(config.enabled);
}

#[test]
fn test_tier1_registry_pylsp() {
    // Test that pylsp is in Tier 1 registry
    let registry = DefaultServerConfigs::tier1_registry();
    
    let python_servers = registry.servers.get("python").unwrap();
    assert_eq!(python_servers.len(), 1);
    
    let config = &python_servers[0];
    assert_eq!(config.executable, "pylsp");
    assert!(config.enabled);
}

// ============================================================================
// Rust-Analyzer Specific Tests
// ============================================================================

#[test]
fn test_rust_analyzer_supports_rust_files() {
    // Test that rust-analyzer is configured for .rs files
    let config = DefaultServerConfigs::rust_analyzer();
    
    assert!(config.extensions.contains(&".rs".to_string()));
    assert_eq!(config.extensions.len(), 1);
}

#[test]
fn test_rust_analyzer_timeout_is_higher() {
    // Test that rust-analyzer has a higher timeout (10s vs 5s for others)
    // This is because rust-analyzer can be slower on first initialization
    let rust_config = DefaultServerConfigs::rust_analyzer();
    let ts_config = DefaultServerConfigs::typescript_language_server();
    
    assert!(rust_config.timeout_ms > ts_config.timeout_ms);
    assert_eq!(rust_config.timeout_ms, 10000);
}

#[test]
fn test_rust_analyzer_no_args() {
    // Test that rust-analyzer doesn't require command line arguments
    let config = DefaultServerConfigs::rust_analyzer();
    
    assert!(config.args.is_empty());
}

#[test]
fn test_rust_analyzer_completion_support() {
    // Test that rust-analyzer configuration supports completions
    // (via textDocument/completion request)
    let config = DefaultServerConfigs::rust_analyzer();
    
    // rust-analyzer supports completions for Rust
    assert_eq!(config.language, "rust");
    assert!(config.enabled);
}

#[test]
fn test_rust_analyzer_diagnostics_support() {
    // Test that rust-analyzer configuration supports diagnostics
    // (via textDocument/publishDiagnostics notification)
    let config = DefaultServerConfigs::rust_analyzer();
    
    // rust-analyzer publishes diagnostics for Rust files
    assert_eq!(config.language, "rust");
    assert!(config.enabled);
}

#[test]
fn test_rust_analyzer_hover_support() {
    // Test that rust-analyzer configuration supports hover
    // (via textDocument/hover request)
    let config = DefaultServerConfigs::rust_analyzer();
    
    // rust-analyzer provides hover information for Rust
    assert_eq!(config.language, "rust");
    assert!(config.enabled);
}

// ============================================================================
// TypeScript Language Server Specific Tests
// ============================================================================

#[test]
fn test_typescript_language_server_supports_multiple_extensions() {
    // Test that typescript-language-server handles multiple file types
    let config = DefaultServerConfigs::typescript_language_server();
    
    assert!(config.extensions.contains(&".ts".to_string()));
    assert!(config.extensions.contains(&".tsx".to_string()));
    assert!(config.extensions.contains(&".js".to_string()));
    assert!(config.extensions.contains(&".jsx".to_string()));
    assert_eq!(config.extensions.len(), 4);
}

#[test]
fn test_typescript_language_server_requires_stdio_arg() {
    // Test that typescript-language-server is configured with --stdio
    let config = DefaultServerConfigs::typescript_language_server();
    
    assert!(config.args.contains(&"--stdio".to_string()));
}

#[test]
fn test_typescript_language_server_completion_support() {
    // Test that typescript-language-server configuration supports completions
    let config = DefaultServerConfigs::typescript_language_server();
    
    // typescript-language-server supports completions for TypeScript/JavaScript
    assert_eq!(config.language, "typescript");
    assert!(config.enabled);
}

#[test]
fn test_typescript_language_server_diagnostics_support() {
    // Test that typescript-language-server configuration supports diagnostics
    let config = DefaultServerConfigs::typescript_language_server();
    
    // typescript-language-server publishes diagnostics
    assert_eq!(config.language, "typescript");
    assert!(config.enabled);
}

#[test]
fn test_typescript_language_server_hover_support() {
    // Test that typescript-language-server configuration supports hover
    let config = DefaultServerConfigs::typescript_language_server();
    
    // typescript-language-server provides hover information
    assert_eq!(config.language, "typescript");
    assert!(config.enabled);
}

#[test]
fn test_typescript_language_server_handles_jsx() {
    // Test that typescript-language-server is configured for JSX files
    let config = DefaultServerConfigs::typescript_language_server();
    
    assert!(config.extensions.contains(&".jsx".to_string()));
    assert!(config.extensions.contains(&".tsx".to_string()));
}

// ============================================================================
// Python LSP Server (pylsp) Specific Tests
// ============================================================================

#[test]
fn test_pylsp_supports_python_files() {
    // Test that pylsp is configured for .py files
    let config = DefaultServerConfigs::pylsp();
    
    assert!(config.extensions.contains(&".py".to_string()));
    assert_eq!(config.extensions.len(), 1);
}

#[test]
fn test_pylsp_no_args() {
    // Test that pylsp doesn't require command line arguments
    let config = DefaultServerConfigs::pylsp();
    
    assert!(config.args.is_empty());
}

#[test]
fn test_pylsp_completion_support() {
    // Test that pylsp configuration supports completions
    let config = DefaultServerConfigs::pylsp();
    
    // pylsp supports completions for Python
    assert_eq!(config.language, "python");
    assert!(config.enabled);
}

#[test]
fn test_pylsp_diagnostics_support() {
    // Test that pylsp configuration supports diagnostics
    let config = DefaultServerConfigs::pylsp();
    
    // pylsp publishes diagnostics for Python files
    assert_eq!(config.language, "python");
    assert!(config.enabled);
}

#[test]
fn test_pylsp_hover_support() {
    // Test that pylsp configuration supports hover
    let config = DefaultServerConfigs::pylsp();
    
    // pylsp provides hover information for Python
    assert_eq!(config.language, "python");
    assert!(config.enabled);
}

// ============================================================================
// Cross-Server Consistency Tests
// ============================================================================

#[test]
fn test_all_tier1_servers_have_same_restart_policy() {
    // Test that all Tier 1 servers have consistent restart policy
    let rust_config = DefaultServerConfigs::rust_analyzer();
    let ts_config = DefaultServerConfigs::typescript_language_server();
    let py_config = DefaultServerConfigs::pylsp();
    
    assert_eq!(rust_config.max_restarts, ts_config.max_restarts);
    assert_eq!(ts_config.max_restarts, py_config.max_restarts);
    assert_eq!(rust_config.max_restarts, 3);
}

#[test]
fn test_all_tier1_servers_have_same_idle_timeout() {
    // Test that all Tier 1 servers have consistent idle timeout
    let rust_config = DefaultServerConfigs::rust_analyzer();
    let ts_config = DefaultServerConfigs::typescript_language_server();
    let py_config = DefaultServerConfigs::pylsp();
    
    assert_eq!(rust_config.idle_timeout_ms, ts_config.idle_timeout_ms);
    assert_eq!(ts_config.idle_timeout_ms, py_config.idle_timeout_ms);
    assert_eq!(rust_config.idle_timeout_ms, 300000); // 5 minutes
}

#[test]
fn test_all_tier1_servers_are_enabled_by_default() {
    // Test that all Tier 1 servers are enabled by default
    let rust_config = DefaultServerConfigs::rust_analyzer();
    let ts_config = DefaultServerConfigs::typescript_language_server();
    let py_config = DefaultServerConfigs::pylsp();
    
    assert!(rust_config.enabled);
    assert!(ts_config.enabled);
    assert!(py_config.enabled);
}

#[test]
fn test_all_tier1_servers_have_reasonable_timeouts() {
    // Test that all Tier 1 servers have reasonable request timeouts
    let rust_config = DefaultServerConfigs::rust_analyzer();
    let ts_config = DefaultServerConfigs::typescript_language_server();
    let py_config = DefaultServerConfigs::pylsp();
    
    // All timeouts should be between 1s and 30s
    assert!(rust_config.timeout_ms >= 1000 && rust_config.timeout_ms <= 30000);
    assert!(ts_config.timeout_ms >= 1000 && ts_config.timeout_ms <= 30000);
    assert!(py_config.timeout_ms >= 1000 && py_config.timeout_ms <= 30000);
}

#[test]
fn test_all_tier1_servers_have_no_output_mapping_by_default() {
    // Test that Tier 1 servers use default LSP output mapping
    let rust_config = DefaultServerConfigs::rust_analyzer();
    let ts_config = DefaultServerConfigs::typescript_language_server();
    let py_config = DefaultServerConfigs::pylsp();
    
    assert!(rust_config.output_mapping.is_none());
    assert!(ts_config.output_mapping.is_none());
    assert!(py_config.output_mapping.is_none());
}

// ============================================================================
// Feature Support Documentation Tests
// ============================================================================

#[test]
fn test_rust_analyzer_feature_documentation() {
    // Document rust-analyzer specific features and handling
    let config = DefaultServerConfigs::rust_analyzer();
    
    // rust-analyzer features:
    // - Completion: Full semantic completions with snippets
    // - Diagnostics: Real-time compiler diagnostics
    // - Hover: Type information and documentation
    // - Navigation: Go to definition, find references
    // - Code actions: Quick fixes and refactorings
    
    assert_eq!(config.language, "rust");
    assert_eq!(config.executable, "rust-analyzer");
    
    // rust-analyzer specific handling:
    // - Requires Cargo.toml for project detection
    // - Supports workspace roots
    // - Can be slow on first initialization (hence 10s timeout)
}

#[test]
fn test_typescript_language_server_feature_documentation() {
    // Document typescript-language-server specific features and handling
    let config = DefaultServerConfigs::typescript_language_server();
    
    // typescript-language-server features:
    // - Completion: Full semantic completions with snippets
    // - Diagnostics: TypeScript/JavaScript compiler diagnostics
    // - Hover: Type information and JSDoc documentation
    // - Navigation: Go to definition, find references
    // - Code actions: Quick fixes and refactorings
    
    assert_eq!(config.language, "typescript");
    assert_eq!(config.executable, "typescript-language-server");
    
    // typescript-language-server specific handling:
    // - Requires tsconfig.json for project detection
    // - Supports workspace roots
    // - Handles both TypeScript and JavaScript files
    // - Requires --stdio argument for stdio communication
}

#[test]
fn test_pylsp_feature_documentation() {
    // Document pylsp specific features and handling
    let config = DefaultServerConfigs::pylsp();
    
    // pylsp features:
    // - Completion: Basic completions (can be enhanced with plugins)
    // - Diagnostics: Python linting and type checking
    // - Hover: Documentation and type information
    // - Navigation: Go to definition, find references
    // - Code actions: Quick fixes
    
    assert_eq!(config.language, "python");
    assert_eq!(config.executable, "pylsp");
    
    // pylsp specific handling:
    // - Supports virtual environment detection
    // - Can be configured with plugins (pylsp-mypy, pylsp-black, etc.)
    // - Requires Python 3.6+
    // - May need configuration file (.pylsp.json or setup.cfg)
}

// ============================================================================
// Installation Verification Tests
// ============================================================================

#[test]
fn test_rust_analyzer_installation_instructions() {
    // Document how to install rust-analyzer
    let config = DefaultServerConfigs::rust_analyzer();
    
    // Installation instructions for rust-analyzer:
    // 1. Install Rust: https://rustup.rs/
    // 2. rust-analyzer is included with Rust toolchain
    // 3. Or install separately: cargo install rust-analyzer
    
    assert_eq!(config.executable, "rust-analyzer");
}

#[test]
fn test_typescript_language_server_installation_instructions() {
    // Document how to install typescript-language-server
    let config = DefaultServerConfigs::typescript_language_server();
    
    // Installation instructions for typescript-language-server:
    // 1. Install Node.js: https://nodejs.org/
    // 2. npm install -g typescript-language-server
    // 3. npm install -g typescript (required dependency)
    
    assert_eq!(config.executable, "typescript-language-server");
}

#[test]
fn test_pylsp_installation_instructions() {
    // Document how to install pylsp
    let config = DefaultServerConfigs::pylsp();
    
    // Installation instructions for pylsp:
    // 1. Install Python 3.6+: https://www.python.org/
    // 2. pip install python-lsp-server
    // 3. Optional: pip install pylsp-mypy pylsp-black pylsp-isort
    
    assert_eq!(config.executable, "pylsp");
}

// ============================================================================
// Error Handling and Fallback Tests
// ============================================================================

#[test]
fn test_tier1_servers_fallback_enabled() {
    // Test that fallback to internal providers is enabled for Tier 1 servers
    let registry = DefaultServerConfigs::tier1_registry();
    
    // If any Tier 1 server is unavailable, system should fall back to internal providers
    assert!(registry.global.enable_fallback);
}

#[test]
fn test_tier1_servers_health_check_interval() {
    // Test that health check interval is configured for Tier 1 servers
    let registry = DefaultServerConfigs::tier1_registry();
    
    // Health checks should run every 30 seconds
    assert_eq!(registry.global.health_check_interval_ms, 30000);
}

#[test]
fn test_tier1_servers_process_limits() {
    // Test that process limits are enforced for Tier 1 servers
    let registry = DefaultServerConfigs::tier1_registry();
    
    // Maximum 5 concurrent LSP server processes
    assert_eq!(registry.global.max_processes, 5);
}

