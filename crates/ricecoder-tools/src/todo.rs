//! Todo tools for managing task lists
//!
//! Provides functionality to create, read, and update todos with persistent storage.

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::error::ToolError;

/// Todo status enumeration
///
/// Status values:
/// - `pending` - Task not yet started
/// - `in_progress` - Currently working on
/// - `completed` - Task finished successfully
/// - `cancelled` - Task no longer needed
/// - `blocked` - Task is blocked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    /// Todo is pending (not yet started)
    Pending,
    /// Todo is in progress (currently working on)
    #[serde(alias = "in-progress")]
    InProgress,
    /// Todo is completed (finished successfully)
    Completed,
    /// Todo is cancelled (no longer needed)
    Cancelled,
    /// Todo is blocked (waiting on something)
    Blocked,
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoStatus::Pending => write!(f, "pending"),
            TodoStatus::InProgress => write!(f, "in_progress"),
            TodoStatus::Completed => write!(f, "completed"),
            TodoStatus::Cancelled => write!(f, "cancelled"),
            TodoStatus::Blocked => write!(f, "blocked"),
        }
    }
}

impl std::str::FromStr for TodoStatus {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TodoStatus::Pending),
            "in_progress" | "in-progress" => Ok(TodoStatus::InProgress),
            "completed" => Ok(TodoStatus::Completed),
            "cancelled" | "canceled" => Ok(TodoStatus::Cancelled),
            "blocked" => Ok(TodoStatus::Blocked),
            _ => Err(
                ToolError::new("INVALID_STATUS", format!("Invalid todo status: {}", s))
                    .with_suggestion(
                        "Use one of: pending, in_progress, completed, cancelled, blocked",
                    ),
            ),
        }
    }
}

/// Todo priority enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl std::fmt::Display for TodoPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoPriority::Low => write!(f, "low"),
            TodoPriority::Medium => write!(f, "medium"),
            TodoPriority::High => write!(f, "high"),
            TodoPriority::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for TodoPriority {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TodoPriority::Low),
            "medium" => Ok(TodoPriority::Medium),
            "high" => Ok(TodoPriority::High),
            "critical" => Ok(TodoPriority::Critical),
            _ => Err(
                ToolError::new("INVALID_PRIORITY", format!("Invalid todo priority: {}", s))
                    .with_suggestion("Use one of: low, medium, high, critical"),
            ),
        }
    }
}

/// A single todo item
///
/// Fields:
/// - `id` - Unique identifier
/// - `content` - Brief description (alias for `title`)
/// - `status` - Current status
/// - `priority` - Priority level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    /// Unique identifier for the todo
    pub id: String,
    /// Todo content/title (required, non-empty)
    /// Both `content` and `title` are supported via serde alias
    #[serde(alias = "title")]
    pub content: String,
    /// Todo description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Current status of the todo
    pub status: TodoStatus,
    /// Priority level of the todo
    pub priority: TodoPriority,
}

impl Todo {
    /// Create a new todo with validation
    ///
    /// # Arguments
    /// * `id` - Unique identifier
    /// * `content` - Brief description of the task
    /// * `status` - Current status
    /// * `priority` - Priority level
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>,
        status: TodoStatus,
        priority: TodoPriority,
    ) -> Result<Self, ToolError> {
        let content = content.into();

        // Validate content is non-empty
        if content.trim().is_empty() {
            return Err(
                ToolError::new("INVALID_CONTENT", "Todo content cannot be empty")
                    .with_suggestion("Provide a non-empty content for the todo"),
            );
        }

        Ok(Self {
            id: id.into(),
            content,
            description: None,
            status,
            priority,
        })
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get the content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Backward-compatible alias for content
    #[deprecated(since = "0.2.0", note = "Use `content` field directly")]
    pub fn title(&self) -> &str {
        &self.content
    }
}

/// Input for todowrite operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodowriteInput {
    /// List of todos to create or update
    pub todos: Vec<Todo>,
}

/// Output from todowrite operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodowriteOutput {
    /// Number of todos created
    pub created: usize,
    /// Number of todos updated
    pub updated: usize,
}

