; Type reference query for D
; Captures type identifiers used in declarations and parameters.
;
; D's grammar is unfielded and `qualified_identifier` is used both as a type
; (Foo x;) AND as an ordinary expression (foo(), obj.method(), writeln("x")).
; A bare `(qualified_identifier) @type.reference` therefore matched every
; call target and member access as a "type" — verified via
; `normalize syntax query` against a probe file: `foo()`, `obj.method()` and
; `writeln("x")` all produced false-positive @type.reference captures. The
; fix constrains the match to the structural positions where the grammar
; actually uses qualified_identifier as a type: directly under var
; declarations, parameters, function return types, alias declarations, and
; the `type`/`basic_type` wrapper nodes (which cover cast targets, `new`
; targets, and everywhere else a type appears). Nested qualified_identifier
; chains (e.g. `std.container.Array!int`) only match at the outermost level
; since inner segments are children of qualified_identifier, not of one of
; these containers — avoiding duplicate captures for the same reference.

(var_declarations
  (qualified_identifier) @type.reference)

(parameter
  (qualified_identifier) @type.reference)

(func_declaration
  (qualified_identifier) @type.reference)

(alias_declaration
  (qualified_identifier) @type.reference)

(alias_assignment
  (qualified_identifier) @type.reference)

(type
  (qualified_identifier) @type.reference)

(basic_type
  (qualified_identifier) @type.reference)
