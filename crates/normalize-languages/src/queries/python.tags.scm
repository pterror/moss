; Vendored from https://github.com/tree-sitter/tree-sitter-python
; License: MIT

; Module-level constant assignment: X = 5, X: int = 5
;
; NOTE (bug fix): `expression_statement` is a grammar *supertype alias* for
; `assignment` in this position, not a real wrapping tree node — this
; grammar never materializes a separate `expression_statement` node for a
; bare top-level assignment (confirmed via direct `tree_sitter::Query`
; testing, not node-types.json alone: `(module (expression_statement) @x)`
; captures a node whose *kind* is `assignment`, not `expression_statement`).
; Nesting a query through it as `(module (expression_statement (assignment
; ...)))` therefore never matches anything — there is no real containment
; relationship to satisfy — so the previous form of this rule silently
; matched zero module-level constants, ever, in any Python file. Matching
; `(module (assignment ...))` directly is the only correct way to scope to
; true module-level assignments (and was verified to correctly exclude
; function-local assignments, unlike the naive unscoped `(expression_statement
; (assignment ...))` form, which matches assignments at any depth).
(module (assignment left: (identifier) @name) @definition.constant)

; Tuple/list-unpacking module-level constants: A, B = 1, 2
(module (assignment left: (pattern_list (identifier) @name)) @definition.constant)

(class_definition
  name: (identifier) @name) @definition.class

(function_definition
  name: (identifier) @name) @definition.function

(call
  function: [
      (identifier) @name
      (attribute
        attribute: (identifier) @name)
      ; Subscript-dispatched call: handlers["key"](), TABLE[i]()
      (subscript
        value: [
          (identifier) @name
          (attribute attribute: (identifier) @name)
        ])
  ]) @reference.call
