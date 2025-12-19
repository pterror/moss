use std::path::Path;
use tree_sitter::Parser;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: u32,
    pub end_line: u32,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Class => "class",
            SymbolKind::Method => "method",
        }
    }
}

pub struct SymbolParser {
    python_parser: Parser,
    rust_parser: Parser,
}

impl SymbolParser {
    pub fn new() -> Self {
        let mut python_parser = Parser::new();
        python_parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to load Python grammar");

        let mut rust_parser = Parser::new();
        rust_parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Failed to load Rust grammar");

        Self {
            python_parser,
            rust_parser,
        }
    }

    pub fn parse_file(&mut self, path: &Path, content: &str) -> Vec<Symbol> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "py" => self.parse_python(content),
            "rs" => self.parse_rust(content),
            _ => Vec::new(),
        }
    }

    fn parse_python(&mut self, content: &str) -> Vec<Symbol> {
        let tree = match self.python_parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut symbols = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();
        self.collect_python_symbols(&mut cursor, content, &mut symbols, None);
        symbols
    }

    fn collect_python_symbols(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        symbols: &mut Vec<Symbol>,
        parent: Option<&str>,
    ) {
        loop {
            let node = cursor.node();
            let kind = node.kind();

            match kind {
                "function_definition" | "async_function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &content[name_node.byte_range()];
                        let symbol_kind = if parent.is_some() {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: symbol_kind,
                            start_line: node.start_position().row as u32 + 1,
                            end_line: node.end_position().row as u32 + 1,
                            parent: parent.map(String::from),
                        });
                    }
                }
                "class_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &content[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Class,
                            start_line: node.start_position().row as u32 + 1,
                            end_line: node.end_position().row as u32 + 1,
                            parent: parent.map(String::from),
                        });

                        if cursor.goto_first_child() {
                            self.collect_python_symbols(cursor, content, symbols, Some(name));
                            cursor.goto_parent();
                        }
                        if cursor.goto_next_sibling() {
                            continue;
                        }
                        break;
                    }
                }
                _ => {}
            }

            if kind != "class_definition" && cursor.goto_first_child() {
                self.collect_python_symbols(cursor, content, symbols, parent);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    fn parse_rust(&mut self, content: &str) -> Vec<Symbol> {
        let tree = match self.rust_parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut symbols = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();
        self.collect_rust_symbols(&mut cursor, content, &mut symbols, None);
        symbols
    }

    fn collect_rust_symbols(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        symbols: &mut Vec<Symbol>,
        parent: Option<&str>,
    ) {
        loop {
            let node = cursor.node();
            let kind = node.kind();

            match kind {
                "function_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &content[name_node.byte_range()];
                        let symbol_kind = if parent.is_some() {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: symbol_kind,
                            start_line: node.start_position().row as u32 + 1,
                            end_line: node.end_position().row as u32 + 1,
                            parent: parent.map(String::from),
                        });
                    }
                }
                "struct_item" | "enum_item" | "trait_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &content[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Class,
                            start_line: node.start_position().row as u32 + 1,
                            end_line: node.end_position().row as u32 + 1,
                            parent: parent.map(String::from),
                        });
                    }
                }
                "impl_item" => {
                    let impl_name = node
                        .child_by_field_name("type")
                        .map(|n| content[n.byte_range()].to_string());

                    if let Some(name) = &impl_name {
                        if cursor.goto_first_child() {
                            self.collect_rust_symbols(cursor, content, symbols, Some(name));
                            cursor.goto_parent();
                        }
                        if cursor.goto_next_sibling() {
                            continue;
                        }
                        break;
                    }
                }
                _ => {}
            }

            if kind != "impl_item" && cursor.goto_first_child() {
                self.collect_rust_symbols(cursor, content, symbols, parent);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    pub fn find_symbol(&mut self, path: &Path, content: &str, name: &str) -> Option<Symbol> {
        let symbols = self.parse_file(path, content);
        symbols.into_iter().find(|s| s.name == name)
    }

    pub fn extract_symbol_source(&mut self, path: &Path, content: &str, name: &str) -> Option<String> {
        let symbol = self.find_symbol(path, content, name)?;
        let lines: Vec<&str> = content.lines().collect();
        let start = (symbol.start_line as usize).saturating_sub(1);
        let end = (symbol.end_line as usize).min(lines.len());
        Some(lines[start..end].join("\n"))
    }

    /// Find function calls in a source snippet, returns (callee_name, line_offset)
    pub fn find_calls_in_source(&mut self, source: &str) -> Vec<(String, u32)> {
        // Try Python first, then Rust
        let py_calls = self.find_python_calls_with_lines(source);
        if !py_calls.is_empty() {
            return py_calls;
        }
        self.find_rust_calls_with_lines(source)
    }

    fn find_python_calls_with_lines(&mut self, source: &str) -> Vec<(String, u32)> {
        let tree = match self.python_parser.parse(source, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut calls = Vec::new();
        let mut cursor = tree.root_node().walk();
        self.collect_python_calls_with_lines(&mut cursor, source, &mut calls);
        calls
    }

    fn collect_python_calls_with_lines(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        calls: &mut Vec<(String, u32)>,
    ) {
        loop {
            let node = cursor.node();

            if node.kind() == "call" {
                if let Some(func_node) = node.child_by_field_name("function") {
                    let func_text = &content[func_node.byte_range()];
                    let name = func_text.split('.').last().unwrap_or(func_text);
                    let line = node.start_position().row as u32;
                    calls.push((name.to_string(), line));
                }
            }

            if cursor.goto_first_child() {
                self.collect_python_calls_with_lines(cursor, content, calls);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    fn find_rust_calls_with_lines(&mut self, source: &str) -> Vec<(String, u32)> {
        let tree = match self.rust_parser.parse(source, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut calls = Vec::new();
        let mut cursor = tree.root_node().walk();
        self.collect_rust_calls_with_lines(&mut cursor, source, &mut calls);
        calls
    }

    fn collect_rust_calls_with_lines(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        calls: &mut Vec<(String, u32)>,
    ) {
        loop {
            let node = cursor.node();

            if node.kind() == "call_expression" {
                if let Some(func_node) = node.child_by_field_name("function") {
                    let func_text = &content[func_node.byte_range()];
                    let name = func_text
                        .split("::")
                        .last()
                        .unwrap_or(func_text)
                        .split('.')
                        .last()
                        .unwrap_or(func_text);
                    let line = node.start_position().row as u32;
                    calls.push((name.to_string(), line));
                }
            }

            if cursor.goto_first_child() {
                self.collect_rust_calls_with_lines(cursor, content, calls);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
