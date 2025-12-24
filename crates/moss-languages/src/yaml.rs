//! YAML language support.

use crate::{Language, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use moss_core::tree_sitter::Node;

/// YAML language support.
pub struct Yaml;

impl Language for Yaml {
    fn name(&self) -> &'static str { "YAML" }
    fn extensions(&self) -> &'static [&'static str] { &["yaml", "yml"] }
    fn grammar_name(&self) -> &'static str { "yaml" }

    // YAML is data, not code - no functions/types/control flow
    fn container_kinds(&self) -> &'static [&'static str] { &["block_mapping", "flow_mapping"] }
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
        if node.kind() == "block_mapping_pair" {
            let key = node.child_by_field_name("key")?;
            let key_text = &content[key.byte_range()];

            return Some(Symbol {
                name: key_text.to_string(),
                kind: SymbolKind::Variable,
                signature: key_text.to_string(),
                docstring: None,
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                visibility: Visibility::Public,
                children: Vec::new(),
            });
        }
        None
    }
}
