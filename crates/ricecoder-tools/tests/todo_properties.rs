//! Property-based tests for todo tools
//!
//! **Feature: ricecoder-tools-enhanced, Property 7: Todo persistence**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

use std::collections::HashSet;

use proptest::prelude::*;
use ricecoder_tools::{Todo, TodoPriority, TodoStatus, TodoTools, TodoreadInput, TodowriteInput};
use tempfile::TempDir;

/// Strategy for generating valid todo IDs
fn todo_id_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}"
}

/// Strategy for generating valid todo content (non-empty, not all whitespace)
fn todo_content_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}"
}

/// Strategy for generating todo statuses
fn todo_status_strategy() -> impl Strategy<Value = TodoStatus> {
    prop_oneof![
        Just(TodoStatus::Pending),
        Just(TodoStatus::InProgress),
        Just(TodoStatus::Completed),
        Just(TodoStatus::Cancelled),
        Just(TodoStatus::Blocked),
    ]
}

/// Strategy for generating todo priorities
fn todo_priority_strategy() -> impl Strategy<Value = TodoPriority> {
    prop_oneof![
        Just(TodoPriority::Low),
        Just(TodoPriority::Medium),
        Just(TodoPriority::High),
        Just(TodoPriority::Critical),
    ]
}

/// Strategy for generating valid todos
fn todo_strategy() -> impl Strategy<Value = Todo> {
    (
        todo_id_strategy(),
        todo_content_strategy(),
        todo_status_strategy(),
        todo_priority_strategy(),
    )
        .prop_flat_map(|(id, content, status, priority)| {
            Just(
                Todo::new(id, content, status, priority)
                    .expect("Generated valid todo")
                    .with_description(format!("Description for {}", status)),
            )
        })
}

/// Strategy for generating lists of todos with unique IDs
fn todos_strategy() -> impl Strategy<Value = Vec<Todo>> {
    (1usize..10)
        .prop_flat_map(|count| {
            prop::collection::vec(
                (
                    todo_content_strategy(),
                    todo_status_strategy(),
                    todo_priority_strategy(),
                ),
                count..=count,
            )
        })
        .prop_map(|items| {
            items
                .into_iter()
                .enumerate()
                .map(|(idx, (content, status, priority))| {
                    Todo::new(format!("todo-{}", idx), content, status, priority)
                        .expect("Generated valid todo")
                        .with_description(format!("Description for {}", status))
                })
                .collect()
        })
}

