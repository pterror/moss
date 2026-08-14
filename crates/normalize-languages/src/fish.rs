//! Fish shell language support.

use crate::{Import, Language, LanguageSymbols};
use tree_sitter::Node;

/// Fish shell language support.
pub struct Fish;

impl Language for Fish {
    fn name(&self) -> &'static str {
        "Fish"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["fish"]
    }
    fn grammar_name(&self) -> &'static str {
        "fish"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        // Mirrors fish.imports.scm's field-based match, not raw text slicing.
        // The prior implementation matched on `text.starts_with("source ")`,
        // which (a) missed the `.` legacy alias (still a working builtin in
        // fish 4.8.0 — confirmed via `fish -c 'type .'`) and (b) swallowed
        // every trailing word into `module` for `source file.fish arg1 arg2`
        // (fish passes trailing words as positional $argv to the sourced
        // script, the same idiom as bash's `source file.sh arg1 arg2`),
        // producing a bogus multi-word module path instead of just the file.
        // `child_by_field_name("argument")` returns only the first
        // `argument` field child, matching the `.` anchor used in
        // fish.imports.scm to restrict to the first argument.
        if node.kind() != "command" {
            return Vec::new();
        }

        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return Vec::new(),
        };
        let name_text = &content[name_node.byte_range()];
        if name_text != "source" && name_text != "." {
            return Vec::new();
        }

        let arg_node = match node.child_by_field_name("argument") {
            Some(n) => n,
            None => return Vec::new(),
        };
        let module = content[arg_node.byte_range()].trim().to_string();
        if module.is_empty() {
            return Vec::new();
        }

        vec![Import {
            module,
            names: Vec::new(),
            alias: None,
            is_wildcard: false,
            is_relative: true,
            line: node.start_position().row + 1,
        }]
    }

    fn format_import(&self, import: &Import, _names: Option<&[&str]>) -> String {
        // Fish: source file
        format!("source {}", import.module)
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

impl LanguageSymbols for Fish {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            "else_clause", "negated_statement", "redirect_statement", "return",
            // control flow — not extracted as symbols
            "begin_statement",
            "switch_statement",
            "for_statement",
            "case_clause",
            "if_statement",
            "else_if_clause",
            "while_statement",
        ];
        validate_unused_kinds_audit(&Fish, documented_unused)
            .expect("Fish unused node kinds audit failed");
    }
}
