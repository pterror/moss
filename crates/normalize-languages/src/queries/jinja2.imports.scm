; Jinja2 imports query for tree-sitter-jinja2 (normalize grammar)
; @import       — the entire statement (for line number)
; @import.path  — the template path being referenced
;
; Captures: extends, import, from, include
;
; `path` on all four statement types is grammar-typed as the full 23-variant
; expression union (node-types.json), not just `string` — Jinja2 supports
; dynamic template selection: {% extends parent_template %},
; {% include "prefix_" ~ suffix %}, {% include templates[key] %}. Verified
; via `normalize syntax query` against a probe file: `path: (identifier)`,
; `path: (concat_expression)` both produce real matches with the expected
; field name. Using the wildcard (_) here (matching the convention already
; used for other wildcard-typed fields like `iterable: (_)` in other
; languages' queries) covers the whole union instead of only enumerating
; `string`, which silently dropped every dynamic-path statement.

; {% extends "base.html" %}
; {% extends parent_template %}
(extends_statement
  path: (_) @import.path) @import

; {% import "macros.html" as m %}
; {% import module_name as m %}
(import_statement
  path: (_) @import.path) @import

; {% from "helpers.html" import helper1 %}
; {% from module_name import helper1 %}
(from_statement
  path: (_) @import.path) @import

; {% include "header.html" %}
; {% include "optional.html" ignore missing %}
; {% include header_template %}
(include_statement
  path: (_) @import.path) @import
