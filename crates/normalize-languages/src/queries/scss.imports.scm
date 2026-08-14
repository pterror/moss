; SCSS imports query
; @import       — the entire @import/@use/@forward statement (for line number)
; @import.path  — the file path being imported

; @import "variables";
(import_statement
  (string_value) @import.path) @import

; @import url("theme.css");
(import_statement
  (call_expression
    (arguments
      (string_value) @import.path))) @import

; @import url(theme.css);  (bare URL without quotes)
(import_statement
  (call_expression
    (arguments
      (plain_value) @import.path))) @import

; @use "sass:math";
(use_statement
  (string_value) @import.path) @import

; @forward "mixins";
(forward_statement
  (string_value) @import.path) @import

; NOT HANDLED — grammar limitation, not a query gap: `@use "variables" as
; vars;` and `@use "variables" as *;` (namespace aliasing) both produce an
; ERROR node for the `as ...` clause in arborium-scss 2.17.0 (confirmed via
; `normalize syntax ast`). The `@import.path` string itself still matches
; correctly since it's a clean sibling of the ERROR node; only the alias
; clause is unparseable. Same for `@forward "list" as list-*;` and
; `@forward "list" show fn hide other;` — the forward-control clauses
; (show/hide/as-prefix) after the string are ERROR, but the path string
; still matches.
