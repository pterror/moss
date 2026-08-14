//! Common Lisp language support.

use crate::traits::{ImportSpec, ModuleId, ModuleResolver, Resolution, ResolverConfig};
use crate::{ContainerBody, Import, Language, LanguageSymbols};
use std::path::Path;
use tree_sitter::Node;

/// Common Lisp language support.
pub struct CommonLisp;

impl Language for CommonLisp {
    fn name(&self) -> &'static str {
        "Common Lisp"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["lisp", "lsp", "cl", "asd"]
    }
    fn grammar_name(&self) -> &'static str {
        "commonlisp"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        if node.kind() != "list_lit" {
            return Vec::new();
        }

        let text = &content[node.byte_range()];
        let line = node.start_position().row + 1;

        for prefix in &["(require ", "(use-package ", "(ql:quickload "] {
            if let Some(rest) = text.strip_prefix(prefix) {
                let module = rest
                    .split(|c: char| c.is_whitespace() || c == ')')
                    .next()
                    .map(|s| s.trim_matches(|c| c == '\'' || c == ':' || c == '"'))
                    .unwrap_or("")
                    .to_string();

                if !module.is_empty() {
                    return vec![Import {
                        module,
                        names: Vec::new(),
                        alias: None,
                        is_wildcard: false,
                        is_relative: false,
                        line,
                    }];
                }
            }
        }

        Vec::new()
    }

    fn format_import(&self, import: &Import, names: Option<&[&str]>) -> String {
        // Common Lisp: (use-package :package) or (use-package :package (:import-from #:a #:b))
        let names_to_use: Vec<&str> = names
            .map(|n| n.to_vec())
            .unwrap_or_else(|| import.names.iter().map(|s| s.as_str()).collect());
        if names_to_use.is_empty() {
            format!("(use-package :{})", import.module)
        } else {
            let symbols: Vec<String> = names_to_use.iter().map(|n| format!("#:{}", n)).collect();
            format!(
                "(use-package :{} (:import-from {}))",
                import.module,
                symbols.join(" ")
            )
        }
    }

    fn is_test_symbol(&self, symbol: &crate::Symbol) -> bool {
        let name = symbol.name.as_str();
        match symbol.kind {
            crate::SymbolKind::Function | crate::SymbolKind::Method => name.starts_with("test_"),
            crate::SymbolKind::Module => name == "tests" || name == "test",
            _ => false,
        }
    }

    fn container_body<'a>(&self, node: &'a Node<'a>) -> Option<Node<'a>> {
        // list/list_lit is itself "( ... )" — use node directly for paren analysis
        Some(*node)
    }
    fn analyze_container_body(
        &self,
        body_node: &Node,
        content: &str,
        inner_indent: &str,
    ) -> Option<ContainerBody> {
        crate::body::analyze_paren_body(body_node, content, inner_indent)
    }

    fn node_name<'a>(&self, node: &Node, content: &'a str) -> Option<&'a str> {
        // Previously always returned `None` — this is the function the actual
        // symbol-extraction pipeline uses for a definition's name (not the
        // tags query's own @name capture text; see
        // `normalize_facts::extract::build_symbol_from_def`), so EVERY Common
        // Lisp definition was silently dropped from `normalize view`/symbol
        // extraction regardless of how correct commonlisp.tags.scm was.
        // Verified live via `normalize view <file>.lisp` returning zero
        // symbols for a file with 9+ definitions, before and after this fix.
        //
        // Two structurally different definition shapes exist in this grammar
        // (confirmed via node-types.json + `normalize syntax ast`):
        // - `defun`/`defgeneric`/`defmethod`/`defmacro` all parse as a single
        //   unified `defun` grammar node with a `defun_header` child whose
        //   `function_name:` field holds the name (a `sym_lit`, or a
        //   `list_lit` for setf-expander forms like `(defun (setf foo) ...)`
        //   — the pre-existing tags query's `(defun_header function_name:
        //   (sym_lit) @name)` pattern already only handled the sym_lit case,
        //   silently dropping setf-form names, but that only affected the
        //   query capture's OWN text, not this always-None node_name bug).
        // - `defclass`/`defstruct`/`defpackage`/`deftype`/`defconstant`/
        //   `defparameter` all still use plain `list_lit` with a leading
        //   `sym_lit` keyword followed by the name (a `sym_lit` for most, but
        //   a `kwd_lit` for `defpackage`, e.g. `(defpackage :my-pkg ...)`).
        if node.kind() == "defun" {
            let mut cursor = node.walk();
            let header = node
                .children(&mut cursor)
                .find(|child| child.kind() == "defun_header")?;
            let name = header.child_by_field_name("function_name")?;
            return Some(&content[name.byte_range()]);
        }
        if node.kind() == "list_lit" {
            let mut cursor = node.walk();
            let mut seen_keyword = false;
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "sym_lit" if !seen_keyword => seen_keyword = true,
                    "sym_lit" | "kwd_lit" if seen_keyword => {
                        return Some(&content[child.byte_range()]);
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn module_resolver(&self) -> Option<&dyn ModuleResolver> {
        static RESOLVER: CommonLispModuleResolver = CommonLispModuleResolver;
        Some(&RESOLVER)
    }
}

impl LanguageSymbols for CommonLisp {}

// =============================================================================
// Common Lisp Module Resolver
// =============================================================================

/// Module resolver for Common Lisp (ASDF/Quicklisp conventions).
///
/// Conservative: looks for `<name>.lisp`, `<name>.cl`, or `<name>.asd`
/// in the workspace root and immediate subdirectories.
pub struct CommonLispModuleResolver;

impl ModuleResolver for CommonLispModuleResolver {
    fn workspace_config(&self, root: &Path) -> ResolverConfig {
        ResolverConfig {
            workspace_root: root.to_path_buf(),
            path_mappings: Vec::new(),
            search_roots: vec![root.to_path_buf()],
        }
    }

    fn module_of_file(&self, _root: &Path, file: &Path, _cfg: &ResolverConfig) -> Vec<ModuleId> {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "lisp" && ext != "cl" && ext != "lsp" && ext != "asd" {
            return Vec::new();
        }
        if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
            return vec![ModuleId {
                canonical_path: stem.to_string(),
            }];
        }
        Vec::new()
    }

    fn resolve(&self, from_file: &Path, spec: &ImportSpec, cfg: &ResolverConfig) -> Resolution {
        let ext = from_file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "lisp" && ext != "cl" && ext != "lsp" && ext != "asd" {
            return Resolution::NotApplicable;
        }
        let raw = &spec.raw;
        let exported_name = raw.rsplit('/').next().unwrap_or(raw).to_string();

        for ext_try in &["lisp", "cl", "lsp", "asd"] {
            // Try directly in workspace root
            let candidate = cfg.workspace_root.join(format!("{}.{}", raw, ext_try));
            if candidate.exists() {
                return Resolution::Resolved(candidate, exported_name.clone());
            }
            // Try in a subdirectory named after the system
            let sub_candidate = cfg
                .workspace_root
                .join(raw)
                .join(format!("{}.{}", raw, ext_try));
            if sub_candidate.exists() {
                return Resolution::Resolved(sub_candidate, exported_name.clone());
            }
        }
        Resolution::NotFound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            // Loop-related clauses. `loop_macro` itself is referenced by
            // commonlisp.complexity.scm's `(loop_macro) @complexity`/
            // `@nesting` patterns, but this audit (validate_unused_kinds_audit)
            // only scans tags.scm for usage, not complexity.scm/cfg.scm — so
            // it stays documented here as "unused" from tags.scm's
            // perspective, matching the audit's own scope.
            "accumulation_clause", "condition_clause", "do_clause", "for_clause",
            "for_clause_word", "loop_clause", "loop_macro", "repeat_clause",
            "termination_clause", "while_clause", "with_clause",
            // Format string specifiers
            "format_directive_type", "format_modifiers", "format_prefix_parameters",
            "format_specifier",
            // Comments
            "block_comment",
        ];
        validate_unused_kinds_audit(&CommonLisp, documented_unused)
            .expect("Common Lisp unused node kinds audit failed");
    }
}
