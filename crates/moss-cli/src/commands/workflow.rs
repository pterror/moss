//! Workflow command - run TOML-defined workflows.

use std::path::Path;

use clap::Subcommand;

use crate::workflow;

#[derive(Subcommand)]
pub enum WorkflowAction {
    /// List available workflows
    List,

    /// Run a workflow
    Run {
        /// Workflow name or path to .toml file
        workflow: String,

        /// Task description (passed to workflow as context)
        #[arg(short, long)]
        task: Option<String>,
    },

    /// Show workflow definition and metadata
    Show {
        /// Workflow name or path to .toml file
        workflow: String,
    },

    /// Create a new workflow from template
    New {
        /// Workflow name (e.g., validate-fix)
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "agentic")]
        template: String,

        /// Overwrite existing workflow
        #[arg(short, long)]
        force: bool,
    },
}

/// Dispatch workflow subcommands
pub fn cmd_workflow(action: WorkflowAction, root: Option<&Path>, json: bool) -> i32 {
    match action {
        WorkflowAction::List => cmd_workflow_list(root, json),
        WorkflowAction::Run { workflow, task } => {
            cmd_workflow_run(&workflow, task.as_deref(), root, json)
        }
        WorkflowAction::Show { workflow } => cmd_workflow_show(&workflow, root, json),
        WorkflowAction::New {
            name,
            template,
            force,
        } => cmd_workflow_new(&name, &template, force, root, json),
    }
}

/// List available workflows in .moss/workflows/
pub fn cmd_workflow_list(root: Option<&Path>, json: bool) -> i32 {
    let root = root.unwrap_or_else(|| Path::new("."));
    let workflows_dir = root.join(".moss").join("workflows");

    if !workflows_dir.exists() {
        if json {
            println!("[]");
        } else {
            println!("No workflows directory found at .moss/workflows/");
            println!("Create TOML workflow files there to get started.");
        }
        return 0;
    }

    let mut workflows = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    // Try to load and get description
                    let description = workflow::load_workflow(&path)
                        .ok()
                        .map(|c| c.workflow.description.clone())
                        .unwrap_or_default();

                    workflows.push((name.to_string(), description));
                }
            }
        }
    }

    if json {
        let items: Vec<_> = workflows
            .iter()
            .map(|(name, desc)| {
                serde_json::json!({
                    "name": name,
                    "description": desc
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        if workflows.is_empty() {
            println!("No workflows found in .moss/workflows/");
        } else {
            println!("Available workflows:");
            for (name, desc) in workflows {
                if desc.is_empty() {
                    println!("  {}", name);
                } else {
                    println!("  {} - {}", name, desc);
                }
            }
        }
    }

    0
}

/// Run a workflow by name or path.
pub fn cmd_workflow_run(
    workflow: &str,
    task: Option<&str>,
    root: Option<&Path>,
    json: bool,
) -> i32 {
    let root = root.unwrap_or_else(|| Path::new("."));

    // Resolve workflow path
    let workflow_path = if workflow.ends_with(".toml") {
        // Explicit path
        root.join(workflow)
    } else {
        // Look in .moss/workflows/
        root.join(".moss")
            .join("workflows")
            .join(format!("{}.toml", workflow))
    };

    if !workflow_path.exists() {
        eprintln!("Workflow not found: {}", workflow_path.display());
        eprintln!("Create it at .moss/workflows/{}.toml", workflow);
        return 1;
    }

    let task_str = task.unwrap_or("");

    match workflow::run_workflow(&workflow_path, task_str, root) {
        Ok(result) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": result.success,
                        "output": result.output,
                        "steps_executed": result.steps_executed
                    })
                );
            } else {
                if !result.output.is_empty() {
                    println!("{}", result.output);
                }
                if result.success {
                    println!("\nWorkflow completed ({} steps)", result.steps_executed);
                } else {
                    eprintln!("\nWorkflow failed after {} steps", result.steps_executed);
                }
            }
            if result.success {
                0
            } else {
                1
            }
        }
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "error": e
                    })
                );
            } else {
                eprintln!("Error: {}", e);
            }
            1
        }
    }
}

/// Show workflow definition and metadata.
pub fn cmd_workflow_show(workflow: &str, root: Option<&Path>, json: bool) -> i32 {
    let root = root.unwrap_or_else(|| Path::new("."));

    let workflow_path = if workflow.ends_with(".toml") {
        root.join(workflow)
    } else {
        root.join(".moss")
            .join("workflows")
            .join(format!("{}.toml", workflow))
    };

    if !workflow_path.exists() {
        eprintln!("Workflow not found: {}", workflow_path.display());
        return 1;
    }

    match workflow::load_workflow(&workflow_path) {
        Ok(config) => {
            if json {
                // Serialize the config
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            } else {
                println!("Workflow: {}", config.workflow.name);
                if !config.workflow.description.is_empty() {
                    println!("Description: {}", config.workflow.description);
                }
                println!();

                if config.is_step_based() {
                    println!("Type: Step-based ({} steps)", config.steps.len());
                    println!();
                    for (i, step) in config.steps.iter().enumerate() {
                        println!("  {}. {} - {}", i + 1, step.name, step.action);
                        if let Some(ref cond) = step.condition {
                            println!("     condition: {}", cond);
                        }
                    }
                } else if config.is_state_machine() {
                    println!("Type: State machine ({} states)", config.states.len());
                    if let Some(ref initial) = config.workflow.initial_state {
                        println!("Initial state: {}", initial);
                    }
                    println!();
                    for state in &config.states {
                        let terminal = if state.terminal { " [terminal]" } else { "" };
                        println!("  State: {}{}", state.name, terminal);
                        if let Some(ref action) = state.action {
                            println!("    action: {}", action);
                        }
                        for trans in &state.transitions {
                            let cond = trans.condition.as_deref().unwrap_or("always");
                            let next = trans.next.as_deref().unwrap_or("(end)");
                            println!("    {} -> {}", cond, next);
                        }
                    }
                }
            }
            0
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"error": e}));
            } else {
                eprintln!("Error loading workflow: {}", e);
            }
            1
        }
    }
}

