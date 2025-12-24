//! CSS language support (parse only, minimal skeleton).

use crate::{LanguageSupport, Symbol};
use moss_core::{tree_sitter::Node, Language};

pub struct CssSupport;

impl LanguageSupport for CssSupport {
    fn language(&self) -> Language { Language::Css }
    fn grammar_name(&self) -> &'static str { "css" }

    // CSS has no functions/containers/types in the traditional sense
    fn container_kinds(&self) -> &'static [&'static str] { &[] }
    fn function_kinds(&self) -> &'static [&'static str] { &[] }
    fn type_kinds(&self) -> &'static [&'static str] { &[] }
    fn import_kinds(&self) -> &'static [&'static str] { &[] }
    fn export_kinds(&self) -> &'static [&'static str] { &[] }
    fn scope_creating_kinds(&self) -> &'static [&'static str] { &[] }
    fn control_flow_kinds(&self) -> &'static [&'static str] { &[] }
    fn complexity_nodes(&self) -> &'static [&'static str] { &[] }
    fn nesting_nodes(&self) -> &'static [&'static str] { &[] }

    fn extract_function(&self, _node: &Node, _content: &str, _in_container: bool) -> Option<Symbol> {
        None
    }

    fn extract_container(&self, _node: &Node, _content: &str) -> Option<Symbol> {
        None
    }
}
