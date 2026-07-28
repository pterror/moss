; Vendored from https://github.com/tree-sitter/tree-sitter-go
; License: MIT

(
  (comment)* @doc
  .
  (function_declaration
    name: (identifier) @name) @definition.function
  (#strip! @doc "^//\\s*")
  (#set-adjacent! @doc @definition.function)
)

(
  (comment)* @doc
  .
  (method_declaration
    name: (field_identifier) @name) @definition.method
  (#strip! @doc "^//\\s*")
  (#set-adjacent! @doc @definition.method)
)

; call_expression.function is the `_expression` supertype (~23 grammar
; variants). Of those, only identifier/parenthesized-identifier and
; selector_expression/parenthesized-selector have a stable, nameable callee
; suitable for @reference.call. Deliberately NOT matched, with reasoning
; (verified via `normalize syntax query` against real Go idioms):
;   - func_literal (IIFE: `func(){}()`, the idiomatic form used by
;     `go func(){}()` / `defer func(){}()`): the callee is an anonymous
;     function; there is no name to capture. Capturing the func_literal's
;     source text as @name would put multi-line function bodies into the
;     name index, corrupting it. Matches javascript.calls.scm's identical
;     omission for `(function(){})()`.
;   - call_expression (curried calls: `adder(1)(2)`): the callee is the
;     *result* of a call, not a named symbol. Rare in idiomatic Go (Go
;     doesn't favor currying).
;   - index_expression (`funcs[0]()`, `handlers[event]()`): the callee is a
;     computed collection element; there is no static name, only a
;     collection identifier and a dynamic index.
(call_expression
  function: [
    (identifier) @name
    (parenthesized_expression (identifier) @name)
    (selector_expression field: (field_identifier) @name)
    (parenthesized_expression (selector_expression field: (field_identifier) @name))
  ]) @reference.call

(type_spec
  name: (type_identifier) @name) @definition.type

; Type alias: `type MyInt = int`. Distinct node type from type_spec (no `=`
; in a type_spec); type_declaration's children field allows both
; [type_alias, type_spec] but only type_spec was handled, silently dropping
; every type-alias definition from tags.
(type_alias
  name: (type_identifier) @name) @definition.type

(type_identifier) @name @reference.type

(package_clause "package" (package_identifier) @name)

(type_declaration (type_spec name: (type_identifier) @name type: (interface_type)))

(type_declaration (type_spec name: (type_identifier) @name type: (struct_type)))

(import_declaration (import_spec) @name)

(var_declaration (var_spec name: (identifier) @name) @definition.var)

; const_spec.name is grammar-documented as `multiple=true` allowing a
; comma-separated identifier list (`const A, B = iota, iota`), but the
; tree-sitter-go grammar only tags the *first* identifier with the `name`
; field — later identifiers in the same spec are unfielded children. A
; field-constrained `name: (identifier)` pattern therefore silently drops
; every name after the first in a multi-name const_spec. Matching
; positionally (no field constraint) catches all of them; verified this
; does not also match the `value:` expression_list contents, since those
; are nested one level deeper (inside expression_list, not a direct child).
(const_declaration (const_spec (identifier) @name) @definition.constant)