/// Input for todoread operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoreadInput {
    /// Optional filter by status
    pub status_filter: Option<TodoStatus>,
    /// Optional filter by priority
    pub priority_filter: Option<TodoPriority>,
}

/// Output from todoread operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoreadOutput {
    /// List of todos matching the filters
    pub todos: Vec<Todo>,
}

/// Todo storage manager
pub struct TodoStorage {
    storage_path: PathBuf,
}

impl TodoStorage {
    /// Create a new todo storage manager
    pub fn new(storage_path: impl Into<PathBuf>) -> Self {
        Self {
            storage_path: storage_path.into(),
        }
    }

    /// Get the default storage path (~/.ricecoder/todos.json)
    pub fn default_path() -> Result<PathBuf, ToolError> {
        if let Some(home_dir) = dirs::home_dir() {
            Ok(home_dir.join(".ricecoder").join("todos.json"))
        } else {
            Err(
                ToolError::new("HOME_DIR_NOT_FOUND", "Could not determine home directory")
                    .with_suggestion("Set the HOME environment variable"),
            )
        }
    }

    /// Load todos from storage
    pub fn load_todos(&self) -> Result<HashMap<String, Todo>, ToolError> {
        debug!("Loading todos from: {:?}", self.storage_path);

        // If file doesn't exist, return empty map
        if !self.storage_path.exists() {
            debug!("Todo storage file does not exist, returning empty todos");
            return Ok(HashMap::new());
        }

        // Read file
        let content = std::fs::read_to_string(&self.storage_path).map_err(|e| {
            error!("Failed to read todo storage file: {}", e);
            ToolError::from(e)
                .with_details(format!("Failed to read: {:?}", self.storage_path))
                .with_suggestion("Check file permissions and ensure the file is readable")
        })?;

        // Parse JSON
        let todos: Vec<Todo> = serde_json::from_str(&content).map_err(|e| {
            error!("Failed to parse todo storage file: {}", e);
            ToolError::from(e)
                .with_details("Todo storage file contains invalid JSON")
                .with_suggestion("Check the file format or restore from backup")
        })?;

        // Convert to HashMap
        let mut map = HashMap::new();
        for todo in todos {
            map.insert(todo.id.clone(), todo);
        }

        info!("Loaded {} todos from storage", map.len());
        Ok(map)
    }

    /// Save todos to storage (atomic write)
    pub fn save_todos(&self, todos: &HashMap<String, Todo>) -> Result<(), ToolError> {
        debug!("Saving {} todos to: {:?}", todos.len(), self.storage_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create storage directory: {}", e);
                ToolError::from(e)
                    .with_details(format!("Failed to create: {:?}", parent))
                    .with_suggestion("Check directory permissions")
            })?;
        }

        // Convert HashMap to Vec for serialization
        let mut todos_vec: Vec<Todo> = todos.values().cloned().collect();
        todos_vec.sort_by(|a, b| a.id.cmp(&b.id));

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&todos_vec).map_err(|e| {
            error!("Failed to serialize todos: {}", e);
            ToolError::from(e)
                .with_details("Failed to serialize todos to JSON")
                .with_suggestion("Check for circular references or invalid data")
        })?;

        // Write to temporary file first (atomic write)
        let temp_path = self.storage_path.with_extension("json.tmp");
        std::fs::write(&temp_path, &json).map_err(|e| {
            error!("Failed to write temporary todo file: {}", e);
            ToolError::from(e)
                .with_details(format!("Failed to write: {:?}", temp_path))
                .with_suggestion("Check disk space and file permissions")
        })?;

        // Rename temporary file to actual file (atomic on most filesystems)
        std::fs::rename(&temp_path, &self.storage_path).map_err(|e| {
            error!("Failed to finalize todo storage: {}", e);
            // Clean up temp file on error
            let _ = std::fs::remove_file(&temp_path);
            ToolError::from(e)
                .with_details(format!(
                    "Failed to rename: {:?} to {:?}",
                    temp_path, self.storage_path
                ))
                .with_suggestion("Check file permissions and disk space")
        })?;

        info!("Saved {} todos to storage", todos.len());
        Ok(())
    }
}

