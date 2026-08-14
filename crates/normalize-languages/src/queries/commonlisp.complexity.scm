; Complexity query for Common Lisp
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Common Lisp is a Lisp — every form (function call, data literal, control
; flow) is a `list_lit` node with no field distinguishing them. The
; previous version of this file was a bare `(list_lit) @complexity` /
; `(list_lit) @nesting`, which counted every single parenthesized
; expression in the file as increasing complexity/nesting — the same bug
; found and fixed in elisp.complexity.scm/scheme.complexity.scm (elisp and
; scheme are Lisp-family too): a zero-branch function like `(defun add (a
; b) (+ a b))` scored non-zero because the parameter list `(a b)` and the
; call `(+ a b)` are both `list_lit` nodes too, and this scales with every
; call/data list in every function body.
;
; Head-symbol set drawn from commonlisp.cfg.scm's existing branch/loop/
; exception coverage (if/when/unless/cond/case/ecase/do/do*/dolist/
; dotimes/loop/handler-case/ignore-errors/with-simple-restart), extended
; with the CL standard's correctable-error siblings ccase/ctypecase and
; typecase/etypecase's own match forms, handler-bind (the non-restarting
; sibling of handler-case, same shape), unwind-protect (an exception-shaped
; control construct), and short-circuit and/or — all verified via
; `normalize syntax query`/`normalize syntax ast` against probe forms to
; still parse as ordinary `list_lit` with a leading `(sym_lit)` naming the
; form (CLHS 3.1.2.1.1 special operators + these macros).
;
; `loop` is the ONE exception: unlike every other form here, CL's `loop`
; macro parses as its own dedicated `loop_macro` grammar node (containing a
; `loop` keyword child plus `loop_clause` children), NOT as a `list_lit`
; headed by a `(sym_lit) "loop"` — confirmed via `normalize syntax ast`
; against `(loop for i from 1 to n collect i)`, and via `normalize syntax
; query` showing `(list_lit . (sym_lit) @h (#eq? @h "loop"))` finds zero
; matches against a real loop form. `loop` is matched as `(loop_macro)`
; directly below, not folded into the list_lit `#match?` pattern (where it
; would silently never fire — noted as a pre-existing dead alternative in
; commonlisp.cfg.scm's own loop pattern, not fixed here since this file is
; scoped to complexity.scm).

; Complexity nodes: branch forms (if adds a branch regardless of else)
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(if|when|unless)$")
) @complexity

; Complexity nodes: match-like forms (cond/case and their correctable and
; type-dispatch siblings)
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(cond|case|ecase|ccase|typecase|etypecase|ctypecase)$")
) @complexity

; Complexity nodes: loop forms (list_lit-headed)
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(do|do\\*|dolist|dotimes)$")
) @complexity

; Complexity nodes: loop forms (dedicated loop_macro node — see header)
(loop_macro) @complexity

; Complexity nodes: exception-handling forms
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(handler-case|handler-bind|ignore-errors|restart-case|unwind-protect|with-simple-restart)$")
) @complexity

; Complexity nodes: short-circuit boolean operators
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(and|or)$")
) @complexity

; Nesting nodes: the same branch/loop/exception forms as containers
(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(if|when|unless|cond|case|ecase|ccase|typecase|etypecase|ctypecase|do|do\\*|dolist|dotimes|handler-case|handler-bind|ignore-errors|restart-case|unwind-protect|with-simple-restart)$")
) @nesting

(loop_macro) @nesting

; Nesting nodes: function/macro/method definitions (the unified `defun`
; node covers defun/defgeneric/defmethod/defmacro alike — see
; commonlisp.tags.scm) and lambda/local-function/lexical-scope forms,
; the same way a nested function body increases nesting in other languages.
(defun) @nesting

(list_lit
  .
  (sym_lit) @_fn
  (#match? @_fn "^(lambda|let|let\\*|flet|labels)$")
) @nesting
