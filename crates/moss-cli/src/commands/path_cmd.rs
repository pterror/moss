//! Path command - resolve file paths with fuzzy matching.

use crate::{daemon, path_resolve};
use std::path::Path;

/// Resolve file paths using fuzzy matching
pub fn cmd_path(query: &str, root: Option<&Path>, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let client = daemon::DaemonClient::new(&root);

    // Try daemon first if available
    if client.is_available() {
        if let Ok(matches) = client.path_query(query) {
            if matches.is_empty() {
                if json {
                    println!("[]");
                } else {
                    eprintln!("No matches for: {}", query);
                }
                return 1;
            }
            if json {
                let output: Vec<_> = matches
                    .iter()
                    .map(|m| serde_json::json!({"path": m.path, "kind": m.kind}))
                    .collect();
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                for m in &matches {
                    println!("{} ({})", m.path, m.kind);
                }
            }
            return 0;
        }
        // Fall through to direct if daemon query failed
    } else {
        // Auto-start daemon in background for future queries
        let client_clone = daemon::DaemonClient::new(&root);
        std::thread::spawn(move || {
            let _ = client_clone.ensure_running();
        });
    }

    // Direct path resolution
    let matches = path_resolve::resolve(query, &root);
    if matches.is_empty() {
        if json {
            println!("[]");
        } else {
            eprintln!("No matches for: {}", query);
        }
        return 1;
    }

    if json {
        let output: Vec<_> = matches
            .iter()
            .map(|m| serde_json::json!({"path": m.path, "kind": m.kind}))
            .collect();
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        for m in &matches {
            println!("{} ({})", m.path, m.kind);
        }
    }

    0
}