proptest! {
    /// Property 7: Todo persistence
    ///
    /// *For any* todo created via todowrite, subsequent todoread operations SHALL return
    /// the same todo with identical data.
    ///
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
    #[test]
    fn prop_todo_persistence(todos in todos_strategy()) {
        // Create temporary storage
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write todos
        let write_result = tools
            .write_todos(TodowriteInput { todos: todos.clone() })
            .expect("Failed to write todos");

        // Verify write result
        prop_assert_eq!(write_result.created, todos.len());
        prop_assert_eq!(write_result.updated, 0);

        // Read todos back
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            })
            .expect("Failed to read todos");

        // Verify all todos were persisted
        prop_assert_eq!(read_result.todos.len(), todos.len());

        // Verify each todo has identical data
        let written_ids: HashSet<String> = todos.iter().map(|t| t.id.clone()).collect();
        let read_ids: HashSet<String> = read_result.todos.iter().map(|t| t.id.clone()).collect();
        prop_assert_eq!(written_ids, read_ids);

        // Verify each todo's data is identical
        for written_todo in &todos {
            let read_todo = read_result
                .todos
                .iter()
                .find(|t| t.id == written_todo.id)
                .expect("Todo not found in read results");

            prop_assert_eq!(&read_todo.id, &written_todo.id);
            prop_assert_eq!(&read_todo.content, &written_todo.content);
            prop_assert_eq!(&read_todo.description, &written_todo.description);
            prop_assert_eq!(read_todo.status, written_todo.status);
            prop_assert_eq!(read_todo.priority, written_todo.priority);
        }
    }

    /// Property 7 variant: Todo persistence across multiple write operations
    ///
    /// *For any* sequence of todos written in multiple operations, subsequent todoread
    /// operations SHALL return all todos with identical data.
    ///
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
    #[test]
    fn prop_todo_persistence_multiple_writes(
        todos_batch1 in todos_strategy(),
        todos_batch2 in todos_strategy(),
    ) {
        // Create temporary storage
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Rename batch 2 todos to have different IDs to avoid overwrites
        let todos_batch2_renamed: Vec<Todo> = todos_batch2
            .iter()
            .enumerate()
            .map(|(idx, todo)| {
                let mut renamed = Todo::new(
                    format!("batch2-{}", idx),
                    todo.content.clone(),
                    todo.status,
                    todo.priority,
                )
                .expect("Failed to create renamed todo");
                if let Some(desc) = &todo.description {
                    renamed = renamed.with_description(desc.clone());
                }
                renamed
            })
            .collect();

        // Write first batch
        let write_result1 = tools
            .write_todos(TodowriteInput {
                todos: todos_batch1.clone(),
            })
            .expect("Failed to write first batch");

        prop_assert_eq!(write_result1.created, todos_batch1.len());

        // Write second batch
        let write_result2 = tools
            .write_todos(TodowriteInput {
                todos: todos_batch2_renamed.clone(),
            })
            .expect("Failed to write second batch");

        prop_assert_eq!(write_result2.created, todos_batch2_renamed.len());

        // Read all todos
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            })
            .expect("Failed to read todos");

        // Verify all todos from both batches are present
        let total_expected = todos_batch1.len() + todos_batch2_renamed.len();
        prop_assert_eq!(read_result.todos.len(), total_expected);

        // Verify all todos from batch 1
        for written_todo in &todos_batch1 {
            let read_todo = read_result
                .todos
                .iter()
                .find(|t| t.id == written_todo.id)
                .expect("Todo from batch 1 not found");

            prop_assert_eq!(&read_todo.id, &written_todo.id);
            prop_assert_eq!(&read_todo.content, &written_todo.content);
            prop_assert_eq!(read_todo.status, written_todo.status);
            prop_assert_eq!(read_todo.priority, written_todo.priority);
        }

        // Verify all todos from batch 2
        for written_todo in &todos_batch2_renamed {
            let read_todo = read_result
                .todos
                .iter()
                .find(|t| t.id == written_todo.id)
                .expect("Todo from batch 2 not found");

            prop_assert_eq!(&read_todo.id, &written_todo.id);
            prop_assert_eq!(&read_todo.content, &written_todo.content);
            prop_assert_eq!(read_todo.status, written_todo.status);
            prop_assert_eq!(read_todo.priority, written_todo.priority);
        }
    }

    /// Property 7 variant: Todo persistence with updates
    ///
    /// *For any* todo that is updated via todowrite, subsequent todoread operations
    /// SHALL return the updated todo with new data.
    ///
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
    #[test]
    fn prop_todo_persistence_with_updates(
        original_todo in todo_strategy(),
        new_status in todo_status_strategy(),
        new_priority in todo_priority_strategy(),
    ) {
        // Create temporary storage
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write original todo
        tools
            .write_todos(TodowriteInput {
                todos: vec![original_todo.clone()],
            })
            .expect("Failed to write original todo");

        // Create updated todo with same ID but different status/priority
        let updated_todo = Todo::new(
            original_todo.id.clone(),
            original_todo.content.clone(),
            new_status,
            new_priority,
        )
        .expect("Failed to create updated todo")
        .with_description("Updated description");

        // Write updated todo
        let write_result = tools
            .write_todos(TodowriteInput {
                todos: vec![updated_todo.clone()],
            })
            .expect("Failed to write updated todo");

        // Verify it was an update, not a create
        prop_assert_eq!(write_result.created, 0);
        prop_assert_eq!(write_result.updated, 1);

        // Read todos
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: None,
                priority_filter: None,
            })
            .expect("Failed to read todos");

        // Verify only one todo exists
        prop_assert_eq!(read_result.todos.len(), 1);

        // Verify the todo has the updated data
        let read_todo = &read_result.todos[0];
        prop_assert_eq!(&read_todo.id, &updated_todo.id);
        prop_assert_eq!(&read_todo.content, &updated_todo.content);
        prop_assert_eq!(read_todo.status, new_status);
        prop_assert_eq!(read_todo.priority, new_priority);
        prop_assert_eq!(&read_todo.description, &updated_todo.description);
    }

    /// Property 7 variant: Todo persistence with filtering
    ///
    /// *For any* todos written and then read with filters, the returned todos
    /// SHALL match the filter criteria and have identical data to what was written.
    ///
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
    #[test]
    fn prop_todo_persistence_with_filtering(
        todos in todos_strategy(),
        filter_status in prop_oneof![
            Just(None),
            Just(Some(TodoStatus::Pending)),
            Just(Some(TodoStatus::InProgress)),
            Just(Some(TodoStatus::Completed)),
            Just(Some(TodoStatus::Cancelled)),
            Just(Some(TodoStatus::Blocked)),
        ],
        filter_priority in prop_oneof![
            Just(None),
            Just(Some(TodoPriority::Low)),
            Just(Some(TodoPriority::Medium)),
            Just(Some(TodoPriority::High)),
            Just(Some(TodoPriority::Critical)),
        ],
    ) {
        // Create temporary storage
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("todos.json");
        let tools = TodoTools::with_storage_path(&storage_path);

        // Write todos
        tools
            .write_todos(TodowriteInput { todos: todos.clone() })
            .expect("Failed to write todos");

        // Read with filters
        let read_result = tools
            .read_todos(TodoreadInput {
                status_filter: filter_status,
                priority_filter: filter_priority,
            })
            .expect("Failed to read todos");

        // Verify all returned todos match the filters
        for read_todo in &read_result.todos {
            if let Some(status) = filter_status {
                prop_assert_eq!(read_todo.status, status);
            }
            if let Some(priority) = filter_priority {
                prop_assert_eq!(read_todo.priority, priority);
            }

            // Verify the todo data matches what was written
            let written_todo = todos
                .iter()
                .find(|t| t.id == read_todo.id)
                .expect("Read todo not found in written todos");

            prop_assert_eq!(&read_todo.id, &written_todo.id);
            prop_assert_eq!(&read_todo.content, &written_todo.content);
            prop_assert_eq!(&read_todo.description, &written_todo.description);
            prop_assert_eq!(read_todo.status, written_todo.status);
            prop_assert_eq!(read_todo.priority, written_todo.priority);
        }
    }
}
