//! Tree command - display directory tree.

use crate::{path_resolve, tree};
use std::path::Path;

/// Display directory tree
pub fn cmd_tree(
    path: &str,
    root: Option<&Path>,
    depth: Option<usize>,
    dirs_only: bool,
    json: bool,
) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Resolve the path using unified resolution (handles trailing slashes)
    let target_dir = if path == "." {
        root.clone()
    } else {
        match path_resolve::resolve_unified(path, &root) {
            Some(u) if u.is_directory => root.join(&u.file_path),
            _ => {
                // Maybe it's an exact path
                let exact = root.join(path);
                if exact.is_dir() {
                    exact
                } else {
                    eprintln!("Directory not found: {}", path);
                    return 1;
                }
            }
        }
    };

    let result = tree::generate_tree(&target_dir, depth, dirs_only);

    if json {
        println!(
            "{}",
            serde_json::json!({
                "root": result.root_name,
                "file_count": result.file_count,
                "dir_count": result.dir_count,
                "tree": result.lines
            })
        );
    } else {
        for line in &result.lines {
            println!("{}", line);
        }
        println!();
        println!(
            "{} directories, {} files",
            result.dir_count, result.file_count
        );
    }

    0
}
