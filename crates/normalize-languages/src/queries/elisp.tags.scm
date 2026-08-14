; Emacs Lisp tags query
;
; In the elisp grammar, (defun name ...) parses as a top-level function_definition
; node with a name: symbol field. Similarly (defmacro ...) parses as macro_definition.
; Other forms remain as list nodes with leading symbol keywords.

; (defun name ...)
; (defsubst name ...)
; (cl-defun name ...)
(function_definition
  name: (symbol) @name) @definition.function

; (defmacro name ...)
; (cl-defmacro name ...)
(macro_definition
  name: (symbol) @name) @definition.macro

; (defvar name ...) / (defconst name ...) — parse as special_form nodes.
;
; The previous pattern here, `(special_form . (symbol) @name)`, anchored to
; the first *named* child of ANY special_form — not just defvar/defconst.
; special_form is also the node kind for if/cond/while/and/or/
; condition-case/let/setq/catch/progn/save-excursion/lambda (a small,
; fixed built-in keyword set — see elisp.complexity.scm and elisp.cfg.scm
; for the full explanation), and the `.` anchor skips over the leading
; anonymous keyword token to match the first named child regardless of
; which keyword it is. Verified via `normalize syntax query`: this
; fabricated bogus @definition.constant tags for `(setq total ...)`
; (captured "total" — a reassignment, not a definition), `(condition-case
; err ...)` (captured "err" — a local exception-binding, not a global),
; and `(and a (or b nil))` (captured both "a" and "b" — plain references,
; not definitions at all). Anchoring to the literal "defvar"/"defconst"
; token scopes the match to the two forms this rule actually intends.
(special_form
  .
  "defvar"
  .
  (symbol) @name) @definition.constant

(special_form
  .
  "defconst"
  .
  (symbol) @name) @definition.constant

; (defcustom name value docstring :type ... :group ...) — a customizable
; variable definition, an extremely common idiom in real Emacs Lisp
; packages (arguably more common than plain defvar for package-level
; config). defcustom is not in the grammar's built-in special-form keyword
; set, so it parses as an ordinary `list` with a `(symbol)` head, verified
; via `normalize syntax ast`. Was entirely absent from this file before —
; every user-facing customization variable in a real package was dropped
; from extraction.
(list
  .
  (symbol) @_kw (#eq? @_kw "defcustom")
  .
  (symbol) @name) @definition.constant

; (cl-defstruct name ...)
(list
  (symbol) @_kw (#eq? @_kw "cl-defstruct")
  .
  (symbol) @name) @definition.class

; (defclass name ...)  — EIEIO
(list
  (symbol) @_kw (#eq? @_kw "defclass")
  .
  (symbol) @name) @definition.class
