//! Directory tree visualization.
//!
//! Git-aware tree display using the `ignore` crate for gitignore support.

use ignore::WalkBuilder;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// Default boilerplate directories that don't count against depth limit.
/// These are common structural directories that add noise without information.
pub const DEFAULT_BOILERPLATE_DIRS: &[&str] = &[
    "src",
    "lib",
    "pkg",
    "packages",
    "crates",
    "internal",
    "cmd",
];

/// Options for tree generation
#[derive(Clone)]
pub struct TreeOptions {
    /// Maximum depth to traverse (None = unlimited)
    pub max_depth: Option<usize>,
    /// Collapse single-child directory chains (src/foo/bar/ → one line)
    pub collapse_single: bool,
    /// Directories that don't count against depth limit (smart depth)
    pub boilerplate_dirs: HashSet<String>,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            collapse_single: true,
            boilerplate_dirs: DEFAULT_BOILERPLATE_DIRS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Result of tree generation
pub struct TreeResult {
    #[allow(dead_code)] // Part of public API
    pub root_name: String,
    pub lines: Vec<String>,
    pub file_count: usize,
    pub dir_count: usize,
}

/// A node in the file tree
#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_dir: bool,
}

impl TreeNode {
    fn add_path(
        &mut self,
        parts: &[&str],
        is_dir: bool,
        max_depth: Option<usize>,
        boilerplate_dirs: &HashSet<String>,
        effective_depth: usize,
    ) {
        if parts.is_empty() {
            return;
        }

        let name = parts[0];

        // Check if we've exceeded max depth (using effective depth that excludes boilerplate)
        // Boilerplate dirs themselves are always shown, but they don't increase depth
        if let Some(max) = max_depth {
            let is_boilerplate = boilerplate_dirs.contains(name);
            // Block if we're at max depth, unless this is a boilerplate dir (which gets a pass)
            if effective_depth >= max && !is_boilerplate {
                return;
            }
        }

        let child = self.children.entry(name.to_string()).or_default();

        if parts.len() == 1 {
            child.is_dir = is_dir;
        } else {
            child.is_dir = true; // intermediate nodes are directories
            // Boilerplate dirs don't count against depth
            let next_depth = if boilerplate_dirs.contains(name) {
                effective_depth
            } else {
                effective_depth + 1
            };
            child.add_path(&parts[1..], is_dir, max_depth, boilerplate_dirs, next_depth);
        }
    }
}

/// Generate a tree visualization for a directory
pub fn generate_tree(root: &Path, options: &TreeOptions) -> TreeResult {
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // Don't use WalkBuilder's max_depth - we handle it with smart depth (boilerplate awareness)
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    let mut tree = TreeNode::default();
    tree.is_dir = true;

    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in walker.flatten() {
        let path = entry.path();
        if path == root {
            continue;
        }

        if let Ok(rel) = path.strip_prefix(root) {
            let rel_str = rel.to_string_lossy();
            if rel_str.is_empty() {
                continue;
            }

            let is_dir = path.is_dir();
            let parts: Vec<&str> = rel_str.split('/').filter(|s| !s.is_empty()).collect();
            if !parts.is_empty() {
                tree.add_path(
                    &parts,
                    is_dir,
                    options.max_depth,
                    &options.boilerplate_dirs,
                    0,
                );

                if is_dir {
                    dir_count += 1;
                } else {
                    file_count += 1;
                }
            }
        }
    }

    let mut lines = vec![root_name.clone()];
    render_tree(&tree, "", &mut lines, options);

    TreeResult {
        root_name,
        lines,
        file_count,
        dir_count,
    }
}

/// Result of collapsing a chain of single-child directories
struct CollapsedChain<'a> {
    path: String,
    end_node: &'a TreeNode,
}

/// Collect a chain of single-child directories into a collapsed path
fn collect_single_chain<'a>(node: &'a TreeNode, name: &str) -> CollapsedChain<'a> {
    let mut current = node;
    let mut path = name.to_string();

    loop {
        // Only collapse if exactly one child and it's a directory
        if current.children.len() != 1 {
            break;
        }
        let (child_name, child_node) = current.children.iter().next().unwrap();
        if !child_node.is_dir {
            break;
        }
        // Append to path and continue down the chain
        path.push('/');
        path.push_str(child_name);
        current = child_node;
    }

    CollapsedChain { path, end_node: current }
}

