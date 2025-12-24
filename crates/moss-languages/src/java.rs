//! Java language support.

use std::path::{Path, PathBuf};
use crate::{Export, LanguageSupport, Symbol, SymbolKind, Visibility, VisibilityMechanism};
use crate::external_packages::{self, ResolvedPackage};
use moss_core::tree_sitter::Node;

/// Java language support.
pub struct Java;

impl LanguageSupport for Java {
    fn name(&self) -> &'static str { "Java" }
    fn extensions(&self) -> &'static [&'static str] { &["java"] }
    fn grammar_name(&self) -> &'static str { "java" }

    fn container_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "interface_declaration", "enum_declaration"]
    }

    fn function_kinds(&self) -> &'static [&'static str] {
        &["method_declaration", "constructor_declaration"]
    }

    fn type_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "interface_declaration", "enum_declaration"]
    }

    fn import_kinds(&self) -> &'static [&'static str] { &["import_declaration"] }

    fn public_symbol_kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "interface_declaration", "enum_declaration", "method_declaration"]
    }

    fn visibility_mechanism(&self) -> VisibilityMechanism {
        VisibilityMechanism::AccessModifier
    }

    fn extract_public_symbols(&self, node: &Node, content: &str) -> Vec<Export> {
        if self.get_visibility(node, content) != Visibility::Public {
            return Vec::new();
        }

        let name = match self.node_name(node, content) {
            Some(n) => n.to_string(),
            None => return Vec::new(),
        };

        let kind = match node.kind() {
            "class_declaration" => SymbolKind::Class,
            "interface_declaration" => SymbolKind::Interface,
            "enum_declaration" => SymbolKind::Enum,
            "method_declaration" | "constructor_declaration" => SymbolKind::Method,
            _ => return Vec::new(),
        };

        vec![Export {
            name,
            kind,
            line: node.start_position().row + 1,
        }]
    }

    fn scope_creating_kinds(&self) -> &'static [&'static str] {
        &["for_statement", "enhanced_for_statement", "while_statement", "do_statement", "try_statement", "catch_clause", "switch_expression", "block"]
    }

    fn control_flow_kinds(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "enhanced_for_statement", "while_statement", "do_statement", "switch_expression", "try_statement", "return_statement", "break_statement", "continue_statement", "throw_statement"]
    }

    fn complexity_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "enhanced_for_statement", "while_statement", "do_statement", "switch_label", "catch_clause", "ternary_expression", "binary_expression"]
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        &["if_statement", "for_statement", "enhanced_for_statement", "while_statement", "do_statement", "switch_expression", "try_statement", "method_declaration", "class_declaration"]
    }

    fn extract_function(&self, node: &Node, content: &str, _in_container: bool) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let params = node.child_by_field_name("parameters")
            .map(|p| content[p.byte_range()].to_string())
            .unwrap_or_else(|| "()".to_string());

        Some(Symbol {
            name: name.to_string(),
            kind: SymbolKind::Method,
            signature: format!("{}{}", name, params),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: self.get_visibility(node, content),
            children: Vec::new(),
        })
    }

    fn extract_container(&self, node: &Node, content: &str) -> Option<Symbol> {
        let name = self.node_name(node, content)?;
        let kind = match node.kind() {
            "interface_declaration" => SymbolKind::Interface,
            "enum_declaration" => SymbolKind::Enum,
            _ => SymbolKind::Class,
        };

        Some(Symbol {
            name: name.to_string(),
            kind,
            signature: format!("{} {}", kind.as_str(), name),
            docstring: None,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            visibility: self.get_visibility(node, content),
            children: Vec::new(),
        })
    }

    fn get_visibility(&self, node: &Node, content: &str) -> Visibility {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let mods = &content[child.byte_range()];
                if mods.contains("private") { return Visibility::Private; }
                if mods.contains("protected") { return Visibility::Protected; }
                // public or no modifier = visible in skeleton
                return Visibility::Public;
            }
        }
        // No modifier = package-private, but still visible for skeleton purposes
        Visibility::Public
    }

    // === Import Resolution ===

    fn lang_key(&self) -> &'static str { "java" }

    fn resolve_local_import(
        &self,
        import: &str,
        current_file: &Path,
        project_root: &Path,
    ) -> Option<PathBuf> {
        // Convert import to path: com.foo.Bar -> com/foo/Bar.java
        let path_part = import.replace('.', "/");

        // Common Java source directories
        let source_dirs = [
            "src/main/java",
            "src/java",
            "src",
            "app/src/main/java", // Android
        ];

        for src_dir in &source_dirs {
            let source_path = project_root.join(src_dir).join(format!("{}.java", path_part));
            if source_path.is_file() {
                return Some(source_path);
            }
        }

        // Also try relative to current file's package structure
        // Find the source root by walking up from current file
        let mut current = current_file.parent()?;
        while current != project_root {
            // Check if this might be a source root
            let potential = current.join(format!("{}.java", path_part));
            if potential.is_file() {
                return Some(potential);
            }
            current = current.parent()?;
        }

        None
    }

    fn resolve_external_import(&self, import_name: &str, _project_root: &Path) -> Option<ResolvedPackage> {
        let maven_repo = external_packages::find_maven_repository();
        let gradle_cache = external_packages::find_gradle_cache();

        external_packages::resolve_java_import(
            import_name,
            maven_repo.as_deref(),
            gradle_cache.as_deref(),
        )
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        external_packages::get_java_version()
    }

    fn find_package_cache(&self, _project_root: &Path) -> Option<PathBuf> {
        external_packages::find_maven_repository()
            .or_else(external_packages::find_gradle_cache)
    }

    fn indexable_extensions(&self) -> &'static [&'static str] {
        &["java"]
    }
}
