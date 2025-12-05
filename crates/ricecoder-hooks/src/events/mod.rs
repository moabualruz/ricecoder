//! Event types for the hooks system
//!
//! This module defines event types for file operations, directory operations, and system events.
//! Events are emitted by the system when something happens and can trigger registered hooks.

pub mod file_operations;
pub mod monitor;
pub mod system;

pub use file_operations::{DirectoryOperationEvent, FileOperationEvent};
pub use monitor::FileSystemMonitor;
pub use system::{
    BuildFailedEvent, BuildSuccessEvent, CustomEvent, DeploymentCompleteEvent, FileSavedEvent,
    GenerationCompleteEvent, RefactoringCompleteEvent, ReviewCompleteEvent, SystemEvent,
    TestFailedEvent, TestPassedEvent,
};
