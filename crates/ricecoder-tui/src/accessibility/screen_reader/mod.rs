//! Screen reader support module

pub mod announcer;
pub mod live_region;

pub use announcer::{Announcement, AnnouncementPriority, ScreenReaderAnnouncer};
pub use live_region::LiveRegion;
// Note: AriaLive and AriaRelevant are exported from crate::accessibility::aria
