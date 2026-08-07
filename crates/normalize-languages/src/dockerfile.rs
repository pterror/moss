//! Dockerfile language support.

use crate::{Import, Language, LanguageSymbols};
use tree_sitter::Node;

/// Dockerfile language support.
pub struct Dockerfile;

impl Language for Dockerfile {
    fn name(&self) -> &'static str {
        "Dockerfile"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["dockerfile"]
    }
    fn grammar_name(&self) -> &'static str {
        "dockerfile"
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }

    // Dockerfiles have stages (FROM ... AS name) that act as containers

    // No functions in Dockerfile

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        match node.kind() {
            "from_instruction" => {
                if let Some(image) = self.extract_image_name(node, content) {
                    return vec![Import {
                        module: image,
                        names: Vec::new(),
                        alias: self.extract_stage_name(node, content),
                        is_wildcard: false,
                        is_relative: false,
                        line: node.start_position().row + 1,
                    }];
                }
                Vec::new()
            }
            // COPY --from=<stage-or-image> references an earlier build stage
            // (by name or index) or an external image — the multi-stage-build
            // analog of a FROM import. Real Docker semantics restrict `--from`
            // to COPY (not ADD, even under BuildKit).
            //
            // The grammar gives `copy_instruction` no fields at all — every
            // `--flag=value` prefix (`--from=`, `--chown=`, ...) parses as an
            // undifferentiated `param` node with no sub-node for the flag name
            // or its value (confirmed via `normalize syntax query`), so the
            // `--from=` prefix has to be identified and stripped from the
            // param's own text here — a `.scm` query has no field to anchor on
            // for this distinction (see imports.scm's matching comment).
            "copy_instruction" => self.extract_copy_from_imports(node, content),
            _ => Vec::new(),
        }
    }

    fn format_import(&self, import: &Import, _names: Option<&[&str]>) -> String {
        // Dockerfile: FROM image
        format!("FROM {}", import.module)
    }

    fn node_name<'a>(&self, _node: &Node, _content: &'a str) -> Option<&'a str> {
        None
    }
}

impl LanguageSymbols for Dockerfile {}

impl Dockerfile {
    /// Extract the image name from a FROM instruction
    fn extract_image_name(&self, node: &Node, content: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "image_spec" {
                return Some(content[child.byte_range()].to_string());
            }
        }
        None
    }

    /// Extract the stage name from a FROM instruction (FROM image AS name).
    ///
    /// `as` is a direct field on `from_instruction` pointing at `image_alias`
    /// — there is no intermediate `as_instruction` node in this grammar (see
    /// imports.scm's comment on the same point).
    fn extract_stage_name(&self, node: &Node, content: &str) -> Option<String> {
        let alias = node.child_by_field_name("as")?;
        Some(content[alias.byte_range()].to_string())
    }

    /// Extract `COPY --from=<stage-or-image>` references as imports.
    ///
    /// `copy_instruction`'s children are an undifferentiated mix of `param`
    /// (any `--flag=value`), `path`, and `heredoc_block` nodes with no
    /// fields to distinguish them — so this walks every `param` child and
    /// text-matches the `--from=` prefix.
    fn extract_copy_from_imports(&self, node: &Node, content: &str) -> Vec<Import> {
        let mut cursor = node.walk();
        let mut imports = Vec::new();
        for child in node.children(&mut cursor) {
            if child.kind() != "param" {
                continue;
            }
            let text = &content[child.byte_range()];
            if let Some(target) = text.strip_prefix("--from=")
                && !target.is_empty()
            {
                imports.push(Import {
                    module: target.to_string(),
                    names: Vec::new(),
                    alias: None,
                    is_wildcard: false,
                    is_relative: false,
                    line: node.start_position().row + 1,
                });
            }
        }
        imports
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
            // Dockerfile instruction types not tracked as symbols
            "add_instruction", "cmd_instruction", "copy_instruction",
            "cross_build_instruction", "entrypoint_instruction",
            "expose_instruction", "healthcheck_instruction", "heredoc_block",
            "label_instruction", "maintainer_instruction", "onbuild_instruction",
            "run_instruction", "shell_instruction", "stopsignal_instruction",
            "user_instruction", "volume_instruction", "workdir_instruction",
        ];

        validate_unused_kinds_audit(&Dockerfile, documented_unused)
            .expect("Dockerfile unused node kinds audit failed");
    }
}
