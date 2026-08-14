; Cap'n Proto imports query
; @import       — the entire using-import statement (for line number)
; @import.path  — the import path string
; @import.alias — the local name bound to the import (the `using X =` identifier)

; using Cxx = import "/capnp/c++.capnp";
;
; NOTE: node-types.json also lists a bare named `import` node type with
; `import_path`/`namespace` children, but no other node in the grammar's
; children/field lists ever nests it — verified via `normalize syntax ast`
; that every `import "..."` construct actually parses as
; `using_directive > import_using`, never a standalone `import` node. A
; query matching `(import ...)` compiles clean but matches nothing in
; practice; don't resurrect it.
(import_using
  (type_identifier) @import.alias
  (import_path) @import.path) @import
