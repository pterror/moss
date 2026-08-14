; Complexity query for Scheme
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Scheme is a Lisp — ALL forms (function calls, data literals, control
; flow) are represented as the same `list` node with no distinguishing
; field (verified via node-types.json: `list` has `"fields": {}`). The
; previous version of this file was a bare `(list) @complexity` /
; `(list) @nesting`, which counted every single parenthesized expression
; in the file — verified via `normalize syntax query`: it produced 82
; @complexity matches across the 47-line sample.scm fixture, and even a
; trivial zero-branch function like `(define (square n) (* n n))` scored
; non-zero because the parameter list `(n)` and the call `(* n n)` are
; both `list` nodes too. This mirrors the exact bug found and fixed in
; elisp.complexity.scm (elisp is Scheme-family), except elisp has a
; `special_form` node for its built-in keyword set and Scheme does not —
; every branch/loop/boolean form here is an ordinary `list` with a leading
; `(symbol)` naming the form, confirmed via `normalize syntax ast`/`normalize
; syntax query` against probe forms (if/when/unless/cond/case/do/guard/
; and/or/lambda/let).
;
; Keyword set below matches scheme.cfg.scm's branch/loop/exception
; coverage, plus `and`/`or` (short-circuit boolean operators, which the
; same elisp fix included for cyclomatic-complexity purposes).

; Complexity nodes: branch forms (if adds a branch regardless of else)
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(if|when|unless)$")
) @complexity

; Complexity nodes: match-like forms (cond/case/case-lambda)
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(cond|case|case-lambda)$")
) @complexity

; Complexity nodes: loop forms
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(do|for-each|string-for-each|vector-for-each)$")
) @complexity

; Complexity nodes: exception-handling forms
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(guard|with-exception-handler)$")
) @complexity

; Complexity nodes: short-circuit boolean operators
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(and|or)$")
) @complexity

; Nesting nodes: same branch/loop/exception forms as containers
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(if|when|unless|cond|case|case-lambda|do|for-each|string-for-each|vector-for-each|guard|with-exception-handler)$")
) @nesting

; Nesting nodes: lambda bodies (closures) and let-bound scopes are new
; nesting levels the same way a nested function body is in other languages
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(lambda|let|let\\*|letrec|letrec\\*)$")
) @nesting
