//! jq language support.

use crate::{Import, Language, LanguageSymbols};
use tree_sitter::Node;

/// jq language support.
pub struct Jq;

impl Language for Jq {
    fn name(&self) -> &'static str {
        "jq"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["jq"]
    }
    fn grammar_name(&self) -> &'static str {
        "jq"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        // The bundled jq.imports.scm handles extraction normally (both consumers
        // of this trait method — normalize-facts and normalize-deps — try the
        // query first and only fall back here if it's absent or fails to
        // compile); this exists as that fallback, so it must stand on its own.
        //
        // `import`/`include` statements both parse as an `import_` node (verified
        // via `normalize syntax ast`: arborium-jq's grammar reuses the same rule
        // for both keywords, differing only in the first token) — not a node of
        // kind `import`, which never actually occurs (`import`/`include` are the
        // anonymous leading keyword tokens, not the statement's own node kind).
        // The previous `node.kind() != "import"` check could therefore never
        // match any real node, silently making this fallback a no-op.
        if node.kind() != "import_" {
            return Vec::new();
        }

        let text = &content[node.byte_range()];
        let line = node.start_position().row + 1;

        // import "path" as name;
        // include "path";  (same node kind, never takes an alias)
        let rest = text
            .strip_prefix("import ")
            .or_else(|| text.strip_prefix("include "));
        if let Some(rest) = rest {
            let module = rest.split('"').nth(1).map(|s| s.to_string());
            let alias = rest
                .split(" as ")
                .nth(1)
                .and_then(|s| s.split(';').next())
                .map(|s| s.trim().to_string());

            if let Some(module) = module {
                return vec![Import {
                    module,
                    names: Vec::new(),
                    alias,
                    is_wildcard: false,
                    is_relative: true,
                    line,
                }];
            }
        }

        Vec::new()
    }

    fn format_import(&self, import: &Import, _names: Option<&[&str]>) -> String {
        // jq: import "module" as name
        format!("import \"{}\"", import.module)
    }

    fn is_test_symbol(&self, symbol: &crate::Symbol) -> bool {
        let name = symbol.name.as_str();
        match symbol.kind {
            crate::SymbolKind::Function | crate::SymbolKind::Method => name.starts_with("test_"),
            crate::SymbolKind::Module => name == "tests" || name == "test",
            _ => false,
        }
    }
}

impl LanguageSymbols for Jq {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            "catch", "elif", "else", "format", "import_", "moduleheader",
            "programbody",
        ];
        validate_unused_kinds_audit(&Jq, documented_unused)
            .expect("jq unused node kinds audit failed");
    }
}
