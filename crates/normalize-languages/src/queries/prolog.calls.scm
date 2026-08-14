; Prolog calls query
; @call — predicate/functor invocation (goal)
; @call.qualifier — not applicable
;
; Prolog's grammar is a generic term reader: a clause HEAD and a genuine
; predicate CALL are both, structurally, an ordinary `functional_notation`
; (functor + arg_list) node — nothing in the CST distinguishes "this is the
; predicate being defined" from "this is a predicate being called". The
; only way to tell them apart is POSITION: a call only ever occurs in goal
; position — an operand of a goal-combinator operator (",", ";", "->", the
; prefix "\+"), the sole body of a ":-" rule, or the entire content of a
; directive_term. Anywhere else (a clause/rule HEAD, or an argument nested
; inside another term's arg_list/list_notation) a functional_notation is
; DATA — a compound term being defined or passed around, not invoked.
; Bare atoms (0-arity goals, including the cut `!`) have the exact same
; head-vs-goal / data-vs-goal ambiguity and are handled the same way.
;
; This file previously matched `(functional_notation function: (atom)
; @call)` completely unconditionally, which was wrong in two ways, both
; verified via `normalize syntax query` against probe files:
;   1. Every fact/rule HEAD was reported as a spurious call to itself (e.g.
;      `ancestor(X, Y) :- parent(X, Y).` reported "ancestor" as a call
;      alongside the real call to "parent").
;   2. Every DATA compound-term ARGUMENT was also reported as a call (e.g.
;      `use_module(library(lists))` reported "library" as a call, when
;      `library(lists)` is a plain data term, never invoked).
; Bare-atom goal calls (!, true, fail, 0-arity user predicates) were not
; handled at all before this fix — see the paired "atom-then-operator" /
; "operator-then-atom" forms below (needed because tree-sitter's ordered
; sibling patterns only match a node on the side it's written relative to
; the operator field).

; Directive: the entire content of a directive_term is executed as a goal
; at load time — no head/body split, so no positional ambiguity here.
(directive_term
  (functional_notation
    function: (atom) @call))

; Rule/directive body that is a single goal (bare functional_notation or
; bare atom): foo(X) :- bar(X). / foo :- true.
(operator_notation
  operator: (binary_operator) @_op
  .
  (functional_notation
    function: (atom) @call)
  (#eq? @_op ":-"))
(operator_notation
  operator: (binary_operator) @_op
  .
  (atom) @call
  (#eq? @_op ":-"))

; Conjunction operands (","), both sides: Goal1, Goal2
(operator_notation
  (functional_notation
    function: (atom) @call)
  operator: (comma))
(operator_notation
  operator: (comma)
  (functional_notation
    function: (atom) @call))
(operator_notation
  (atom) @call
  operator: (comma))
(operator_notation
  operator: (comma)
  (atom) @call)

; Disjunction operands (";"), both sides: Goal1 ; Goal2
(operator_notation
  (functional_notation
    function: (atom) @call)
  operator: (semicolon))
(operator_notation
  operator: (semicolon)
  (functional_notation
    function: (atom) @call))
(operator_notation
  (atom) @call
  operator: (semicolon))
(operator_notation
  operator: (semicolon)
  (atom) @call)

; If-then operands ("->"), both sides: Cond -> Then
(operator_notation
  (functional_notation
    function: (atom) @call)
  operator: (binary_operator) @_op
  (#eq? @_op "->"))
(operator_notation
  operator: (binary_operator) @_op
  (functional_notation
    function: (atom) @call)
  (#eq? @_op "->"))
(operator_notation
  (atom) @call
  operator: (binary_operator) @_op
  (#eq? @_op "->"))
(operator_notation
  operator: (binary_operator) @_op
  (atom) @call
  (#eq? @_op "->"))

; Negation as failure (\+ Goal) — a prefix operator, single operand.
(operator_notation
  operator: (prefix_operator) @_pfx
  (functional_notation
    function: (atom) @call)
  (#eq? @_pfx "\\+"))
(operator_notation
  operator: (prefix_operator) @_pfx
  (atom) @call
  (#eq? @_pfx "\\+"))
