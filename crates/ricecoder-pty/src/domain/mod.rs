//! Domain layer - business logic and entities

pub mod session;
pub mod config;
pub mod events;

pub use session::{PtySession, SessionInfo, SessionStatus};
pub use config::PtyConfig;
pub use events::SessionEvent;
