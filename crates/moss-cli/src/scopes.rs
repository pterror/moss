//! Variable scope analysis.
//!
//! Tracks variable definitions and their scopes in source files.
//! Supports finding where a variable is defined, what's in scope at a position,
//! and detecting variable shadowing.

use std::path::Path;
use tree_sitter::{Node, Parser};

/// A scope in the code
#[derive(Debug, Clone)]
pub struct Scope {
    pub kind: ScopeKind,
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub bindings: Vec<Binding>,
    pub children: Vec<Scope>,
}

/// Type of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Some variants reserved for future language support
pub enum ScopeKind {
    Module,
    Function,
    Class,
    Method,
    Lambda,
    Comprehension,
    Loop,
    With,
    Try,
    Block, // Generic block scope (Rust)
    Impl,
}

impl ScopeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScopeKind::Module => "module",
            ScopeKind::Function => "function",
            ScopeKind::Class => "class",
            ScopeKind::Method => "method",
            ScopeKind::Lambda => "lambda",
            ScopeKind::Comprehension => "comprehension",
            ScopeKind::Loop => "loop",
            ScopeKind::With => "with",
            ScopeKind::Try => "try",
            ScopeKind::Block => "block",
            ScopeKind::Impl => "impl",
        }
    }
}

/// A variable binding (definition)
#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub kind: BindingKind,
    pub line: usize,
    pub column: usize,
}

/// Type of binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Variable,
    Parameter,
    Function,
    Class,
    Import,
    ForLoop,
    WithItem,
    ExceptHandler,
}

impl BindingKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BindingKind::Variable => "variable",
            BindingKind::Parameter => "parameter",
            BindingKind::Function => "function",
            BindingKind::Class => "class",
            BindingKind::Import => "import",
            BindingKind::ForLoop => "for",
            BindingKind::WithItem => "with",
            BindingKind::ExceptHandler => "except",
        }
    }
}

/// Result of scope analysis
pub struct ScopeResult {
    pub root: Scope,
    pub file_path: String,
}

impl ScopeResult {
    /// Find all bindings visible at a given line
    pub fn bindings_at_line(&self, line: usize) -> Vec<&Binding> {
        let mut result = Vec::new();
        self.collect_bindings_at(&self.root, line, &mut result);
        result
    }

    fn collect_bindings_at<'a>(
        &'a self,
        scope: &'a Scope,
        line: usize,
        result: &mut Vec<&'a Binding>,
    ) {
        // Check if line is within this scope
        if line < scope.start_line || line > scope.end_line {
            return;
        }

        // Add bindings from this scope that are defined before the line
        for binding in &scope.bindings {
            if binding.line <= line {
                result.push(binding);
            }
        }

        // Recurse into child scopes
        for child in &scope.children {
            self.collect_bindings_at(child, line, result);
        }
    }

    /// Find where a name is defined at a given line
    pub fn find_definition(&self, name: &str, line: usize) -> Option<&Binding> {
        let bindings = self.bindings_at_line(line);
        // Return the most recent binding (last one shadowing previous)
        bindings
            .into_iter()
            .filter(|b| b.name == name)
            .last()
    }

    /// Format the scope tree for display
    pub fn format(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("# Scopes in {}", self.file_path));
        lines.push(String::new());
        self.format_scope(&self.root, 0, &mut lines);
        lines.join("\n")
    }

    fn format_scope(&self, scope: &Scope, indent: usize, lines: &mut Vec<String>) {
        let prefix = "  ".repeat(indent);
        let name = scope.name.as_deref().unwrap_or("<anonymous>");
        lines.push(format!(
            "{}{} {} (lines {}-{})",
            prefix,
            scope.kind.as_str(),
            name,
            scope.start_line,
            scope.end_line
        ));

        if !scope.bindings.is_empty() {
            for binding in &scope.bindings {
                lines.push(format!(
                    "{}  {} {} (line {})",
                    prefix,
                    binding.kind.as_str(),
                    binding.name,
                    binding.line
                ));
            }
        }

        for child in &scope.children {
            self.format_scope(child, indent + 1, lines);
        }
    }
}

