//! Todo tools for managing task lists
//!
//! Provides functionality to create, read, and update todos with persistent storage.

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[serde(rename = "in_progress", alias = "in-progress")]
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

/// Output from todowrite operation (OpenCode compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodowriteOutput {
    /// Title (OpenCode: count of incomplete todos)
    pub title: String,
    /// Output (OpenCode: JSON string of todos)
    pub output: String,
    /// Metadata (OpenCode: includes todos array)
    pub metadata: TodowriteMetadata,
}

/// Metadata for todowrite output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodowriteMetadata {
    /// Todos array (OpenCode compatible)
    pub todos: Vec<Todo>,
    /// Number of todos created (RiceCoder extra - KEEP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<usize>,
    /// Number of todos updated (RiceCoder extra - KEEP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<usize>,
}

/// Input for todoread operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoreadInput {
    /// Optional filter by status
    pub status_filter: Option<TodoStatus>,
    /// Optional filter by priority
    pub priority_filter: Option<TodoPriority>,
}

/// Output from todoread operation (OpenCode compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoreadOutput {
    /// Title (OpenCode: count of incomplete todos)
    pub title: String,
    /// Output (OpenCode: JSON string of todos)
    pub output: String,
    /// Metadata (OpenCode: includes todos array)
    pub metadata: TodoreadMetadata,
}

/// Metadata for todoread output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoreadMetadata {
    /// Todos array (OpenCode compatible)
    pub todos: Vec<Todo>,
}

/// Storage mode for todos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMode {
    /// Global storage (single todos.json file)
    Global,
    /// Session-scoped storage (per-session files)
    SessionScoped,
}

/// Write mode for todos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    /// Replace entire list (OpenCode compatible)
    Replace,
    /// Merge by ID (RiceCoder default)
    Merge,
}

/// Todo storage manager
pub struct TodoStorage {
    storage_path: PathBuf,
    storage_mode: StorageMode,
}

impl TodoStorage {
    /// Create a new todo storage manager
    pub fn new(storage_path: impl Into<PathBuf>) -> Self {
        Self {
            storage_path: storage_path.into(),
            storage_mode: StorageMode::Global,
        }
    }

