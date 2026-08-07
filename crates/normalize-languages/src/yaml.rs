//! YAML language support.

use crate::{Language, LanguageSymbols};
use tree_sitter::Node;

/// YAML language support.
pub struct Yaml;

impl Language for Yaml {
    fn name(&self) -> &'static str {
        "YAML"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["yaml", "yml"]
    }
    fn grammar_name(&self) -> &'static str {
        "yaml"
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
        // Pairs whose value nests a mapping (block or flow) are containers.
        // Both block_mapping_pair (`key: value`) and flow_pair
        // (`{key: value}`) can nest either kind of mapping — e.g. a
        // block-style key can have an inline flow-mapping value
        // (`flow_map: {a: 1, b: 2}`), and a flow-style pair can nest another
        // flow mapping (`outer: {inner: {a: 1}}`). Verified via `normalize
        // syntax query` against real parse output.
        if matches!(node.kind(), "block_mapping_pair" | "flow_pair")
            && let Some(value) = node.child_by_field_name("value")
            && find_mapping_container(value).is_some()
        {
            return crate::SymbolKind::Module;
        }
        tag_kind
    }

    fn node_name<'a>(&self, node: &Node, content: &'a str) -> Option<&'a str> {
        if matches!(node.kind(), "block_mapping_pair" | "flow_pair")
            && let Some(key) = node.child_by_field_name("key")
        {
            return find_scalar_text(key, content);
        }
        None
    }

    fn container_body<'a>(&self, node: &'a Node<'a>) -> Option<Node<'a>> {
        if matches!(node.kind(), "block_mapping_pair" | "flow_pair")
            && let Some(value) = node.child_by_field_name("value")
        {
            return find_mapping_container(value);
        }
        None
    }

    fn build_signature(&self, node: &Node, content: &str) -> String {
        if let Some(key) = self.node_name(node, content) {
            if let Some(value) = node.child_by_field_name("value") {
                if value.kind() == "block_node" {
                    return format!("{}:", key);
                }
                let val_text = content[value.byte_range()].trim();
                if val_text.len() > 40 {
                    return format!("{}: {}…", key, &val_text[..37]);
                }
                return format!("{}: {}", key, val_text);
            }
            return key.to_string();
        }
        content[node.byte_range()]
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

impl LanguageSymbols for Yaml {}

/// Find the nested `block_mapping`/`flow_mapping` a pair's `value` node
/// (a `block_node` or `flow_node`) wraps, if any — the container to recurse
/// into for nested symbols. Returns `None` for scalar/sequence/alias values.
fn find_mapping_container(node: Node) -> Option<Node> {
    if matches!(node.kind(), "block_node" | "flow_node") {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "block_mapping" | "flow_mapping") {
                return Some(child);
            }
        }
    }
    None
}

