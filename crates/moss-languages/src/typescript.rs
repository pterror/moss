//! TypeScript language support.

use std::path::{Path, PathBuf};
use crate::{LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use crate::external_packages::{self, ResolvedPackage};
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

    // === Import Resolution ===

    fn lang_key(&self) -> &'static str { "js" } // Uses same cache as JS

    fn resolve_local_import(
        &self,
        module: &str,
        current_file: &Path,
        _project_root: &Path,
    ) -> Option<PathBuf> {
        // Only handle relative imports
        if !module.starts_with('.') {
            return None;
        }

        let current_dir = current_file.parent()?;

        // Normalize the path
        let target = if module.starts_with("./") {
            current_dir.join(&module[2..])
        } else if module.starts_with("../") {
            current_dir.join(module)
        } else {
            return None;
        };

        // Try various extensions in order of preference
        let extensions = ["ts", "tsx", "js", "jsx", "mts", "mjs"];

        // First try exact path (might already have extension)
        if target.exists() && target.is_file() {
            return Some(target);
        }

        // Try adding extensions
        for ext in &extensions {
            let with_ext = target.with_extension(ext);
            if with_ext.exists() && with_ext.is_file() {
                return Some(with_ext);
            }
        }

        // Try index files in directory
        if target.is_dir() {
            for ext in &extensions {
                let index = target.join(format!("index.{}", ext));
                if index.exists() && index.is_file() {
                    return Some(index);
                }
            }
        }

        None
    }

    fn resolve_external_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
        if import_name.starts_with('.') || import_name.starts_with('/') {
            return None;
        }

        let node_modules = external_packages::find_node_modules(project_root)?;
        external_packages::resolve_node_import(import_name, &node_modules)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        external_packages::get_node_version()
    }

    fn find_package_cache(&self, project_root: &Path) -> Option<PathBuf> {
        external_packages::find_node_modules(project_root)
    }

    fn indexable_extensions(&self) -> &'static [&'static str] {
        &["ts", "mts", "cts", "js", "mjs", "cjs"]
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

    // === Import Resolution (delegates to TypeScript) ===

    fn lang_key(&self) -> &'static str { "js" }

    fn resolve_local_import(
        &self,
        module: &str,
        current_file: &Path,
        project_root: &Path,
    ) -> Option<PathBuf> {
        crate::TypeScript.resolve_local_import(module, current_file, project_root)
    }

    fn resolve_external_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
        crate::TypeScript.resolve_external_import(import_name, project_root)
    }

    fn get_version(&self, project_root: &Path) -> Option<String> {
        crate::TypeScript.get_version(project_root)
    }

    fn find_package_cache(&self, project_root: &Path) -> Option<PathBuf> {
        crate::TypeScript.find_package_cache(project_root)
    }

    fn indexable_extensions(&self) -> &'static [&'static str] {
        &["tsx", "ts", "js"]
    }
}
