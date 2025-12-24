//! JSON language support.

use crate::{Language, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use moss_core::tree_sitter::Node;

/// JSON language support.
pub struct Json;

impl Language for Json {
    fn name(&self) -> &'static str { "JSON" }
    fn extensions(&self) -> &'static [&'static str] { &["json", "jsonc"] }
    fn grammar_name(&self) -> &'static str { "json" }

    // JSON is data, not code - no functions/types/control flow
    fn container_kinds(&self) -> &'static [&'static str] { &["object"] }
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
        // Extract top-level object keys
        if node.kind() == "pair" {
            let key = node.child_by_field_name("key")?;
            let key_text = content[key.byte_range()].trim_matches('"');

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

    fn should_skip_package_entry(&self, name: &str, is_dir: bool) -> bool {
        use crate::traits::{skip_dotfiles, has_extension};
        if skip_dotfiles(name) { return true; }
        !is_dir && !has_extension(name, &["json", "jsonc"])
    }
}
