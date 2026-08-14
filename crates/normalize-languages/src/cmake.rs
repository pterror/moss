//! CMake language support.

use crate::{ContainerBody, Import, Language, LanguageSymbols};
use tree_sitter::Node;

/// CMake language support.
pub struct CMake;

impl Language for CMake {
    fn name(&self) -> &'static str {
        "CMake"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["cmake"]
    }
    fn grammar_name(&self) -> &'static str {
        "cmake"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        if node.kind() != "normal_command" {
            return Vec::new();
        }

        let text = &content[node.byte_range()];
        let line = node.start_position().row + 1;

        // include(file), find_package(pkg)
        if text.starts_with("include(") || text.starts_with("find_package(") {
            let inner = text
                .split('(')
                .nth(1)
                .and_then(|s| s.split(')').next())
                .map(|s| s.trim().to_string());

            if let Some(module) = inner {
                return vec![Import {
                    module,
                    names: Vec::new(),
                    alias: None,
                    is_wildcard: false,
                    is_relative: text.starts_with("include("),
                    line,
                }];
            }
        }

        Vec::new()
    }

    fn format_import(&self, import: &Import, _names: Option<&[&str]>) -> String {
        // CMake: include(file) or find_package(pkg)
        format!("include({})", import.module)
    }

    fn container_body<'a>(&self, node: &'a Node<'a>) -> Option<Node<'a>> {
        // CMake function_def/macro_def: body is an unnamed child of kind "body"
        let mut c = node.walk();
        node.children(&mut c).find(|&child| child.kind() == "body")
    }

    fn analyze_container_body(
        &self,
        body_node: &Node,
        content: &str,
        inner_indent: &str,
    ) -> Option<ContainerBody> {
        // body node: "\n  message(...)\n  set(...)" — raw statements after opening newline
        crate::body::analyze_end_body(body_node, content, inner_indent)
    }

    fn node_name<'a>(&self, node: &Node, content: &'a str) -> Option<&'a str> {
        // `node` is the @definition.function capture: a `function_def`/`macro_def`
        // node whose DIRECT children are only `function_command`/`macro_command`,
        // `body`, `endfunction_command`/`endmacro_command` (confirmed against
        // arborium-cmake 2.17.0's node-types.json — `function_def`'s `children`
        // list contains no `argument`). The function/macro NAME lives two levels
        // deeper: `function_def` -> `function_command` -> `argument_list` ->
        // (first) `argument`. The previous single-level `node.children()` scan
        // for an `argument` child could therefore never find one — confirmed via
        // `normalize view <file>.cmake --types-only` against a file with three
        // functions/macros returning zero symbols. Walk into the leading
        // `function_command`/`macro_command` child first, then take its first
        // `argument`.
        let mut cursor = node.walk();
        let command = node
            .children(&mut cursor)
            .find(|child| child.kind() == "function_command" || child.kind() == "macro_command")?;
        let mut cmd_cursor = command.walk();
        let argument_list = command
            .children(&mut cmd_cursor)
            .find(|child| child.kind() == "argument_list")?;
        let mut arg_cursor = argument_list.walk();
        let name_arg = argument_list
            .children(&mut arg_cursor)
            .find(|child| child.kind() == "argument")?;
        Some(&content[name_arg.byte_range()])
    }
}

impl LanguageSymbols for CMake {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            "block", "block_command", "block_def", "body", "else", "else_command",
            "elseif", "endblock", "endblock_command", "endforeach", "endforeach_command",
            "endfunction", "endfunction_command", "endif", "endif_command", "endwhile",
            "endwhile_command", "foreach", "foreach_command", "function",
            "identifier", "if", "if_command", "while",
            "while_command",
            // control flow — not extracted as symbols
            "if_condition",
            "foreach_loop",
            "while_loop",
            "elseif_command",
        ];
        validate_unused_kinds_audit(&CMake, documented_unused)
            .expect("CMake unused node kinds audit failed");
    }
}
