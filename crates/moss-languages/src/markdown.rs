//! Markdown language support.

use crate::{Language, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use moss_core::tree_sitter::Node;

/// Markdown language support.
pub struct Markdown;

impl Language for Markdown {
    fn name(&self) -> &'static str { "Markdown" }
    fn extensions(&self) -> &'static [&'static str] { &["md", "markdown"] }
    fn grammar_name(&self) -> &'static str { "markdown" }

    // Markdown is documentation, not code - no functions/types/control flow
    fn container_kinds(&self) -> &'static [&'static str] { &["atx_heading", "setext_heading"] }
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
        // Extract heading text
        let mut cursor = node.walk();
        let text = node.children(&mut cursor)
            .find(|c| c.kind() == "heading_content" || c.kind() == "inline")
            .map(|c| content[c.byte_range()].trim().to_string())
            .unwrap_or_default();

        if text.is_empty() {
            return None;
        }

        // Determine heading level
        let level = node.children(&mut cursor)
            .find(|c| c.kind().starts_with("atx_h"))
            .map(|c| c.kind().chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize)
            .unwrap_or(1);

        Some(Symbol {
            name: text.clone(),
            kind: SymbolKind::Heading,
            signature: format!("{} {}", "#".repeat(level), text),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn should_skip_package_entry(&self, name: &str, is_dir: bool) -> bool {
        use crate::traits::{skip_dotfiles, has_extension};
        if skip_dotfiles(name) { return true; }
        !is_dir && !has_extension(name, &["md", "markdown"])
    }
}
