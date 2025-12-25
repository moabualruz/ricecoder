//! Live region for dynamic content updates (ARIA live regions)

use crate::accessibility::aria::{AriaLive, AriaRelevant};

/// Live region for dynamic content updates (ARIA live regions)
#[derive(Debug, Clone, PartialEq)]
pub struct LiveRegion {
    pub id: String,
    pub content: String,
    pub aria_live: AriaLive,
    pub aria_atomic: bool,
    pub aria_relevant: AriaRelevant,
    pub last_update: std::time::Instant,
}
