; PowerShell tags query

; function_statement has no name: field; function_name is a child node
(function_statement
  (function_name) @name) @definition.function

; class_statement has no name: field; simple_name is a child node
(class_statement
  (simple_name) @name) @definition.class

(enum_statement
  (simple_name) @name) @definition.type

; class_method_definition has no name: field either; simple_name is a
; child. Note the constructor's simple_name and the return-type simple_name
; are BOTH present as children for typed methods ([string] Greet() {...}) —
; verified via `normalize syntax query` that the FIRST (simple_name) child
; in source order is always the method name, never the return type's
; type_identifier (that lives inside type_literal/type_spec/type_name, a
; different node type), so this pattern is unambiguous.
(class_method_definition
  (simple_name) @name) @definition.method

; enum_member has no name: field; simple_name is a child. Tagged as
; @definition.constant to match the convention used by kotlin.tags.scm's
; enum_entry and swift.tags.scm's enum_entry (both enum-member-as-constant).
(enum_member
  (simple_name) @name) @definition.constant

; class_property_definition (e.g. `[int]$Precision`) is intentionally NOT
; tagged here: no other language's tags.scm in this workspace taxonomizes
; plain data fields/properties as their own @definition.* kind (verified via
; grep over crates/normalize-languages/src/queries/*.tags.scm — the
; vocabulary only goes down to function/class/method/module/type/var/
; interface/constant/macro/enum/heading), so adding one here would be a new,
; unprecedented category rather than filling an existing gap.
