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

; import module/path.{Type as Alias, function as alias} — unqualified_import
; has its own optional `alias` field (constructor_name/identifier/
; type_identifier, same variants as `name`), distinct from the whole-import
; `alias:` field above. Verified common in real Gleam: the wisp.gleam sample
; itself imports `type Response as HttpResponse` and
; `type Request as HttpRequest`.
(import
  module: (module) @import.path
  imports: (unqualified_imports
    (unqualified_import
      name: (_) @import.name
      alias: (_) @import.alias))) @import
