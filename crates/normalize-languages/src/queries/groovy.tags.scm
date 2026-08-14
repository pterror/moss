; Groovy tags query
;
; Completeness matrix verified against arborium-groovy 2.17.0's
; node-types.json via `normalize syntax ast`/`normalize syntax query`.

; Class definitions: class Foo { ... }
; `class_definition` is the single node type for both classes and
; interfaces — there is no dedicated `interface_definition` node. The two
; forms are distinguished only by the leading anonymous keyword token
; ("class" vs "interface"), matched here as a literal.
(class_definition
  "class"
  name: (identifier) @name) @definition.class

; Interface definitions: interface Foo { ... } — see note above.
; The grammar has no `trait` or `enum` construct at all: `trait Foo { ... }`
; and `enum Foo { ... }` do not parse as any recognized declaration (they
; produce bare identifier tokens followed by an unrelated closure, with no
; ERROR node) — confirmed via `normalize syntax ast`. There is nothing to
; query for either.
(class_definition
  "interface"
  name: (identifier) @name) @definition.interface

; Method/function definitions (with a body): def foo() {} / Type m() {}
(function_definition
  function: (identifier) @name) @definition.function

; Method declarations (no body): abstract/interface method signatures,
; e.g. `double area()` inside `interface Shape { ... }`.
(function_declaration
  function: (identifier) @name) @definition.function

; Closure assigned to a name — Groovy's common alternative to `def foo() {}`,
; heavily used in Gradle/Jenkins DSL scripts: `def f = { x -> ... }`.
(declaration
  name: (identifier) @name
  value: (closure)) @definition.function

; Same idiom via plain (re)assignment rather than `def`: `f = { x -> ... }`.
; `assignment` has no fields in this grammar (just a flat children list), so
; the pattern is positional: identifier immediately followed by a closure.
(assignment
  (identifier) @name
  .
  (closure)) @definition.function

; Call references — same variant coverage as groovy.calls.scm (see that
; file's header comment for the full completeness-matrix rationale, including
; why `closure` as a call target is not queried).

(function_call
  function: (identifier) @name) @reference.call

(juxt_function_call
  function: (identifier) @name) @reference.call

(function_call
  function: (dotted_identifier
    (_)
    (identifier) @name)) @reference.call

(juxt_function_call
  function: (dotted_identifier
    (_)
    (identifier) @name)) @reference.call

(function_call
  function: (function_call) @name) @reference.call

(function_call
  function: (index) @name) @reference.call

(juxt_function_call
  function: (index) @name) @reference.call

(function_call
  function: (parenthesized_expression) @name) @reference.call

; this(args) call form — mirrors the super/this-as-@name convention used by
; java.tags.scm's explicit_constructor_invocation patterns. `this` is an
; anonymous token in this grammar, matched as a literal rather than `(this)`.
; See groovy.calls.scm's note: this grammar has no constructor node at all,
; so this only matches the `this(args)` call *shape*, not real constructor
; delegation (which never parses cleanly here).
(function_call
  function: "this" @name) @reference.call
