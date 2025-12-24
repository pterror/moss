//! Symbols command - list symbols in a file.

use crate::{path_resolve, symbols};
use std::path::Path;

/// List symbols in a file
pub fn cmd_symbols(file: &str, root: Option<&Path>, json: bool) -> i32 {
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

    let parser = symbols::SymbolParser::new();
    let symbols = parser.parse_file(&file_path, &content);

    if json {
        let output: Vec<_> = symbols
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "kind": s.kind.as_str(),
                    "start_line": s.start_line,
                    "end_line": s.end_line,
                    "parent": s.parent
                })
            })
            .collect();
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        for s in &symbols {
            let parent_str = s
                .parent
                .as_ref()
                .map(|p| format!(" (in {})", p))
                .unwrap_or_default();
            println!(
                "{}:{}-{} {} {}{}",
                file_match.path,
                s.start_line,
                s.end_line,
                s.kind.as_str(),
                s.name,
                parent_str
            );
        }
    }

    0
}