/// Workflow templates.
mod templates {
    pub const AGENTIC: &str = r#"# {name} workflow - agentic loop with LLM decision-making
[workflow]
name = "{name}"
description = "Agentic workflow for {name}"
version = "1.0"

[workflow.limits]
max_turns = 20
timeout_seconds = 300

[workflow.context]
strategy = "flat"

[workflow.cache]
strategy = "in_memory"
preview_length = 500

[workflow.retry]
strategy = "exponential"
max_attempts = 3
base_delay = 1.0
max_delay = 60.0

[workflow.llm]
strategy = "simple"
model = "claude-sonnet-4-20250514"
system_prompt = """Commands:
- view [path] - show directory tree, file skeleton, or symbol
- edit <file> "task" - apply an edit to a file
- analyze [--health|--complexity] - analyze codebase
- done - signal completion

Think between commands. One command per line.
"""
"#;

    pub const STEP_BASED: &str = r#"# {name} workflow - sequential steps
[workflow]
name = "{name}"
description = "Step-based workflow for {name}"
version = "1.0"

[[steps]]
name = "analyze"
action = "analyze --health"

[[steps]]
name = "view"
action = "view ."
"#;

    pub const STATE_MACHINE: &str = r#"# {name} workflow - state machine
[workflow]
name = "{name}"
description = "State machine workflow for {name}"
version = "1.0"
initial_state = "start"

[workflow.limits]
max_turns = 10

[[states]]
name = "start"
action = "analyze --health"

[[states.transitions]]
condition = "has_errors"
next = "fix"

[[states.transitions]]
next = "done"

[[states]]
name = "fix"
action = "edit . 'Fix the errors'"

[[states.transitions]]
next = "verify"

[[states]]
name = "verify"
action = "analyze --health"

[[states.transitions]]
condition = "has_errors"
next = "fix"

[[states.transitions]]
next = "done"

[[states]]
name = "done"
terminal = true
"#;

    pub fn get(name: &str) -> Option<&'static str> {
        match name {
            "agentic" => Some(AGENTIC),
            "step" | "step-based" | "steps" => Some(STEP_BASED),
            "state" | "state-machine" | "fsm" => Some(STATE_MACHINE),
            _ => None,
        }
    }

    pub fn list() -> &'static [&'static str] {
        &["agentic", "step-based", "state-machine"]
    }
}

/// Create a new workflow from template.
pub fn cmd_workflow_new(
    name: &str,
    template: &str,
    force: bool,
    root: Option<&Path>,
    json: bool,
) -> i32 {
    let root = root.unwrap_or_else(|| Path::new("."));
    let workflows_dir = root.join(".moss").join("workflows");
    let workflow_path = workflows_dir.join(format!("{}.toml", name));

    // Check if already exists
    if workflow_path.exists() && !force {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "error": format!("Workflow '{}' already exists", name),
                    "path": workflow_path.display().to_string()
                })
            );
        } else {
            eprintln!(
                "Workflow '{}' already exists at {}",
                name,
                workflow_path.display()
            );
            eprintln!("Use --force to overwrite");
        }
        return 1;
    }

    // Get template
    let template_content = match templates::get(template) {
        Some(t) => t,
        None => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "error": format!("Unknown template: {}", template),
                        "available": templates::list()
                    })
                );
            } else {
                eprintln!("Unknown template: {}", template);
                eprintln!("Available: {}", templates::list().join(", "));
            }
            return 1;
        }
    };

    // Create directory if needed
    if let Err(e) = std::fs::create_dir_all(&workflows_dir) {
        if json {
            println!("{}", serde_json::json!({"error": e.to_string()}));
        } else {
            eprintln!("Failed to create workflows directory: {}", e);
        }
        return 1;
    }

    // Write workflow file
    let content = template_content.replace("{name}", name);
    if let Err(e) = std::fs::write(&workflow_path, content) {
        if json {
            println!("{}", serde_json::json!({"error": e.to_string()}));
        } else {
            eprintln!("Failed to write workflow file: {}", e);
        }
        return 1;
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "path": workflow_path.display().to_string(),
                "name": name,
                "template": template
            })
        );
    } else {
        println!("Created workflow '{}' at {}", name, workflow_path.display());
        println!();
        println!("Next steps:");
        println!("  1. Edit {}", workflow_path.display());
        println!("  2. Run with: moss workflow run {}", name);
    }

    0
}
