; Elm type reference query
; Captures type constructor references used in type annotations and expressions.
;
; Elm is statically typed. Type names are uppercase identifiers.
; `type_ref` nodes (inside `type_expression`) represent concrete type references.
; `type_ref`'s own child is a single `upper_case_qid`, whose children are a
; flat, dot-separated sequence of `upper_case_identifier`s (`Html`,
; `Html.Attribute`, `Json.Decode.Decoder`, …) — every segment is captured,
; consistent with treating a qualifier segment (`Html` in `Html.Attribute`)
; as itself a valid, searchable reference.
;
; Correctness bug fixed here (verified via `normalize syntax query`): the
; OLD version had a second, separate pattern hardcoding exactly TWO
; `(upper_case_identifier)` children to handle the 2-segment qualified case
; (`Html.Attribute`). For a 3+-segment qualified name
; (`Json.Decode.Decoder`), tree-sitter's repeated-field matching for that
; 2-identifier pattern produced every OVERLAPPING adjacent pair
; (`Json,Decode` AND `Decode,Decoder`), and the single-identifier pattern
; ALSO matched each identifier again — multiplying `Decode` into 3 separate
; captures and `Json`/`Decoder` into 2 each, for what should be one capture
; per identifier. `type_ref`/`upper_case_qid` recurse into any nesting
; depth (`List (Maybe Int)`, record/tuple type parts) without needing a
; separate pattern per depth, since tree-sitter matches every `type_ref`
; node position in the tree independently — a single, unqualified pattern
; is both correct and sufficient.
(type_ref
  (upper_case_qid
    (upper_case_identifier) @type.reference))
