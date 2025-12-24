//! Go language support.

use std::path::{Path, PathBuf};
use crate::{Export, Import, LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use crate::external_packages::{self, ResolvedPackage};
use moss_core::tree_sitter::Node;

// ============================================================================
// Go module parsing (inlined from go_mod.rs)
// ============================================================================

/// Information from a go.mod file
#[derive(Debug, Clone)]
struct GoModule {
    /// Module path (e.g., "github.com/user/project")
    path: String,
    /// Go version (e.g., "1.21")
    #[allow(dead_code)]
    go_version: Option<String>,
}

/// Parse a go.mod file to extract module information.
fn parse_go_mod(path: &Path) -> Option<GoModule> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_go_mod_content(&content)
}

/// Parse go.mod content string.
fn parse_go_mod_content(content: &str) -> Option<GoModule> {
    let mut module_path = None;
    let mut go_version = None;

    for line in content.lines() {
        let line = line.trim();

        // module github.com/user/project
        if line.starts_with("module ") {
            module_path = Some(line.trim_start_matches("module ").trim().to_string());
        }

        // go 1.21
        if line.starts_with("go ") {
            go_version = Some(line.trim_start_matches("go ").trim().to_string());
        }
    }

    module_path.map(|path| GoModule { path, go_version })
}

/// Find go.mod by walking up from a directory.
fn find_go_mod(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let go_mod = current.join("go.mod");
        if go_mod.exists() {
            return Some(go_mod);
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Resolve a Go import path to a local directory path.
///
/// Returns the computed path if the import is within the module, None for external imports.
/// Does not check if the path exists - caller should verify.
fn resolve_go_import(import_path: &str, module: &GoModule, project_root: &Path) -> Option<PathBuf> {
    // Check if import is within our module
    if !import_path.starts_with(&module.path) {
        return None; // External import
    }

    // Get the relative path after the module prefix
    let rel_path = import_path.strip_prefix(&module.path)?;
    let rel_path = rel_path.trim_start_matches('/');

    let target = if rel_path.is_empty() {
        project_root.to_path_buf()
    } else {
        project_root.join(rel_path)
    };

    Some(target)
}

// ============================================================================
// Go language support
// ============================================================================

/// Go language support.
pub struct Go;

impl LanguageSupport for Go {
    fn name(&self) -> &'static str { "Go" }
    fn extensions(&self) -> &'static [&'static str] { &["go"] }
    fn grammar_name(&self) -> &'static str { "go" }

    fn container_kinds(&self) -> &'static [&'static str] {
        &[] // Go types don't have children in the tree-sitter sense
    }

    fn function_kinds(&self) -> &'static [&'static str] {
        &["function_declaration", "method_declaration"]
    }

    fn type_kinds(&self) -> &'static [&'static str] {
        &["type_spec"] // The actual type is in type_spec, not type_declaration
    }

    fn import_kinds(&self) -> &'static [&'static str] {
        &["import_declaration"]
    }

    fn public_symbol_kinds(&self) -> &'static [&'static str] {
        &["function_declaration", "method_declaration", "type_spec", "const_spec", "var_spec"]
    }

    fn visibility_mechanism(&self) -> VisibilityMechanism {
        VisibilityMechanism::NamingConvention
    }

    fn scope_creating_kinds(&self) -> &'static [&'static str] {
        &["for_statement", "if_statement", "switch_statement", "select_statement", "block"]
    }

    fn control_flow_kinds(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "switch_statement", "select_statement", "return_statement", "break_statement", "continue_statement", "goto_statement", "defer_statement"]
    }

    fn complexity_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "switch_statement", "select_statement", "expression_case", "type_case", "communication_case", "binary_expression"]
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "switch_statement", "select_statement", "function_declaration", "method_declaration"]
    }

    fn extract_function(&self, node: &Node, content: &str, in_container: bool) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let params = node.child_by_field_name("parameters")
            .map(|p| content[p.byte_range()].to_string())
            .unwrap_or_else(|| "()".to_string());

        Some(Symbol {
            name: name.to_string(),
            kind: if in_container { SymbolKind::Method } else { SymbolKind::Function },
            signature: format!("func {}{}", name, params),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                Visibility::Public
            } else {
                Visibility::Private
            },
            children: Vec::new(),
        })
    }

    fn extract_container(&self, _node: &Node, _content: &str) -> Option<Symbol> {
        None // Go types are extracted via extract_type
    }

    fn extract_type(&self, node: &Node, content: &str) -> Option<Symbol> {
        // Go type_spec: name field + type field (struct_type, interface_type, etc.)
        let name_node = node.child_by_field_name("name")?;
        let name = content[name_node.byte_range()].to_string();

        let type_node = node.child_by_field_name("type");
        let type_kind = type_node.map(|t| t.kind()).unwrap_or("");

        let kind = match type_kind {
            "struct_type" => SymbolKind::Struct,
            "interface_type" => SymbolKind::Interface,
            _ => SymbolKind::Type,
        };

        Some(Symbol {
            name: name.clone(),
            kind,
            signature: format!("type {}", name),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                Visibility::Public
            } else {
                Visibility::Private
            },
            children: Vec::new(),
        })
    }

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        if node.kind() != "import_declaration" {
            return Vec::new();
        }

        let mut imports = Vec::new();
        let line = node.start_position().row + 1;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "import_spec" => {
                    // import "path" or import alias "path"
                    if let Some(imp) = Self::parse_import_spec(&child, content, line) {
                        imports.push(imp);
                    }
                }
                "import_spec_list" => {
                    // Grouped imports
                    let mut list_cursor = child.walk();
                    for spec in child.children(&mut list_cursor) {
                        if spec.kind() == "import_spec" {
                            if let Some(imp) = Self::parse_import_spec(&spec, content, line) {
                                imports.push(imp);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        imports
    }

    fn extract_public_symbols(&self, node: &Node, content: &str) -> Vec<Export> {
        // Go exports are determined by uppercase first letter
        let name = match self.node_name(node, content) {
            Some(n) if n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) => n,
            _ => return Vec::new(),
        };

        let line = node.start_position().row + 1;
        let kind = match node.kind() {
            "function_declaration" => SymbolKind::Function,
            "method_declaration" => SymbolKind::Method,
            "type_spec" => SymbolKind::Type,
            "const_spec" => SymbolKind::Constant,
            "var_spec" => SymbolKind::Variable,
            _ => return Vec::new(),
        };

        vec![Export {
            name: name.to_string(),
            kind,
            line,
        }]
    }

    fn is_public(&self, node: &Node, content: &str) -> bool {
        self.node_name(node, content)
            .and_then(|n| n.chars().next())
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    // === Import Resolution ===

    fn lang_key(&self) -> &'static str { "go" }

    fn resolve_local_import(
        &self,
        import_path: &str,
        current_file: &Path,
        _project_root: &Path,
    ) -> Option<PathBuf> {
        // Find go.mod to understand module boundaries
        if let Some(go_mod_path) = find_go_mod(current_file) {
            if let Some(module) = parse_go_mod(&go_mod_path) {
                // Try local resolution within the module
                let module_root = go_mod_path.parent()?;
                if let Some(local_path) = resolve_go_import(import_path, &module, module_root) {
                    if local_path.exists() && local_path.is_dir() {
                        return Some(local_path);
                    }
                }
            }
        }
        None
    }

    fn resolve_external_import(&self, import_name: &str, _project_root: &Path) -> Option<ResolvedPackage> {
        // Check stdlib first
        if external_packages::is_go_stdlib_import(import_name) {
            if let Some(stdlib) = external_packages::find_go_stdlib() {
                if let Some(pkg) = external_packages::resolve_go_stdlib_import(import_name, &stdlib) {
                    return Some(pkg);
                }
            }
        }

        // Then mod cache
        if let Some(mod_cache) = external_packages::find_go_mod_cache() {
            return external_packages::resolve_go_import(import_name, &mod_cache);
        }

        None
    }

    fn is_stdlib_import(&self, import_name: &str, _project_root: &Path) -> bool {
        external_packages::is_go_stdlib_import(import_name)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        external_packages::get_go_version()
    }

    fn find_package_cache(&self, _project_root: &Path) -> Option<PathBuf> {
        external_packages::find_go_mod_cache()
    }

    fn indexable_extensions(&self) -> &'static [&'static str] {
        &["go"]
    }
}

impl Go {
    fn parse_import_spec(node: &Node, content: &str, line: usize) -> Option<Import> {
        let mut path = String::new();
        let mut alias = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "interpreted_string_literal" => {
                    let text = &content[child.byte_range()];
                    path = text.trim_matches('"').to_string();
                }
                "package_identifier" | "blank_identifier" | "dot" => {
                    alias = Some(content[child.byte_range()].to_string());
                }
                _ => {}
            }
        }

        if path.is_empty() {
            return None;
        }

        let is_wildcard = alias.as_deref() == Some(".");
        Some(Import {
            module: path,
            names: Vec::new(),
            alias,
            is_wildcard,
            is_relative: false, // Go doesn't have relative imports in the traditional sense
            line,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_go_mod() {
        let content = r#"
module github.com/user/project

go 1.21

require (
    github.com/pkg/errors v0.9.1
    golang.org/x/sync v0.3.0
)
"#;
        let module = parse_go_mod_content(content).unwrap();
        assert_eq!(module.path, "github.com/user/project");
        assert_eq!(module.go_version, Some("1.21".to_string()));
    }

    #[test]
    fn test_resolve_internal_import() {
        let module = GoModule {
            path: "github.com/user/project".to_string(),
            go_version: Some("1.21".to_string()),
        };

        // Internal import
        let result = resolve_go_import(
            "github.com/user/project/pkg/utils",
            &module,
            Path::new("/fake/root"),
        );
        assert_eq!(result, Some(PathBuf::from("/fake/root/pkg/utils")));

        // External import
        let result = resolve_go_import("github.com/other/lib", &module, Path::new("/fake/root"));
        assert!(result.is_none());
    }
}
