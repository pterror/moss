; Python type references (PEP 484/585/604/612/646/695 annotations)
; Captures identifiers used in type annotation positions.
;
; `type`'s node-types.json `children` field lists: constrained_type,
; expression (→ identifier/attribute/... via the primary_expression
; supertype), generic_type, member_type, splat_type, union_type. Verified
; each variant against real parse output (`normalize syntax ast`) rather
; than trusting node-types.json alone — one listed variant (`union_type`)
; turned out to never be produced by this grammar version; `X | Y` parses
; as a plain `binary_operator`, not `union_type`.

; Type annotations on parameters and variables: x: Foo
(type
  (identifier) @type.reference)

; Dotted type annotations: x: foo.Bar
(type
  (attribute
    object: (identifier) @type.reference
    attribute: (identifier) @type.reference))

; Multi-segment dotted type annotations: x: foo.bar.Baz (verified via real
; parse output: `attribute.object` nests as another `attribute`, not just
; `identifier`, so the single-level rule above misses 3+-segment chains).
(type
  (attribute
    object: (attribute
      object: (identifier) @type.reference
      attribute: (identifier) @type.reference)
    attribute: (identifier) @type.reference))

; Generic type annotations: List[int], Optional[str], Dict[str, int],
; and PEP 695 generic aliases: Box[T]. `generic_type`'s base name is a
; bare `identifier` child (not wrapped in another `type` node), so it's
; otherwise invisible to the two rules above; the bracketed type arguments
; are each individually wrapped in `(type ...)` and so are already picked
; up recursively by the rules above/below without any extra pattern.
(generic_type
  (identifier) @type.reference)

; Dotted-module generic annotations: typing.List[int], typing.Dict[str, x.Y]
; Unlike the bare-identifier form above, `typing.List[int]` parses as a
; `subscript` (not `generic_type`) with a `value`/`subscript` field pair —
; confirmed via real parse output, since node-types.json's `type.children`
; list doesn't call this out as a distinct case from `generic_type`.
(type
  (subscript
    value: [
      (identifier) @type.reference
      (attribute attribute: (identifier) @type.reference)
    ]
    subscript: [
      (identifier) @type.reference
      (attribute attribute: (identifier) @type.reference)
    ]))

; PEP 604 union types: int | str, int | str | None
; Parses as (possibly chained/nested) `binary_operator` with a `|`
; operator, scoped here to being directly reachable from a `(type ...)`
; ancestor to avoid misfiring on ordinary runtime bitwise-or expressions
; (e.g. `re.IGNORECASE | re.MULTILINE`), which use the identical
; `binary_operator` node shape outside annotation position. Covers 2- and
; 3-member unions (by far the common case); a 4+-member chain nests one
; level deeper than handled here and is a known, deliberately-undone edge
; case — vanishingly rare in real annotations.
(type
  (binary_operator
    left: [
      (identifier) @type.reference
      (attribute attribute: (identifier) @type.reference)
      (binary_operator
        left: [
          (identifier) @type.reference
          (attribute attribute: (identifier) @type.reference)
        ]
        right: [
          (identifier) @type.reference
          (attribute attribute: (identifier) @type.reference)
        ])
    ]
    right: [
      (identifier) @type.reference
      (attribute attribute: (identifier) @type.reference)
    ]))

; PEP 695/646/612 variadic/paramspec type parameters: def f[*Ts](...),
; def f[**P](...)
(type
  (splat_type
    (identifier) @type.reference))

; Callable argument-list generics: Callable[[int, str], bool],
; Callable[[*Ts], bool] — the argument list is a plain `list` node nested
; inside `generic_type`'s `type_parameter`, scoped explicitly to that
; position so ordinary list literals elsewhere in the code are never
; mistaken for type references.
(generic_type
  (type_parameter
    (type
      (list
        [
          (identifier) @type.reference
          (attribute attribute: (identifier) @type.reference)
          (list_splat (identifier) @type.reference)
        ]))))

; NOTE: `member_type` (listed in node-types.json as a `type.children`
; variant) was not reproducible against any realistic annotation this
; investigation tried (dotted names consistently parse as `attribute` or
; `subscript`, never `member_type`); left unhandled per "don't add a
; clause for a shape the grammar doesn't actually produce."

; Class definitions
(class_definition name: (identifier) @name) @definition.type
