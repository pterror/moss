// Completeness fixture: one construct per grammar-legal variant of each field
// the javascript.{tags,calls,imports}.scm queries constrain, cross-referenced
// against arborium-javascript's node-types.json. Every construct here is
// *expected to be captured*; see query_fixtures.rs `javascript_*_completeness_*`
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
export { plainName as reexported } from "mod-reexport-alias"; // re-export, aliased
export { renamedDefault } from "mod-reexport-plain"; // re-export, plain
export * from "mod-wildcard"; // wildcard re-export (bare star)
export * as wildcardNs from "mod-wildcard-ns"; // namespace re-export
export { default } from "mod-default-reexport"; // bare default re-export
export { default as renamedDefaultReexport } from "mod-default-reexport-alias"; // aliased default re-export

const { statSync } = require("fs"); // destructured require — shorthand
const { readFile: readFileAliased } = require("fs-alias"); // destructured require — aliased member
const wholeModule = require("mod-require"); // plain require binding
require("mod-side-effect"); // side-effect-only require

function useDynamicImport() {
  return import("mod-dynamic"); // dynamic import() expression
}

// --- call_expression.function variants ----------------------------------

function identity(x) {
  return x;
}

function plainCall() {
  identity(1); // function: identifier
}

function methodCall() {
  const arr = [];
  arr.push(1); // function: member_expression, property: property_identifier
}

class PrivateHolder {
  #compute() {
    return 1;
  }
  callPrivate() {
    return this.#compute(); // function: member_expression, property: private_property_identifier
  }
}

function computedCall() {
  const arr = [1, 2, 3];
  arr[0](); // function: subscript_expression — will throw at runtime, but grammar-legal and must still be captured
}

function parenthesizedCall() {
  (function iife() {})(); // function: parenthesized_expression (IIFE)
}

function chainedCall() {
  function curried() {
    return identity;
  }
  curried()(1); // function: call_expression (chained/curried call)
}

function taggedTemplateCall(strings, ...vals) {
  return strings.join("");
}
taggedTemplateCall`hello ${1}`; // arguments: template_string (not the usual `arguments` node) — already
// matched by the plain-identifier clause, which doesn't constrain `arguments`

// --- method_definition.name variants --------------------------------------

class MethodNameHolder {
  plainMethod() {
    return 1;
  } // name: property_identifier

  #privateMethod() {
    return 2;
  } // name: private_property_identifier

  ["computedMethod"]() {
    return 3;
  } // name: computed_property_name

  static staticMethod() {
    return 4;
  } // name: property_identifier (static modifier doesn't change the name field)

  get accessor() {
    return 5;
  } // name: property_identifier (getter)

  set accessor(v) {} // name: property_identifier (setter)
}

// --- new_expression.constructor variants ---------------------------------

const nsObj = { Ctor: class {} };

function plainNew() {
  return new PrivateHolder(); // constructor: identifier
}

function qualifiedNew() {
  return new nsObj.Ctor(); // constructor: member_expression (namespaced constructor)
}

// --- class_heritage variants (extends) ------------------------------------

class Base {}

class ExtendsIdentifier extends Base {} // class_heritage child: identifier

class ExtendsMemberExpression extends nsObj.Ctor {} // class_heritage child: member_expression

function Mixin(Ctor) {
  return class extends Ctor {};
}
class ExtendsMixinCall extends Mixin(Base) {} // class_heritage child: call_expression (mixin pattern)

// --- NEGATIVE cases: must not be captured as calls/definitions -------------

class NegativeHolder {
  field = 0;
}

function negativeCases(holder) {
  // A closure literal is not a function_declaration/method_definition. It
  // must never appear in tags @definition.function/@definition.method
  // captures.
  const addOne = (x) => x + 1;

  // A bare field read with no call parens must never appear in any @call
  // capture (regression guard against over-eager member_expression patterns
  // matching plain property access).
  const readField = holder.field;

  // `const`-bound call results are plain reads — this is just a normal call,
  // included here to confirm it's captured exactly once (not duplicated by
  // an over-broad negative-adjacent clause).
  const bound = addOne(1);

  return readField + bound;
}
