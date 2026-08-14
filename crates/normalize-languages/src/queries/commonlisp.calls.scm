; Common Lisp calls query
; @call — call expression nodes
; @call.qualifier — not applicable (Lisp has no method dispatch at this level)
;
; Common Lisp is a Lisp: function application is `(f args...)` where f is the
; first element of a list. In the tree-sitter grammar, this is a `list_lit`
; with a leading `sym_lit` (symbol) as the function position.
;
; We match the outer list_lit as the call context and the first sym_lit as
; @call (the callee name). Qualified symbols like `pkg:func` or `pkg::func`
; appear as `sym_lit` nodes and are captured as-is.
;
; The syntax for a function call and a special-form/macro invocation is
; IDENTICAL in a Lisp — both are `(leading-symbol args...)`. Without
; excluding known special forms and definition/binding/control-flow macros,
; every `(defpackage ...)`, `(let (...) ...)`, `(if ...)`, `(dolist ...)`
; etc. was counted as a "call" — confirmed via `normalize syntax query`
; against `sample.lisp`, where `defpackage`/`in-package`/`require`/
; `use-package`/`defstruct`/`defclass`/`if`/`let`/`dolist`/`when`/`setf` were
; all reported as calls. List drawn from Common Lisp's standard special
; operators (CLHS 3.1.2.1.1) plus the definition/binding/control-flow macros
; this codebase's own `commonlisp.tags.scm`/`commonlisp.cfg.scm` already
; treat as structural, not call, constructs.
;
; KNOWN LIMITATION, not fixed here: a lambda-list like `(n)` in `(defun
; factorial (n) ...)` is structurally IDENTICAL to a zero-arg call `(n)` —
; both are a `list_lit` whose first (and only) child is a `sym_lit`. Common
; Lisp's grammar gives no field distinguishing "this list is a binding form"
; from "this list is a call", and disambiguating would require tracking
; which parent form (defun/let/dolist/...) the list sits inside, which a
; single flat tree-sitter pattern cannot express. Confirmed via `normalize
; syntax query` that parameter names like `n`/`name`/`lst`/`pred`/`item`
; still appear as bogus @call captures after the special-form exclusion
; below. This is a genuine grammar/query-language limitation, not a gap to
; special-case away — documented per CLAUDE.md's "be honest about
; capabilities" rather than fabricated structure.
(list_lit
  .
  (sym_lit) @call
  (#not-match?
    @call
    "^(defun|defgeneric|defmethod|defmacro|defclass|defstruct|defpackage|deftype|defconstant|defparameter|defvar|in-package|require|use-package|ql:quickload|let|let\\*|flet|labels|letf|letf\\*|symbol-macrolet|macrolet|loop|do|do\\*|dolist|dotimes|if|when|unless|cond|case|ecase|ccase|typecase|etypecase|ctypecase|progn|prog1|prog2|block|return|return-from|catch|throw|unwind-protect|handler-case|handler-bind|ignore-errors|with-simple-restart|restart-case|multiple-value-bind|multiple-value-call|destructuring-bind|declare|declaim|the|quote|function|lambda|setf|setq|psetf|psetq|and|or|not)$"))

; Keyword-named call: (:method args...) — less common but valid
(list_lit
  .
  (kwd_lit) @call)
