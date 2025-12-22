//! Hook execution engine

pub mod condition;
pub mod runner;
pub mod substitution;

pub use condition::ConditionEvaluator;
pub use runner::DefaultHookExecutor;
pub use substitution::VariableSubstitutor;

use crate::{
    error::Result,
    types::{EventContext, Hook, HookResult},
};

/// Trait for executing hooks
///
/// The HookExecutor is responsible for:
/// 1. Receiving hooks and event context
/// 2. Evaluating hook conditions
/// 3. Executing hook actions (commands, tool calls, AI prompts, chains)
/// 4. Handling timeouts and errors
/// 5. Capturing output and results
/// 6. Providing hook isolation (failures don't affect other hooks)
///
/// # Examples
///
/// ```ignore
/// let executor = DefaultHookExecutor::new();
/// let hook = Hook { ... };
/// let context = EventContext { ... };
/// let result = executor.execute_hook(&hook, &context)?;
/// ```
pub trait HookExecutor: Send + Sync {
    /// Execute a hook with the given context
    ///
    /// This method:
    /// 1. Evaluates the hook condition (if present)
    /// 2. Executes the hook action
    /// 3. Captures output and errors
    /// 4. Returns a HookResult with status and output
    fn execute_hook(&self, hook: &Hook, context: &EventContext) -> Result<HookResult>;

    /// Execute a hook action
    ///
    /// Routes to the appropriate executor based on action type:
    /// - CommandAction: Execute shell command
    /// - ToolCallAction: Call tool with parameters
    /// - AiPromptAction: Send prompt to AI assistant
    /// - ChainAction: Execute hooks in sequence
    fn execute_action(&self, hook: &Hook, context: &EventContext) -> Result<String>;
}
