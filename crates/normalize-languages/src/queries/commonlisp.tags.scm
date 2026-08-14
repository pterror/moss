; Common Lisp tags query
;
; In the CL grammar, `defun`/`defgeneric`/`defmethod`/`defmacro` all parse as
; a SINGLE unified `defun` grammar node — confirmed via `normalize syntax
; query` against arborium-commonlisp 2.17.0, which showed all four forms
; producing a `defun` node whose `defun_header.keyword` field (a
; `defun_keyword` node) holds the actual form name as text. The previous
; version of this file assumed `defmethod`/`defmacro` still parsed as plain
; `list_lit` with a leading `sym_lit` keyword (as `defclass`/`defstruct`/
; etc. still do below) — that assumption was wrong for this grammar version,
; so the dedicated `defmethod`/`defmacro` list_lit patterns never matched
; anything, and BOTH forms fell through to being tagged
; @definition.function (via the unqualified `defun`-node pattern) instead of
; @definition.method/@definition.macro. Fixed by discriminating on the
; `keyword` field's text.
;
; `function_name:` also allows a `list_lit` (not just `sym_lit`), for
; setf-expander forms like `(defun (setf point-x) (new-value p) ...)` /
; `(defmethod (setf point-y) (new-value p) ...)` — a real, idiomatic CL
; construct (defining a custom setf place). The previous query only handled
; `(sym_lit) @name`, silently dropping every setf-form definition's name
; capture. `@name` below matches `(_)` to cover both shapes.

; (defun name ...) — plain function
(defun
  (defun_header
    keyword: (defun_keyword) @_kw (#eq? @_kw "defun")
    function_name: (_) @name)) @definition.function

; (defgeneric name ...)
(defun
  (defun_header
    keyword: (defun_keyword) @_kw (#eq? @_kw "defgeneric")
    function_name: (_) @name)) @definition.function

; (defmethod name ...) / (defmethod name :before ...) / setf-expander methods
(defun
  (defun_header
    keyword: (defun_keyword) @_kw (#eq? @_kw "defmethod")
    function_name: (_) @name)) @definition.method

; (defmacro name ...)
(defun
  (defun_header
    keyword: (defun_keyword) @_kw (#eq? @_kw "defmacro")
    function_name: (_) @name)) @definition.macro

; (defclass name ...)
(list_lit
  .
  (sym_lit) @_kw (#eq? @_kw "defclass")
  .
  (_) @name) @definition.class

; (defstruct name ...)
(list_lit
  .
  (sym_lit) @_kw (#eq? @_kw "defstruct")
  .
  (sym_lit) @name) @definition.class

; (defpackage name ...)
(list_lit
  .
  (sym_lit) @_kw (#eq? @_kw "defpackage")
  .
  (_) @name) @definition.module

; (deftype name ...)
(list_lit
  .
  (sym_lit) @_kw (#eq? @_kw "deftype")
  .
  (sym_lit) @name) @definition.type

; (defconstant name ...)
(list_lit
  .
  (sym_lit) @_kw (#eq? @_kw "defconstant")
  .
  (sym_lit) @name) @definition.constant

; (defparameter name ...)
(list_lit
  .
  (sym_lit) @_kw (#eq? @_kw "defparameter")
  .
  (sym_lit) @name) @definition.constant