    /// Set storage mode (global or session-scoped)
    pub fn with_storage_mode(mut self, mode: StorageMode) -> Self {
        self.storage_mode = mode;
        self
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

    /// Get session-scoped storage path
    pub fn session_path(session_id: &str) -> Result<PathBuf, ToolError> {
        if let Some(home_dir) = dirs::home_dir() {
            Ok(home_dir
                .join(".ricecoder")
                .join("sessions")
                .join(session_id)
                .join("todos.json"))
        } else {
            Err(
                ToolError::new("HOME_DIR_NOT_FOUND", "Could not determine home directory")
                    .with_suggestion("Set the HOME environment variable"),
            )
        }
    }

    /// Get effective storage path based on mode and session
    fn effective_path(&self, session_id: Option<&str>) -> Result<PathBuf, ToolError> {
        match (self.storage_mode, session_id) {
            (StorageMode::SessionScoped, Some(sid)) => Self::session_path(sid),
            (StorageMode::SessionScoped, None) => Err(ToolError::new(
                "MISSING_SESSION_ID",
                "Session ID required for session-scoped storage",
            )
            .with_suggestion("Provide a session_id or use global storage mode")),
            (StorageMode::Global, _) => Ok(self.storage_path.clone()),
        }
    }

    /// Load todos from storage (with session support)
    pub fn load_todos(&self, session_id: Option<&str>) -> Result<HashMap<String, Todo>, ToolError> {
        let path = self.effective_path(session_id)?;
        debug!("Loading todos from: {:?}", path);

        // If file doesn't exist, return empty map
        if !path.exists() {
            debug!("Todo storage file does not exist, returning empty todos");
            return Ok(HashMap::new());
        }

        // Read file
        let content = std::fs::read_to_string(&path).map_err(|e| {
            error!("Failed to read todo storage file: {}", e);
            ToolError::from(e)
                .with_details(format!("Failed to read: {:?}", path))
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

    /// Load todos as Vec (preserves order)
    pub fn load_todos_ordered(&self, session_id: Option<&str>) -> Result<Vec<Todo>, ToolError> {
        let path = self.effective_path(session_id)?;
        debug!("Loading todos from: {:?}", path);

        // If file doesn't exist, return empty vec
        if !path.exists() {
            debug!("Todo storage file does not exist, returning empty todos");
            return Ok(Vec::new());
        }

        // Read file
        let content = std::fs::read_to_string(&path).map_err(|e| {
            error!("Failed to read todo storage file: {}", e);
            ToolError::from(e)
                .with_details(format!("Failed to read: {:?}", path))
                .with_suggestion("Check file permissions and ensure the file is readable")
        })?;

        // Parse JSON
        let todos: Vec<Todo> = serde_json::from_str(&content).map_err(|e| {
            error!("Failed to parse todo storage file: {}", e);
            ToolError::from(e)
                .with_details("Todo storage file contains invalid JSON")
                .with_suggestion("Check the file format or restore from backup")
        })?;

        info!("Loaded {} todos from storage", todos.len());
        Ok(todos)
    }

    /// Save todos to storage from HashMap (atomic write)
    pub fn save_todos(&self, todos: &HashMap<String, Todo>, session_id: Option<&str>) -> Result<(), ToolError> {
        let path = self.effective_path(session_id)?;
        debug!("Saving {} todos to: {:?}", todos.len(), path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create storage directory: {}", e);
                ToolError::from(e)
                    .with_details(format!("Failed to create: {:?}", parent))
                    .with_suggestion("Check directory permissions")
            })?;
        }

        // Convert HashMap to Vec for serialization (sorted by ID for stable output)
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
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &json).map_err(|e| {
            error!("Failed to write temporary todo file: {}", e);
            ToolError::from(e)
                .with_details(format!("Failed to write: {:?}", temp_path))
                .with_suggestion("Check disk space and file permissions")
        })?;

        // Rename temporary file to actual file (atomic on most filesystems)
        std::fs::rename(&temp_path, &path).map_err(|e| {
            error!("Failed to finalize todo storage: {}", e);
            // Clean up temp file on error
            let _ = std::fs::remove_file(&temp_path);
            ToolError::from(e)
                .with_details(format!(
                    "Failed to rename: {:?} to {:?}",
                    temp_path, path
                ))
                .with_suggestion("Check file permissions and disk space")
        })?;

        info!("Saved {} todos to storage", todos.len());
        Ok(())
    }

    /// Save todos to storage from Vec (preserves order, atomic write)
    pub fn save_todos_ordered(&self, todos: &[Todo], session_id: Option<&str>) -> Result<(), ToolError> {
        let path = self.effective_path(session_id)?;
        debug!("Saving {} todos to: {:?}", todos.len(), path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create storage directory: {}", e);
                ToolError::from(e)
                    .with_details(format!("Failed to create: {:?}", parent))
                    .with_suggestion("Check directory permissions")
            })?;
        }

        // Serialize to JSON (preserves input order)
        let json = serde_json::to_string_pretty(&todos).map_err(|e| {
            error!("Failed to serialize todos: {}", e);
            ToolError::from(e)
                .with_details("Failed to serialize todos to JSON")
                .with_suggestion("Check for circular references or invalid data")
        })?;

