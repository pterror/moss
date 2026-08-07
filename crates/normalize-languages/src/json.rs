//! JSON language support.

use crate::{Language, LanguageSymbols};
use tree_sitter::Node;

/// JSON language support.
pub struct Json;

impl Language for Json {
    fn name(&self) -> &'static str {
        "JSON"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["json", "jsonc"]
    }
    fn grammar_name(&self) -> &'static str {
        "json"
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
        // Pairs with object values act as containers (sections/namespaces)
        if node.kind() == "pair"
            && let Some(value) = node.child_by_field_name("value")
            && value.kind() == "object"
        {
            return crate::SymbolKind::Module;
        }
        tag_kind
    }

    fn node_name<'a>(&self, node: &Node, content: &'a str) -> Option<&'a str> {
        // For pair nodes, extract the key string content by slicing between
        // the key `string` node's own opening/closing quotes rather than
        // reading its `string_content` child(ren) directly. Iterating
        // children breaks on two grammar shapes (verified via
        // `normalize syntax ast`):
        //   - Empty-string keys (`"": value`) parse as a `string` node with
        //     NO `string_content` child at all -- the key is legitimately
        //     "", not absent, but a child-search would find nothing and
        //     return None, silently dropping the pair from extraction.
        //   - Keys containing an escape sequence (`"a\nb"`) parse as a
        //     `string` node with *multiple* `string_content` children (one
        //     per literal run around each `escape_sequence`) -- returning
        //     just the first child would silently truncate the name to "a".
        // Byte-slicing between the node's own start/end quotes handles both
        // uniformly and matches what json.tags.scm's `@name` capture (the
        // whole `string` node) actually matches on.
        if node.kind() == "pair"
            && let Some(key) = node.child_by_field_name("key")
            && key.kind() == "string"
        {
            let start = key.start_byte() + 1;
            let end = key.end_byte().saturating_sub(1);
            if start <= end && end <= content.len() {
                return Some(&content[start..end]);
            }
        }
        None
    }

    fn container_body<'a>(&self, node: &'a Node<'a>) -> Option<Node<'a>> {
        if node.kind() == "pair"
            && let Some(value) = node.child_by_field_name("value")
            && value.kind() == "object"
        {
            return Some(value);
        }
        None
    }

    fn build_signature(&self, node: &Node, content: &str) -> String {
        if node.kind() == "pair"
            && let Some(key) = self.node_name(node, content)
        {
            if let Some(value) = node.child_by_field_name("value") {
                return match value.kind() {
                    "object" => format!("{}: {{}}", key),
                    "array" => format!("{}: []", key),
                    _ => {
                        let val_text = &content[value.byte_range()];
                        if val_text.len() > 40 {
                            let mut end = 37;
                            while end > 0 && !val_text.is_char_boundary(end) {
                                end -= 1;
                            }
                            format!("{}: {}…", key, &val_text[..end])
                        } else {
                            format!("{}: {}", key, val_text)
                        }
                    }
                };
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

impl LanguageSymbols for Json {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        // JSON has no "interesting" unused kinds matching our patterns
        let documented_unused: &[&str] = &[];
        validate_unused_kinds_audit(&Json, documented_unused)
            .expect("JSON unused node kinds audit failed");
    }
}
