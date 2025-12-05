//! LSP server registry and configuration management

pub mod config;
pub mod defaults;
pub mod discovery;

pub use config::ConfigLoader;
pub use defaults::DefaultServerConfigs;
pub use discovery::ServerDiscovery;
