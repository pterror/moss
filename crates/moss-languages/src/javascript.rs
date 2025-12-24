//! JavaScript language support.

use std::path::{Path, PathBuf};
use crate::{Export, Import, LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use crate::external_packages::{self, ResolvedPackage};
use moss_core::tree_sitter::Node;

/// JavaScript language support.
pub struct JavaScript;

impl LanguageSupport for JavaScript {
    fn name(&self) -> &'static str { "JavaScript" }
    fn extensions(&self) -> &'static [&'static str] { &["js", "mjs", "cjs", "jsx"] }
    fn grammar_name(&self) -> &'static str { "javascript" }

    fn container_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "class"]
    }

    fn function_kinds(&self) -> &'static [&'static str] {
        &["function_declaration", "method_definition", "generator_function_declaration"]
    }

    fn type_kinds(&self) -> &'static [&'static str] {
        &["class_declaration"]
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
        &[
            "if_statement", "for_statement", "for_in_statement", "while_statement",
            "do_statement", "switch_case", "catch_clause", "ternary_expression",
            "binary_expression", // for && and ||
        ]
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

        let signature = if node.kind() == "method_definition" {
            format!("{}{}", name, params)
        } else {
            format!("function {}{}", name, params)
        };

        Some(Symbol {
            name: name.to_string(),
            kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
            signature,
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

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        if node.kind() != "import_statement" {
            return Vec::new();
        }

        let line = node.start_position().row + 1;
        let mut module = String::new();
        let mut names = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "string" | "string_fragment" => {
                    let text = &content[child.byte_range()];
                    module = text.trim_matches(|c| c == '"' || c == '\'').to_string();
                }
                "import_clause" => {
                    Self::collect_import_names(&child, content, &mut names);
                }
                _ => {}
            }
        }

        if module.is_empty() {
            return Vec::new();
        }

        vec![Import {
            module: module.clone(),
            names,
            alias: None,
            is_wildcard: false,
            is_relative: module.starts_with('.'),
            line,
        }]
    }

    fn extract_public_symbols(&self, node: &Node, content: &str) -> Vec<Export> {
        if node.kind() != "export_statement" {
            return Vec::new();
        }

        let line = node.start_position().row + 1;
        let mut exports = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" | "generator_function_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        exports.push(Export {
                            name: content[name_node.byte_range()].to_string(),
                            kind: SymbolKind::Function,
                            line,
                        });
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        exports.push(Export {
                            name: content[name_node.byte_range()].to_string(),
                            kind: SymbolKind::Class,
                            line,
                        });
                    }
                }
                "lexical_declaration" => {
                    // export const foo = ...
                    let mut decl_cursor = child.walk();
                    for decl_child in child.children(&mut decl_cursor) {
                        if decl_child.kind() == "variable_declarator" {
                            if let Some(name_node) = decl_child.child_by_field_name("name") {
                                exports.push(Export {
                                    name: content[name_node.byte_range()].to_string(),
                                    kind: SymbolKind::Variable,
                                    line,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        exports
    }

    // === Import Resolution ===

    fn lang_key(&self) -> &'static str { "js" }

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
        let extensions = ["js", "jsx", "mjs", "cjs"];

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
        // Skip relative imports
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
        &["js", "mjs", "cjs"]
    }
}

impl JavaScript {
    fn collect_import_names(import_clause: &Node, content: &str, names: &mut Vec<String>) {
        let mut cursor = import_clause.walk();
        for child in import_clause.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Default import: import foo from './module'
                    names.push(content[child.byte_range()].to_string());
                }
                "named_imports" => {
                    // { foo, bar }
                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        if inner.kind() == "import_specifier" {
                            if let Some(name_node) = inner.child_by_field_name("name") {
                                names.push(content[name_node.byte_range()].to_string());
                            }
                        }
                    }
                }
                "namespace_import" => {
                    // import * as foo
                    if let Some(name_node) = child.child_by_field_name("name") {
                        names.push(format!("* as {}", &content[name_node.byte_range()]));
                    }
                }
                _ => {}
            }
        }
    }
}
