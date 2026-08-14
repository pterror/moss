; Nix tags query
; Covers: attribute bindings in attrsets, let expressions, rec attrsets, and
; inherit statements.
; Nix's main declaration form is: name = value; inside attrsets/let
; The binding node has: attrpath (with attr: identifier) and expression fields.

; ---------------------------------------------------------------------------
; Attribute bindings: name = value;
; ---------------------------------------------------------------------------
; attrpath.attr is `multiple: true` — a dotted path like `foo.bar = 1;` (sugar
; for `foo = { bar = 1; };`) produces MULTIPLE attr children. The binding
; being *declared* is only the FIRST segment (`foo`); without the leading `.`
; anchor below, this query previously matched every segment (`foo` AND
; `bar`), producing a spurious duplicate/wrong definition for `bar` on every
; dotted-path binding — a common Nix idiom (e.g. NixOS module options like
; `services.foo.enable = true;`). Verified via `normalize syntax query`
; against a probe file with `foo.bar = 1; foo.baz = 2;`: unanchored captured
; both "foo" and "bar"/"baz"; anchored to the first attr correctly captures
; only "foo" twice.
;
; attrpath.attr also allows string_expression (quoted names, e.g.
; `"foo-bar" = 1;`) in addition to identifier — both are captured below.
; interpolation (dynamic names, e.g. `${dynamicKey} = 2;`) is deliberately
; NOT captured: the key is a runtime-computed expression, no static name
; exists, matching lua.tags.scm's identical exclusion of computed
; assignment targets.
(binding
  attrpath: (attrpath
    . attr: [
      (identifier) @name
      (string_expression) @name
    ])) @definition.var

; Function-valued bindings (heuristic: binding where expression is a function)
; e.g. mkDerivation = { ... }: ... — same first-attr/quoted-name coverage as
; above. Curried multi-arg functions (`add = a: b: a + b;`) are not a gap:
; the outer function_expression is still the binding's direct `expression`,
; so this matches on the first parameter and doesn't need to unwrap nesting.
(binding
  attrpath: (attrpath
    . attr: [
      (identifier) @name
      (string_expression) @name
    ])
  expression: (function_expression)) @definition.function

; ---------------------------------------------------------------------------
; Inherit statements
; ---------------------------------------------------------------------------
; `inherit foo bar;` brings `foo`/`bar` from the enclosing scope into the
; current attrset/let as new bindings (`foo = <outer>.foo;`). `inherit (expr)
; foo bar;` (`inherit_from`) does the same but sources from an arbitrary
; expression instead of the enclosing scope. Both create real, named
; bindings — the same kind of declaration a plain `name = value;` binding
; is — and were previously entirely unmatched by this query despite being a
; very common idiom (used repeatedly in this repo's own flake.nix and in
; nix.tags.scm's own sample.nix fixture). inherited_attrs.attr allows
; identifier, interpolation, string_expression, matching attrpath's variant
; set; only identifier is realistic here (inherit's target names are always
; plain identifiers in practice — interpolation/string_expression as an
; inherited-attr name is not idiomatic Nix and not exercised by any fixture
; found).
(inherit
  (inherited_attrs
    (identifier) @name)) @definition.var

(inherit_from
  (inherited_attrs
    (identifier) @name)) @definition.var
