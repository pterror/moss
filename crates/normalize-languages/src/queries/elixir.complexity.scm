; Elixir complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; In Elixir's tree-sitter grammar, control flow constructs (if, unless, case,
; cond, with, for, try, receive) are represented as `call` nodes rather than
; dedicated AST nodes — they are macros, not special forms. A blanket
; `(call) @complexity` (the previous version of this file) therefore matches
; every single function call in the program, not just branching constructs —
; including `IO.puts`, `Enum.reduce`, `def`/`defmodule` themselves, and every
; other ordinary call — which grossly overcounts complexity for any function
; that calls anything at all. Verified via `normalize syntax query`: the
; call-target identifier is the only way to distinguish a genuine branching
; macro from an ordinary function call, so every complexity-contributing
; `call` pattern below is scoped to the specific keyword set that actually
; branches, mirroring how `ruby.complexity.scm` scopes to named control-flow
; node types rather than every expression.

; Branching macros: each of these represents one decision point.
(call
  target: (identifier) @_kw
  (#match? @_kw "^(if|unless|case|cond|with|for|try|receive)$")) @complexity

; Each `stab_clause` (`pattern -> body`) is an independent branch arm inside
; a `case`/`cond`/`receive`/`rescue`/`catch`/`with`-else block, or a clause
; of a multi-clause anonymous function (`fn 1 -> :a; 2 -> :b end`) — verified
; via `normalize syntax query` against case/cond/anonymous-function/rescue/
; catch samples. Counting only the parent `call` once (as above) would
; undercount a `case`/`cond` with N branches as a single decision point
; instead of N; counting each `stab_clause` independently fixes that,
; mirroring how `ruby.complexity.scm` counts each `elsif`/`in_clause`
; independently rather than folding a branch chain into one point.
(stab_clause) @complexity

; Boolean short-circuit operators genuinely add a branch (the right operand
; may not evaluate). Arithmetic/comparison operators (+, -, ==, etc.) do not
; branch and are intentionally excluded — the previous blanket
; `(binary_operator) @complexity` counted every arithmetic expression as a
; decision point, which is not what cyclomatic complexity means.
(binary_operator operator: "&&") @complexity
(binary_operator operator: "||") @complexity
(binary_operator operator: "and") @complexity
(binary_operator operator: "or") @complexity

; Nesting nodes — blocks and anonymous functions that introduce new scopes.
(call
  target: (identifier) @_kw
  (#match? @_kw "^(if|unless|case|cond|with|for|try|receive)$")) @nesting
(do_block) @nesting
(anonymous_function) @nesting
