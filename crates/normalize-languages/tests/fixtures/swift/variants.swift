// variants.swift is a completeness matrix: one small, commented construct per
// grammar-legal node-type variant found while applying
// docs/query-testing-methodology.md to swift.{tags,calls,imports,complexity,types}.scm.
// Every construct here is annotated with which field/variant it exercises,
// verified against real parse output via `normalize syntax query`/
// `normalize syntax ast` (node-types.json for arborium-swift 2.17.0), not
// assumed. A NEGATIVE section at the bottom holds constructs that must NOT
// match, so regressions in either direction are caught.

import Foundation
// import_declaration: dotted path (identifier node covers the whole
// "Foundation.NSObject" span, including the `.`, and the import-kind
// keyword doesn't interfere).
import class Foundation.NSObject

// --- tags.scm: function_declaration.name variants --------------------------

// plain_name — name: simple_identifier
func plainFunction() {}

infix operator +++: AdditionPrecedence

struct Vector {
    var x: Int
    var y: Int

    // custom_operator — name: custom_operator (multi-char operator overload)
    static func +++ (lhs: Vector, rhs: Vector) -> Vector {
        return Vector(x: lhs.x + rhs.x, y: lhs.y + rhs.y)
    }

    // standard-operator overload — name: literal operator token ("==").
    // Verified this is a REAL field child (`normalize syntax query
    // '(function_declaration name: "==" @name)'` matches), not the
    // return-type-mislabeling quirk documented in swift.tags.scm.
    static func == (lhs: Vector, rhs: Vector) -> Bool {
        return lhs.x == rhs.x && lhs.y == rhs.y
    }

    // compound-assignment operator overload — name: "+=" literal token.
    static func += (lhs: inout Vector, rhs: Vector) {
        lhs.x += rhs.x
        lhs.y += rhs.y
    }
}

// --- tags.scm: class_declaration.name variants ------------------------------

// plain_type_name — name: bare type_identifier
class PlainClass {}

// extension_name — name: user_type -> type_identifier. Distinct shape from
// PlainClass above; silently unhandled before this fix.
extension PlainClass {
    func extraMethod() {}
}

// extension with a `where` clause on a generic stdlib type — same
// user_type-wrapped name shape as the plain extension above.
extension Array where Element == Int {
    func sumAll() -> Int {
        return self.reduce(0, +)
    }
}

// --- tags.scm: enum_entry multi-name completeness ---------------------------

enum Status {
    // single case — name: simple_identifier
    case ready
    // associated-value case — name: simple_identifier (the payload type
    // doesn't change how `name` is tagged)
    case failed(String)
    // comma-separated multi-name case — name field is multiple=true and (per
    // real parse output) tags BOTH identifiers, unlike the analogous Go
    // const_spec bug from batch 1.
    case paused, cancelled
}

// --- tags.scm: member property let/var + local-variable exclusion ----------

class Holder {
    // member let — @definition.constant
    let readOnly: Int = 1
    // member var — @definition.var
    var mutable: Int = 2
    // computed property (still a property_declaration, still a direct child
    // of class_body) — @definition.var
    var computed: Int {
        return readOnly + mutable
    }

    func useLocals() {
        // NEGATIVE: local let/var inside a function body — same node kind
        // (property_declaration) as the member properties above, but must
        // NOT be captured as @definition.constant/@definition.var (the
        // ancestor-scoped class_body/enum_class_body restriction excludes
        // these; they are children of `statements`, not class_body).
        let localReadOnly = 5
        var localMutable = 6
        localMutable += localReadOnly
        print(localMutable)
    }
}

enum WithComputed {
    case a, b

    // computed property inside an enum — enum_class_body variant of the
    // class_body ancestor restriction above.
    var isA: Bool {
        return self == .a
    }
}

// --- tags.scm: protocol requirements ----------------------------------------

protocol Describable {
    // protocol_property_declaration — name: pattern -> simple_identifier
    var label: String { get }
    // protocol_function_declaration — name: simple_identifier
    func describe() -> String
    // associatedtype_declaration — name: type_identifier
    associatedtype Value
}

// --- calls.scm: call_expression.function variants ---------------------------

// plain_call — function: identifier
func identity(_ x: Int) -> Int {
    return x
}

struct Box {
    var value: Int
    // method_call — function: navigation_expression -> simple_identifier
    func get() -> Int {
        return value
    }
}

func callVariants() {
    _ = identity(1) // plain_call
    let b = Box(value: 5)
    _ = b.get() // method_call: navigation_expression, target: simple_identifier
    _ = Optional<String>(nil) // NOT relevant here; see constructor call below
}

// --- calls.scm: force-unwrap call (postfix_expression) ----------------------

final class Runner {
    var completion: (() -> Void)?

    func run() {
        // force-unwrap call — callee: postfix_expression(target:
        // simple_identifier, operation: bang). Previously unmatched.
        completion!()
        // optional-chaining call — callee: plain simple_identifier (the `?`
        // does not change call_expression's shape); already matched by the
        // plain_call pattern.
        completion?()
    }
}

// --- calls.scm: generic type instantiation call (constructor_expression) ---

struct GenericBox<T> {
    var value: T
}

func constructorVariants() {
    // NOT a call_expression — a distinct constructor_expression node with a
    // `constructed_type: (user_type (type_identifier))` field. Previously
    // unmatched by any calls.scm pattern.
    _ = GenericBox<Int>(value: 1)
    _ = Optional<String>(nil)
}

// --- complexity.scm: guard / switch_entry / conjunction / disjunction ------

func complexityVariants(_ n: Int, flag: Bool) -> String {
    // guard_statement — early-exit branch, previously uncounted.
    guard n > 0 else {
        return "non-positive"
    }
    // conjunction_expression (&&) / disjunction_expression (||) — previously
    // uncounted.
    if n > 0 && flag || n < 100 {
        return "complex"
    }
    switch n {
    case 1:
        return "one"
    case 2, 3:
        return "two-or-three"
    case let x where x > 10:
        return "big"
    default:
        return "other"
    }
}

// --- NEGATIVE cases: constructs that must NOT match -------------------------

func negativeCallVariants() {
    // NEGATIVE: function: call_expression (curried call) — the outer call
    // must not produce a @call/@name capture.
    _ = makeAdder()(1)

    // NEGATIVE: function: lambda_literal (immediately-invoked closure) — an
    // anonymous callee, must not produce a @call/@name capture.
    _ = { (x: Int) -> Int in x * 2 }(5)

    // NEGATIVE: function: array_type literal ([Int](...)) — a bracket
    // type-literal callee, not a declared symbol name, must not produce a
    // @call/@name capture.
    _ = [Int](repeating: 0, count: 3)
}

func makeAdder() -> (Int) -> Int {
    return { delta in delta }
}

func negativeLocalPropertyVariants() {
    // NEGATIVE: local let/var must never be tagged @definition.constant/
    // @definition.var (covered structurally by Holder.useLocals above; this
    // top-level-function copy exercises the same shape outside any class
    // body at all).
    let notAMember = 1
    var alsoNotAMember = 2
    alsoNotAMember += notAMember
    print(alsoNotAMember)
}
