//! Domain knowledge base management

use crate::error::{DomainAgentError, Result};
use crate::models::{Domain, KnowledgeBase, KnowledgeEntry};
use ricecoder_storage::PathResolver;
use tracing::{debug, info};

/// Knowledge base manager for domain-specific agents
pub struct KnowledgeBaseManager {
    knowledge_bases: std::collections::HashMap<String, KnowledgeBase>,
}

impl KnowledgeBaseManager {
    /// Create a new knowledge base manager
    pub fn new() -> Self {
        Self {
            knowledge_bases: std::collections::HashMap::new(),
        }
    }
}

impl Default for KnowledgeBaseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeBaseManager {

    /// Load knowledge base for a domain
    pub async fn load_knowledge_base(&mut self, domain: &str) -> Result<()> {
        debug!("Loading knowledge base for domain: {}", domain);

        let kb_path = self.get_knowledge_base_path(domain);

        if kb_path.exists() {
            let content = tokio::fs::read_to_string(&kb_path).await?;
            let kb: KnowledgeBase = serde_yaml::from_str(&content)?;
            self.knowledge_bases.insert(domain.to_string(), kb);
            info!("Loaded knowledge base for domain: {}", domain);
        } else {
            debug!("Knowledge base not found for domain: {}, creating empty", domain);
            let kb = KnowledgeBase::new(domain, "1.0.0");
            self.knowledge_bases.insert(domain.to_string(), kb);
        }

        Ok(())
    }

    /// Get knowledge base for a domain
    pub fn get_knowledge_base(&self, domain: &str) -> Result<&KnowledgeBase> {
        self.knowledge_bases
            .get(domain)
            .ok_or_else(|| DomainAgentError::KnowledgeNotAvailable(domain.to_string()))
    }

    /// Get mutable knowledge base for a domain
    pub fn get_knowledge_base_mut(&mut self, domain: &str) -> Result<&mut KnowledgeBase> {
        self.knowledge_bases
            .get_mut(domain)
            .ok_or_else(|| DomainAgentError::KnowledgeNotAvailable(domain.to_string()))
    }

    /// Add knowledge entry to a domain
    pub fn add_knowledge_entry(&mut self, domain: &str, entry: KnowledgeEntry) -> Result<()> {
        let kb = self.get_knowledge_base_mut(domain)?;
        kb.add_entry(entry);
        Ok(())
    }

    /// Search knowledge by tag
    pub fn search_by_tag(&self, domain: &str, tag: &str) -> Result<Vec<&KnowledgeEntry>> {
        let kb = self.get_knowledge_base(domain)?;
        Ok(kb.search_by_tag(tag))
    }

    /// Search knowledge by category
    pub fn search_by_category(&self, domain: &str, category: &str) -> Result<Vec<&KnowledgeEntry>> {
        let kb = self.get_knowledge_base(domain)?;
        Ok(kb.search_by_category(category))
    }

    /// Save knowledge base to disk
    pub async fn save_knowledge_base(&self, domain: &str) -> Result<()> {
        let kb = self.get_knowledge_base(domain)?;
        let kb_path = self.get_knowledge_base_path(domain);

        // Create parent directory if it doesn't exist
        if let Some(parent) = kb_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_yaml::to_string(kb)?;
        tokio::fs::write(&kb_path, content).await?;
        info!("Saved knowledge base for domain: {}", domain);

        Ok(())
    }

    /// Get path to knowledge base file
    fn get_knowledge_base_path(&self, domain: &str) -> std::path::PathBuf {
        PathResolver::resolve_project_path()
            .join("knowledge_bases")
            .join(format!("{}.yaml", domain))
    }

    /// Load all knowledge bases for all domains
    pub async fn load_all_knowledge_bases(&mut self) -> Result<()> {
        debug!("Loading all knowledge bases");

        for domain in Domain::all() {
            self.load_knowledge_base(domain.as_str()).await?;
        }

        info!("Loaded all knowledge bases");
        Ok(())
    }

    /// Get all loaded domains
    pub fn get_loaded_domains(&self) -> Vec<&str> {
        self.knowledge_bases.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_creation() {
        let kb = KnowledgeBase::new("frontend", "1.0.0");
        assert_eq!(kb.domain, "frontend");
        assert_eq!(kb.version, "1.0.0");
        assert!(kb.entries.is_empty());
    }

    #[test]
    fn test_add_knowledge_entry() {
        let mut kb = KnowledgeBase::new("frontend", "1.0.0");
        let entry = KnowledgeEntry {
            id: "react-hooks".to_string(),
            category: "patterns".to_string(),
            title: "React Hooks".to_string(),
            description: "Best practices for using React Hooks".to_string(),
            tags: vec!["react".to_string(), "hooks".to_string()],
            example: Some("const [count, setCount] = useState(0);".to_string()),
            references: vec!["https://react.dev/reference/react/hooks".to_string()],
        };

        kb.add_entry(entry);
        assert_eq!(kb.entries.len(), 1);
    }

    #[test]
    fn test_search_by_tag() {
        let mut kb = KnowledgeBase::new("frontend", "1.0.0");
        let entry = KnowledgeEntry {
            id: "react-hooks".to_string(),
            category: "patterns".to_string(),
            title: "React Hooks".to_string(),
            description: "Best practices for using React Hooks".to_string(),
            tags: vec!["react".to_string(), "hooks".to_string()],
            example: None,
            references: vec![],
        };

        kb.add_entry(entry);
        let results = kb.search_by_tag("react");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_category() {
        let mut kb = KnowledgeBase::new("frontend", "1.0.0");
        let entry = KnowledgeEntry {
            id: "react-hooks".to_string(),
            category: "patterns".to_string(),
            title: "React Hooks".to_string(),
            description: "Best practices for using React Hooks".to_string(),
            tags: vec![],
            example: None,
            references: vec![],
        };

        kb.add_entry(entry);
        let results = kb.search_by_category("patterns");
        assert_eq!(results.len(), 1);
    }
}
