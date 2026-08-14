; VHDL type reference query
; Captures type marks (type names) used in subtype indications.
;
; In VHDL, `type_mark` appears wherever a type is referenced —
; in signal declarations, port lists, variable declarations, etc.
; `subtype_indication` wraps type marks with optional constraints.
;
; `type_mark`'s child allows `attribute_name`, `extended_simple_name`,
; `selected_name`, or `simple_name` per node-types.json — all four verified
; against real parse output (see
; crates/normalize-languages/tests/query_fixtures.rs
; vhdl_types_completeness):
;   - simple_name: plain type name, e.g. `std_logic`
;   - extended_simple_name: extended-identifier type name, e.g.
;     `\Extended Type\`
;   - attribute_name: type attribute, e.g. `std_logic'subtype`,
;     `x'base` (the type-valued attributes)
;   - selected_name: package-qualified type, e.g. `ieee.std_logic_1164
;     .std_logic`, with a `simple_name` or `extended_simple_name` suffix

; Plain type mark: std_logic, integer, MyType
(type_mark
  (simple_name) @type.reference)

; Extended-identifier type mark: \Extended Type\
(type_mark
  (extended_simple_name) @type.reference)

; Type attribute: std_logic'subtype, x'base
(type_mark
  (attribute_name) @type.reference)

; Package-qualified type: ieee.std_logic_1164.std_logic / pkg.\My Type\
(type_mark
  (selected_name
    suffix: [(simple_name) (extended_simple_name)] @type.reference))
