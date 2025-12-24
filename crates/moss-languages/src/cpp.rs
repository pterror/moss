//! C++ language support.

use crate::{LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use moss_core::{tree_sitter::Node, Language};

pub struct CppSupport;

impl LanguageSupport for CppSupport {
    fn language(&self) -> Language { Language::Cpp }
    fn grammar_name(&self) -> &'static str { "cpp" }

    fn container_kinds(&self) -> &'static [&'static str] { &["class_specifier", "struct_specifier"] }
    fn function_kinds(&self) -> &'static [&'static str] { &["function_definition"] }
    fn type_kinds(&self) -> &'static [&'static str] { &["class_specifier", "struct_specifier", "enum_specifier", "type_definition"] }
    fn import_kinds(&self) -> &'static [&'static str] { &["preproc_include"] }

    fn public_symbol_kinds(&self) -> &'static [&'static str] {
        &["function_definition", "class_specifier", "struct_specifier"]
    }

    fn visibility_mechanism(&self) -> VisibilityMechanism {
        VisibilityMechanism::HeaderBased // Also has public/private in classes, but header-based is primary
    }
    fn scope_creating_kinds(&self) -> &'static [&'static str] { todo!("cpp: scope_creating_kinds") }
    fn control_flow_kinds(&self) -> &'static [&'static str] { todo!("cpp: control_flow_kinds") }
    fn complexity_nodes(&self) -> &'static [&'static str] { todo!("cpp: complexity_nodes") }
    fn nesting_nodes(&self) -> &'static [&'static str] { todo!("cpp: nesting_nodes") }

    fn extract_function(&self, node: &Node, content: &str, in_container: bool) -> Option<Symbol> {
        let declarator = node.child_by_field_name("declarator")?;
        let name = find_identifier(&declarator, content)?;

        Some(Symbol {
            name: name.to_string(),
            kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
            signature: name.to_string(),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn extract_container(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let kind = if node.kind() == "class_specifier" { SymbolKind::Class } else { SymbolKind::Struct };

        Some(Symbol {
            name: name.to_string(),
            kind,
            signature: format!("{} {}", kind.as_str(), name),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }
}

fn find_identifier<'a>(node: &Node, content: &'a str) -> Option<&'a str> {
    if node.kind() == "identifier" || node.kind() == "field_identifier" {
        return Some(&content[node.byte_range()]);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(id) = find_identifier(&child, content) {
            return Some(id);
        }
    }
    None
}