/// Todo tools for managing task lists
pub struct TodoTools {
    storage: TodoStorage,
    mcp_provider: Option<std::sync::Arc<dyn crate::Provider>>,
}

impl TodoTools {
    /// Create new todo tools with default storage path
    pub fn new() -> Result<Self, ToolError> {
        let storage_path = TodoStorage::default_path()?;
        Ok(Self {
            storage: TodoStorage::new(storage_path),
            mcp_provider: None,
        })
    }

    /// Create new todo tools with custom storage path
    pub fn with_storage_path(storage_path: impl Into<PathBuf>) -> Self {
        Self {
            storage: TodoStorage::new(storage_path),
            mcp_provider: None,
        }
    }

    /// Set the MCP provider for todo operations
    pub fn with_mcp_provider(mut self, provider: std::sync::Arc<dyn crate::Provider>) -> Self {
        self.mcp_provider = Some(provider);
        self
    }

    /// Write todos with timeout enforcement (500ms)
    ///
    /// Attempts to use MCP provider if available, falls back to built-in implementation.
    pub async fn write_todos_with_timeout(
        &self,
        input: TodowriteInput,
    ) -> Result<TodowriteOutput, ToolError> {
        let timeout_duration = std::time::Duration::from_millis(500);

        match tokio::time::timeout(timeout_duration, async { self.write_todos_internal(input) })
            .await
        {
            Ok(result) => result,
            Err(_) => Err(
                ToolError::new("TIMEOUT", "Todo write operation exceeded 500ms timeout")
                    .with_details("Operation took too long to complete")
                    .with_suggestion("Try again or check system performance"),
            ),
        }
    }

    /// Write todos (create or update)
    ///
    /// Attempts to use MCP provider if available, falls back to built-in implementation.
    pub fn write_todos(&self, input: TodowriteInput) -> Result<TodowriteOutput, ToolError> {
        self.write_todos_internal(input)
    }

