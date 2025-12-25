//! MCP Tool Input Types
//!
//! JSON schema-compatible input types for all MCP tools.

use rmcp::schemars::JsonSchema;

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct GrepToolInput {
    pub pattern: String,
    pub include: Option<String>,
    pub path: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct NlSearchToolInput {
    pub query: String,
    pub include: Option<String>,
    pub path: Option<String>,
    pub max_results: Option<usize>,
    pub answer: Option<bool>,
    pub no_rerank: Option<bool>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct GlobToolInput {
    pub pattern: String,
    pub path: Option<String>,
    pub include_dirs: Option<bool>,
    pub ignore_case: Option<bool>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct ListToolInput {
    pub path: Option<String>,
    pub pattern: Option<String>,
    pub ignore_case: Option<bool>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct ReadToolInput {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct EditToolInput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    #[serde(default)]
    pub replace_all: Option<bool>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct WriteToolInput {
    pub file_path: String,
    pub content: String,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}
