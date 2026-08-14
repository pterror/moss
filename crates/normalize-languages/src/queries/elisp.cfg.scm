; Emacs Lisp CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Emacs Lisp has a small, fixed built-in keyword set (if/cond/while/and/or/
; condition-case/let/setq/lambda/...) whose forms parse as a distinct
; `special_form` node with the keyword as a literal anonymous token — NOT
; `(list (symbol) ...)`. Every other named control-flow-shaped form (when,
; unless, dolist, dotimes, cl-loop, pcase, cl-case, until,
; condition-case-unless-debug, ignore-errors, ...) is *not* in that
; built-in set and stays an ordinary `list` with a `(symbol)` head.
; Verified per-keyword via `normalize syntax ast`. The previous version of
; this file matched every one of these forms via `(list . (symbol) @_fn
; ...)`, which only ever matches the list-headed forms — "if", "cond",
; "while", and "condition-case" are all special_form-headed and were
; entirely invisible to CFG construction (confirmed via `normalize syntax
; query`: a function built entirely around a `while` loop produced zero
; @cfg.loop matches).

; ---------------------------------------------------------------------------
; if (branch) — special_form-headed
; ---------------------------------------------------------------------------

(special_form
  .
  "if"
  .
  (_) @cfg.branch.condition
  .
  (_) @cfg.branch.then
  .
  (_)? @cfg.branch.else
) @cfg.branch

; when / unless (branch without else) — list-headed
(list
  .
  (symbol) @_fn
  .
  (_) @cfg.branch.condition
  (#match? @_fn "^(when|unless)$")
) @cfg.branch

; ---------------------------------------------------------------------------
; cond (special_form-headed) / case / cl-case / pcase (list-headed) (match-like)
; ---------------------------------------------------------------------------

(special_form
  .
  "cond"
) @cfg.match

(list
  .
  (symbol) @_fn
  (#match? @_fn "^(case|cl-case|pcase|pcase-exhaustive)$")
) @cfg.match

; ---------------------------------------------------------------------------
; while (special_form-headed) / until / dotimes / dolist / cl-loop
; (list-headed) (loop constructs)
; ---------------------------------------------------------------------------

(special_form
  .
  "while"
  .
  (_) @cfg.loop.condition
) @cfg.loop

(list
  .
  (symbol) @_fn
  .
  (_) @cfg.loop.condition
  (#match? @_fn "^(until|dotimes|dolist|cl-loop|cl-do|cl-dolist)$")
) @cfg.loop

; ---------------------------------------------------------------------------
; condition-case (special_form-headed) / condition-case-unless-debug /
; ignore-errors (list-headed) (exception handling)
; ---------------------------------------------------------------------------

(special_form
  .
  "condition-case"
  .
  (_)
  .
  (_) @cfg.try.body
) @cfg.try

(list
  .
  (symbol) @_fn
  .
  (_)
  .
  (_) @cfg.try.body
  (#match? @_fn "^(condition-case-unless-debug|ignore-errors)$")
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; throw / error / signal — all list-headed (not in the built-in
; special-form keyword set), verified via `normalize syntax ast`.
(list
  .
  (symbol) @_fn
  (#match? @_fn "^(throw|error|signal|user-error)$")
) @cfg.exit.throw

; return
(list
  .
  (symbol) @_fn
  (#eq? @_fn "cl-return")
) @cfg.exit.return