pub struct ScopeAnalyzer {
    python_parser: Parser,
    rust_parser: Parser,
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        let mut python_parser = Parser::new();
        python_parser
            .set_language(&moss_core::tree_sitter_python::LANGUAGE.into())
            .expect("Failed to load Python grammar");

        let mut rust_parser = Parser::new();
        rust_parser
            .set_language(&moss_core::tree_sitter_rust::LANGUAGE.into())
            .expect("Failed to load Rust grammar");

        Self {
            python_parser,
            rust_parser,
        }
    }

    pub fn analyze(&mut self, path: &Path, content: &str) -> ScopeResult {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let root = match ext {
            "py" => self.analyze_python(content),
            "rs" => self.analyze_rust(content),
            _ => Scope {
                kind: ScopeKind::Module,
                name: None,
                start_line: 1,
                end_line: content.lines().count(),
                bindings: Vec::new(),
                children: Vec::new(),
            },
        };

        ScopeResult {
            root,
            file_path: path.to_string_lossy().to_string(),
        }
    }

    fn analyze_python(&mut self, content: &str) -> Scope {
        let tree = match self.python_parser.parse(content, None) {
            Some(t) => t,
            None => {
                return Scope {
                    kind: ScopeKind::Module,
                    name: None,
                    start_line: 1,
                    end_line: content.lines().count(),
                    bindings: Vec::new(),
                    children: Vec::new(),
                }
            }
        };

        let root = tree.root_node();
        let source = content.as_bytes();

        self.build_python_scope(root, source, ScopeKind::Module, None)
    }

    fn build_python_scope(
        &self,
        node: Node,
        source: &[u8],
        kind: ScopeKind,
        name: Option<String>,
    ) -> Scope {
        let mut bindings = Vec::new();
        let mut children = Vec::new();

        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                // Function definitions create new scopes
                "function_definition" => {
                    let func_name = child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .map(|s| s.to_string());

                    // Add function name as binding in current scope
                    if let Some(ref name) = func_name {
                        bindings.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::Function,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }

                    // Create child scope for function body
                    let scope_kind = if kind == ScopeKind::Class {
                        ScopeKind::Method
                    } else {
                        ScopeKind::Function
                    };
                    let mut func_scope = self.build_python_scope(child, source, scope_kind, func_name);

                    // Extract parameters
                    if let Some(params) = child.child_by_field_name("parameters") {
                        self.extract_python_params(params, source, &mut func_scope.bindings);
                    }

                    children.push(func_scope);
                }

                // Class definitions create new scopes
                "class_definition" => {
                    let class_name = child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .map(|s| s.to_string());

                    // Add class name as binding in current scope
                    if let Some(ref name) = class_name {
                        bindings.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::Class,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }

                    children.push(self.build_python_scope(child, source, ScopeKind::Class, class_name));
                }

                // Assignments create bindings
                "assignment" | "augmented_assignment" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        self.extract_python_targets(left, source, &mut bindings, BindingKind::Variable);
                    }
                }

                // Annotated assignments
                "annotated_assignment" => {
                    // First child is typically the target (use named_child to avoid borrow issues)
                    if let Some(target) = child.named_child(0) {
                        if target.kind() == "identifier" {
                            if let Ok(name) = target.utf8_text(source) {
                                bindings.push(Binding {
                                    name: name.to_string(),
                                    kind: BindingKind::Variable,
                                    line: target.start_position().row + 1,
                                    column: target.start_position().column,
                                });
                            }
                        }
                    }
                }

                // Import statements
                "import_statement" | "import_from_statement" => {
                    self.extract_python_imports(child, source, &mut bindings);
                }

                // For loops
                "for_statement" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        self.extract_python_targets(left, source, &mut bindings, BindingKind::ForLoop);
                    }
                    // Recurse into body
                    let mut c = child.walk();
                    for grandchild in child.children(&mut c) {
                        if grandchild.kind() == "block" {
                            let loop_scope = self.build_python_scope(grandchild, source, ScopeKind::Loop, None);
                            if !loop_scope.bindings.is_empty() || !loop_scope.children.is_empty() {
                                children.push(loop_scope);
                            }
                        }
                    }
                }

                // With statements
                "with_statement" => {
                    let mut c = child.walk();
                    for grandchild in child.children(&mut c) {
                        if grandchild.kind() == "with_clause" {
                            let mut cc = grandchild.walk();
                            for item in grandchild.children(&mut cc) {
                                if item.kind() == "with_item" {
                                    // Look for "as" alias
                                    if let Some(alias) = item.child_by_field_name("alias") {
                                        self.extract_python_targets(alias, source, &mut bindings, BindingKind::WithItem);
                                    }
                                }
                            }
                        }
                    }
                }

                // Except handlers
                "except_clause" => {
                    // Look for the name after "as"
                    let mut c = child.walk();
                    for grandchild in child.children(&mut c) {
                        if grandchild.kind() == "identifier" {
                            if let Ok(name) = grandchild.utf8_text(source) {
                                bindings.push(Binding {
                                    name: name.to_string(),
                                    kind: BindingKind::ExceptHandler,
                                    line: grandchild.start_position().row + 1,
                                    column: grandchild.start_position().column,
                                });
                            }
                        }
                    }
                }

                // Comprehensions create their own scope
                "list_comprehension" | "set_comprehension" | "dictionary_comprehension" | "generator_expression" => {
                    let comp_scope = self.build_python_scope(child, source, ScopeKind::Comprehension, None);
                    if !comp_scope.bindings.is_empty() {
                        children.push(comp_scope);
                    }
                }

                // For clauses in comprehensions
                "for_in_clause" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        self.extract_python_targets(left, source, &mut bindings, BindingKind::ForLoop);
                    }
                }

                // Lambda expressions
                "lambda" => {
                    let mut lambda_scope = Scope {
                        kind: ScopeKind::Lambda,
                        name: None,
                        start_line: child.start_position().row + 1,
                        end_line: child.end_position().row + 1,
                        bindings: Vec::new(),
                        children: Vec::new(),
                    };
                    if let Some(params) = child.child_by_field_name("parameters") {
                        self.extract_python_params(params, source, &mut lambda_scope.bindings);
                    }
                    if !lambda_scope.bindings.is_empty() {
                        children.push(lambda_scope);
                    }
                }

                // Other nodes: recurse
                _ => {
                    if child.child_count() > 0 {
                        let nested = self.build_python_scope(child, source, kind, None);
                        bindings.extend(nested.bindings);
                        children.extend(nested.children);
                    }
                }
            }
        }

        Scope {
            kind,
            name,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            bindings,
            children,
        }
    }

    fn extract_python_targets(&self, node: Node, source: &[u8], bindings: &mut Vec<Binding>, kind: BindingKind) {
        match node.kind() {
            "identifier" => {
                if let Ok(name) = node.utf8_text(source) {
                    bindings.push(Binding {
                        name: name.to_string(),
                        kind,
                        line: node.start_position().row + 1,
                        column: node.start_position().column,
                    });
                }
            }
            "tuple_pattern" | "list_pattern" | "pattern_list" | "tuple" | "list" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_python_targets(child, source, bindings, kind);
                }
            }
            _ => {}
        }
    }

    fn extract_python_params(&self, node: Node, source: &[u8], bindings: &mut Vec<Binding>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if let Ok(name) = child.utf8_text(source) {
                        bindings.push(Binding {
                            name: name.to_string(),
                            kind: BindingKind::Parameter,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }
                }
                "typed_parameter" | "default_parameter" | "typed_default_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source) {
                            bindings.push(Binding {
                                name: name.to_string(),
                                kind: BindingKind::Parameter,
                                line: name_node.start_position().row + 1,
                                column: name_node.start_position().column,
                            });
                        }
                    }
                }
                "list_splat_pattern" | "dictionary_splat_pattern" => {
                    let mut c = child.walk();
                    for grandchild in child.children(&mut c) {
                        if grandchild.kind() == "identifier" {
                            if let Ok(name) = grandchild.utf8_text(source) {
                                bindings.push(Binding {
                                    name: name.to_string(),
                                    kind: BindingKind::Parameter,
                                    line: grandchild.start_position().row + 1,
                                    column: grandchild.start_position().column,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_python_imports(&self, node: Node, source: &[u8], bindings: &mut Vec<Binding>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "dotted_name" => {
                    // For "import x", the first identifier is the binding
                    if let Some(first) = child.named_child(0) {
                        if first.kind() == "identifier" {
                            if let Ok(name) = first.utf8_text(source) {
                                bindings.push(Binding {
                                    name: name.to_string(),
                                    kind: BindingKind::Import,
                                    line: first.start_position().row + 1,
                                    column: first.start_position().column,
                                });
                            }
                        }
                    }
                }
                "aliased_import" => {
                    // Use alias if present, otherwise use name
                    let alias_name = child.child_by_field_name("alias")
                        .or_else(|| child.child_by_field_name("name"));
                    if let Some(name_node) = alias_name {
                        if let Ok(name) = name_node.utf8_text(source) {
                            bindings.push(Binding {
                                name: name.to_string(),
                                kind: BindingKind::Import,
                                line: name_node.start_position().row + 1,
                                column: name_node.start_position().column,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn analyze_rust(&mut self, content: &str) -> Scope {
        let tree = match self.rust_parser.parse(content, None) {
            Some(t) => t,
            None => {
                return Scope {
                    kind: ScopeKind::Module,
                    name: None,
                    start_line: 1,
                    end_line: content.lines().count(),
                    bindings: Vec::new(),
                    children: Vec::new(),
                }
            }
        };

        let root = tree.root_node();
        let source = content.as_bytes();

        self.build_rust_scope(root, source, ScopeKind::Module, None)
    }

    fn build_rust_scope(
        &self,
        node: Node,
        source: &[u8],
        kind: ScopeKind,
        name: Option<String>,
    ) -> Scope {
        let mut bindings = Vec::new();
        let mut children = Vec::new();

        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                // Function definitions
                "function_item" => {
                    let func_name = child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .map(|s| s.to_string());

                    if let Some(ref name) = func_name {
                        bindings.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::Function,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }

                    let scope_kind = if kind == ScopeKind::Impl {
                        ScopeKind::Method
                    } else {
                        ScopeKind::Function
                    };
                    let mut func_scope = self.build_rust_scope(child, source, scope_kind, func_name);

                    // Extract parameters
                    if let Some(params) = child.child_by_field_name("parameters") {
                        self.extract_rust_params(params, source, &mut func_scope.bindings);
                    }

                    children.push(func_scope);
                }

                // Struct definitions
                "struct_item" => {
                    let struct_name = child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .map(|s| s.to_string());

                    if let Some(ref name) = struct_name {
                        bindings.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::Class,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }
                }

                // Enum definitions
                "enum_item" => {
                    let enum_name = child
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .map(|s| s.to_string());

                    if let Some(ref name) = enum_name {
                        bindings.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::Class,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }
                }

                // Impl blocks
                "impl_item" => {
                    let impl_name = child
                        .child_by_field_name("type")
                        .and_then(|n| n.utf8_text(source).ok())
                        .map(|s| s.to_string());

                    children.push(self.build_rust_scope(child, source, ScopeKind::Impl, impl_name));
                }

                // Let bindings
                "let_declaration" => {
                    if let Some(pattern) = child.child_by_field_name("pattern") {
                        self.extract_rust_pattern(pattern, source, &mut bindings);
                    }
                }

                // For loops
                "for_expression" => {
                    if let Some(pattern) = child.child_by_field_name("pattern") {
                        self.extract_rust_pattern(pattern, source, &mut bindings);
                    }
                    // Recurse into body
                    if let Some(body) = child.child_by_field_name("body") {
                        let loop_scope = self.build_rust_scope(body, source, ScopeKind::Loop, None);
                        if !loop_scope.bindings.is_empty() || !loop_scope.children.is_empty() {
                            children.push(loop_scope);
                        }
                    }
                }

                // Block expressions (create new scope)
                "block" => {
                    let block_scope = self.build_rust_scope(child, source, ScopeKind::Block, None);
                    if !block_scope.bindings.is_empty() || !block_scope.children.is_empty() {
                        children.push(block_scope);
                    }
                }

                // Use declarations
                "use_declaration" => {
                    self.extract_rust_use(child, source, &mut bindings);
                }

                // Other nodes: recurse (but not into blocks which we handle separately)
                _ => {
                    if child.child_count() > 0 && child.kind() != "block" {
                        let nested = self.build_rust_scope(child, source, kind, None);
                        bindings.extend(nested.bindings);
                        children.extend(nested.children);
                    }
                }
            }
        }

        Scope {
            kind,
            name,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            bindings,
            children,
        }
    }

    fn extract_rust_params(&self, node: Node, source: &[u8], bindings: &mut Vec<Binding>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parameter" {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    self.extract_rust_pattern(pattern, source, bindings);
                }
            } else if child.kind() == "self_parameter" {
                bindings.push(Binding {
                    name: "self".to_string(),
                    kind: BindingKind::Parameter,
                    line: child.start_position().row + 1,
                    column: child.start_position().column,
                });
            }
        }
    }

    fn extract_rust_pattern(&self, node: Node, source: &[u8], bindings: &mut Vec<Binding>) {
        match node.kind() {
            "identifier" => {
                if let Ok(name) = node.utf8_text(source) {
                    // Skip _ patterns
                    if name != "_" {
                        bindings.push(Binding {
                            name: name.to_string(),
                            kind: BindingKind::Variable,
                            line: node.start_position().row + 1,
                            column: node.start_position().column,
                        });
                    }
                }
            }
            "tuple_pattern" | "slice_pattern" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_rust_pattern(child, source, bindings);
                }
            }
            "struct_pattern" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "field_pattern" {
                        // Could be `name` or `name: pattern`
                        if let Some(pattern) = child.child_by_field_name("pattern") {
                            self.extract_rust_pattern(pattern, source, bindings);
                        } else if let Some(name) = child.child_by_field_name("name") {
                            self.extract_rust_pattern(name, source, bindings);
                        }
                    }
                }
            }
            "ref_pattern" | "mut_pattern" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        self.extract_rust_pattern(child, source, bindings);
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_rust_use(&self, node: Node, source: &[u8], bindings: &mut Vec<Binding>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "use_as_clause" => {
                    // use x as y - y is the binding
                    if let Some(alias) = child.child_by_field_name("alias") {
                        if let Ok(name) = alias.utf8_text(source) {
                            bindings.push(Binding {
                                name: name.to_string(),
                                kind: BindingKind::Import,
                                line: alias.start_position().row + 1,
                                column: alias.start_position().column,
                            });
                        }
                    }
                }
                "use_list" => {
                    let mut c = child.walk();
                    for item in child.children(&mut c) {
                        if item.kind() == "identifier" {
                            if let Ok(name) = item.utf8_text(source) {
                                bindings.push(Binding {
                                    name: name.to_string(),
                                    kind: BindingKind::Import,
                                    line: item.start_position().row + 1,
                                    column: item.start_position().column,
                                });
                            }
                        } else if item.kind() == "use_as_clause" {
                            if let Some(alias) = item.child_by_field_name("alias") {
                                if let Ok(name) = alias.utf8_text(source) {
                                    bindings.push(Binding {
                                        name: name.to_string(),
                                        kind: BindingKind::Import,
                                        line: alias.start_position().row + 1,
                                        column: alias.start_position().column,
                                    });
                                }
                            } else {
                                // No alias, use the path's last component
                                let mut cc = item.walk();
                                if let Some(last) = item.children(&mut cc).last() {
                                    if last.kind() == "identifier" {
                                        if let Ok(name) = last.utf8_text(source) {
                                            bindings.push(Binding {
                                                name: name.to_string(),
                                                kind: BindingKind::Import,
                                                line: last.start_position().row + 1,
                                                column: last.start_position().column,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "scoped_identifier" => {
                    // use foo::bar - bar is the binding
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source) {
                            bindings.push(Binding {
                                name: name.to_string(),
                                kind: BindingKind::Import,
                                line: name_node.start_position().row + 1,
                                column: name_node.start_position().column,
                            });
                        }
                    }
                }
                "identifier" => {
                    if let Ok(name) = child.utf8_text(source) {
                        bindings.push(Binding {
                            name: name.to_string(),
                            kind: BindingKind::Import,
                            line: child.start_position().row + 1,
                            column: child.start_position().column,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}
