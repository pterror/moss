; Haskell tags query
; Covers: functions, data types, newtypes, type synonyms, typeclasses, instances

; Function definitions (bind + equation sets)
; Type signatures are intentionally excluded: they are not definition sites and
; would produce duplicate symbols when a function has both a signature and a body.
;
; SCOPING: both patterns below are anchored under `(declarations ...)` — the
; container of top-level module declarations — rather than matching `function`
; anywhere in the tree. `function` is also the node type used for where/let-
; bound local helper definitions (`outer n = go n where go x = ...`), and an
; earlier, unscoped `(function name: (variable) @name)` leaked those local
; helpers in as top-level symbols (confirmed: `go`/`helper` from a `where`
; clause both appeared in `normalize view` output before this fix). Anchoring
; to `declarations` restricts matches to direct top-level children only.
(declarations
  (function
    name: (variable) @name) @definition.function)

; Operator function definitions: `(+++) xs ys = ...` — a custom infix operator
; defined via parenthesized prefix syntax. `function.name` allows `prefix_id`
; in addition to `variable`; custom operators are a pervasive Haskell idiom
; (lens, servant-style type-level combinators, etc.), not a rare edge case.
(declarations
  (function
    name: (prefix_id) @name) @definition.function)

; Zero-argument / point-free top-level definitions: `main = do ...`,
; `frequencyMap = foldr (...) Map.empty`. These use a `bind` node, not
; `function` — `function` requires at least one pattern argument at the
; grammar level. Point-free style and `main` itself (the single most
; fundamental top-level definition in any Haskell program) were both
; entirely absent from tags before this fix — confirmed via `normalize view`
; on the existing sample.hs fixture, which already contained `main` and
; `frequencyMap` and neither ever appeared in symbol output.
; Scoped to `declarations` for the same reason as `function` above: `bind` is
; also the node type for local `let`-bindings inside `do`/`where` blocks
; (e.g. `let t = insert 3 (...)` inside `main`), which must NOT be tagged as
; top-level symbols.
(declarations
  (bind
    name: (variable) @name) @definition.function)
(declarations
  (bind
    name: (prefix_id) @name) @definition.function)

; Data type declarations
(data_type
  name: (name) @name) @definition.class

; Operator data type declarations: `data (:+:) a b = L a | R b` — the
; parenthesized-prefix form of an infix type constructor. `data_type.name`
; also allows `prefix_list`/`qualified`/`unit`, but those variants only arise
; from compiler-builtin tuple/unit syntax, never hand-written user code — left
; unhandled per "verify real-world usage density" rather than fabricated.
(data_type
  name: (prefix_id) @name) @definition.class

; Newtype declarations
(newtype
  name: (name) @name) @definition.type

; Operator newtype declarations: `newtype (:*:) a = Wrap a` (see data_type
; comment above for why prefix_list/qualified/unit are left unhandled).
(newtype
  name: (prefix_id) @name) @definition.type

; Type synonym declarations
(type_synomym
  name: (name) @name) @definition.type

; Operator type synonym declarations: `type (:->) a b = a -> b`.
(type_synomym
  name: (prefix_id) @name) @definition.type

; Typeclass declarations (interfaces)
(class
  name: (name) @name) @definition.interface

; Operator typeclass declarations: `class (:~:) a b where ...`.
(class
  name: (prefix_id) @name) @definition.interface

; Instance declarations — captured as definition.module so extract_container
; can populate the implements list from the typeclass name.
(instance
  name: (name) @name) @definition.module

; Operator instance declarations: `instance (:~:) Int Int where ...`.
(instance
  name: (prefix_id) @name) @definition.module
