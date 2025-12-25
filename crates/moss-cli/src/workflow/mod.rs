//! TOML-based workflow engine.
//!
//! Workflows orchestrate moss primitives (view, edit, analyze) through:
//! - Step-based execution (linear sequence)
//! - State machine execution (conditional transitions)
//!
//! LLM is an optional plugin, not required for workflow execution.

mod config;
mod execute;
mod strategies;

pub use config::{load_workflow, WorkflowConfig};
pub use execute::run_workflow;
