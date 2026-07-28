; Vendored from https://github.com/fwcd/tree-sitter-kotlin
; License: MIT

; Classes
(class_declaration
  (type_identifier) @name) @definition.class

; Objects
(object_declaration
  (type_identifier) @name) @definition.class

; Functions (top-level and member)
(function_declaration
  (simple_identifier) @name) @definition.function

; Properties (class-level only via class body filter)
; NOTE: property_declaration is intentionally omitted here because the Kotlin
; grammar uses the same node kind for class-level properties AND local val/var
; declarations inside function bodies. The extraction layer has no way to
; distinguish them without ancestor traversal, and including them causes all
; symbols to be silently dropped (the first property_declaration with an
; un-resolvable name causes collect_symbols_from_tags to return None).

; Enum entries
(enum_entry
  (simple_identifier) @name) @definition.constant

; Type aliases
(type_alias
  (type_identifier) @name) @definition.type

; Companion objects (only named ones)
(companion_object
  (type_identifier) @name) @definition.class

; Function calls
(call_expression
  (simple_identifier) @name) @reference.call

; Method calls via navigation
(call_expression
  (navigation_expression
    (navigation_suffix
      (simple_identifier) @name))) @reference.call

; Explicit constructor delegation: `this(...)` / `super(...)` inside a
; secondary constructor. `constructor_delegation_call` is a distinct node
; kind (its only named child is `value_arguments`; "this"/"super" are
; anonymous keyword tokens) — entirely unmatched before, silently dropping
; every secondary-constructor delegation from call extraction.
(constructor_delegation_call
  ("this") @name) @reference.call

(constructor_delegation_call
  ("super") @name) @reference.call

; Constructor invocations (superclass call with args, or an annotation
; usage with args — `@Deprecated("old")`). Restricted to the
; `delegation_specifier` context: `constructor_invocation` is also a legal
; child of `annotation`/`file_annotation`, and an unconstrained pattern
; here previously misclassified every argument-carrying annotation usage
; (e.g. `@Suppress("unused")`) as a @reference.class.
(delegation_specifier
  (constructor_invocation
    (user_type
      (type_identifier) @name))) @reference.class

; Superclass/interface reference with no invocation (no parens) — by far
; the most common Kotlin idiom for implementing an interface, e.g.
; `class Person(...) : Greeter { ... }`. This is a distinct shape from the
; constructor_invocation case above (`delegation_specifier`'s `user_type`
; child directly, not wrapped) and was entirely unmatched: the grammar
; cannot syntactically distinguish "extends" from "implements" here (both
; use the same bare `: Type` form), so — consistent with tags.scm not
; fabricating a distinction the CST doesn't support — both map to the same
; @reference.class capture as the constructor_invocation form above.
(delegation_specifier
  (user_type
    (type_identifier) @name)) @reference.class

; Interface delegation (`by`): `class Derived(b: Base) : Base by b`. The
; delegate type lives two levels deep (delegation_specifier ->
; explicit_delegation -> user_type), not a direct delegation_specifier
; child like the two forms above, so it needs its own pattern.
(delegation_specifier
  (explicit_delegation
    (user_type
      (type_identifier) @name))) @reference.class
