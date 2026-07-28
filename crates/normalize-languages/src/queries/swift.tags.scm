; Swift tags query
; In the Swift grammar, class/struct/enum/actor/extension all use
; class_declaration with declaration_kind distinguishing them. Protocol has
; its own protocol_declaration.

; Function declarations. `name` is multiple=true and, verified via real parse
; output (`normalize syntax query`/`normalize syntax ast`), also covers
; operator-overload forms: plain simple_identifier, custom_operator
; (multi-char custom operators like `+++`), and literal operator tokens
; (`==`, `+`, `+=`, `<`, …) for standard operator overloading
; (`static func == (lhs: Foo, rhs: Foo) -> Bool`) — how every
; Equatable/Comparable/Hashable/arithmetic conformance is written in real
; Swift code. `node-types.json`'s `name` field for function_declaration ALSO
; lists type-expression variants (array_type, user_type, opaque_type, …);
; verified this is a grammar quirk where the RETURN type (`-> Bool`) gets
; mistakenly tagged with the same `name` field on the same node — a wildcard
; `name: (_)` pattern silently captures "Bool"/"Int" as the function's name
; instead of the real one. Do NOT widen this to a wildcard.
(function_declaration
  name: [
    (simple_identifier)
    (custom_operator)
    "!=" "%" "%=" "&" "*" "*=" "+" "++" "+=" "-" "--" "-="
    "/" "/=" "<" "<<" "<=" "==" ">" ">=" ">>" "^" "|" "~"
  ] @name) @definition.function

; Class/struct/enum/actor declarations (distinguished by declaration_kind);
; the plain form uses a bare type_identifier.
(class_declaration
  name: (type_identifier) @name) @definition.class

; Extension declarations (declaration_kind == "extension") use the SAME
; class_declaration node, but their target type is wrapped in `user_type`
; instead of a bare type_identifier — verified via `normalize syntax ast`:
; both `extension Foo { }` and `extension Array where Element == Int { }`
; parse `name` as `user_type -> type_identifier`. Without this pattern, every
; extension in a Swift codebase — a heavily-used idiom (protocol conformance,
; computed properties, organizing code by feature) — is completely invisible
; to tags: no container symbol, and every method/property declared inside is
; never nested under anything.
(class_declaration
  name: (user_type
    (type_identifier) @name)) @definition.class

; Protocol declarations (interfaces)
(protocol_declaration
  name: (type_identifier) @name) @definition.interface

; Type alias declarations
(typealias_declaration
  name: (type_identifier) @name) @definition.type

; Enum cases (`case success(String)`, `case pending, cancelled`). `name` is
; multiple=true and, verified via real parse output, tags EVERY identifier in
; a comma-separated case list (unlike the analogous Go const_spec bug found in
; batch 1) — no positional workaround needed here.
(enum_entry
  name: (simple_identifier) @name) @definition.constant

; Member properties (`var`/`let` directly inside a class/struct/enum/
; extension body). property_declaration is used for BOTH member-level
; properties AND local var/let declarations inside function bodies (same
; node kind — confirmed via `normalize syntax ast`), so these patterns are
; deliberately restricted to a DIRECT child of class_body/enum_class_body to
; exclude locals: a local `var x = 5` inside a method body is a child of
; `statements`, never of class_body/enum_class_body directly. `let`/`var` are
; distinguished via value_binding_pattern's `mutability` field, mirroring the
; Go convention of separate @definition.constant/@definition.var captures.
[
  (class_body
    (property_declaration
      (value_binding_pattern mutability: "let")
      name: (pattern (simple_identifier) @name)) @definition.constant)
  (enum_class_body
    (property_declaration
      (value_binding_pattern mutability: "let")
      name: (pattern (simple_identifier) @name)) @definition.constant)
]

[
  (class_body
    (property_declaration
      (value_binding_pattern mutability: "var")
      name: (pattern (simple_identifier) @name)) @definition.var)
  (enum_class_body
    (property_declaration
      (value_binding_pattern mutability: "var")
      name: (pattern (simple_identifier) @name)) @definition.var)
]

; Protocol requirements use distinct node types from their concrete-body
; counterparts (protocol_property_declaration / protocol_function_declaration
; / associatedtype_declaration, not property_declaration/function_declaration)
; — none of these were previously captured at all, so every protocol's
; requirement list was invisible to tags.
(protocol_property_declaration
  name: (pattern (simple_identifier) @name)) @definition.var

(protocol_function_declaration
  name: (simple_identifier) @name) @definition.method

(associatedtype_declaration
  name: (type_identifier) @name) @definition.type

; init/deinit/subscript declarations are deliberately NOT captured here.
; Verified via `normalize syntax query`: `(init_declaration name: (_) @name)`
; and the equivalent for deinit_declaration find nothing in the real parse
; tree, even though node-types.json claims init_declaration has a required
; `name: (init)` field — it is declared but never populated (the same class
; of grammar quirk the CFG remediation found elsewhere). subscript_declaration
; DOES populate a `name` field, but only with the same return-type-mislabeled
; type-expression variants as function_declaration above (not a real
; subscript name — subscripts are genuinely unnamed in Swift, accessed via
; `obj[index]`). `Language::node_name` (the default impl symbol extraction
; relies on) resolves a symbol's name via `child_by_field_name("name")`
; directly on the AST node, independent of what a query captures; without a
; populated field there, matching these node types here would produce query
; captures that silently vanish downstream (no symbol emitted), which is
; worse than not capturing them at all. Fixing this needs a
; `Language::node_name` override in swift.rs for init/deinit specifically —
; out of scope for this query-only sweep.
