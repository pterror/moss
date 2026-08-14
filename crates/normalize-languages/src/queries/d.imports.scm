; D imports query
; @import       — the entire import declaration (for line number)
; @import.path  — the module being imported
; @import.alias — module alias binding (import io = std.stdio;)

; import std.stdio;  and  import io = std.stdio; (module alias form, the
; alias capture is optional so both shapes match with a single pattern and
; no duplicate @import per declaration).
(import_declaration
  (import_list
    (import
      (module_alias_identifier)? @import.alias
      (module_fully_qualified_name) @import.path))) @import

; import std.math : sqrt;  (bindings form)
(import_declaration
  (import_list
    (import_bindings
      (import
        (module_fully_qualified_name) @import.path)))) @import
