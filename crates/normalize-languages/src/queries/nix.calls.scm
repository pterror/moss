; Nix calls query
; @call — function application expression
; @call.qualifier — not applicable
;
; Nix uses juxtaposition for function application: `f x` (no parentheses).
; The tree-sitter grammar represents this as `apply_expression` with a
; `function` field (the callee) and an `argument` field.
;
; Per arborium-nix's node-types.json, `apply_expression.function` allows 15
; node types: apply_expression, attrset_expression, float_expression,
; hpath_expression, indented_string_expression, integer_expression,
; let_attrset_expression, list_expression, parenthesized_expression,
; path_expression, rec_attrset_expression, select_expression, spath_expression,
; string_expression, uri_expression, variable_expression. Only
; `variable_expression` (simple name), `select_expression` (attribute path
; like `builtins.map`), and `parenthesized_expression` (see below) produce a
; meaningful callable name in real Nix code; the rest are calling a literal
; value, which is a runtime type error, not something real programs do.
;
; Curried/multi-arg application (`f a b`) is NOT a gap here: it parses as
; nested apply_expression nodes (`apply_expression(function: apply_expression
; (function: f, argument: a), argument: b)`), and each pattern below matches
; every node in the tree independently — the innermost apply_expression
; (whose function is the plain variable/select expression) still matches and
; captures the callee name once. Verified via `normalize syntax ast` on
; `add 1 2` (arborium-nix 2.17.0).

; Simple application: f arg
(apply_expression
  function: (variable_expression
    (identifier) @call))

; Attribute-path application: builtins.map, lib.lists.map, etc.
; attrpath.attr allows identifier, interpolation, string_expression; only the
; LAST attr matters here (anchored with trailing `.`) since that's the actual
; member being called — mid-path segments (e.g. `pkgs.${system}.foo`) are
; irrelevant to the callee name.
(apply_expression
  function: (select_expression
    attrpath: (attrpath
      attr: (identifier) @call .)))

; Quoted-attr application: attrs."my-fn" arg — rare but grammar-legal
; (attrpath.attr's string_expression variant). Best-effort: captures the
; quoted string text (including quotes) as @call.
(apply_expression
  function: (select_expression
    attrpath: (attrpath
      attr: (string_expression) @call .)))

; Dynamic-attr application: attrs.${key} arg — no statically resolvable
; callee name (the key is a runtime expression); captured best-effort as the
; interpolation's own text, matching the convention used for other languages'
; dynamic/computed call targets (e.g. lua.calls.scm's bracket-index call).
(apply_expression
  function: (select_expression
    attrpath: (attrpath
      attr: (interpolation) @call .)))

; Parenthesized call target: (import ./module.nix) { ... } — a common NixOS
; module / flake-utils idiom (calling the result of an expression wrapped in
; parens). No statically resolvable single name; captured best-effort as the
; whole parenthesized expression's text, matching lua.calls.scm's identical
; IIFE case (`(function() ... end)()`).
(apply_expression
  function: (parenthesized_expression) @call)
