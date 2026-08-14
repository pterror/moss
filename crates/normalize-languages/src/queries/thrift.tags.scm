; Thrift tags query
; @name            — the symbol name
; @definition.*    — the definition node

; Struct definitions.
; struct_definition's name identifier is exposed as field `type` (grammar
; quirk — verified via `normalize syntax query`, not a copy/paste error).
(struct_definition
  type: (identifier) @name) @definition.class

; Union definitions (same `type`-field shape as struct_definition).
(union_definition
  type: (identifier) @name) @definition.class

; Exception definitions. exception_definition has no named fields at all
; (verified against node-types.json), so the name is matched positionally.
; Safe because "message"/other field identifiers are nested inside `field`
; child nodes, not direct children — and exception modifiers (`safe`,
; `transient`, `permanent`, `client`, `server`) precede the `"exception"`
; keyword, not the identifier, so they don't interfere.
(exception_definition
  "exception" (identifier) @name) @definition.class

; Enum definitions. MUST use the `type` field, not a positional match:
; enum_definition's direct children also include one `identifier` per enum
; value (e.g. `ACTIVE`, `INACTIVE` in `enum Status { ACTIVE = 1, ... }`), so
; `(enum_definition "enum" (identifier) @name)` (the prior, positional form)
; matched every enum value as an additional @definition.class with the same
; span as the enum itself — a real, verified false-positive bug.
(enum_definition
  type: (identifier) @name) @definition.class

; Legacy string enum (`senum Color { "RED", "GREEN" }`, deprecated but still
; parsed by the grammar and present in older Apache/Facebook Thrift IDL).
; Same `type`-field shape as enum_definition.
(senum_definition
  type: (identifier) @name) @definition.class

; Service definitions (interface-like containers).
; service_definition's `type` field is `multiple: true` in node-types.json:
; it covers BOTH the service's own name AND, when present, the identifier
; in an `extends` clause (`service Derived extends Base { ... }`) — verified
; via `normalize syntax query`, both identifiers carry the same field name.
; The `.` anchor immediately after the `"service"` token restricts the match
; to the service's own name; without it the query yielded two @name captures
; (`Derived` and `Base`) for the single Derived @definition.interface node.
(service_definition
  "service" . (identifier) @name) @definition.interface

; The `extends` target of a service, tagged as a reference (mirrors the
; class-extends convention in e.g. java.tags.scm's `@reference.class`).
(service_definition
  "extends" . (identifier) @name) @reference.interface

; Interaction definitions (Thrift's RPC "interaction" construct — a
; service-like container of function_definitions, entered via a service's
; `performs` statement). Same `type`-field shape as service_definition.
(interaction_definition
  type: (identifier) @name) @definition.interface

; Function definitions (methods inside services/interactions).
; function_definition has no named fields; its only direct `identifier`
; child is the function name (the return type is wrapped in a `type` child
; node, and parameter names are nested inside `parameters` -> `parameter`),
; so the positional match is unambiguous — verified including `oneway`
; modifiers and `throws` clauses.
(function_definition
  (identifier) @name) @definition.function

; Typedef definitions
(typedef_definition
  (typedef_identifier) @name) @definition.type

; Constant definitions. const_definition has no named fields; its only
; direct `identifier` child is the constant name (the declared type is
; wrapped in a `type` child, and any identifier-valued literal — e.g.
; `const Status s = Status.ACTIVE` referencing an enum value — is nested
; inside a `literal` child), so the positional match is unambiguous.
(const_definition
  (identifier) @name) @definition.constant
