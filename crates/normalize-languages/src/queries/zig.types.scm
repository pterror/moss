; Type reference query for Zig
; Captures type identifiers used in structural type positions: parameter
; types, function return types, variable/const type annotations, and struct
; field types.
;
; Verified against arborium-zig 2.17.0's node-types.json and real parse
; output (`normalize syntax ast` / `normalize syntax query`).
;
; A bare `(IDENTIFIER) @type.reference` (the prior version of this file)
; matched EVERY identifier in the source — variable names, field names,
; function names, call targets, everything — because Zig's grammar reuses
; the plain `IDENTIFIER` node kind pervasively (types are first-class
; values, so there is no dedicated `type_identifier` node kind the way
; Rust or C have). Verified: 103 false-positive captures on the 45-line
; zig sample fixture alone. This mirrors the same root cause documented in
; `d.types.scm` for D's `qualified_identifier`.
;
; The fix constrains matches to the structural positions where the grammar
; actually places a type: `ParamType` (parameter types), the trailing
; `ErrorUnionExpr` of `FnProto` (return types — always exactly one, so no
; anchor is needed to disambiguate it from a value), and — for `VarDecl`
; and `ContainerField`, which can hold BOTH a type annotation AND an
; unrelated value/default expression as sibling `ErrorUnionExpr` nodes —
; anchored either immediately before the `=` token (has an
; initializer/default) or as the last child (no initializer/default,
; e.g. `extern var x: T;` or a field with no default value).
;
; For a qualified/generic type (`std.ArrayList(u8)`, `std.mem.Allocator`),
; the grammar chains `FieldOrFnCall` siblings inside the `SuffixExpr`
; (`field_access:` for plain member access, `function_call:` for the
; call-like generic-instantiation syntax `Foo(Args)` — Zig generics ARE
; ordinary function calls, so there is no distinct "generic type" node).
; Only the LAST link in the chain is captured (trailing `.` anchor within
; `SuffixExpr`, verified against a 3-level chain
; `std.mem.Allocator.Error` — captures only `Error`), matching the leaf-name
; convention `rust.types.scm` uses for `scoped_type_identifier`. The
; `PrefixTypeOp` wrapper for slices/pointers/optionals (`[]const T`, `*T`,
; `?T`) is a sibling of the type's `ErrorUnionExpr`, not an ancestor, so it
; does not interfere with any of the patterns below (verified against
; `[]const Point`, `?Point`, `*Point` in parameter and field position).

; ---------------------------------------------------------------------------
; Parameter types — ParamType always has exactly one ErrorUnionExpr child
; (the type); no ambiguity with a value expression to anchor against.
; ---------------------------------------------------------------------------

(ParamType
  (ErrorUnionExpr
    (SuffixExpr
      variable_type_function: (IDENTIFIER) @type.reference .)))

(ParamType
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall field_access: (IDENTIFIER) @type.reference) .)))

(ParamType
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall function_call: (IDENTIFIER) @type.reference) .)))

; ---------------------------------------------------------------------------
; Function return types — FnProto has exactly one ErrorUnionExpr child (the
; return type); same no-ambiguity reasoning as ParamType.
; ---------------------------------------------------------------------------

(FnProto
  (ErrorUnionExpr
    (SuffixExpr
      variable_type_function: (IDENTIFIER) @type.reference .)))

(FnProto
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall field_access: (IDENTIFIER) @type.reference) .)))

(FnProto
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall function_call: (IDENTIFIER) @type.reference) .)))

; ---------------------------------------------------------------------------
; var/const type annotations — `const x: T = value;` / `extern var x: T;`.
; VarDecl can hold a type ErrorUnionExpr AND a separate value ErrorUnionExpr
; as siblings; anchor to the type slot specifically (immediately before "="
; when an initializer follows, or last child when it doesn't).
; ---------------------------------------------------------------------------

(VarDecl
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      variable_type_function: (IDENTIFIER) @type.reference .)) . "=")

(VarDecl
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall field_access: (IDENTIFIER) @type.reference) .)) . "=")

(VarDecl
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall function_call: (IDENTIFIER) @type.reference) .)) . "=")

(VarDecl
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      variable_type_function: (IDENTIFIER) @type.reference .)) .)

(VarDecl
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall field_access: (IDENTIFIER) @type.reference) .)) .)

(VarDecl
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall function_call: (IDENTIFIER) @type.reference) .)) .)

; ---------------------------------------------------------------------------
; Struct/union field types — same type-vs-default ambiguity as VarDecl.
; ---------------------------------------------------------------------------

(ContainerField
  field_member: (_)
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      variable_type_function: (IDENTIFIER) @type.reference .)) . "=")

(ContainerField
  field_member: (_)
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall field_access: (IDENTIFIER) @type.reference) .)) . "=")

(ContainerField
  field_member: (_)
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall function_call: (IDENTIFIER) @type.reference) .)) . "=")

(ContainerField
  field_member: (_)
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      variable_type_function: (IDENTIFIER) @type.reference .)) .)

(ContainerField
  field_member: (_)
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall field_access: (IDENTIFIER) @type.reference) .)) .)

(ContainerField
  field_member: (_)
  ":"
  (ErrorUnionExpr
    (SuffixExpr
      (FieldOrFnCall function_call: (IDENTIFIER) @type.reference) .)) .)
