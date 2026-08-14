# Nix completeness-matrix fixture.
#
# One small, commented construct per node-type variant identified by
# cross-referencing nix.{calls,tags,complexity}.scm's field constraints
# against arborium-nix 2.17.0's node-types.json (see
# docs/query-testing-methodology.md). A NEGATIVE section at the bottom holds
# near-miss constructs that must NOT match the tags/calls/complexity queries.

let

  # --- tags.scm: binding attrpath.attr variants -----------------------------

  # identifier attr (plain binding name) — @definition.var, @name = "plain"
  plain = 1;

  # string_expression attr (quoted binding name) — @definition.var,
  # @name = "\"quoted-name\"" (captured with surrounding quotes)
  "quoted-name" = 2;

  # dotted attrpath — only the FIRST identifier ("outer") is the declared
  # binding; "inner" is a nested attribute, not a separate top-level
  # definition. See NEGATIVE section below for the explicit non-match check.
  outer.inner = 3;

  # function-valued binding (first identifier attr, single-arg) —
  # @definition.function in addition to @definition.var
  fn = x: x + 1;

  # function-valued binding via quoted name — @definition.function with a
  # string_expression @name
  "quoted-fn" = x: x + 1;

  # curried (multi-arg) function-valued binding — still matches on the
  # binding's direct (outer) function_expression
  curried = a: b: a + b;

  # --- tags.scm: inherit / inherit_from --------------------------------------

  # inherit_from: sources names from an arbitrary expression rather than the
  # enclosing scope — @definition.var, @name = "attrValues"
  inherit (builtins) attrValues;

  # --- calls.scm: apply_expression.function variants ------------------------

  # variable_expression callee (simple application) — @call = "plain"
  simpleCallResult = plain;
  # (plain is a value, not callable; use a real function for the call itself)
  fnCallResult = fn 1;

  # select_expression callee, identifier last-attr (attribute-path
  # application, e.g. `builtins.map`) — @call = "attrNames"
  selectCallResult = builtins.attrNames { a = 1; };

  # select_expression callee, string_expression last-attr (quoted-attr
  # application) — @call = "\"my-fn\"" (rare; grammar-legal, verified via
  # `normalize syntax query`)
  quotedSelectHolder = { "my-fn" = x: x; };
  quotedCallResult = quotedSelectHolder."my-fn" 1;

  # parenthesized_expression callee (paren-wrapped call target — common
  # NixOS-module / flake-utils idiom for applying an expression's result)
  parenCallResult = (fn) 1;

in {
  inherit plain fn curried;
  inherit simpleCallResult fnCallResult selectCallResult quotedCallResult
    parenCallResult;

  # --- complexity.scm: complexity nodes --------------------------------------

  # if_expression — @complexity, @nesting
  ifResult = if plain > 0 then 1 else 0;

  # assert_expression — @complexity
  assertResult = assert plain > 0; plain;

  # short-circuiting && — @complexity
  andResult = plain > 0 && plain < 10;

  # short-circuiting || — @complexity
  orResult = plain > 100 || plain < 10;

  # --- complexity.scm: nesting-only nodes ------------------------------------

  # with_expression — @nesting (not @complexity: no branching)
  withResult = with { a = 1; }; a;

  # let_expression — @nesting (not @complexity: no branching)
  letResult = let x = 1; in x;

  # function_expression — @nesting (not @complexity: no branching)
  funcResult = x: x;

  # ---------------------------------------------------------------------------
  # NEGATIVE cases — must NOT produce the captures noted
  # ---------------------------------------------------------------------------

  # Dotted attrpath's inner segment ("negInner") must NOT itself appear as a
  # separate @name/@definition.var capture distinct from the outer ("negOuter")
  # binding it belongs to — see the anchored-first-attr fix in nix.tags.scm.
  negOuter.negInner = 4;

  # Interpolation attr (dynamic key) has no static name and must NOT produce
  # a @name capture at all — matches lua.tags.scm's identical exclusion of
  # computed assignment targets.
  ${if true then "dyn" else "dyn"} = 5;

  # Plain arithmetic/comparison operators are NOT short-circuiting and must
  # NOT be tagged @complexity (only && and || are).
  arithResult = plain + 1;
  compareResult = plain == 1;

  # A bare non-applied value reference is not a call and must NOT produce a
  # @call capture (no apply_expression node exists here at all).
  bareRef = plain;
}
