; Complexity query for Emacs Lisp
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Emacs Lisp is a Lisp, but NOT every list form is a control-flow construct
; — ordinary function calls and data literals are `list` nodes too. The
; previous version of this file was a bare `(list) @complexity` /
; `(list) @nesting`, which counted every single parenthesized expression in
; the file as increasing complexity/nesting — verified via `normalize
; syntax query`: a zero-branch function like `(defun add (a b) (+ a b))`
; produced 2 false @complexity/@nesting hits (the parameter list `(a b)`
; and the call `(+ a b)`), and this scales with every call/data list in
; every function body, massively inflating the metric for all Emacs Lisp
; code.
;
; The grammar additionally has a small, fixed built-in keyword set
; (if/cond/while/and/or/condition-case/let/setq/lambda/...) whose forms
; parse as a distinct `special_form` node with the keyword as a literal
; anonymous token — NOT `(list (symbol) ...)`. Every other named
; control-flow-shaped form (when, unless, dolist, dotimes, cl-loop, pcase,
; cl-case, until, ...) is *not* in that built-in set and stays an ordinary
; `list` with a `(symbol)` head. Verified per-keyword via `normalize syntax
; ast`. The pre-existing `(list . (symbol) @_fn ...)` shape only ever
; matched the list-headed forms — "while", the single most fundamental
; Lisp loop construct, is special_form-headed and was silently invisible
; to both complexity and CFG construction (see elisp.cfg.scm) before this
; fix.

; Complexity nodes: special_form-headed branch/loop/boolean forms.
(special_form
  .
  ["if" "cond" "while" "and" "or" "condition-case"]
) @complexity

; Complexity nodes: list-headed forms (not in the grammar's built-in
; special-form keyword set, so they parse with an ordinary `(symbol)` head).
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(when|unless|case|cl-case|pcase|pcase-exhaustive|dolist|dotimes|until|cl-loop|cl-do|cl-dolist|condition-case-unless-debug|ignore-errors)$")
) @complexity

; Nesting nodes: the same branch/loop/exception forms as containers, plus
; function/macro definitions and lambda (closures increase nesting the same
; way a nested function body does in other languages).
(special_form
  .
  ["if" "cond" "while" "condition-case" "lambda"]
) @nesting

(list
  .
  (symbol) @_fn
  (#match? @_fn "^(when|unless|case|cl-case|pcase|pcase-exhaustive|dolist|dotimes|until|cl-loop|cl-do|cl-dolist|condition-case-unless-debug|ignore-errors)$")
) @nesting

(function_definition) @nesting
(macro_definition) @nesting
