//! Find symbols command - search for symbols across the codebase.

use crate::index;
use std::path::Path;

/// Search for symbols across the codebase
pub fn cmd_find_symbols(
    name: &str,
    root: Option<&Path>,
    kind: Option<&str>,
    fuzzy: bool,
    limit: usize,
    json: bool,
) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Open or create index
    let idx = match index::FileIndex::open(&root) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open index: {}", e);
            return 1;
        }
    };

    // Check if call graph is populated (symbols are indexed there)
    let (symbol_count, _, _) = idx.call_graph_stats().unwrap_or((0, 0, 0));
    if symbol_count == 0 {
        eprintln!("Symbol index empty. Run: moss reindex --call-graph");
        return 1;
    }

    // Query symbols
    match idx.find_symbols(name, kind, fuzzy, limit) {
        Ok(symbols) => {
            if symbols.is_empty() {
                if json {
                    println!("[]");
                } else {
                    eprintln!("No symbols found matching: {}", name);
                }
                return 1;
            }

            if json {
                let output: Vec<_> = symbols
                    .iter()
                    .map(|(sym_name, sym_kind, file, start, end, parent)| {
                        serde_json::json!({
                            "name": sym_name,
                            "kind": sym_kind,
                            "file": file,
                            "line": start,
                            "end_line": end,
                            "parent": parent
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                for (sym_name, sym_kind, file, start, _end, parent) in &symbols {
                    let parent_str = parent
                        .as_ref()
                        .map(|p| format!(" (in {})", p))
                        .unwrap_or_default();
                    println!("{}:{} {} {}{}", file, start, sym_kind, sym_name, parent_str);
                }
            }
            0
        }
        Err(e) => {
            eprintln!("Query failed: {}", e);
            1
        }
    }
}
