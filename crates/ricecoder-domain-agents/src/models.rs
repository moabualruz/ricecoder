//! Data models for domain-specific agents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported domains for specialized agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    /// Frontend development (React, Vue, Angular, etc.)
    Frontend,
    /// Backend development (Node.js, Python, Go, etc.)
    Backend,
    /// DevOps and infrastructure
    DevOps,
    /// Mobile development (iOS, Android, Flutter)
    Mobile,
    /// Data science and ML
    DataScience,
    /// Security and compliance
    Security,
    /// Database design and optimization
    Database,
    /// API design and development
    Api,
    /// Testing and QA
    Testing,
    /// Documentation
    Documentation,
}

impl Domain {
    /// Get all available domains
    pub fn all() -> &'static [Domain] {
        &[
            Domain::Frontend,
            Domain::Backend,
            Domain::DevOps,
            Domain::Mobile,
            Domain::DataScience,
            Domain::Security,
            Domain::Database,
            Domain::Api,
            Domain::Testing,
            Domain::Documentation,
        ]
    }

    /// Get domain name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::Frontend => "frontend",
            Domain::Backend => "backend",
            Domain::DevOps => "devops",
            Domain::Mobile => "mobile",
            Domain::DataScience => "data_science",
            Domain::Security => "security",
            Domain::Database => "database",
            Domain::Api => "api",
            Domain::Testing => "testing",
            Domain::Documentation => "documentation",
        }
    }

    /// Parse domain from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "frontend" => Some(Domain::Frontend),
            "backend" => Some(Domain::Backend),
            "devops" => Some(Domain::DevOps),
            "mobile" => Some(Domain::Mobile),
            "data_science" | "datascience" => Some(Domain::DataScience),
            "security" => Some(Domain::Security),
            "database" => Some(Domain::Database),
            "api" => Some(Domain::Api),
            "testing" => Some(Domain::Testing),
            "documentation" => Some(Domain::Documentation),
            _ => None,
        }
    }
}

/// Domain-specific knowledge entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Unique identifier for the knowledge entry
    pub id: String,
    /// Category of knowledge (e.g., "best_practices", "patterns", "tools")
    pub category: String,
    /// Title or name of the knowledge
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Related tags for searching
    pub tags: Vec<String>,
    /// Example code or usage
    pub example: Option<String>,
    /// References or links
    pub references: Vec<String>,
}

/// Domain knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Domain this knowledge base covers
    pub domain: String,
    /// Version of the knowledge base
    pub version: String,
    /// Last updated timestamp
    pub last_updated: String,
    /// Collection of knowledge entries
    pub entries: Vec<KnowledgeEntry>,
    /// Metadata about the knowledge base
    pub metadata: HashMap<String, String>,
}

impl KnowledgeBase {
    /// Create a new knowledge base for a domain
    pub fn new(domain: &str, version: &str) -> Self {
        Self {
            domain: domain.to_string(),
            version: version.to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            entries: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a knowledge entry
    pub fn add_entry(&mut self, entry: KnowledgeEntry) {
        self.entries.push(entry);
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Search knowledge entries by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&KnowledgeEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.tags.iter().any(|t| t.contains(tag)))
            .collect()
    }

    /// Search knowledge entries by category
    pub fn search_by_category(&self, category: &str) -> Vec<&KnowledgeEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.category == category)
            .collect()
    }
}

/// Configuration for a domain-specific agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgentConfig {
    /// Domain this agent specializes in
    pub domain: String,
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: String,
    /// System prompt for the agent
    pub system_prompt: String,
    /// Tools available to this agent
    pub tools: Vec<String>,
    /// Model to use for this agent
    pub model: Option<String>,
    /// Temperature for generation
    pub temperature: Option<f32>,
    /// Max tokens for generation
    pub max_tokens: Option<usize>,
    /// Custom configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

/// Domain agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgentMetadata {
    /// Domain identifier
    pub domain: String,
    /// Agent name
    pub name: String,
    /// Agent version
    pub version: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Required tools
    pub required_tools: Vec<String>,
    /// Optional tools
    pub optional_tools: Vec<String>,
}

/// Domain agent registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgentRegistry {
    /// Registered agents by domain
    pub agents: HashMap<String, DomainAgentMetadata>,
    /// Knowledge bases by domain
    pub knowledge_bases: HashMap<String, String>, // domain -> path to knowledge base
}

impl DomainAgentRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            knowledge_bases: HashMap::new(),
        }
    }

    /// Register an agent
    pub fn register_agent(&mut self, domain: &str, metadata: DomainAgentMetadata) {
        self.agents.insert(domain.to_string(), metadata);
    }

    /// Register a knowledge base
    pub fn register_knowledge_base(&mut self, domain: &str, path: String) {
        self.knowledge_bases.insert(domain.to_string(), path);
    }

    /// Get agent for domain
    pub fn get_agent(&self, domain: &str) -> Option<&DomainAgentMetadata> {
        self.agents.get(domain)
    }

    /// Get knowledge base path for domain
    pub fn get_knowledge_base(&self, domain: &str) -> Option<&str> {
        self.knowledge_bases.get(domain).map(|s| s.as_str())
    }
}

impl Default for DomainAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
