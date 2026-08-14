; PowerShell type reference query
; Captures type names used in type literals and cast expressions.
;
; PowerShell uses [TypeName] syntax for type references (type literals).
; The `type_literal` wraps a `type_spec` which contains `type_name`.

; Type literal: [int], [string]
;
; `type_spec`'s children allow FIVE variants (verified against
; node-types.json's `type_spec.children.types` and against real parse
; output via `normalize syntax ast`/`normalize syntax query`):
;   type_name, array_type_name, dimension, generic_type_arguments,
;   generic_type_name
; Only the plain `type_name` case was handled; array types ([int[]],
; [string[]]) and the outer generic type name ([List[int]]'s "List",
; [System.Collections.Generic.List[int]]'s dotted path) were silently
; dropped — confirmed via `normalize syntax query` returning zero matches
; for `[int[]]` and for the "List"/dotted-path portion of a generic type,
; while only the inner `int` type argument (nested inside
; generic_type_arguments, itself containing an ordinary type_spec/type_name)
; was ever captured. [int[]] appears in this crate's own
; fixtures/powershell/sample.ps1 (`param([int[]]$Numbers)`), so this gap
; was already silently active in the shipped fixture.

(type_literal
  (type_spec
    (type_name) @type.reference))

; Array type: [int[]], [string[]]
(type_literal
  (type_spec
    (array_type_name
      (type_name) @type.reference)))

; Generic type name (the part before the brackets): [List[int]]'s "List",
; [System.Collections.Generic.List[int]]'s dotted path. The generic type
; argument(s) themselves ("int") are already covered by the plain
; type_spec/type_name pattern above, since generic_type_arguments wraps an
; ordinary type_spec.
(type_literal
  (type_spec
    (generic_type_name
      (type_name) @type.reference)))
