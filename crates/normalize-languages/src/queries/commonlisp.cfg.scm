; Common Lisp CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Common Lisp grammar node types.
;
; Common Lisp is a Lisp — most forms are list_lit nodes with a leading
; symbol. We match specific branching forms: if, when, unless, cond, case,
; ecase, do, dolist, dotimes. throw/return-from are exit forms.
;
; `loop` is the ONE exception: unlike every other form here, CL's `loop`
; macro parses as its own dedicated `loop_macro` grammar node (a `loop`
; keyword plus `loop_clause` children), NOT as a `list_lit` headed by a
; `(sym_lit) "loop"` — confirmed via `normalize syntax ast`/`normalize
; syntax query` against `(loop for i from 1 to n collect i)`: the
; `list_lit . (sym_lit) @h (#eq? @h "loop")` shape matches zero nodes, so
; `loop` as an alternative in the list_lit `#match?` regex below was
; unreachable dead weight (same finding already documented in
; commonlisp.complexity.scm, see 19261a0a). Matched below as its own
; `(loop_macro)` pattern instead.

; ---------------------------------------------------------------------------
; if (branch)
; ---------------------------------------------------------------------------

(list_lit
  .
  (sym_lit) @_fn
  .
  (_) @cfg.branch.condition
  .
  (_) @cfg.branch.then
  .
  (_)? @cfg.branch.else
  (#eq? @_fn "if")
) @cfg.branch

; when / unless (branch without else)
(list_lit
  .
  (sym_lit) @_fn
  .
  (_) @cfg.branch.condition
  (#match? @_fn "^(when|unless)$")
) @cfg.branch

; ---------------------------------------------------------------------------
; cond / case / ecase (match-like)
; ---------------------------------------------------------------------------

(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(cond|case|ecase|typecase|etypecase)$")
) @cfg.match

; ---------------------------------------------------------------------------
; do / dolist / dotimes (loop constructs, list_lit-headed)
; ---------------------------------------------------------------------------

(list_lit
  .
  (sym_lit) @_fn
  .
  (_) @cfg.loop.condition
  (#match? @_fn "^(do|do\\*|dolist|dotimes)$")
) @cfg.loop

; loop (dedicated loop_macro node — see header)
(loop_macro) @cfg.loop

; ---------------------------------------------------------------------------
; handler-case / handler-bind / ignore-errors (exception handling)
; ---------------------------------------------------------------------------

(list_lit
  .
  (sym_lit) @_fn
  .
  (_) @cfg.try.body
  (#match? @_fn "^(handler-case|ignore-errors|with-simple-restart)$")
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; throw / error / signal
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(throw|error|signal|cerror)$")
) @cfg.exit.throw

; return / return-from
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(return|return-from)$")
) @cfg.exit.return
