; ReScript imports query
; @import       — the entire open statement (for line number)
; @import.path  — the module being opened

; open Module
; open Module.Sub
(open_statement
  (_) @import.path) @import

; include Module — a distinct statement/node type from `open` (inlines a
; module's contents rather than just bringing names into scope), entirely
; unhandled before this fix. Restricted to the `module_expression` child
; (the common `include Module`/`include Module.Sub` form); `functor`,
; `block`, and `extension_expression` children (functor application,
; anonymous module bodies, decorator-wrapped includes) have no single
; static path worth reporting.
(include_statement
  (module_expression) @import.path) @import
