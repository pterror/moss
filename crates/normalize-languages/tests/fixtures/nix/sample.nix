# Sample Nix expression file

{ pkgs ? import <nixpkgs> {} }:

let
  version = "1.0.0";

  greet = name: "Hello, ${name}!";

  factorial = n:
    if n <= 1
    then 1
    else n * factorial (n - 1);

  filterEvens = lst:
    builtins.filter (x: builtins.div x 2 * 2 == x) lst;

  makePackage = { name, src, buildInputs ? [] }:
    pkgs.stdenv.mkDerivation {
      inherit name src buildInputs;
      installPhase = ''
        mkdir -p $out/bin
        cp $src $out/bin/${name}
      '';
    };

  utils = {
    join = sep: lst:
      builtins.concatStringsSep sep lst;

    mapValues = f: attrs:
      builtins.mapAttrs (_: v: f v) attrs;

    defaultTo = default: value:
      if value == null then default else value;
  };

  # Curried multi-arg function: outer function_expression's body is another
  # function_expression. `add 1 2` below parses as nested apply_expression
  # nodes; the innermost (function: variable_expression(add)) still matches
  # nix.calls.scm's simple-application pattern and captures "add" once.
  add = a: b: a + b;

  # inherit-from: brings names into scope sourced from an arbitrary
  # expression (here, `builtins`) rather than the enclosing scope.
  inherit (builtins) attrNames;

  # assert + short-circuiting && — both nix.complexity.scm complexity nodes.
  checked = assert version != "" && pkgs != null; version;

  # Parenthesized call target: a common NixOS-module / flake-utils idiom for
  # applying the result of an expression (here `import`) to arguments.
  configuredModule = (import ./module.nix) { inherit pkgs; };

in {
  inherit greet factorial filterEvens;
  inherit utils;

  # Dotted attrpath binding (sugar for `meta = { description = ...; };`).
  # attrpath.attr's first segment ("meta") is the binding being declared;
  # nested segments ("description") are NOT separate top-level bindings.
  meta.description = "A sample derivation";

  # Quoted attribute name (attrpath.attr's string_expression variant) —
  # common for names containing characters identifiers can't hold.
  "with-dash" = true;

  samplePackage = makePackage {
    name = "sample";
    src = ./src;
    buildInputs = with pkgs; [ bash coreutils ];
  };

  message = greet "World";
  fact5 = factorial 5;
  evens = filterEvens [ 1 2 3 4 5 6 ];
  total = add 1 2;
}
