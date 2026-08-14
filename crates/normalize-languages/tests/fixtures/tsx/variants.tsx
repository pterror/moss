// Completeness matrix for tsx.{calls,tags,imports,types,cfg}.scm.
//
// Each section is commented with which query file + field/variant it
// exercises, cross-referenced against arborium-tsx's node-types.json (see
// docs/query-testing-methodology.md). A dedicated NEGATIVE section at the
// bottom holds constructs that must NOT match specific patterns.

// --- calls.scm: call_expression.function variants --------------------------

function plainCall() {}
plainCall(); // function: (identifier)

const obj = { method() {}, ['computed']() {} };
obj.method(); // function: (member_expression), property: (property_identifier)

class WithPrivate {
    #privateMethod() {}
    run() {
        this.#privateMethod(); // function: (member_expression), property: (private_property_identifier)
    }
}

const bracketTarget: Record<string, () => void> = { key: () => {} };
bracketTarget['key'](); // function: (subscript_expression)

const parenTarget = (Math.random() > 0.5 ? plainCall : plainCall);
(parenTarget)(); // function: (parenthesized_expression)

function maybeFn(): (() => void) | undefined {
    return undefined;
}
maybeFn()!(); // outer function: (non_null_expression)

const curried = () => () => {};
curried()(); // outer function: (call_expression)

async function dynamicLoader() {
    return import('./variant-dynamic-import'); // imports.scm: dynamic import() call
}

// --- tags.scm: name field variants ------------------------------------------

function funcDecl() {} // definition.function via function_declaration

declare function ambientSig(): void; // definition.function via function_signature

interface HasMethods {
    sigMethod(): void; // definition.method via method_signature
}

abstract class AbstractBase {
    abstract absMethod(): void; // definition.method via abstract_method_signature
    #privateDef() {} // definition.method via method_definition, name: private_property_identifier
    ['computedDef']() {} // definition.method via method_definition, name: computed_property_name
}

class PlainClass {} // definition.class via class_declaration
abstract class AbstractClass {} // definition.class via abstract_class_declaration

module LegacyModule { // definition.module via `module` keyword, name: identifier
    export const x = 1;
}
module LegacyModule.Nested { // definition.module via `module` keyword, name: nested_identifier
    export const y = 2;
}
declare module "ambient-module-name" { // definition.module via `module` keyword, name: string
    export const z: number;
}

namespace ModernNamespace { // definition.module via internal_module, name: identifier
    export const a = 1;
}
namespace ModernNamespace.Nested { // definition.module via internal_module, name: nested_identifier
    export const b = 1;
}

interface PlainInterface {} // definition.interface

enum PlainEnum { A, B } // definition.enum

type PlainAlias = { a: number }; // definition.type via type_alias_declaration

class Base {}
class DerivedPlain extends Base {} // reference.class via extends_clause, value: identifier

const NsHolder = { Ctor: class {} };
class DerivedFromMember extends NsHolder.Ctor {} // reference.class via extends_clause, value: member_expression (not identifier — falls to generic `value: (_)` clause)

interface LoggerLike {}
class ImplementsPlain extends Base implements LoggerLike {} // reference.implementation via implements_clause, plain type_identifier

interface ComparableLike<T> {}
class ImplementsGeneric extends Base implements ComparableLike<ImplementsGeneric> {} // reference.implementation via implements_clause, generic_type

new Base(); // reference.class via new_expression, constructor: identifier

const NamespacedCtor = { Widget: class {} };
new NamespacedCtor.Widget(); // reference.class via new_expression, constructor: member_expression

// --- cfg.scm: effect captures ------------------------------------------------

async function effectAwait() {
    await Promise.resolve(); // cfg.effect.await
}

function* effectYield() {
    yield 1; // cfg.effect.yield
}

// --- NEGATIVE: constructs that must NOT match -------------------------------

// A bare property access is not a call — must never surface as @call.
const notACallTarget = obj.method;

// A plain function type annotation reference is not a `new` target.
type NotANewTarget = typeof Base;

// An interface property (not a method) must not surface as definition.method.
interface HasProps {
    justAField: number;
}

// A numeric-keyed method name is grammar-legal (method_signature.name allows
// `number`) but tsx.tags.scm intentionally doesn't handle it — see the
// header comment in tsx.tags.scm. Confirms it stays uncaptured.
interface HasNumericKey {
    123(): void;
}
