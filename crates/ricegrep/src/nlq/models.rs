use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuery {
    pub original: String,
    pub tokens: Vec<String>,
    pub language: String,
    pub complexity: QueryComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedQuery {
    pub parsed: ParsedQuery,
    pub intent: QueryIntent,
    pub expanded_terms: Vec<String>,
    pub filters: Vec<QueryFilter>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryIntent {
    CodeSearch,
    APISearch,
    DocumentationSearch,
    BugReportSearch,
    GeneralSearch,
}
