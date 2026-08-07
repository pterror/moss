//! TOML language support.

use crate::{Language, LanguageSymbols};
use tree_sitter::Node;

/// TOML language support.
pub struct Toml;

impl Language for Toml {
    fn name(&self) -> &'static str {
        "TOML"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["toml"]
    }
    fn grammar_name(&self) -> &'static str {
        "toml"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    fn node_name<'a>(&self, node: &Node, content: &'a str) -> Option<&'a str> {
        match node.kind() {
            // `array_table` is not a node kind arborium-toml 2.17.0's grammar
            // actually produces (confirmed against node-types.json) —
            // `table_array_element` is the only `[[...]]` node kind.
            "table" | "table_array_element" => {
                // The header key is a bare_key, quoted_key, or dotted_key
                // child (e.g. [foo], ["quoted table"], [tool.poetry]).
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if matches!(child.kind(), "bare_key" | "quoted_key" | "dotted_key") {
                        return Some(&content[child.byte_range()]);
                    }
                }
                None
            }
            "pair" => {
                // Skip pairs inside inline_table (they appear as noise siblings)
                if is_inside_inline_table(node) {
                    return None;
                }
                // The key is a bare_key, quoted_key, or dotted_key child
                // (e.g. foo = 1, "quoted key" = 1, a.b.c = 1).
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if matches!(child.kind(), "bare_key" | "quoted_key" | "dotted_key") {
                        return Some(&content[child.byte_range()]);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn build_signature(&self, node: &Node, content: &str) -> String {
        if let Some(key) = self.node_name(node, content) {
            match node.kind() {
                "table" | "table_array_element" => {
                    let brackets = if node.kind() == "table_array_element" {
                        ("[[", "]]")
                    } else {
                        ("[", "]")
                    };
                    return format!("{}{}{}", brackets.0, key, brackets.1);
                }
                "pair" => {
                    // Find value child (after the = sign)
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        let k = child.kind();
                        if !matches!(k, "bare_key" | "quoted_key" | "dotted_key" | "=") {
                            let val_text = &content[child.byte_range()];
                            if val_text.len() > 40 {
                                return format!("{} = {}…", key, &val_text[..37]);
                            }
                            return format!("{} = {}", key, val_text);
                        }
                    }
                    return key.to_string();
                }
                _ => {}
            }
        }
        content[node.byte_range()]
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

impl LanguageSymbols for Toml {}

/// Check if a node is inside an inline_table by walking up the parent chain.
fn is_inside_inline_table(node: &Node) -> bool {
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind() == "inline_table" {
            return true;
        }
        current = n.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        // TOML has no "interesting" unused kinds matching our patterns
        let documented_unused: &[&str] = &[];
        validate_unused_kinds_audit(&Toml, documented_unused)
            .expect("TOML unused node kinds audit failed");
    }

    /// `node_name`/`build_signature` must recognize `dotted_key` and
    /// `quoted_key` headers/keys, not just `bare_key` — arborium-toml
    /// 2.17.0 uses whichever of the three appears as a direct child, with
    /// no named field to distinguish them (node-types.json shows an empty
    /// `fields` object for `table`/`table_array_element`/`pair`). Skips
    /// gracefully if the `toml` grammar `.so` isn't built locally (same
    /// convention as `query_fixtures.rs`'s `grammar_dir()` skip).
    #[test]
    fn node_name_handles_dotted_and_quoted_keys() {
        // Use the locally-built `target/grammars/` copy (same convention as
        // `query_fixtures.rs`'s `grammar_dir()`), not the installed
        // `~/.config/normalize/grammars` copy `GrammarLoader::new()` resolves
        // by default — the installed copy can be stale relative to a
        // grammar rebuilt for this session (`cargo xtask build-grammars`).
        let crate_root = std::env::current_dir().unwrap();
        let Some(workspace_root) = crate_root
            .ancestors()
            .find(|p| p.join("Cargo.lock").exists())
        else {
            eprintln!(
                "Skipping node_name_handles_dotted_and_quoted_keys: workspace root not found"
            );
            return;
        };
        let grammars_dir = workspace_root.join("target/grammars");
        if !grammars_dir.exists() {
            eprintln!(
                "Skipping node_name_handles_dotted_and_quoted_keys: run `cargo xtask build-grammars` first"
            );
            return;
        }
        let loader = crate::GrammarLoader::with_paths(vec![grammars_dir]);
        let Ok(ts_lang) = loader.get("toml") else {
            eprintln!(
                "Skipping node_name_handles_dotted_and_quoted_keys: toml grammar .so not found"
            );
            return;
        };

        let source = "[tool.poetry]\na.b.c = 1\n[\"quoted table\"]\nx = 2\n";
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&ts_lang).expect("set_language failed");
        let tree = parser.parse(source, None).expect("parse failed");
        let root = tree.root_node();

        let toml = Toml;
        let mut found_dotted_table = false;
        let mut found_dotted_pair = false;
        let mut found_quoted_table = false;
        // `a.b.c = 1` is a pair *inside* the preceding `[tool.poetry]` table
        // (TOML has no nested block scoping — a pair belongs to whichever
        // table header precedes it), so walk every descendant, not just
        // direct children of the document root.
        fn walk(
            node: Node,
            source: &str,
            toml: &Toml,
            found_dotted_table: &mut bool,
            found_dotted_pair: &mut bool,
            found_quoted_table: &mut bool,
        ) {
            match node.kind() {
                "table" => {
                    let name = toml.node_name(&node, source);
                    if name == Some("tool.poetry") {
                        *found_dotted_table = true;
                        assert_eq!(toml.build_signature(&node, source), "[tool.poetry]");
                    } else if name == Some("\"quoted table\"") {
                        *found_quoted_table = true;
                        assert_eq!(toml.build_signature(&node, source), "[\"quoted table\"]");
                    }
                }
                "pair" if toml.node_name(&node, source) == Some("a.b.c") => {
                    *found_dotted_pair = true;
                    assert_eq!(toml.build_signature(&node, source), "a.b.c = 1");
                }
                _ => {}
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk(
                    child,
                    source,
                    toml,
                    found_dotted_table,
                    found_dotted_pair,
                    found_quoted_table,
                );
            }
        }
        walk(
            root,
            source,
            &toml,
            &mut found_dotted_table,
            &mut found_dotted_pair,
            &mut found_quoted_table,
        );
        assert!(
            found_dotted_table,
            "expected node_name to return Some(\"tool.poetry\") for a dotted table header"
        );
        assert!(
            found_dotted_pair,
            "expected node_name to return Some(\"a.b.c\") for a dotted pair key"
        );
        assert!(
            found_quoted_table,
            "expected node_name to return Some(\"\\\"quoted table\\\"\") for a quoted table header"
        );
    }
}
