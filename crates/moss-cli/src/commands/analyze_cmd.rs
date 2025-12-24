//! Analyze command - run analysis on target.

use crate::analyze;
use std::path::Path;

/// Run analysis on a target (file or directory)
pub fn cmd_analyze(
    target: Option<&str>,
    root: Option<&Path>,
    health: bool,
    complexity: bool,
    security: bool,
    threshold: Option<usize>,
    kind_filter: Option<&str>,
    json: bool,
) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // If no specific flags, run all analyses
    let any_flag = health || complexity || security;
    let (run_health, run_complexity, run_security) = if !any_flag {
        (true, true, true)
    } else {
        (health, complexity, security)
    };

    let report = analyze::analyze(
        target,
        &root,
        run_health,
        run_complexity,
        run_security,
        threshold,
        kind_filter,
    );

    if json {
        println!("{}", report.to_json());
    } else {
        println!("{}", report.format());
    }

    0
}
