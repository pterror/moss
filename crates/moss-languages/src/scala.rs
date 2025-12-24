//! Scala language support.

use crate::{LanguageSupport, Symbol, SymbolKind, Visibility};
use moss_core::{tree_sitter::Node, Language};

pub struct ScalaSupport;

impl LanguageSupport for ScalaSupport {
    fn language(&self) -> Language { Language::Scala }
    fn grammar_name(&self) -> &'static str { "scala" }

    fn container_kinds(&self) -> &'static [&'static str] { &["class_definition", "object_definition", "trait_definition"] }
    fn function_kinds(&self) -> &'static [&'static str] { &["function_definition"] }
    fn type_kinds(&self) -> &'static [&'static str] { &["class_definition", "trait_definition"] }
    fn import_kinds(&self) -> &'static [&'static str] { todo!("scala: import_kinds") }
    fn export_kinds(&self) -> &'static [&'static str] { &[] } // Scala uses visibility modifiers, not export statements
    fn scope_creating_kinds(&self) -> &'static [&'static str] { todo!("scala: scope_creating_kinds") }
    fn control_flow_kinds(&self) -> &'static [&'static str] { todo!("scala: control_flow_kinds") }
    fn complexity_nodes(&self) -> &'static [&'static str] { todo!("scala: complexity_nodes") }
    fn nesting_nodes(&self) -> &'static [&'static str] { todo!("scala: nesting_nodes") }

    fn extract_function(&self, node: &Node, content: &str, in_container: bool) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let params = node.child_by_field_name("parameters")
            .map(|p| content[p.byte_range()].to_string())
            .unwrap_or_else(|| "()".to_string());
        let ret = node.child_by_field_name("return_type")
            .map(|r| format!(": {}", &content[r.byte_range()]))
            .unwrap_or_default();

        Some(Symbol {
            name: name.to_string(),
            kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
            signature: format!("def {}{}{}", name, params, ret),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn extract_container(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let (kind, keyword) = match node.kind() {
            "object_definition" => (SymbolKind::Module, "object"),
            "trait_definition" => (SymbolKind::Trait, "trait"),
            _ => (SymbolKind::Class, "class"),
        };

        Some(Symbol {
            name: name.to_string(),
            kind,
            signature: format!("{} {}", keyword, name),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }
}
