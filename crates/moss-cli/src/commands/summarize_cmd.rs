//! Summarize command - generate module summary.

use crate::{path_resolve, summarize};
use std::path::Path;

/// Generate a summary of a module
pub fn cmd_summarize(file: &str, root: Option<&Path>, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Resolve the file
    let matches = path_resolve::resolve(file, &root);
    let file_match = match matches.iter().find(|m| m.kind == "file") {
        Some(m) => m,
        None => {
            eprintln!("File not found: {}", file);
            return 1;
        }
    };

    let file_path = root.join(&file_match.path);
    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return 1;
        }
    };

    let summary = summarize::summarize_module(&file_path, &content);

    if json {
        let exports: Vec<_> = summary
            .main_exports
            .iter()
            .map(|e| {
                serde_json::json!({
                    "name": e.name,
                    "kind": e.kind,
                    "signature": e.signature,
                    "docstring": e.docstring
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::json!({
                "file": file_match.path,
                "module_name": summary.module_name,
                "purpose": summary.purpose,
                "exports": exports,
                "dependencies": summary.dependencies,
                "line_count": summary.line_count
            })
        );
    } else {
        println!("{}", summary.format());
    }

    0
}
