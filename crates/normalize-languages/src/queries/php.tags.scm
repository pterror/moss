; PHP tags query

(function_definition
  name: (name) @name) @definition.function

(method_declaration
  name: (name) @name) @definition.method

(class_declaration
  name: (name) @name) @definition.class

(interface_declaration
  name: (name) @name) @definition.interface

(trait_declaration
  name: (name) @name) @definition.class

(enum_declaration
  name: (name) @name) @definition.class

(namespace_definition
  name: (namespace_name) @name) @definition.module

; References
;
; This file previously had no @reference.* captures at all — every call
; site, `extends`, and `implements` was invisible to tags, unlike the
; sibling languages' tags.scm (see java.tags.scm/ruby.tags.scm for the same
; construct categories). All patterns below were verified against real
; parse output via `normalize syntax query`/`normalize syntax ast`, cross-
; referenced field-by-field against arborium-php 2.17.0's node-types.json.

; Function calls: func(args), $func(args), \Ns\func(args), Ns\func(args).
; `function_call_expression.function` allows 17 variants total; the ones
; below cover every identifier-bearing form. The remaining variants
; (array_creation_expression, parenthesized_expression, subscript_expression,
; string/heredoc/nowdoc, and nested call/`new` results) are genuinely
; computed callees with no stable "name" — captured in php.calls.scm (call
; sites still matter for call-graph/complexity purposes) but intentionally
; left out of tags, where @name is meant to be a legible identifier.
(function_call_expression
  function: (name) @name) @reference.call

(function_call_expression
  function: (variable_name
    (name) @name)) @reference.call

(function_call_expression
  function: [
    (qualified_name)
    (relative_name)
  ] @name) @reference.call

; Static method call: Class::method(args), Class::$dynamicMethod(args),
; self::{$expr}(args). `scoped_call_expression.name` allows exactly these
; four variants (name/variable_name/dynamic_variable_name/expression); the
; `expression` entry is a real tree-sitter supertype here (verified it binds
; to the concrete node, e.g. `binary_expression`, not a literal "expression"
; node — and verified it does NOT also match name/variable_name, so this
; list is not a double-count risk).
(scoped_call_expression
  name: [
    (name)
    (variable_name)
    (dynamic_variable_name)
    (expression)
  ] @name) @reference.call

; Instance method call: $obj->method(args), $obj->$method(args),
; $obj->{$expr}(args). Same four-variant field shape as scoped_call_expression.
(member_call_expression
  name: [
    (name)
    (variable_name)
    (dynamic_variable_name)
    (expression)
  ] @name) @reference.call

; Nullsafe method call: $obj?->method(args), etc. Same field shape again.
(nullsafe_member_call_expression
  name: [
    (name)
    (variable_name)
    (dynamic_variable_name)
    (expression)
  ] @name) @reference.call

; Constructor invocation: new Foo(), new \App\Models\Bar(), new $className().
; `object_creation_expression`'s constructor-name is an unnamed (positional)
; first child, not a named field — anchored with `.` so only that position
; is captured, not the `arguments` node that follows it. Limited to the
; identifier-bearing variants (name/qualified_name/relative_name/
; variable_name/dynamic_variable_name); `anonymous_class` (`new class {...}`)
; is deliberately excluded — it has no name field at all, and capturing the
; whole class body as "name" text would be dishonest, not just incomplete
; (verified: an unconstrained wildcard here does match the full multi-line
; anonymous_class node). member_access_expression/subscript_expression/
; parenthesized_expression forms (`new $this->classProp()`, `new $arr['k']()`)
; are rarer computed-class forms, left undocumented as a known gap rather
; than guessed at.
(object_creation_expression
  .
  [
    (name)
    (qualified_name)
    (relative_name)
    (variable_name)
    (dynamic_variable_name)
  ] @name) @reference.class

; extends BaseClass — `base_clause`'s children allow name/qualified_name/
; relative_name (plain, namespaced, and namespace-relative superclass refs).
(base_clause
  [
    (name)
    (qualified_name)
    (relative_name)
  ] @name) @reference.class

; implements Interface1, Interface2 — same three-variant shape.
(class_interface_clause
  [
    (name)
    (qualified_name)
    (relative_name)
  ] @name) @reference.implementation
