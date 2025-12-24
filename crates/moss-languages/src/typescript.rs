//! TypeScript language support.

use crate::{LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use moss_core::tree_sitter::Node;

/// TypeScript language support.
pub struct TypeScript;

/// TSX language support (TypeScript + JSX).
pub struct Tsx;

impl LanguageSupport for TypeScript {
    fn name(&self) -> &'static str { "TypeScript" }
    fn extensions(&self) -> &'static [&'static str] { &["ts", "mts", "cts"] }
    fn grammar_name(&self) -> &'static str { "typescript" }

    fn container_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "class"]
    }

    fn function_kinds(&self) -> &'static [&'static str] {
        &["function_declaration", "method_definition"]
    }

    fn type_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "interface_declaration", "type_alias_declaration", "enum_declaration"]
    }

    fn import_kinds(&self) -> &'static [&'static str] {
        &["import_statement"]
    }

    fn public_symbol_kinds(&self) -> &'static [&'static str] {
        &["export_statement"]
    }

    fn visibility_mechanism(&self) -> VisibilityMechanism {
        VisibilityMechanism::ExplicitExport
    }

    fn scope_creating_kinds(&self) -> &'static [&'static str] {
        &["for_statement", "for_in_statement", "while_statement", "do_statement", "try_statement", "catch_clause", "switch_statement", "arrow_function"]
    }

    fn control_flow_kinds(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "for_in_statement", "while_statement", "do_statement", "switch_statement", "try_statement", "return_statement", "break_statement", "continue_statement", "throw_statement"]
    }

    fn complexity_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "for_in_statement", "while_statement", "do_statement", "switch_case", "catch_clause", "ternary_expression", "binary_expression"]
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "for_in_statement", "while_statement", "do_statement", "switch_statement", "try_statement", "function_declaration", "method_definition", "class_declaration"]
    }

    fn extract_function(&self, node: &Node, content: &str, in_container: bool) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let params = node
            .child_by_field_name("parameters")
            .map(|p| content[p.byte_range()].to_string())
            .unwrap_or_else(|| "()".to_string());

        Some(Symbol {
            name: name.to_string(),
            kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
            signature: format!("function {}{}", name, params),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn extract_container(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;

        Some(Symbol {
            name: name.to_string(),
            kind: SymbolKind::Class,
            signature: format!("class {}", name),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn extract_type(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let (kind, keyword) = match node.kind() {
            "interface_declaration" => (SymbolKind::Interface, "interface"),
            "type_alias_declaration" => (SymbolKind::Type, "type"),
            "enum_declaration" => (SymbolKind::Enum, "enum"),
            "class_declaration" => (SymbolKind::Class, "class"),
            _ => return None,
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

// TSX shares the same implementation as TypeScript
impl LanguageSupport for Tsx {
    fn name(&self) -> &'static str { "TSX" }
    fn extensions(&self) -> &'static [&'static str] { &["tsx"] }
    fn grammar_name(&self) -> &'static str { "tsx" }

    fn container_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "class"]
    }

    fn function_kinds(&self) -> &'static [&'static str] {
        &["function_declaration", "method_definition"]
    }

    fn type_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "interface_declaration", "type_alias_declaration", "enum_declaration"]
    }

    fn import_kinds(&self) -> &'static [&'static str] {
        &["import_statement"]
    }

    fn public_symbol_kinds(&self) -> &'static [&'static str] {
        &["export_statement"]
    }

    fn visibility_mechanism(&self) -> VisibilityMechanism {
        VisibilityMechanism::ExplicitExport
    }

    fn scope_creating_kinds(&self) -> &'static [&'static str] {
        &["for_statement", "for_in_statement", "while_statement", "do_statement", "try_statement", "catch_clause", "switch_statement", "arrow_function"]
    }

    fn control_flow_kinds(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "for_in_statement", "while_statement", "do_statement", "switch_statement", "try_statement", "return_statement", "break_statement", "continue_statement", "throw_statement"]
    }

    fn complexity_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "for_in_statement", "while_statement", "do_statement", "switch_case", "catch_clause", "ternary_expression", "binary_expression"]
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "for_in_statement", "while_statement", "do_statement", "switch_statement", "try_statement", "function_declaration", "method_definition", "class_declaration"]
    }

    fn extract_function(&self, node: &Node, content: &str, in_container: bool) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let params = node
            .child_by_field_name("parameters")
            .map(|p| content[p.byte_range()].to_string())
            .unwrap_or_else(|| "()".to_string());

        Some(Symbol {
            name: name.to_string(),
            kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
            signature: format!("function {}{}", name, params),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn extract_container(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;

        Some(Symbol {
            name: name.to_string(),
            kind: SymbolKind::Class,
            signature: format!("class {}", name),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: Visibility::Public,
            children: Vec::new(),
        })
    }

    fn extract_type(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let (kind, keyword) = match node.kind() {
            "interface_declaration" => (SymbolKind::Interface, "interface"),
            "type_alias_declaration" => (SymbolKind::Type, "type"),
            "enum_declaration" => (SymbolKind::Enum, "enum"),
            "class_declaration" => (SymbolKind::Class, "class"),
            _ => return None,
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
