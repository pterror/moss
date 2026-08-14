; Prolog tags query
; Covers: predicate definitions (facts and rules) and directives
; Prolog programs consist of clause_term nodes (facts/rules) and directive_term nodes.
; Each clause is either a plain atom, a functional_notation (functor/arity),
; or an operator_notation with :- (head :- body rule).

; Simple fact: foo.
; The clause_term contains an atom directly.
(clause_term
  (atom) @name) @definition.function

; Compound fact or rule head: foo(X, Y). or foo(X) :- bar(X).
; The functional_notation has a function: (atom) field.
(clause_term
  (functional_notation
    function: (atom) @name)) @definition.function

; Rule with :- operator: head :- body.
; The operator_notation's first child (before :-) is the head — anchored
; with `.` so this only matches the head position. Unanchored, this also
; matched a body that happens to be a bare atom/functional_notation (e.g.
; `foo :- bar.` or `findall_test(L) :- findall(...).`, where the body is a
; single goal with no wrapping comma/semicolon operator_notation): the head
; atom/functional_notation and the *entire body* are both direct children
; of the same operator_notation node, so the unanchored pattern produced a
; spurious second @name/@definition.function for the called predicate
; alongside the real one. Verified via `normalize syntax query` against a
; probe file — this doubled tags for every rule whose body is a single
; goal, a very common Prolog idiom (thin wrapper predicates).
; Head can be atom or functional_notation.
(clause_term
  (operator_notation
    .
    (atom) @name)) @definition.function

(clause_term
  (operator_notation
    .
    (functional_notation
      function: (atom) @name))) @definition.function

; Directives: :- module(foo, [...]).
; directive_term contains a directive_head node or functional_notation.
; Two bugs fixed here, both verified via `normalize syntax query`:
;   1. Unrestricted, this matched *every* directive (use_module/1,
;      use_module/2, dynamic/1, initialization/1, set_prolog_flag/2, ...),
;      not just `:- module(...)`. Restricted to functor "module" via #eq?.
;   2. Even for an actual `:- module(...)` directive, @name captured the
;      *functor* atom ("module" — always that literal string, useless as a
;      name) instead of the module's own name (its first argument). Fixed
;      by anchoring `.` on the first arg_list element instead.
(directive_term
  (functional_notation
    function: (atom) @_fn
    (arg_list
      .
      (atom) @name))
    (#eq? @_fn "module")) @definition.module
