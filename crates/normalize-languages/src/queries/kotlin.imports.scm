; Kotlin imports query
; @import       — the entire import header (for line number)
; @import.path  — the module/class path
; @import.alias — alias after 'as'
; @import.glob  — wildcard marker (presence means is_wildcard=true)

; import pkg.Class
; The `.` anchor is required: without it, this pattern is unconstrained and
; ALSO matches every aliased (`import pkg.Class as Alias`) and wildcard
; (`import pkg.*`) import below, since it only requires *some* `identifier`
; child, not the sole one — producing a duplicate @import per aliased/
; wildcard import statement (same bug class as the Java import fix).
(import_header
  (identifier) @import.path .) @import

; import pkg.Class // trailing comment
; `line_comment`/`multiline_comment` are declared `extra` in this grammar,
; but verified against real parse output: a comment on the same or
; following line attaches as a literal trailing CHILD of the preceding
; `import_header` (not skipped as an "extra" for anchor purposes the way
; tree-sitter's docs describe) — so a trailing anchor alone silently drops
; every plain import immediately followed by a comment. Handled with two
; explicit non-optional variants (NOT `(line_comment)? .`, which was tried
; first and reintroduced the exact duplicate-match bug above — a quantified
; sibling combined with a trailing anchor does not constrain matches the
; way it does without the `?` in this query engine).
(import_header
  (identifier) @import.path . (line_comment) .) @import

(import_header
  (identifier) @import.path . (multiline_comment) .) @import

; import pkg.Class as Alias
(import_header
  (identifier) @import.path
  (import_alias
    (type_identifier) @import.alias)) @import

; import pkg.*
(import_header
  (identifier) @import.path
  (wildcard_import) @import.glob) @import
