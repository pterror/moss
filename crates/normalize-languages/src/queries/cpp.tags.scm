; Vendored from https://github.com/tree-sitter/tree-sitter-cpp
; License: MIT

(struct_specifier name: (type_identifier) @name body:(_)) @definition.class

; Union definitions: same asymmetry bug as C's c.tags.scm had. Bare
; (`union Foo { ... };`) and typedef'd (`typedef union Baz { ... } BazT;`)
; unions both parse as a top-level `union_specifier` with name+body, exactly
; like `struct_specifier` above — never as `declaration type: (union_specifier
; ...)`, which only matches a *bodyless* union-typed variable declaration (a
; usage, not a definition) and mislabeled it @definition.class while missing
; every real union definition. Verified via `normalize syntax query`.
(union_specifier name: (type_identifier) @name body: (_)) @definition.class

(function_declarator declarator: (identifier) @name) @definition.function

(function_declarator declarator: (field_identifier) @name) @definition.function

; Destructors and operator overloads, defined inline in the class body:
; `~Foo() {}`, `Foo& operator+(...) {}`. Previously entirely untagged — no
; C++ class with a user-defined destructor or overloaded operator produced
; any @definition.method for it.
(function_declarator declarator: (destructor_name) @name) @definition.method

(function_declarator declarator: (operator_name) @name) @definition.method

; #define NAME value / #define NAME(args) body — shared C preprocessor
; syntax; same gap as c.tags.scm (macro definitions had zero tags coverage).
(preproc_def name: (identifier) @name) @definition.macro
(preproc_function_def name: (identifier) @name) @definition.macro

(function_declarator declarator: (qualified_identifier scope: (namespace_identifier) @local.scope name: (identifier) @name)) @definition.method

; Out-of-line destructor/operator definitions: `Stack::~Stack() {}`,
; `Stack::operator=(...) {}`. The qualified name's `name` field is
; `destructor_name`/`operator_name`, not `identifier`, so the pattern above
; never matched these — silently dropping every out-of-line destructor and
; operator overload (an extremely common pattern outside header-only code).
(function_declarator declarator: (qualified_identifier scope: (namespace_identifier) @local.scope name: (destructor_name) @name)) @definition.method

(function_declarator declarator: (qualified_identifier scope: (namespace_identifier) @local.scope name: (operator_name) @name)) @definition.method

; Out-of-line method of a template class: `template <typename T> T Box<T>::get() {}`
; — the qualifier before `::` is `template_type` (`Box<T>`), not
; `namespace_identifier`, because the class name itself carries template
; arguments. Same root cause as Rust's generic-impl gap (docs/query-testing-methodology.md).
(function_declarator declarator: (qualified_identifier scope: (template_type) @local.scope name: (identifier) @name)) @definition.method

(type_definition declarator: (type_identifier) @name) @definition.type

(enum_specifier name: (type_identifier) @name) @definition.type

(class_specifier name: (type_identifier) @name) @definition.class

; Explicit template specialization: `template <> class Box<int> { ... };` —
; the class name is wrapped in `template_type` (`Box<int>`), not a bare
; `type_identifier`; the plain pattern above silently dropped every
; specialization (a routine pattern for trait-style dispatch in C++).
(class_specifier name: (template_type name: (type_identifier) @name) body: (_)) @definition.class

; Namespaces (including nested `namespace a::b::c { ... }`) had zero tags
; coverage — every symbol inside one lost its enclosing container, the same
; class of bug Rust's generic/path-qualified `impl` gap was.
(namespace_definition name: (namespace_identifier) @name) @definition.module

(namespace_definition name: (nested_namespace_specifier) @name) @definition.module