/// Walk into nested scalar nodes to find the text content.
fn find_scalar_text<'a>(node: Node, content: &'a str) -> Option<&'a str> {
    let kind = node.kind();
    if kind == "string_scalar" || kind == "string_content" {
        return Some(&content[node.byte_range()]);
    }
    if kind == "double_quote_scalar" || kind == "single_quote_scalar" {
        // Neither variant tokenizes its inner text as a distinct child node
        // (only escape sequences get their own node) — the surrounding
        // quote characters are the node's own first/last byte, so trimming
        // them is the only way to get the unquoted text. Verified via
        // `normalize syntax ast` against real parse output.
        let range = node.byte_range();
        if range.end.saturating_sub(range.start) >= 2 {
            return Some(&content[range.start + 1..range.end - 1]);
        }
        return Some(&content[range]);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(text) = find_scalar_text(child, content) {
            return Some(text);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GrammarLoader;
    use crate::validate_unused_kinds_audit;
    use tree_sitter::Parser;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            "block_node", "block_scalar",
            "block_sequence", "block_sequence_item",
            // structural node, not extracted as symbols
            "block_mapping",
        ];
        validate_unused_kinds_audit(&Yaml, documented_unused)
            .expect("YAML unused node kinds audit failed");
    }

    struct ParseResult {
        tree: tree_sitter::Tree,
        #[allow(dead_code)]
        loader: GrammarLoader,
    }

    fn parse_yaml(content: &str) -> ParseResult {
        let loader = GrammarLoader::new();
        let Ok(language) = loader.get("yaml") else {
            // Grammar `.so` not built in this environment — same skip
            // convention as `validate_unused_kinds_audit`.
            panic!("yaml grammar not found; run `cargo xtask build-grammars`");
        };
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        ParseResult {
            tree: parser.parse(content, None).unwrap(),
            loader,
        }
    }

    /// Depth-first find the first node of `kind` in the tree.
    fn find_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_kind(child, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn node_name_extracts_quoted_block_keys() {
        let content = "\"double quoted key\": v1\n'single quoted key': v2\nplain_key: v3\n";
        let result = parse_yaml(content);
        let root = result.tree.root_node();
        let support = Yaml;

        fn collect<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
            if node.kind() == "block_mapping_pair" {
                out.push(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect(child, out);
            }
        }
        let mut pairs = Vec::new();
        collect(root, &mut pairs);
        assert_eq!(pairs.len(), 3, "expected 3 block_mapping_pair nodes");
        let names: Vec<&str> = pairs
            .iter()
            .filter_map(|p| support.node_name(p, content))
            .collect();
        assert_eq!(
            names,
            vec!["double quoted key", "single quoted key", "plain_key"]
        );
    }

    #[test]
    fn node_name_extracts_flow_pair_keys() {
        let content = "m: {plain: 1, \"double quoted\": 2, 'single quoted': 3}\n";
        let result = parse_yaml(content);
        let root = result.tree.root_node();
        let support = Yaml;

        fn collect<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
            if node.kind() == "flow_pair" {
                out.push(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect(child, out);
            }
        }
        let mut pairs = Vec::new();
        collect(root, &mut pairs);
        assert_eq!(pairs.len(), 3, "expected 3 flow_pair nodes");
        let names: Vec<&str> = pairs
            .iter()
            .filter_map(|p| support.node_name(p, content))
            .collect();
        assert_eq!(names, vec!["plain", "double quoted", "single quoted"]);
    }

    #[test]
    fn refine_kind_treats_inline_flow_mapping_value_as_container() {
        // A block-style key whose value is an inline flow mapping
        // (`flow_map: {a: 1}`) must be classified as a container (Module),
        // matching a block-style nested block_mapping value.
        let content = "flow_map: {a: 1, b: 2}\n";
        let result = parse_yaml(content);
        let root = result.tree.root_node();
        let support = Yaml;

        let pair = find_kind(root, "block_mapping_pair").expect("block_mapping_pair");
        let kind = support.refine_kind(&pair, content, crate::SymbolKind::Variable);
        assert_eq!(kind, crate::SymbolKind::Module);

        let body = support.container_body(&pair).expect("container_body");
        assert_eq!(body.kind(), "flow_mapping");
    }

    #[test]
    fn refine_kind_treats_nested_flow_pair_value_as_container() {
        // A flow-style pair whose value is itself a flow mapping
        // (`outer: {inner: {a: 1}}`) must also be classified as a container.
        let content = "top: {outer: {inner: 1}}\n";
        let result = parse_yaml(content);
        let root = result.tree.root_node();
        let support = Yaml;

        // `outer` is the flow_pair whose value nests another flow_mapping.
        fn collect<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
            if node.kind() == "flow_pair" {
                out.push(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect(child, out);
            }
        }
        let mut pairs = Vec::new();
        collect(root, &mut pairs);
        let outer = pairs
            .iter()
            .find(|p| support.node_name(p, content) == Some("outer"))
            .expect("outer flow_pair");
        let kind = support.refine_kind(outer, content, crate::SymbolKind::Variable);
        assert_eq!(kind, crate::SymbolKind::Module);

        let inner = pairs
            .iter()
            .find(|p| support.node_name(p, content) == Some("inner"))
            .expect("inner flow_pair");
        let kind = support.refine_kind(inner, content, crate::SymbolKind::Variable);
        assert_eq!(
            kind,
            crate::SymbolKind::Variable,
            "scalar-valued pair must not be reclassified as a container"
        );
    }
}
