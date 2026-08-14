; OCaml imports query
; @import       — the entire open/include statement (for line number)
; @import.path  — the module path being opened/included

; open Module
; open Module.Sub
; open! Module (unused-open warning suppressed — the `!` is an anonymous
; token, not a node, so this pattern already covers it)
(open_module
  (_) @import.path) @import

; include Module — merges Module's whole signature into the current
; structure/signature, unqualified (a stronger form of `open`). Was entirely
; unhandled: `include_module` is a distinct node type from `open_module`.
; Also covers functor-application includes: `include Make(X)`.
(include_module
  (_) @import.path) @import
