//! Shared ECMAScript (JavaScript/TypeScript) support functions.
//!
//! This module contains common logic shared between JavaScript, TypeScript, and TSX.
//! Each language struct delegates to these functions for DRY implementation.

use std::path::{Path, PathBuf};
use crate::{Export, Import, Symbol, SymbolKind, Visibility};
use crate::external_packages::{self, ResolvedPackage};
use moss_core::tree_sitter::Node;

// ============================================================================
// Node kind constants
// ============================================================================

pub const CONTAINER_KINDS: &[&str] = &["class_declaration", "class"];

pub const JS_FUNCTION_KINDS: &[&str] = &["function_declaration", "method_definition", "generator_function_declaration"];
pub const TS_FUNCTION_KINDS: &[&str] = &["function_declaration", "method_definition"];

pub const JS_TYPE_KINDS: &[&str] = &["class_declaration"];
pub const TS_TYPE_KINDS: &[&str] = &["class_declaration", "interface_declaration", "type_alias_declaration", "enum_declaration"];

pub const IMPORT_KINDS: &[&str] = &["import_statement"];
pub const PUBLIC_SYMBOL_KINDS: &[&str] = &["export_statement"];

pub const SCOPE_CREATING_KINDS: &[&str] = &[
    "for_statement", "for_in_statement", "while_statement", "do_statement",
    "try_statement", "catch_clause", "switch_statement", "arrow_function"
];

pub const CONTROL_FLOW_KINDS: &[&str] = &[
    "if_statement", "for_statement", "for_in_statement", "while_statement",
    "do_statement", "switch_statement", "try_statement", "return_statement",
    "break_statement", "continue_statement", "throw_statement"
];

pub const COMPLEXITY_NODES: &[&str] = &[
    "if_statement", "for_statement", "for_in_statement", "while_statement",
    "do_statement", "switch_case", "catch_clause", "ternary_expression",
    "binary_expression"
];

pub const NESTING_NODES: &[&str] = &[
    "if_statement", "for_statement", "for_in_statement", "while_statement",
    "do_statement", "switch_statement", "try_statement", "function_declaration",
    "method_definition", "class_declaration"
];

// ============================================================================
// Symbol extraction
// ============================================================================

/// Extract a function/method symbol from a node.
pub fn extract_function(node: &Node, content: &str, in_container: bool, name: &str) -> Symbol {
    let params = node
        .child_by_field_name("parameters")
        .map(|p| content[p.byte_range()].to_string())
        .unwrap_or_else(|| "()".to_string());

    let signature = if node.kind() == "method_definition" {
        format!("{}{}", name, params)
    } else {
        format!("function {}{}", name, params)
    };

    Symbol {
        name: name.to_string(),
        kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
        signature,
        docstring: None,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        visibility: Visibility::Public,
        children: Vec::new(),
    }
}

/// Extract a class container symbol from a node.
pub fn extract_container(node: &Node, name: &str) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: SymbolKind::Class,
        signature: format!("class {}", name),
        docstring: None,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        visibility: Visibility::Public,
        children: Vec::new(),
    }
}

/// Extract a TypeScript type symbol (interface, type alias, enum).
pub fn extract_type(node: &Node, name: &str) -> Option<Symbol> {
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

// ============================================================================
// Import/Export extraction
// ============================================================================

/// Extract imports from an import_statement node.
pub fn extract_imports(node: &Node, content: &str) -> Vec<Import> {
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
                collect_import_names(&child, content, &mut names);
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

/// Extract exports from an export_statement node.
pub fn extract_public_symbols(node: &Node, content: &str) -> Vec<Export> {
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

// ============================================================================
// Import resolution
// ============================================================================

/// Resolve a relative import to a local file path.
pub fn resolve_local_import(
    module: &str,
    current_file: &Path,
    extensions: &[&str],
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

    // First try exact path (might already have extension)
    if target.exists() && target.is_file() {
        return Some(target);
    }

    // Try adding extensions
    for ext in extensions {
        let with_ext = target.with_extension(ext);
        if with_ext.exists() && with_ext.is_file() {
            return Some(with_ext);
        }
    }

    // Try index files in directory
    if target.is_dir() {
        for ext in extensions {
            let index = target.join(format!("index.{}", ext));
            if index.exists() && index.is_file() {
                return Some(index);
            }
        }
    }

    None
}

/// Resolve an external (node_modules) import.
pub fn resolve_external_import(import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
    if import_name.starts_with('.') || import_name.starts_with('/') {
        return None;
    }

    let node_modules = external_packages::find_node_modules(project_root)?;
    external_packages::resolve_node_import(import_name, &node_modules)
}

/// Get the Node.js version.
pub fn get_version() -> Option<String> {
    external_packages::get_node_version()
}

/// Find the node_modules directory.
pub fn find_package_cache(project_root: &Path) -> Option<PathBuf> {
    external_packages::find_node_modules(project_root)
}

// Extension preferences for each language variant
pub const JS_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "cjs"];
pub const TS_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "mjs"];
