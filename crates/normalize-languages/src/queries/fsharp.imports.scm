; F# imports query
; @import       — the entire open declaration (for line number)
; @import.path  — the namespace/module being opened

; open System.Collections
; open Microsoft.FSharp.Core
(import_decl
  (long_identifier) @import.path) @import

; KNOWN GRAMMAR LIMITATION, not fixable at the query level: F# 5+'s
; `open type System.Math` (opening a type's static members) is NOT parsed
; as a distinct construct by arborium-fsharp 2.17.0 — confirmed via
; `normalize syntax ast`: it parses `open type` as an ordinary
; `import_decl` whose `long_identifier` is the single bogus identifier
; "type", and the actual namespace (`System.Math`) is left as a
; disconnected top-level `long_identifier_or_op` expression, not attached
; to the import_decl at all (no `(ERROR)` node — the grammar "succeeds"
; with a semantically wrong tree). There is no query pattern that can
; recover the real import path from this shape since the grammar itself
; never associates it with the `open` declaration.
