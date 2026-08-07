//! CSS language support with symbol extraction.
//!
//! CSS symbols: rule_set (selectors → Class), media/supports/keyframes → Module,
//! declarations → Variable. Nested rule_sets inside at-rules become children.

use crate::{Language, LanguageSymbols};
use tree_sitter::Node;

/// CSS language support.
pub struct Css;

impl Language for Css {
    fn name(&self) -> &'static str {
        "CSS"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["css"]
    }
    fn grammar_name(&self) -> &'static str {
        "css"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    fn refine_kind(
        &self,
        node: &Node,
        _content: &str,
        tag_kind: crate::SymbolKind,
    ) -> crate::SymbolKind {
        match node.kind() {
            // At-rules containing blocks are containers
            "media_statement" | "supports_statement" | "keyframes_statement"
            | "scope_statement"
            // Generic at-rule fallback (@font-face, @layer, @property,
            // @container, @page, …) — see css.tags.scm for why this grammar
            // can't distinguish them syntactically. Statement-form at-rules
            // without a block (e.g. `@layer a, b, c;`) also land here; they
            // just never get children via `container_body`.
            | "at_rule" => crate::SymbolKind::Module,
            _ => tag_kind,
        }
    }

    fn node_name<'a>(&self, node: &Node, content: &'a str) -> Option<&'a str> {
        match node.kind() {
            "rule_set" => {
                // Extract the selectors text
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "selectors" {
                        return Some(content[child.byte_range()].trim());
                    }
                }
                None
            }
            "media_statement" => {
                // Extract feature_query or keyword after @media
                extract_at_rule_name(node, content, "@media")
            }
            "supports_statement" => extract_at_rule_name(node, content, "@supports"),
            "scope_statement" => extract_at_rule_name(node, content, "@scope"),
            "at_rule" => extract_generic_at_rule_name(node, content),
            "keyframes_statement" => {
                // keyframes_name child
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "keyframes_name" {
                        return Some(content[child.byte_range()].trim());
                    }
                }
                None
            }
            "declaration" => {
                // property_name child
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "property_name" {
                        return Some(content[child.byte_range()].trim());
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn container_body<'a>(&self, node: &'a Node<'a>) -> Option<Node<'a>> {
        match node.kind() {
            "rule_set"
            | "media_statement"
            | "supports_statement"
            | "keyframes_statement"
            | "scope_statement"
            | "at_rule" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "block" || child.kind() == "keyframe_block_list" {
                        return Some(child);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn build_signature(&self, node: &Node, content: &str) -> String {
        if let Some(name) = self.node_name(node, content) {
            match node.kind() {
                "rule_set" => format!("{} {{ … }}", name),
                "media_statement" => format!("@media {} {{ … }}", name),
                "supports_statement" => format!("@supports {} {{ … }}", name),
                "keyframes_statement" => format!("@keyframes {} {{ … }}", name),
                "scope_statement" => format!("@scope {} {{ … }}", name),
                // `name` already includes the at-keyword (there's no per-rule
                // prefix to add — see `extract_generic_at_rule_name`).
                // Statement-form at-rules (`@layer a, b, c;`) have no block,
                // so only append the `{ … }` placeholder when one exists.
                "at_rule" => {
                    if self.container_body(node).is_some() {
                        format!("{} {{ … }}", name)
                    } else {
                        name.to_string()
                    }
                }
                "declaration" => {
                    // Render the full value (every child after `property_name`
                    // and `:`, up to the trailing `;`), not just the first
                    // value token. Multi-token shorthand values
                    // (`margin: 0 auto;`, `font-weight: 400 700;`,
                    // `src: url(...) format(...), url(...) format(...);`)
                    // previously got silently truncated to their first token
                    // — confirmed via `normalize view` on a real declaration.
                    let mut cursor = node.walk();
                    let mut found_name = false;
                    let mut value_range: Option<(usize, usize)> = None;
                    for child in node.children(&mut cursor) {
                        if child.kind() == "property_name" {
                            found_name = true;
                        } else if found_name && child.kind() != ":" && child.kind() != ";" {
                            value_range = Some((
                                value_range.map_or(child.start_byte(), |(s, _)| s),
                                child.end_byte(),
                            ));
                        }
                    }
                    if let Some((start, end)) = value_range {
                        let val = content[start..end].trim();
                        if val.chars().count() > 40 {
                            let truncated: String = val.chars().take(37).collect();
                            return format!("{}: {}…", name, truncated);
                        }
                        return format!("{}: {}", name, val);
                    }
                    name.to_string()
                }
                _ => name.to_string(),
            }
        } else {
            content[node.byte_range()]
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
    }
}

impl LanguageSymbols for Css {}

/// Extract the text between an at-rule keyword and its block.
fn extract_at_rule_name<'a>(node: &Node, content: &'a str, keyword: &str) -> Option<&'a str> {
    let full = &content[node.byte_range()];
    let after_keyword = full.strip_prefix(keyword)?.trim_start();
    // Take everything up to the opening brace
    let name = after_keyword.split('{').next()?.trim();
    if name.is_empty() {
        return None;
    }
    // Find the offset within the node and return a reference into content
    let start = node.start_byte() + full.find(name)?;
    let end = start + name.len();
    Some(&content[start..end])
}

/// Extract the name of a generic `at_rule` node (@font-face, @layer,
/// @property, @container, @page, and any other at-rule this grammar has no
/// dedicated node type for — see css.tags.scm). Unlike
/// [`extract_at_rule_name`] there's no single fixed keyword to strip since
/// the at-keyword varies per rule; instead this returns everything from the
/// at-keyword itself up to (but not including) the opening `{` of the block
/// or the terminating `;` of a blockless statement form (`@layer a, b, c;`),
/// whichever comes first.
fn extract_generic_at_rule_name<'a>(node: &Node, content: &'a str) -> Option<&'a str> {
    let full = &content[node.byte_range()];
    let brace = full.find('{');
    let semi = full.find(';');
    let stop = match (brace, semi) {
        (Some(b), Some(s)) => b.min(s),
        (Some(b), None) => b,
        (None, Some(s)) => s,
        (None, None) => full.len(),
    };
    let name = full[..stop].trim_end();
    if name.is_empty() {
        return None;
    }
    let start = node.start_byte();
    Some(&content[start..start + name.len()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            "binary_expression", "block", "call_expression", "charset_statement",
            "class_name", "class_selector",
            "function_name",
            "identifier", "import_statement", "important", "important_value",
            "keyframe_block", "keyframe_block_list",
            "namespace_statement", "postcss_statement",
            "pseudo_class_selector",
        ];
        validate_unused_kinds_audit(&Css, documented_unused)
            .expect("CSS unused node kinds audit failed");
    }

    struct ParseResult {
        tree: tree_sitter::Tree,
        #[allow(dead_code)]
        loader: crate::GrammarLoader,
    }

    fn parse_css(content: &str) -> Option<ParseResult> {
        let loader = crate::GrammarLoader::new();
        let language = loader.get("css").ok()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).ok()?;
        Some(ParseResult {
            tree: parser.parse(content, None)?,
            loader,
        })
    }

    fn find_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        node.children(&mut cursor).find(|n| n.kind() == kind)
    }

    /// Regression test for the declaration-value truncation bug: before this
    /// fix, `build_signature` only rendered the *first* value token after
    /// `property_name`, silently dropping the rest of any multi-token
    /// shorthand value (`margin: 0 auto;` rendered as `margin: 0`).
    #[test]
    fn declaration_signature_includes_full_multi_token_value() {
        let Some(parsed) = parse_css(".x { margin: 0 auto; }") else {
            eprintln!("Skipping: css grammar .so not found");
            return;
        };
        let root = parsed.tree.root_node();
        let rule_set = find_kind(&root, "rule_set").expect("rule_set");
        let block = find_kind(&rule_set, "block").expect("block");
        let declaration = find_kind(&block, "declaration").expect("declaration");
        let content = ".x { margin: 0 auto; }";

        let sig = Css.build_signature(&declaration, content);
        assert_eq!(sig, "margin: 0 auto");
    }

    /// @font-face has no dedicated grammar node type — it parses as the
    /// generic `at_rule` fallback (see css.tags.scm). Confirms the name is
    /// the at-keyword plus everything up to the block, and the signature
    /// renders a ` { … }` placeholder since a block is present.
    #[test]
    fn generic_at_rule_name_and_signature_for_font_face() {
        let content = "@font-face { font-family: \"Foo\"; }";
        let Some(parsed) = parse_css(content) else {
            eprintln!("Skipping: css grammar .so not found");
            return;
        };
        let root = parsed.tree.root_node();
        let at_rule = find_kind(&root, "at_rule").expect("at_rule");

        assert_eq!(Css.node_name(&at_rule, content), Some("@font-face"));
        assert_eq!(Css.build_signature(&at_rule, content), "@font-face { … }");
        assert_eq!(
            Css.refine_kind(&at_rule, content, crate::SymbolKind::Module),
            crate::SymbolKind::Module
        );
    }

    /// Blockless at-rule statement form (`@layer a, b;`) must not get a
    /// synthesized ` { … }` suffix — there is no block to summarize.
    #[test]
    fn generic_at_rule_blockless_signature_has_no_block_suffix() {
        let content = "@layer a, b;";
        let Some(parsed) = parse_css(content) else {
            eprintln!("Skipping: css grammar .so not found");
            return;
        };
        let root = parsed.tree.root_node();
        let at_rule = find_kind(&root, "at_rule").expect("at_rule");

        assert_eq!(Css.node_name(&at_rule, content), Some("@layer a, b"));
        assert_eq!(Css.build_signature(&at_rule, content), "@layer a, b");
        assert!(Css.container_body(&at_rule).is_none());
    }

    /// `@scope (.card) to (.content) { ... }` uses its own dedicated
    /// `scope_statement` node type (distinct from the generic `at_rule`
    /// fallback); its name should be the condition only (matching the
    /// existing @media/@supports convention), with the `@scope` prefix
    /// re-added by `build_signature`.
    #[test]
    fn scope_statement_name_and_signature() {
        let content = "@scope (.card) to (.content) { p { color: red; } }";
        let Some(parsed) = parse_css(content) else {
            eprintln!("Skipping: css grammar .so not found");
            return;
        };
        let root = parsed.tree.root_node();
        let scope = find_kind(&root, "scope_statement").expect("scope_statement");

        assert_eq!(
            Css.node_name(&scope, content),
            Some("(.card) to (.content)")
        );
        assert_eq!(
            Css.build_signature(&scope, content),
            "@scope (.card) to (.content) { … }"
        );
        assert!(Css.container_body(&scope).is_some());
    }
}
