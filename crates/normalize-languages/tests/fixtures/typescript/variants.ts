// Completeness fixture: one construct per grammar-legal variant of each field
// the typescript.{tags,calls,imports}.scm queries constrain, cross-referenced
// against arborium-typescript's node-types.json. Every construct here is
// *expected to be captured*; see query_fixtures.rs `typescript_*_completeness_*`
// tests for the matrix.
//
// This file also carries a handful of deliberate near-miss constructs (marked
// NEGATIVE) that must NOT be captured by the query under test, to guard
// against over-broad patterns.

// --- import variants ---------------------------------------------------

import { plainName } from "mod-plain"; // import_specifier.name = identifier
import { plainName as plainAlias } from "mod-plain-alias"; // + alias
import { default as renamedDefault } from "mod-default"; // name = "default" (anonymous token)
import * as ns from "mod-namespace"; // namespace_import
import fsThing = require("fs"); // import_require_clause (TS import-equals)
export { plainName as reexported } from "mod-reexport-alias"; // re-export, aliased
export { renamedDefault } from "mod-reexport-plain"; // re-export, plain
export * from "mod-wildcard"; // wildcard re-export (bare star)
export * as wildcardNs from "mod-wildcard-ns"; // namespace re-export
export { default } from "mod-default-reexport"; // bare default re-export
export { default as renamedDefaultReexport } from "mod-default-reexport-alias"; // aliased default re-export

export function useDynamicImport(): Promise<unknown> {
  return import("mod-dynamic"); // dynamic import() expression
}

// --- call_expression.function variants ----------------------------------

function identity(x: number): number {
  return x;
}

function plainCall() {
  identity(1); // function: identifier
}

function methodCall() {
  const arr: number[] = [];
  arr.push(1); // function: member_expression, property: property_identifier
}

class PrivateHolder {
  #compute(): number {
    return 1;
  }
  callPrivate(): number {
    return this.#compute(); // function: member_expression, property: private_property_identifier
  }
}

function computedCall() {
  const arr = [1, 2, 3];
  arr[0](); // function: subscript_expression — will throw at runtime, but grammar-legal and must still be captured
}

function parenthesizedCall() {
  (identity)(1); // function: parenthesized_expression
}

function nonNullCall() {
  identity!(1); // function: non_null_expression (TS non-null assertion before call)
}

function chainedCall() {
  function curried() {
    return identity;
  }
  curried()(1); // function: call_expression (chained/curried call)
}

// --- method_definition/method_signature/abstract_method_signature.name variants --

class MethodNameHolder {
  plainMethod(): number {
    return 1;
  } // name: property_identifier

  #privateMethod(): number {
    return 2;
  } // name: private_property_identifier

  ["computedMethod"](): number {
    return 3;
  } // name: computed_property_name
}

// --- new_expression.constructor variants ---------------------------------

const ns2 = { Ctor: class {} };

function plainNew() {
  return new PrivateHolder(); // constructor: identifier
}

function qualifiedNew() {
  return new ns2.Ctor(); // constructor: member_expression (namespaced constructor)
}

// --- module / internal_module name variants ------------------------------

module LegacyModule {
  export const value = 1;
} // `module` keyword — name: identifier

module Legacy.Dotted {
  export const value = 2;
} // `module` keyword — name: nested_identifier

declare module "ambient-module-name" {
  export function ambientFn(): void;
} // `module` keyword — name: string (ambient module declaration)

namespace SimpleNamespace {
  export const value = 3;
} // `namespace` keyword — parses as internal_module, name: identifier

namespace Dotted.Nested {
  export const value = 4;
} // `namespace` keyword — parses as internal_module, name: nested_identifier

// --- extends_clause.value / implements_clause variants --------------------

class Base {}
interface Iface {
  method(): void;
}
interface GenericIface<T> {
  method(x: T): void;
}

class ExtendsIdentifier extends Base {} // extends_clause.value: identifier

class ExtendsMemberExpression extends ns2.Ctor {} // extends_clause.value: member_expression (as a base, structurally valid even though Ctor is a class expression)

function Mixin<T extends new (...args: unknown[]) => object>(Ctor: T) {
  return class extends Ctor {};
}
class ExtendsMixinCall extends Mixin(Base) {} // extends_clause.value: call_expression (mixin pattern)

class ImplementsPlain implements Iface {
  method(): void {}
} // implements_clause: type_identifier

class ImplementsGeneric implements GenericIface<number> {
  method(x: number): void {}
} // implements_clause: generic_type -> name: type_identifier

// --- NEGATIVE cases: must not be captured as calls/definitions -------------

class NegativeHolder {
  field: number = 0;
}

function negativeCases(holder: NegativeHolder): number {
  // A closure literal is not a function_declaration/method_definition. It
  // must never appear in tags @definition.function/@definition.method
  // captures.
  const addOne = (x: number): number => x + 1;

  // A bare field read with no call parens must never appear in any @call
  // capture (regression guard against over-eager member_expression patterns
  // matching plain property access).
  const readField = holder.field;

  // `let`/`const`-bound call results are plain reads, never anything else —
  // this is just a normal call, included here to confirm it's captured
  // exactly once (not duplicated by an over-broad negative-adjacent clause).
  const bound = addOne(1);

  return readField + bound;
}