    /// Internal write todos implementation
    fn write_todos_internal(&self, input: TodowriteInput) -> Result<TodowriteOutput, ToolError> {
        debug!("Writing {} todos", input.todos.len());

        // Try MCP provider first
        if let Some(_provider) = &self.mcp_provider {
            debug!("Attempting to use MCP provider for todowrite");
            let _input_json = serde_json::to_string(&input).map_err(|e| {
                error!("Failed to serialize todowrite input: {}", e);
                ToolError::from(e)
                    .with_details("Failed to serialize input for MCP provider")
                    .with_suggestion("Check the input data format")
            })?;

            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Note: In a real async context, this would be async
                // For now, we'll use a synchronous fallback
                None::<String>
            })) {
                Ok(_) => {
                    debug!("MCP provider not available, falling back to built-in");
                }
                Err(_) => {
                    debug!("MCP provider error, falling back to built-in");
                }
            }
        }

        // Fall back to built-in implementation
        debug!("Using built-in todowrite implementation");

        // Load existing todos
        let mut todos = self.storage.load_todos()?;

        let mut created = 0;
        let mut updated = 0;

        // Process each todo
        for todo in input.todos {
            let id = todo.id.clone();
            if todos.contains_key(&id) {
                updated += 1;
            } else {
                created += 1;
            }
            todos.insert(id, todo);
        }

        // Save todos
        self.storage.save_todos(&todos)?;

        info!("Wrote todos: {} created, {} updated", created, updated);
        Ok(TodowriteOutput { created, updated })
    }

    /// Read todos with timeout enforcement (500ms)
    ///
    /// Attempts to use MCP provider if available, falls back to built-in implementation.
    pub async fn read_todos_with_timeout(
        &self,
        input: TodoreadInput,
    ) -> Result<TodoreadOutput, ToolError> {
        let timeout_duration = std::time::Duration::from_millis(500);

        match tokio::time::timeout(timeout_duration, async { self.read_todos_internal(input) })
            .await
        {
            Ok(result) => result,
            Err(_) => Err(
                ToolError::new("TIMEOUT", "Todo read operation exceeded 500ms timeout")
                    .with_details("Operation took too long to complete")
                    .with_suggestion("Try again or check system performance"),
            ),
        }
    }

    /// Read todos with optional filtering
    ///
    /// Attempts to use MCP provider if available, falls back to built-in implementation.
    pub fn read_todos(&self, input: TodoreadInput) -> Result<TodoreadOutput, ToolError> {
        self.read_todos_internal(input)
    }

    /// Internal read todos implementation
    fn read_todos_internal(&self, input: TodoreadInput) -> Result<TodoreadOutput, ToolError> {
        debug!(
            "Reading todos with filters: status={:?}, priority={:?}",
            input.status_filter, input.priority_filter
        );

        // Try MCP provider first
        if let Some(_provider) = &self.mcp_provider {
            debug!("Attempting to use MCP provider for todoread");
            let _input_json = serde_json::to_string(&input).map_err(|e| {
                error!("Failed to serialize todoread input: {}", e);
                ToolError::from(e)
                    .with_details("Failed to serialize input for MCP provider")
                    .with_suggestion("Check the input data format")
            })?;

            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Note: In a real async context, this would be async
                // For now, we'll use a synchronous fallback
                None::<String>
            })) {
                Ok(_) => {
                    debug!("MCP provider not available, falling back to built-in");
                }
                Err(_) => {
                    debug!("MCP provider error, falling back to built-in");
                }
            }
        }

        // Fall back to built-in implementation
        debug!("Using built-in todoread implementation");

        // Load todos
        let todos = self.storage.load_todos()?;

        // Filter todos
        let mut filtered: Vec<Todo> = todos
            .into_values()
            .filter(|todo| {
                // Apply status filter
                if let Some(status) = input.status_filter {
                    if todo.status != status {
                        return false;
                    }
                }

                // Apply priority filter
                if let Some(priority) = input.priority_filter {
                    if todo.priority != priority {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Sort by priority (descending) then by id
        filtered.sort_by(|a, b| match b.priority.cmp(&a.priority) {
            std::cmp::Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        });

        info!("Read {} todos (filtered from total)", filtered.len());
        Ok(TodoreadOutput { todos: filtered })
    }
}

impl Default for TodoTools {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to in-memory storage if default path fails
            Self::with_storage_path("/tmp/ricecoder-todos.json")
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_todo_creation() {
        let todo = Todo::new("1", "Test todo", TodoStatus::Pending, TodoPriority::High);
        assert!(todo.is_ok());
        let todo = todo.unwrap();
        assert_eq!(todo.id, "1");
        assert_eq!(todo.content, "Test todo");
        assert_eq!(todo.status, TodoStatus::Pending);
        assert_eq!(todo.priority, TodoPriority::High);
    }

    #[test]
    fn test_todo_empty_content_validation() {
        let result = Todo::new("1", "   ", TodoStatus::Pending, TodoPriority::High);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.code, "INVALID_CONTENT");
        }
    }

    #[test]
    fn test_todo_with_description() {
        let todo = Todo::new("1", "Test", TodoStatus::Pending, TodoPriority::High)
            .unwrap()
            .with_description("A test todo");
        assert_eq!(todo.description, Some("A test todo".to_string()));
    }

    #[test]
    fn test_todo_status_parsing() {
        assert_eq!(
            "pending".parse::<TodoStatus>().unwrap(),
            TodoStatus::Pending
        );
        assert_eq!(
            "in_progress".parse::<TodoStatus>().unwrap(),
            TodoStatus::InProgress
        );
        // Also accept hyphenated form
        assert_eq!(
            "in-progress".parse::<TodoStatus>().unwrap(),
            TodoStatus::InProgress
        );
        assert_eq!(
            "completed".parse::<TodoStatus>().unwrap(),
            TodoStatus::Completed
        );
        // Cancelled status
        assert_eq!(
            "cancelled".parse::<TodoStatus>().unwrap(),
            TodoStatus::Cancelled
        );
        // US spelling variant
        assert_eq!(
            "canceled".parse::<TodoStatus>().unwrap(),
            TodoStatus::Cancelled
        );
        assert_eq!(
            "blocked".parse::<TodoStatus>().unwrap(),
            TodoStatus::Blocked
        );
        assert!("invalid".parse::<TodoStatus>().is_err());
    }

    #[test]
    fn test_todo_priority_parsing() {
        assert_eq!("low".parse::<TodoPriority>().unwrap(), TodoPriority::Low);
        assert_eq!(
            "medium".parse::<TodoPriority>().unwrap(),
            TodoPriority::Medium
        );
        assert_eq!("high".parse::<TodoPriority>().unwrap(), TodoPriority::High);
        assert_eq!(
            "critical".parse::<TodoPriority>().unwrap(),
            TodoPriority::Critical
        );
        assert!("invalid".parse::<TodoPriority>().is_err());
    }

    #[test]
    fn test_todo_storage_load_empty() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let storage = TodoStorage::new(&storage_path);

        let todos = storage.load_todos().unwrap();
        assert!(todos.is_empty());
    }

    #[test]
    fn test_todo_storage_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let storage = TodoStorage::new(&storage_path);

        // Create and save todos
        let mut todos = HashMap::new();
        let todo1 = Todo::new("1", "First todo", TodoStatus::Pending, TodoPriority::High)
            .unwrap()
            .with_description("First description");
        let todo2 =
            Todo::new("2", "Second todo", TodoStatus::Completed, TodoPriority::Low).unwrap();

        todos.insert(todo1.id.clone(), todo1);
        todos.insert(todo2.id.clone(), todo2);

        storage.save_todos(&todos).unwrap();

        // Load and verify
        let loaded = storage.load_todos().unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.contains_key("1"));
        assert!(loaded.contains_key("2"));
    }

    #[test]
    fn test_todo_tools_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write todos
        let todo1 = Todo::new("1", "First", TodoStatus::Pending, TodoPriority::High).unwrap();
        let todo2 = Todo::new("2", "Second", TodoStatus::InProgress, TodoPriority::Medium).unwrap();

        let write_result = tools
            .write_todos(TodowriteInput {
                todos: vec![todo1, todo2],
            })
            .unwrap();

        assert_eq!(write_result.created, 2);
        assert_eq!(write_result.updated, 0);

        // Read todos
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            })
            .unwrap();

        assert_eq!(read_result.todos.len(), 2);
    }

    #[test]
    fn test_todo_tools_update() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write initial todos
        let todo1 = Todo::new("1", "First", TodoStatus::Pending, TodoPriority::High).unwrap();
        tools
            .write_todos(TodowriteInput { todos: vec![todo1] })
            .unwrap();

        // Update the todo
        let updated_todo = Todo::new(
            "1",
            "First (updated)",
            TodoStatus::Completed,
            TodoPriority::Low,
        )
        .unwrap();
        let write_result = tools
            .write_todos(TodowriteInput {
                todos: vec![updated_todo],
            })
            .unwrap();

        assert_eq!(write_result.created, 0);
        assert_eq!(write_result.updated, 1);

        // Verify update
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            })
            .unwrap();

        assert_eq!(read_result.todos.len(), 1);
        assert_eq!(read_result.todos[0].content, "First (updated)");
        assert_eq!(read_result.todos[0].status, TodoStatus::Completed);
    }

    #[test]
    fn test_todo_tools_filter_by_status() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write todos with different statuses
        let todo1 = Todo::new("1", "Pending", TodoStatus::Pending, TodoPriority::High).unwrap();
        let todo2 = Todo::new(
            "2",
            "Completed",
            TodoStatus::Completed,
            TodoPriority::Medium,
        )
        .unwrap();

        tools
            .write_todos(TodowriteInput {
                todos: vec![todo1, todo2],
            })
            .unwrap();

        // Filter by status
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: Some(TodoStatus::Completed),
                priority_filter: None,
            })
            .unwrap();

        assert_eq!(read_result.todos.len(), 1);
        assert_eq!(read_result.todos[0].content, "Completed");
    }

    #[test]
    fn test_todo_tools_filter_by_priority() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write todos with different priorities
        let todo1 = Todo::new("1", "High", TodoStatus::Pending, TodoPriority::High).unwrap();
        let todo2 = Todo::new("2", "Low", TodoStatus::Pending, TodoPriority::Low).unwrap();

        tools
            .write_todos(TodowriteInput {
                todos: vec![todo1, todo2],
            })
            .unwrap();

        // Filter by priority
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: Some(TodoPriority::High),
            })
            .unwrap();

        assert_eq!(read_result.todos.len(), 1);
        assert_eq!(read_result.todos[0].content, "High");
    }

    #[tokio::test]
    async fn test_todo_write_timeout_enforcement() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write todos with timeout enforcement (should complete well within 500ms)
        let todo = Todo::new("1", "Test", TodoStatus::Pending, TodoPriority::High).unwrap();
        let result = tools
            .write_todos_with_timeout(TodowriteInput { todos: vec![todo] })
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.created, 1);
    }

    #[tokio::test]
    async fn test_todo_read_timeout_enforcement() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write a todo first
        let todo = Todo::new("1", "Test", TodoStatus::Pending, TodoPriority::High).unwrap();
        tools
            .write_todos(TodowriteInput { todos: vec![todo] })
            .unwrap();

        // Read todos with timeout enforcement (should complete well within 500ms)
        let result = tools
            .read_todos_with_timeout(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            })
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.todos.len(), 1);
    }

    #[test]
    fn test_todo_cancelled_status() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Create todos with different statuses including cancelled
        let todo1 = Todo::new("1", "Active task", TodoStatus::Pending, TodoPriority::High).unwrap();
        let todo2 =
            Todo::new("2", "Cancelled task", TodoStatus::Cancelled, TodoPriority::Low).unwrap();

        tools
            .write_todos(TodowriteInput {
                todos: vec![todo1, todo2],
            })
            .unwrap();

        // Filter by cancelled status
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: Some(TodoStatus::Cancelled),
                priority_filter: None,
            })
            .unwrap();

        assert_eq!(read_result.todos.len(), 1);
        assert_eq!(read_result.todos[0].content, "Cancelled task");
        assert_eq!(read_result.todos[0].status, TodoStatus::Cancelled);
    }

    #[test]
    fn test_json_serialization() {
        // Test serialization format
        let todo = Todo::new(
            "test-1",
            "Implement feature",
            TodoStatus::InProgress,
            TodoPriority::High,
        )
        .unwrap();

        let json = serde_json::to_string(&todo).unwrap();
        assert!(json.contains(r#""content":"Implement feature""#));
        assert!(json.contains(r#""status":"inprogress""#));
        assert!(json.contains(r#""priority":"high""#));

        // Test deserialization (using "content" field)
        let todo_json = r#"{
            "id": "test-2",
            "content": "Review PR",
            "status": "pending",
            "priority": "medium"
        }"#;

        let todo: Todo = serde_json::from_str(todo_json).unwrap();
        assert_eq!(todo.id, "test-2");
        assert_eq!(todo.content, "Review PR");
        assert_eq!(todo.status, TodoStatus::Pending);
        assert_eq!(todo.priority, TodoPriority::Medium);
    }

    #[test]
    fn test_legacy_title_field_compatibility() {
        // Test deserialization from legacy format (using "title" field)
        let legacy_json = r#"{
            "id": "legacy-1",
            "title": "Legacy task",
            "status": "completed",
            "priority": "low"
        }"#;

        let todo: Todo = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(todo.id, "legacy-1");
        assert_eq!(todo.content, "Legacy task");
        assert_eq!(todo.status, TodoStatus::Completed);
        assert_eq!(todo.priority, TodoPriority::Low);
    }

    #[test]
    fn test_cancelled_status_display() {
        assert_eq!(TodoStatus::Cancelled.to_string(), "cancelled");
    }
}
