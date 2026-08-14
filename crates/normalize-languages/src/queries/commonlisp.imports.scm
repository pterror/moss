; Common Lisp imports query
; @import       — the entire require/use-package form (for line number)
; @import.path  — the package being imported

; (require 'package) or (require "package")
; Anchored `.` on the first argument only: `require` takes an optional
; second `pathname-list` argument, and an unanchored `(_) @import.path` (as
; this file previously had) would capture that too as a bogus second path.
(list_lit
  (sym_lit) @_f (#eq? @_f "require")
  .
  (_) @import.path) @import

; (use-package :package)
(list_lit
  (sym_lit) @_f (#eq? @_f "use-package")
  .
  (_) @import.path) @import

; (ql:quickload :package) — package-qualified symbols like `ql:quickload`
; parse as a distinct `package_lit` node (with `package`/`symbol` fields),
; NOT `sym_lit` — confirmed via `normalize syntax query`. The previous
; `(sym_lit) @_f (#eq? @_f "ql:quickload")` pattern required node kind
; `sym_lit`, so it could never match: every `(ql:quickload ...)` form —
; THE standard Quicklisp package-loading idiom — was silently dropped from
; import extraction.
(list_lit
  (package_lit) @_f (#eq? @_f "ql:quickload")
  .
  (_) @import.path) @import
