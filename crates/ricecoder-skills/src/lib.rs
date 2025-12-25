//! Skills system for RiceCoder
//! 
//! Provides discoverable prompt assets (markdown + frontmatter) loaded at runtime
//! from config directories, with agent-level permission gating (allow/deny/ask + wildcard rules).
//!
//! ## Gap Fixes Implemented
//! - G-17-01: Skill registry/discovery with filesystem scanning
//! - G-17-02: Skill tool API surface (OpenCode-compatible contract)
//! - G-17-03: Permission semantics with wildcard allow/deny/ask
//! - G-17-04: Per-session approved caching for skill permissions
//! - G-17-05: Config directory parity (OpenCode-compatible search paths)
//! - G-17-06: Duplicate skill name handling (warn + last writer wins)
//! - G-17-07: Error taxonomy (skill-specific error types)
//! - G-17-08: Skill CLI (list discovered skills)
//! - G-17-09: Skills are data (markdown), not executable code
//! - G-17-10: Tool output formatting parity
//! - G-17-11: Dynamic tool description with <available_skills>
//! - G-17-12: Agent permission model integration

pub mod errors;
pub mod models;
pub mod permissions;
pub mod registry;
pub mod tool;

pub use errors::SkillError;
pub use models::{SkillInfo, SkillMetadata};
pub use permissions::{SkillPermission, SkillPermissionAction, SkillPermissionChecker};
pub use registry::SkillRegistry;
pub use tool::SkillToolProvider;
