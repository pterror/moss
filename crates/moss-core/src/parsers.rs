//! Tree-sitter parser initialization and management.

use crate::Language;
use tree_sitter::Parser;

/// Collection of tree-sitter parsers for all supported languages.
pub struct Parsers {
    python: Parser,
    rust: Parser,
    javascript: Parser,
    typescript: Parser,
    tsx: Parser,
    markdown: Parser,
    json: Parser,
    yaml: Parser,
    html: Parser,
    css: Parser,
    go: Parser,
    c: Parser,
    cpp: Parser,
    java: Parser,
    ruby: Parser,
    bash: Parser,
    toml: Parser,
}

impl Parsers {
    /// Create new parser collection with all languages initialized.
    pub fn new() -> Self {
        Self {
            python: Self::create_parser(&tree_sitter_python::LANGUAGE.into()),
            rust: Self::create_parser(&tree_sitter_rust::LANGUAGE.into()),
            javascript: Self::create_parser(&tree_sitter_javascript::LANGUAGE.into()),
            typescript: Self::create_parser(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            tsx: Self::create_parser(&tree_sitter_typescript::LANGUAGE_TSX.into()),
            markdown: Self::create_parser(&tree_sitter_md::LANGUAGE.into()),
            json: Self::create_parser(&tree_sitter_json::LANGUAGE.into()),
            yaml: Self::create_parser(&tree_sitter_yaml::LANGUAGE.into()),
            html: Self::create_parser(&tree_sitter_html::LANGUAGE.into()),
            css: Self::create_parser(&tree_sitter_css::LANGUAGE.into()),
            go: Self::create_parser(&tree_sitter_go::LANGUAGE.into()),
            c: Self::create_parser(&tree_sitter_c::LANGUAGE.into()),
            cpp: Self::create_parser(&tree_sitter_cpp::LANGUAGE.into()),
            java: Self::create_parser(&tree_sitter_java::LANGUAGE.into()),
            ruby: Self::create_parser(&tree_sitter_ruby::LANGUAGE.into()),
            bash: Self::create_parser(&tree_sitter_bash::LANGUAGE.into()),
            // tree-sitter-toml-updated uses old API with language() function
            toml: Self::create_parser_old(tree_sitter_toml_updated::language()),
        }
    }

    fn create_parser(lang: &tree_sitter::Language) -> Parser {
        let mut parser = Parser::new();
        parser.set_language(lang).expect("Failed to load grammar");
        parser
    }

    // For older grammars that return tree_sitter::Language directly
    fn create_parser_old(lang: tree_sitter::Language) -> Parser {
        let mut parser = Parser::new();
        parser.set_language(&lang).expect("Failed to load grammar");
        parser
    }

    /// Get parser for a specific language.
    pub fn get(&mut self, lang: Language) -> &mut Parser {
        match lang {
            Language::Python => &mut self.python,
            Language::Rust => &mut self.rust,
            Language::JavaScript => &mut self.javascript,
            Language::TypeScript => &mut self.typescript,
            Language::Tsx => &mut self.tsx,
            Language::Markdown => &mut self.markdown,
            Language::Json => &mut self.json,
            Language::Yaml => &mut self.yaml,
            Language::Html => &mut self.html,
            Language::Css => &mut self.css,
            Language::Go => &mut self.go,
            Language::C => &mut self.c,
            Language::Cpp => &mut self.cpp,
            Language::Java => &mut self.java,
            Language::Ruby => &mut self.ruby,
            Language::Bash => &mut self.bash,
            Language::Toml => &mut self.toml,
        }
    }

    /// Parse source code, auto-detecting language from path.
    pub fn parse(
        &mut self,
        path: &std::path::Path,
        source: &str,
    ) -> Option<(Language, tree_sitter::Tree)> {
        let lang = Language::from_path(path)?;
        let parser = self.get(lang);
        let tree = parser.parse(source, None)?;
        Some((lang, tree))
    }
}

impl Default for Parsers {
    fn default() -> Self {
        Self::new()
    }
}