fn render_tree(node: &TreeNode, prefix: &str, lines: &mut Vec<String>, options: &TreeOptions) {
    // Sort children: directories first, then alphabetically
    let mut children: Vec<_> = node.children.iter().collect();
    children.sort_by(
        |(a_name, a_node), (b_name, b_node)| match (b_node.is_dir, a_node.is_dir) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a_name.to_lowercase().cmp(&b_name.to_lowercase()),
        },
    );

    let count = children.len();
    for (i, (name, child)) in children.into_iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };

        // Collapse single-child directory chains if enabled
        let (display_name, effective_child) = if options.collapse_single && child.is_dir {
            let chain = collect_single_chain(child, name);
            (chain.path, chain.end_node)
        } else {
            (name.clone(), child)
        };

        lines.push(format!("{}{}{}", prefix, connector, display_name));

        // Recurse into directories
        if effective_child.is_dir && !effective_child.children.is_empty() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            render_tree(effective_child, &new_prefix, lines, options);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_basic_tree() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/foo")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "").unwrap();
        fs::write(dir.path().join("src/foo/bar.rs"), "").unwrap();
        fs::write(dir.path().join("README.md"), "").unwrap();

        let result = generate_tree(dir.path(), &TreeOptions::default());

        assert!(result.file_count >= 3);
        assert!(result.dir_count >= 2);
        assert!(result.lines.len() > 1);
    }

    #[test]
    fn test_max_depth() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("alpha/beta/gamma/delta")).unwrap();
        fs::write(dir.path().join("alpha/beta/gamma/delta/file.txt"), "").unwrap();

        let result = generate_tree(
            dir.path(),
            &TreeOptions {
                max_depth: Some(2),
                collapse_single: false, // disable collapse to see raw depth
                boilerplate_dirs: HashSet::new(), // no boilerplate
            },
        );

        let tree_text = result.lines.join("\n");
        // Should stop at depth 2 (alpha, alpha/beta) - not show gamma or delta
        assert!(
            tree_text.contains("alpha") && tree_text.contains("beta"),
            "Should show alpha and beta: {}",
            tree_text
        );
        assert!(
            !tree_text.contains("gamma") && !tree_text.contains("delta"),
            "Should not show gamma or delta at depth 2: {}",
            tree_text
        );
    }

    #[test]
    fn test_collapse_single_child() {
        let dir = tempdir().unwrap();
        // Create a/b/c chain with file at end
        fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
        fs::write(dir.path().join("a/b/c/file.txt"), "").unwrap();

        // With collapse enabled (default)
        let result = generate_tree(dir.path(), &TreeOptions::default());
        // Should show "a/b/c" as single line, not 3 separate entries
        let tree_text = result.lines.join("\n");
        assert!(
            tree_text.contains("a/b/c"),
            "Should collapse single-child chain: {}",
            tree_text
        );

        // With collapse disabled
        let result_raw = generate_tree(
            dir.path(),
            &TreeOptions {
                collapse_single: false,
                ..Default::default()
            },
        );
        let raw_text = result_raw.lines.join("\n");
        // Should show separate entries
        assert!(
            !raw_text.contains("a/b/c"),
            "Should not collapse when disabled: {}",
            raw_text
        );
    }

    #[test]
    fn test_collapse_stops_at_multiple_children() {
        let dir = tempdir().unwrap();
        // Create a/b with two children under b
        fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
        fs::create_dir_all(dir.path().join("a/b/d")).unwrap();
        fs::write(dir.path().join("a/b/c/file.txt"), "").unwrap();
        fs::write(dir.path().join("a/b/d/file.txt"), "").unwrap();

        let result = generate_tree(dir.path(), &TreeOptions::default());
        let tree_text = result.lines.join("\n");
        // Should collapse a/b but not further since b has 2 children
        assert!(
            tree_text.contains("a/b"),
            "Should collapse a/b: {}",
            tree_text
        );
        assert!(
            !tree_text.contains("a/b/c"),
            "Should not collapse past fork: {}",
            tree_text
        );
    }

    #[test]
    fn test_boilerplate_dirs_dont_count_against_depth() {
        let dir = tempdir().unwrap();
        // Create src/commands/view.rs - 'src' is boilerplate
        fs::create_dir_all(dir.path().join("src/commands")).unwrap();
        fs::write(dir.path().join("src/commands/view.rs"), "").unwrap();
        fs::write(dir.path().join("README.md"), "").unwrap();

        // With max_depth=1, without boilerplate awareness we'd only see src/
        // With boilerplate awareness, src/ doesn't count, so we see src/commands/
        let mut boilerplate = HashSet::new();
        boilerplate.insert("src".to_string());

        let result = generate_tree(
            dir.path(),
            &TreeOptions {
                max_depth: Some(1),
                collapse_single: false, // disable collapse to see raw structure
                boilerplate_dirs: boilerplate,
            },
        );
        let tree_text = result.lines.join("\n");

        // Should show src/commands because src doesn't count against depth
        assert!(
            tree_text.contains("commands"),
            "Should show commands inside boilerplate src: {}",
            tree_text
        );
    }

    #[test]
    fn test_depth_with_no_boilerplate() {
        let dir = tempdir().unwrap();
        // Create alpha/beta/gamma structure
        fs::create_dir_all(dir.path().join("alpha/beta/gamma")).unwrap();
        fs::write(dir.path().join("alpha/beta/gamma/file.txt"), "").unwrap();

        // With max_depth=1 and no boilerplate, only 'alpha' should be shown
        let result = generate_tree(
            dir.path(),
            &TreeOptions {
                max_depth: Some(1),
                collapse_single: false,
                boilerplate_dirs: HashSet::new(), // no boilerplate
            },
        );
        let tree_text = result.lines.join("\n");

        assert!(
            tree_text.contains("alpha"),
            "Should show 'alpha': {}",
            tree_text
        );
        assert!(
            !tree_text.contains("beta"),
            "Should not show 'beta' at depth 1: {}",
            tree_text
        );
    }
}
