; Haskell imports query
; @import       — the entire import declaration (for line number)
; @import.path  — the module name (first module_id in module path)
; @import.name  — a single imported name
; @import.alias — module alias (as M)

; import Data.Map / import qualified Data.Map
; Matches any import with a module
(import
  module: (module
    (module_id) @import.path)) @import

; import Data.Map as M / import qualified Data.Map as M
; Matches imports with an alias
(import
  module: (module
    (module_id) @import.path)
  alias: (module
    (module_id) @import.alias)) @import

; Named imports: import Data.List (sort, nub) / import Data.Map (Map, empty)
; `import.names` is an `import_list` of `import_name` nodes; each `import_name`
; carries the actual identifier under one of three mutually-exclusive fields
; depending on what's imported — a plain value/function (`variable`), a
; qualified re-export inside the list (`qualified`), a type/class name
; (`name`/`qualified`), or an operator (`prefix_id`, e.g. `(<|>)`). This
; capture was entirely absent before: named imports were never extracted.
(import_name
  variable: (variable) @import.name)
(import_name
  variable: (qualified
    id: (variable) @import.name))
(import_name
  type: (name) @import.name)
(import_name
  type: (qualified
    id: (name) @import.name))
(import_name
  operator: (prefix_id) @import.name)

; import Data.Set hiding (map, filter) / import Prelude hiding (lookup)
; `hiding` imports use the same `import_list`/`import_name` shape as normal
; named imports — the clauses above already cover both. There is no separate
; grammar node distinguishing "hiding" from an ordinary import list, so no
; @import.hiding capture is possible from the CST alone (the `hiding` keyword
; is a plain anonymous token on the `import` node, not a field).
