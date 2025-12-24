//! Analyze command - run analysis on target.

use crate::analyze;
use crate::overview;
use std::path::Path;

/// Run analysis on a target (file or directory)
pub fn cmd_analyze(
    target: Option<&str>,
    root: Option<&Path>,
    health: bool,
    complexity: bool,
    security: bool,
    show_overview: bool,
    compact: bool,
    threshold: Option<usize>,
    kind_filter: Option<&str>,
    json: bool,
) -> i32 {
    // --overview runs the overview report
    if show_overview {
        return cmd_overview(root, compact, json);
    }

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

/// Analyze codebase overview
fn cmd_overview(root: Option<&Path>, compact: bool, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let report = overview::analyze_overview(&root);

    if json {
        println!(
            "{}",
            serde_json::json!({
                "total_files": report.total_files,
                "files_by_language": report.files_by_language,
                "total_lines": report.total_lines,
                "total_functions": report.total_functions,
                "total_classes": report.total_classes,
                "total_methods": report.total_methods,
                "avg_complexity": (report.avg_complexity * 10.0).round() / 10.0,
                "max_complexity": report.max_complexity,
                "high_risk_functions": report.high_risk_functions,
                "functions_with_docs": report.functions_with_docs,
                "doc_coverage": (report.doc_coverage * 100.0).round() / 100.0,
                "total_imports": report.total_imports,
                "unique_modules": report.unique_modules,
                "todo_count": report.todo_count,
                "fixme_count": report.fixme_count,
                "health_score": (report.health_score * 100.0).round() / 100.0,
                "grade": report.grade
            })
        );
    } else if compact {
        println!("{}", report.format_compact());
    } else {
        println!("{}", report.format());
    }

    0
}
