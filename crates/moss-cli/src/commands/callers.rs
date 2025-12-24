//! Callers command - find callers of a symbol.

use crate::index;
use std::path::Path;

/// Find callers of a symbol
pub fn cmd_callers(symbol: &str, root: Option<&Path>, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Try index first (fast path)
    if let Ok(idx) = index::FileIndex::open(&root) {
        // Check if call graph is populated
        let (_, calls, _) = idx.call_graph_stats().unwrap_or((0, 0, 0));
        if calls > 0 {
            // Index is populated - use it exclusively (don't fall back to slow scan)
            if let Ok(callers) = idx.find_callers(symbol) {
                if !callers.is_empty() {
                    if json {
                        let output: Vec<_> = callers
                            .iter()
                            .map(|(file, sym, line)| {
                                serde_json::json!({"file": file, "symbol": sym, "line": line})
                            })
                            .collect();
                        println!("{}", serde_json::to_string(&output).unwrap());
                    } else {
                        println!("Callers of {}:", symbol);
                        for (file, sym, line) in &callers {
                            println!("  {}:{}:{}", file, line, sym);
                        }
                    }
                    return 0;
                }
            }
            // Index populated but no results - symbol not called anywhere
            eprintln!(
                "No callers found for: {} (index has {} calls)",
                symbol, calls
            );
            return 1;
        }
    }

    // Index empty or stale - auto-reindex (incremental is faster)
    eprintln!("Call graph not indexed. Building now...");

    if let Ok(mut idx) = index::FileIndex::open(&root) {
        // Ensure file index is populated first
        if idx.needs_refresh() {
            if let Err(e) = idx.incremental_refresh() {
                eprintln!("Failed to refresh file index: {}", e);
                return 1;
            }
        }

        // Now build call graph (incremental uses mtime to skip unchanged files)
        match idx.incremental_call_graph_refresh() {
            Ok((symbols, calls, imports)) => {
                eprintln!(
                    "Indexed {} symbols, {} calls, {} imports",
                    symbols, calls, imports
                );

                // Retry the query
                if let Ok(callers) = idx.find_callers(symbol) {
                    if !callers.is_empty() {
                        if json {
                            let output: Vec<_> = callers
                                .iter()
                                .map(|(file, sym, line)| {
                                    serde_json::json!({"file": file, "symbol": sym, "line": line})
                                })
                                .collect();
                            println!("{}", serde_json::to_string(&output).unwrap());
                        } else {
                            println!("Callers of {}:", symbol);
                            for (file, sym, line) in &callers {
                                println!("  {}:{}:{}", file, line, sym);
                            }
                        }
                        return 0;
                    }
                }
                eprintln!("No callers found for: {}", symbol);
                return 1;
            }
            Err(e) => {
                eprintln!("Failed to build call graph: {}", e);
                return 1;
            }
        }
    }

    eprintln!("Failed to open index");
    1
}
