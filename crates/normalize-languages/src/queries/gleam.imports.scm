; Gleam imports query
; @import       — the entire import statement (for line number)
; @import.path  — the module path being imported
; @import.name  — a single unqualified import name
; @import.alias — module alias

; import module/path
(import
  module: (module) @import.path) @import

; import module/path as alias
(import
  module: (module) @import.path
  alias: (identifier) @import.alias) @import

; import module/path.{Type, function}
(import
  module: (module) @import.path
  imports: (unqualified_imports
    (unqualified_import
      name: (_) @import.name))) @import
