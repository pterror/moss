//! TOML language support.

use crate::{LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use moss_core::tree_sitter::Node;

/// TOML language support.
pub struct Toml;

impl LanguageSupport for Toml {
    fn name(&self) -> &'static str { "TOML" }
    fn extensions(&self) -> &'static [&'static str] { &["toml"] }
    fn grammar_name(&self) -> &'static str { "toml" }

    // TOML is config, not code - no functions/types/control flow
    fn container_kinds(&self) -> &'static [&'static str] { &["table", "table_array_element"] }
    fn function_kinds(&self) -> &'static [&'static str] { &[] }
    fn type_kinds(&self) -> &'static [&'static str] { &[] }
    fn import_kinds(&self) -> &'static [&'static str] { &[] }
    fn public_symbol_kinds(&self) -> &'static [&'static str] { &[] }
    fn visibility_mechanism(&self) -> VisibilityMechanism { VisibilityMechanism::NotApplicable }
    fn scope_creating_kinds(&self) -> &'static [&'static str] { &[] }
    fn control_flow_kinds(&self) -> &'static [&'static str] { &[] }
    fn complexity_nodes(&self) -> &'static [&'static str] { &[] }
    fn nesting_nodes(&self) -> &'static [&'static str] { &[] }

    fn extract_function(&self, _node: &Node, _content: &str, _in_container: bool) -> Option<Symbol> {
        None
    }

    fn extract_container(&self, node: &Node, content: &str) -> Option<Symbol> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "bare_key" || child.kind() == "dotted_key" || child.kind() == "quoted_key" {
                let name = content[child.byte_range()].to_string();
                return Some(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Module,
                    signature: format!("[{}]", name),
                    docstring: None,
                    start_line: node.start_position().row + 1,
                    end_line: node.end_position().row + 1,
                    visibility: Visibility::Public,
                    children: Vec::new(),
                });
            }
        }
        None
    }
}
