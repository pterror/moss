; Type reference query for Erlang
; Captures type names used in -spec/-callback/-type declarations, record
; field type annotations, and `fun((...) -> ...)` type expressions.
;
; Grammar quirk (verified via `normalize syntax ast`/`normalize syntax
; query`): arborium-erlang has no distinct "type expression" grammar — a
; type reference like `integer()` parses identically to an ordinary
; function call (`call` with `expr: (atom)`), and a bare type name like
; `ok` parses identically to a bare atom literal. The only way to tell a
; type *reference* apart from an ordinary call/atom is by anchoring on the
; known type-position fields below (`ty:`/`args:`/`expr:` on the nodes that
; only appear inside -type/-spec/-callback/record-field syntax) — there is
; no node kind that means "this is definitely a type usage" on its own.
;
; `(type_name)` (the OLD version of this file's only pattern) is NOT a type
; *usage* — it's the name header of a `-type NAME(...) :: ...` declaration
; itself (a definition, already covered by erlang.tags.scm's
; `@definition.type`), so it must not be captured here as a reference.
;
; `|` unions parse as a right-nested `pipe` node (`a | b | c` ==
; `pipe(lhs: a, rhs: pipe(lhs: b, rhs: c))`), and `pipe` is a generic `_expr`
; subtype reused elsewhere (e.g. `[H | T]` cons patterns) — so it is only
; matched here anchored under a known type-position field, never bare.
; Patterns below unroll two levels of `pipe` nesting (covers 2–3-member
; unions, the common case); a 4th+ union member nested deeper than that is
; a real but unhandled edge case — a query-expressiveness limit (no
; arbitrary-depth recursion in tree-sitter query syntax), not a grammar
; limitation, since the CST itself represents it just fine.

; -type NAME() :: Ty.  (Ty is the `ty:` field of `type_alias`)
(type_alias ty: (atom) @type.reference)
(type_alias ty: (call expr: (atom) @type.reference))
(type_alias ty: (tuple expr: (atom) @type.reference))
(type_alias ty: (tuple expr: (call expr: (atom) @type.reference)))
(type_alias ty: (pipe lhs: (atom) @type.reference))
(type_alias ty: (pipe lhs: (call expr: (atom) @type.reference)))
(type_alias ty: (pipe rhs: (atom) @type.reference))
(type_alias ty: (pipe rhs: (call expr: (atom) @type.reference)))
(type_alias ty: (pipe rhs: (pipe lhs: (atom) @type.reference)))
(type_alias ty: (pipe rhs: (pipe rhs: (atom) @type.reference)))

; -spec name(Args) -> Ty.  (return type: `ty:` field of `type_sig`;
; argument types: elements of `args:` -> `expr_args.args`)
(type_sig ty: (atom) @type.reference)
(type_sig ty: (call expr: (atom) @type.reference))
(type_sig ty: (tuple expr: (atom) @type.reference))
(type_sig ty: (tuple expr: (call expr: (atom) @type.reference)))
(type_sig ty: (pipe lhs: (atom) @type.reference))
(type_sig ty: (pipe lhs: (call expr: (atom) @type.reference)))
(type_sig ty: (pipe rhs: (atom) @type.reference))
(type_sig ty: (pipe rhs: (call expr: (atom) @type.reference)))
(type_sig ty: (pipe rhs: (pipe lhs: (atom) @type.reference)))
(type_sig ty: (pipe rhs: (pipe rhs: (atom) @type.reference)))
(type_sig args: (expr_args args: (atom) @type.reference))
(type_sig args: (expr_args args: (call expr: (atom) @type.reference)))

; `fun((Args) -> Ty)` type expressions (e.g. inside `-type handler() ::
; fun((msg()) -> ok | error).`) — same shape as `type_sig` minus `guard:`.
(fun_type_sig ty: (atom) @type.reference)
(fun_type_sig ty: (call expr: (atom) @type.reference))
(fun_type_sig ty: (pipe lhs: (atom) @type.reference))
(fun_type_sig ty: (pipe lhs: (call expr: (atom) @type.reference)))
(fun_type_sig ty: (pipe rhs: (atom) @type.reference))
(fun_type_sig ty: (pipe rhs: (call expr: (atom) @type.reference)))
(fun_type_sig args: (expr_args args: (atom) @type.reference))
(fun_type_sig args: (expr_args args: (call expr: (atom) @type.reference)))

; Record field type annotation: `-record(name, {field :: Ty}).`
(record_field ty: (field_type expr: (atom) @type.reference))
(record_field ty: (field_type expr: (call expr: (atom) @type.reference)))
(record_field ty: (field_type expr: (pipe lhs: (atom) @type.reference)))
(record_field ty: (field_type expr: (pipe rhs: (atom) @type.reference)))

; Annotated spec argument: `-spec f(Name :: Ty) -> ...`
(ann_type ty: (atom) @type.reference)
(ann_type ty: (call expr: (atom) @type.reference))
