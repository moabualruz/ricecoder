//! Audit log querying and filtering

use super::models::{AuditAction, AuditLogEntry, AuditResult};
use chrono::{DateTime, Utc};

/// Filter criteria for audit log queries
#[derive(Debug, Clone)]
pub struct QueryFilter {
    /// Filter by tool name (optional)
    pub tool: Option<String>,
    /// Filter by action (optional)
    pub action: Option<AuditAction>,
    /// Filter by result (optional)
    pub result: Option<AuditResult>,
    /// Filter by agent (optional)
    pub agent: Option<String>,
    /// Filter by start date (optional)
    pub start_date: Option<DateTime<Utc>>,
    /// Filter by end date (optional)
    pub end_date: Option<DateTime<Utc>>,
}

impl QueryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self {
            tool: None,
            action: None,
            result: None,
            agent: None,
            start_date: None,
            end_date: None,
        }
    }

    /// Filter by tool name
    pub fn with_tool(mut self, tool: String) -> Self {
        self.tool = Some(tool);
        self
    }

    /// Filter by action
    pub fn with_action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Filter by result
    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = Some(result);
        self
    }

    /// Filter by agent
    pub fn with_agent(mut self, agent: String) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Filter by start date
    pub fn with_start_date(mut self, date: DateTime<Utc>) -> Self {
        self.start_date = Some(date);
        self
    }

    /// Filter by end date
    pub fn with_end_date(mut self, date: DateTime<Utc>) -> Self {
        self.end_date = Some(date);
        self
    }

    /// Check if an entry matches this filter
    fn matches(&self, entry: &AuditLogEntry) -> bool {
        // Check tool filter
        if let Some(ref tool) = self.tool {
            if entry.tool != *tool {
                return false;
            }
        }

        // Check action filter
        if let Some(action) = self.action {
            if entry.action != action {
                return false;
            }
        }

        // Check result filter
        if let Some(result) = self.result {
            if entry.result != result {
                return false;
            }
        }

        // Check agent filter
        if let Some(ref agent) = self.agent {
            if entry.agent.as_ref() != Some(agent) {
                return false;
            }
        }

        // Check start date filter
        if let Some(start_date) = self.start_date {
            if entry.timestamp < start_date {
                return false;
            }
        }

        // Check end date filter
        if let Some(end_date) = self.end_date {
            if entry.timestamp > end_date {
                return false;
            }
        }

        true
    }
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination parameters
#[derive(Debug, Clone)]
pub struct Pagination {
    /// Number of results per page
    pub limit: usize,
    /// Number of results to skip
    pub offset: usize,
}

impl Pagination {
    /// Create a new pagination with limit and offset
    pub fn new(limit: usize, offset: usize) -> Self {
        Self { limit, offset }
    }

    /// Create pagination for the first page
    pub fn first_page(limit: usize) -> Self {
        Self { limit, offset: 0 }
    }

    /// Get the next page pagination
    pub fn next_page(&self) -> Self {
        Self {
            limit: self.limit,
            offset: self.offset + self.limit,
        }
    }

    /// Get the previous page pagination
    pub fn prev_page(&self) -> Option<Self> {
        if self.offset >= self.limit {
            Some(Self {
                limit: self.limit,
                offset: self.offset - self.limit,
            })
        } else {
            None
        }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(10, 0)
    }
}

/// Query result with pagination metadata
#[derive(Debug, Clone)]
pub struct AuditQuery {
    /// Filtered and paginated entries
    pub entries: Vec<AuditLogEntry>,
    /// Total number of entries matching the filter
    pub total: usize,
    /// Current pagination
    pub pagination: Pagination,
}

impl AuditQuery {
    /// Execute a query on the given entries
    pub fn execute(
        entries: &[AuditLogEntry],
        filter: &QueryFilter,
        pagination: &Pagination,
    ) -> Self {
        // Filter entries
        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| filter.matches(e))
            .cloned()
            .collect();

        let total = filtered.len();

        // Apply pagination
        let start = pagination.offset;
        let end = std::cmp::min(start + pagination.limit, total);

        let paginated: Vec<_> = if start < total {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };

        Self {
            entries: paginated,
            total,
            pagination: pagination.clone(),
        }
    }

    /// Get the total number of pages
    pub fn total_pages(&self) -> usize {
        if self.pagination.limit == 0 {
            return 0;
        }
        self.total.div_ceil(self.pagination.limit)
    }

    /// Get the current page number (1-indexed)
    pub fn current_page(&self) -> usize {
        if self.pagination.limit == 0 {
            return 0;
        }
        (self.pagination.offset / self.pagination.limit) + 1
    }

    /// Check if there is a next page
    pub fn has_next_page(&self) -> bool {
        self.pagination.offset + self.pagination.limit < self.total
    }

    /// Check if there is a previous page
    pub fn has_prev_page(&self) -> bool {
        self.pagination.offset > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entries() -> Vec<AuditLogEntry> {
        vec![
            AuditLogEntry::new(
                "tool1".to_string(),
                AuditAction::Allowed,
                AuditResult::Success,
            ),
            AuditLogEntry::with_agent(
                "tool2".to_string(),
                AuditAction::Denied,
                AuditResult::Blocked,
                "agent1".to_string(),
            ),
            AuditLogEntry::new(
                "tool1".to_string(),
                AuditAction::Prompted,
                AuditResult::Success,
            ),
            AuditLogEntry::with_agent(
                "tool3".to_string(),
                AuditAction::Allowed,
                AuditResult::Success,
                "agent2".to_string(),
            ),
        ]
    }

    #[test]
    fn test_filter_by_tool() {
        let entries = create_test_entries();
        let filter = QueryFilter::new().with_tool("tool1".to_string());
        let pagination = Pagination::first_page(10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 2);
        assert_eq!(result.entries.len(), 2);
        assert!(result.entries.iter().all(|e| e.tool == "tool1"));
    }

    #[test]
    fn test_filter_by_action() {
        let entries = create_test_entries();
        let filter = QueryFilter::new().with_action(AuditAction::Allowed);
        let pagination = Pagination::first_page(10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 2);
        assert!(result
            .entries
            .iter()
            .all(|e| e.action == AuditAction::Allowed));
    }

    #[test]
    fn test_filter_by_result() {
        let entries = create_test_entries();
        let filter = QueryFilter::new().with_result(AuditResult::Success);
        let pagination = Pagination::first_page(10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 3);
        assert!(result
            .entries
            .iter()
            .all(|e| e.result == AuditResult::Success));
    }

    #[test]
    fn test_filter_by_agent() {
        let entries = create_test_entries();
        let filter = QueryFilter::new().with_agent("agent1".to_string());
        let pagination = Pagination::first_page(10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 1);
        assert_eq!(result.entries[0].agent, Some("agent1".to_string()));
    }

    #[test]
    fn test_combined_filters() {
        let entries = create_test_entries();
        let filter = QueryFilter::new()
            .with_tool("tool1".to_string())
            .with_action(AuditAction::Allowed);
        let pagination = Pagination::first_page(10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 1);
        assert_eq!(result.entries[0].tool, "tool1");
        assert_eq!(result.entries[0].action, AuditAction::Allowed);
    }

    #[test]
    fn test_pagination_first_page() {
        let entries = create_test_entries();
        let filter = QueryFilter::new();
        let pagination = Pagination::first_page(2);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 4);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.current_page(), 1);
        assert!(result.has_next_page());
        assert!(!result.has_prev_page());
    }

    #[test]
    fn test_pagination_second_page() {
        let entries = create_test_entries();
        let filter = QueryFilter::new();
        let pagination = Pagination::new(2, 2);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total, 4);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.current_page(), 2);
        assert!(!result.has_next_page());
        assert!(result.has_prev_page());
    }

    #[test]
    fn test_pagination_total_pages() {
        let entries = create_test_entries();
        let filter = QueryFilter::new();
        let pagination = Pagination::first_page(2);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.total_pages(), 2);
    }

    #[test]
    fn test_pagination_offset_beyond_total() {
        let entries = create_test_entries();
        let filter = QueryFilter::new();
        let pagination = Pagination::new(2, 10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        assert_eq!(result.entries.len(), 0);
    }

    #[test]
    fn test_pagination_next_page() {
        let pagination = Pagination::first_page(2);
        let next = pagination.next_page();

        assert_eq!(next.offset, 2);
        assert_eq!(next.limit, 2);
    }

    #[test]
    fn test_pagination_prev_page() {
        let pagination = Pagination::new(2, 2);
        let prev = pagination.prev_page();

        assert!(prev.is_some());
        assert_eq!(prev.unwrap().offset, 0);
    }

    #[test]
    fn test_pagination_prev_page_first_page() {
        let pagination = Pagination::first_page(2);
        let prev = pagination.prev_page();

        assert!(prev.is_none());
    }

    #[test]
    fn test_query_filter_default() {
        let filter = QueryFilter::default();
        let entries = create_test_entries();

        assert!(entries.iter().all(|e| filter.matches(e)));
    }

    #[test]
    fn test_date_range_filter() {
        let entries = create_test_entries();
        let now = Utc::now();
        let future = now + chrono::Duration::hours(1);

        let filter = QueryFilter::new()
            .with_start_date(now - chrono::Duration::hours(1))
            .with_end_date(future);
        let pagination = Pagination::first_page(10);

        let result = AuditQuery::execute(&entries, &filter, &pagination);

        // All entries should be within the date range
        assert_eq!(result.total, entries.len());
    }
}
