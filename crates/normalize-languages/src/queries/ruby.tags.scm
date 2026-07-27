; Vendored from https://github.com/tree-sitter/tree-sitter-ruby
; License: MIT

; Method definitions

(
  (comment)* @doc
  .
  [
    (method
      name: (_) @name) @definition.method
    (singleton_method
      name: (_) @name) @definition.method
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.method)
)

(alias
  name: (_) @name) @definition.method

(setter
  (identifier) @ignore)

; Class definitions

(
  (comment)* @doc
  .
  [
    (class
      name: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
    (singleton_class
      value: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.class)
)

; Module definitions

(
  (module
    name: [
      (constant) @name
      (scope_resolution
        name: (_) @name)
    ]) @definition.module
)

; `class << self` (and, more rarely, `class << some_object`) opens a
; singleton class whose `value` field is `_arg` — in the overwhelmingly
; common `self` case that's the bare `self` keyword node, which has no name
; text at all. There is no grammar-level name to report, so this container
; is intentionally NOT captured as @definition.class/@definition.module:
; capturing it would require fabricating a synthetic name (e.g. the literal
; string "self"), which CLAUDE.md's "be honest about capabilities" rule
; forbids. Methods defined inside `class << self ... end` still parse as
; plain `method` nodes (not `singleton_method`) and are captured normally by
; the method-definition pattern above; they simply nest under the enclosing
; `class`/`module`, not under a distinct singleton-class symbol.

; Calls

(call method: (identifier) @name) @reference.call

; Bare Kernel-style calls where the callee name is itself a constant
; (`Integer(x)`, `Array(x)`, `String(x)`, `Float(x)`) — `call.method` allows
; `constant` in addition to `identifier`; verified via `normalize syntax
; query` that these are real `call` nodes, not some other shape.
(call method: (constant) @name) @reference.call

(
  [(identifier) (constant)] @name @reference.call
  (#is-not? local)
  (#not-match? @name "^(lambda|load|require|require_relative|__FILE__|__LINE__)$")
)