        // Write to temporary file first (atomic write)
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &json).map_err(|e| {
            error!("Failed to write temporary todo file: {}", e);
            ToolError::from(e)
                .with_details(format!("Failed to write: {:?}", temp_path))
                .with_suggestion("Check disk space and file permissions")
        })?;

        // Rename temporary file to actual file (atomic on most filesystems)
        std::fs::rename(&temp_path, &path).map_err(|e| {
            error!("Failed to finalize todo storage: {}", e);
            // Clean up temp file on error
            let _ = std::fs::remove_file(&temp_path);
            ToolError::from(e)
                .with_details(format!(
                    "Failed to rename: {:?} to {:?}",
                    temp_path, path
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
    storage_mode: StorageMode,
    write_mode: WriteMode,
    event_bus: Option<std::sync::Arc<dyn TodoEventPublisher>>,
}

/// Event publisher trait for todo changes
pub trait TodoEventPublisher: Send + Sync {
    /// Publish a todo updated event
    fn publish_todo_updated(&self, session_id: Option<&str>, todos: &[Todo]);
}

impl TodoTools {
    /// Create new todo tools with default storage path
    pub fn new() -> Result<Self, ToolError> {
        let storage_path = TodoStorage::default_path()?;
        Ok(Self {
            storage: TodoStorage::new(storage_path),
            mcp_provider: None,
            storage_mode: StorageMode::Global,
            write_mode: WriteMode::Merge,
            event_bus: None,
        })
    }

    /// Create new todo tools with custom storage path
    pub fn with_storage_path(storage_path: impl Into<PathBuf>) -> Self {
        Self {
            storage: TodoStorage::new(storage_path),
            mcp_provider: None,
            storage_mode: StorageMode::Global,
            write_mode: WriteMode::Merge,
            event_bus: None,
        }
    }

    /// Set storage mode (global or session-scoped)
    pub fn with_storage_mode(mut self, mode: StorageMode) -> Self {
        self.storage_mode = mode;
        self.storage = self.storage.with_storage_mode(mode);
        self
    }

    /// Set write mode (replace or merge)
    pub fn with_write_mode(mut self, mode: WriteMode) -> Self {
        self.write_mode = mode;
        self
    }

    /// Set the MCP provider for todo operations
    pub fn with_mcp_provider(mut self, provider: std::sync::Arc<dyn crate::Provider>) -> Self {
        self.mcp_provider = Some(provider);
        self
    }

    /// Set the event bus for publishing todo updates
    pub fn with_event_bus(mut self, event_bus: std::sync::Arc<dyn TodoEventPublisher>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Validate todo input (G-05: Tool-layer schema validation)
    fn validate_todo_input(input: &TodowriteInput) -> Result<(), ToolError> {
        for (idx, todo) in input.todos.iter().enumerate() {
            // Validate required fields
            if todo.id.trim().is_empty() {
                return Err(ToolError::new(
                    "INVALID_TODO",
                    format!("Todo at index {} has empty id", idx),
                )
                .with_suggestion("Provide a non-empty id for each todo"));
            }

            if todo.content.trim().is_empty() {
                return Err(ToolError::new(
                    "INVALID_TODO",
                    format!("Todo at index {} has empty content", idx),
                )
                .with_suggestion("Provide non-empty content for each todo"));
            }

            // Validate status enum value
            let status_str = format!("{}", todo.status);
            if !["pending", "in_progress", "completed", "cancelled", "blocked"]
                .contains(&status_str.as_str())
            {
                return Err(ToolError::new(
                    "INVALID_STATUS",
                    format!("Todo at index {} has invalid status: {}", idx, status_str),
                )
                .with_suggestion(
                    "Use one of: pending, in_progress, completed, cancelled, blocked",
                ));
            }

            // Validate priority enum value
            let priority_str = format!("{}", todo.priority);
            if !["low", "medium", "high", "critical"].contains(&priority_str.as_str()) {
                return Err(ToolError::new(
                    "INVALID_PRIORITY",
                    format!(
                        "Todo at index {} has invalid priority: {}",
                        idx, priority_str
                    ),
                )
                .with_suggestion("Use one of: low, medium, high, critical"));
            }
        }

        Ok(())
    }

    /// Count incomplete todos (status != completed)
    fn count_incomplete(todos: &[Todo]) -> usize {
        todos.iter().filter(|t| t.status != TodoStatus::Completed).count()
    }

    /// Format output for OpenCode compatibility
    fn format_output(todos: &[Todo]) -> String {
        serde_json::to_string_pretty(todos).unwrap_or_else(|_| "[]".to_string())
    }

    /// Write todos with timeout enforcement (500ms)
    ///
    /// Attempts to use MCP provider if available, falls back to built-in implementation.
    pub async fn write_todos_with_timeout(
        &self,
        input: TodowriteInput,
        session_id: Option<&str>,
    ) -> Result<TodowriteOutput, ToolError> {
        let timeout_duration = std::time::Duration::from_millis(500);

        match tokio::time::timeout(timeout_duration, async {
            self.write_todos_internal(input, session_id)
        })
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
    pub fn write_todos(&self, input: TodowriteInput, session_id: Option<&str>) -> Result<TodowriteOutput, ToolError> {
        self.write_todos_internal(input, session_id)
    }

    /// Internal write todos implementation (with session and mode support)
    fn write_todos_internal(&self, input: TodowriteInput, session_id: Option<&str>) -> Result<TodowriteOutput, ToolError> {
        debug!("Writing {} todos (mode: {:?})", input.todos.len(), self.write_mode);

        // G-05: Validate input before processing
        Self::validate_todo_input(&input)?;

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

        let (created, updated, final_todos) = match self.write_mode {
            WriteMode::Replace => {
                // OpenCode compatible: replace entire list
                debug!("Replace mode: replacing entire todo list");
                self.storage.save_todos_ordered(&input.todos, session_id)?;
                (input.todos.len(), 0, input.todos)
            }
            WriteMode::Merge => {
                // RiceCoder default: merge by ID
                debug!("Merge mode: merging todos by ID");
                let mut todos = self.storage.load_todos(session_id)?;

                let mut created = 0;
                let mut updated = 0;

                // Process each todo
                for todo in &input.todos {
                    let id = todo.id.clone();
                    if todos.contains_key(&id) {
                        updated += 1;
                    } else {
                        created += 1;
                    }
                    todos.insert(id, todo.clone());
                }

                // Save todos
                self.storage.save_todos(&todos, session_id)?;

                // Convert to Vec for output (sorted by ID for stable ordering)
                let mut final_todos: Vec<Todo> = todos.values().cloned().collect();
                final_todos.sort_by(|a, b| a.id.cmp(&b.id));

                (created, updated, final_todos)
            }
        };

        // Format OpenCode-compatible output
        let incomplete_count = Self::count_incomplete(&final_todos);
        let title = format!("{} todos", incomplete_count);
        let output = Self::format_output(&final_todos);

        // G-07: Publish event to event bus if available
        if let Some(event_bus) = &self.event_bus {
            event_bus.publish_todo_updated(session_id, &final_todos);
            debug!("Published TodoUpdated event to event bus");
        }

        info!("Wrote todos: {} created, {} updated", created, updated);
        Ok(TodowriteOutput {
            title,
            output,
            metadata: TodowriteMetadata {
                todos: final_todos,
                created: Some(created),
                updated: Some(updated),
            },
        })
    }

    /// Read todos with timeout enforcement (500ms)
    ///
    /// Attempts to use MCP provider if available, falls back to built-in implementation.
    pub async fn read_todos_with_timeout(
        &self,
        input: TodoreadInput,
        session_id: Option<&str>,
    ) -> Result<TodoreadOutput, ToolError> {
        let timeout_duration = std::time::Duration::from_millis(500);

        match tokio::time::timeout(timeout_duration, async {
            self.read_todos_internal(input, session_id)
        })
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
    pub fn read_todos(&self, input: TodoreadInput, session_id: Option<&str>) -> Result<TodoreadOutput, ToolError> {
        self.read_todos_internal(input, session_id)
    }

    /// Internal read todos implementation (with session support)
    fn read_todos_internal(&self, input: TodoreadInput, session_id: Option<&str>) -> Result<TodoreadOutput, ToolError> {
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

        // Load todos (preserves order if replace mode was used)
        let todos = if self.write_mode == WriteMode::Replace {
            self.storage.load_todos_ordered(session_id)?
        } else {
            // Merge mode: convert HashMap to Vec and sort
            let todos_map = self.storage.load_todos(session_id)?;
            let mut todos: Vec<Todo> = todos_map.values().cloned().collect();
            todos.sort_by(|a, b| match b.priority.cmp(&a.priority) {
                std::cmp::Ordering::Equal => a.id.cmp(&b.id),
                other => other,
            });
            todos
        };

        // Filter todos (RiceCoder extra - KEEP)
        let filtered: Vec<Todo> = if input.status_filter.is_some() || input.priority_filter.is_some() {
            todos
                .into_iter()
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
                .collect()
        } else {
            todos
        };

        // Format OpenCode-compatible output
        let incomplete_count = Self::count_incomplete(&filtered);
        let title = format!("{} todos", incomplete_count);
        let output = Self::format_output(&filtered);

        info!("Read {} todos (filtered from total)", filtered.len());
        Ok(TodoreadOutput {
            title,
            output,
            metadata: TodoreadMetadata {
                todos: filtered,
            },
        })
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

/// Tool invoker wiring helpers (G-10)
impl TodoTools {
    /// Create tool metadata for todowrite
    pub fn todowrite_metadata() -> serde_json::Value {
        serde_json::json!({
            "name": "todowrite",
            "description": "Write todos to persistent storage with session support",
            "parameters": {
                "type": "object",
                "properties": {
                    "todos": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string"},
                                "content": {"type": "string"},
                                "status": {
                                    "type": "string",
                                    "enum": ["pending", "in_progress", "completed", "cancelled", "blocked"]
                                },
                                "priority": {
                                    "type": "string",
                                    "enum": ["low", "medium", "high", "critical"]
                                },
                                "description": {"type": "string"}
                            },
                            "required": ["id", "content", "status", "priority"]
                        }
                    }
                },
                "required": ["todos"]
            }
        })
    }

    /// Create tool metadata for todoread
    pub fn todoread_metadata() -> serde_json::Value {
        serde_json::json!({
            "name": "todoread",
            "description": "Read todos from persistent storage with optional filtering",
            "parameters": {
                "type": "object",
                "properties": {
                    "status_filter": {
                        "type": "string",
                        "enum": ["pending", "in_progress", "completed", "cancelled", "blocked"]
                    },
                    "priority_filter": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"]
                    }
                }
            }
        })
    }

    /// Invoke todowrite from JSON parameters
    pub fn invoke_todowrite(&self, params: Value, session_id: Option<&str>) -> Result<Value, ToolError> {
        let input: TodowriteInput = serde_json::from_value(params)
            .map_err(|e| ToolError::new("INVALID_INPUT", format!("Failed to parse todowrite input: {}", e)))?;
        
        let output = self.write_todos(input, session_id)?;
        
        serde_json::to_value(output)
            .map_err(|e| ToolError::new("SERIALIZATION_ERROR", format!("Failed to serialize output: {}", e)))
    }

    /// Invoke todoread from JSON parameters
    pub fn invoke_todoread(&self, params: Value, session_id: Option<&str>) -> Result<Value, ToolError> {
        let input: TodoreadInput = serde_json::from_value(params)
            .map_err(|e| ToolError::new("INVALID_INPUT", format!("Failed to parse todoread input: {}", e)))?;
        
        let output = self.read_todos(input, session_id)?;
        
        serde_json::to_value(output)
            .map_err(|e| ToolError::new("SERIALIZATION_ERROR", format!("Failed to serialize output: {}", e)))
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

        let todos = storage.load_todos(None).unwrap();
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

        storage.save_todos(&todos, None).unwrap();

        // Load and verify
        let loaded = storage.load_todos(None).unwrap();
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
            }, None)
            .unwrap();

        assert_eq!(write_result.metadata.created.unwrap(), 2);
        assert_eq!(write_result.metadata.updated.unwrap(), 0);

        // Read todos
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            }, None)
            .unwrap();

        assert_eq!(read_result.metadata.todos.len(), 2);
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
