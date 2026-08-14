; Emacs Lisp imports query
; @import       — the entire require/load form (for line number)
; @import.path  — the feature/file being required

; (require 'feature)
(list
  (symbol) @_f (#eq? @_f "require")
  (_) @import.path) @import

; (load "file.el")
(list
  (symbol) @_f (#eq? @_f "load")
  (_) @import.path) @import

; (load-theme 'theme) — activates (and loads) a theme.
(list
  (symbol) @_f (#eq? @_f "load-theme")
  (_) @import.path) @import

; (require-theme 'theme) — Emacs 29+, the require-analog for theme files
; (declares a static dependency, byte-compiler-tracked, distinct from
; load-theme's runtime activation). The comment above previously documented
; this form but the code matched "load-theme" instead, so require-theme was
; entirely unmatched — verified via `normalize syntax query`.
(list
  (symbol) @_f (#eq? @_f "require-theme")
  (_) @import.path) @import
