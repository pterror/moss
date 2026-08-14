// Completeness-matrix fixture for Groovy queries.
//
// One small, commented construct per node-type field-variant found by
// cross-referencing groovy.{calls,tags,decorations,refactor,types,cfg,
// complexity}.scm against arborium-groovy 2.17.0's node-types.json (see
// docs/query-testing-methodology.md). A dedicated NEGATIVE section at the
// bottom holds near-miss constructs that must NOT match the relevant
// queries.
//
// Verified to parse with zero `(ERROR)` nodes via:
//   normalize syntax query -p variants.groovy "(ERROR) @e" --show-source

// ---------------------------------------------------------------------------
// tags.scm / class_definition: class vs interface (same node type, only the
// leading anonymous keyword token differs — no dedicated interface node).
// ---------------------------------------------------------------------------

class PlainClass {
    int field
}

interface PlainInterface {
    // function_declaration: a method signature with no body (abstract).
    void doThing()
}

// class_definition.superclass: extends works for both classes and
// interfaces; `implements` is NOT supported by this grammar at all (see
// NEGATIVE section below and groovy.calls.scm's header comment).
class ExtendsClass extends PlainClass {
}

interface ExtendsInterface extends PlainInterface {
}

// ---------------------------------------------------------------------------
// tags.scm: closure assigned to a name (def-bound and reassignment forms)
// ---------------------------------------------------------------------------

def declaredClosure = { x -> x }

reassignedClosure = { x -> x }

// ---------------------------------------------------------------------------
// calls.scm / tags.scm: function_call.function field variants
// ---------------------------------------------------------------------------

// function: identifier
plainCall()

// function: dotted_identifier (qualified method call)
Collections.sort(declaredClosure)

// function: function_call (chained/curried call)
def chained = declaredClosure()()

// function: index (bracket-index dispatch-table call) — only reachable as a
// declaration/assignment RHS or nested expression, not a bare top-level
// statement (see NEGATIVE section).
def dispatch = [k: { -> 1 }]
def dispatchResult = dispatch['k']()

// function: parenthesized_expression (ternary-selected call target)
def ternaryCall = (true ? declaredClosure : declaredClosure)()

// function: this (call shape only — this grammar has no constructor node,
// so real constructor delegation never parses; see groovy.calls.scm)
def thisCall = this("x")

// ---------------------------------------------------------------------------
// calls.scm / tags.scm: juxt_function_call.function field variants
// (no-parens "juxtaposition" calls)
// ---------------------------------------------------------------------------

// function: identifier — reliable regardless of surrounding file content.
println "juxt plain";

// function: dotted_identifier (e.g. `Collections.sort numbers`) and
// function: index (e.g. `dispatch['k'] 1`) are both legal per node-types.json
// and DO produce a `juxt_function_call` when parsed in a small, isolated
// snippet — confirmed via `normalize syntax ast`/`normalize syntax query`.
// But in a larger composite file (this one included, at every position
// tried: start, middle, end, with or without an intervening `def` binding)
// the same two constructs silently reparse as separate sibling statements
// instead — no ERROR node, just a different, wrong tree shape. This appears
// to be a genuine GLR ambiguity-resolution artifact in arborium-groovy
// 2.17.0 that depends on total surrounding parse context, not on anything
// local to the construct itself (bisecting the file's prefix length toggles
// pass/fail non-monotonically). Since this fixture can't reliably
// demonstrate the shape without misrepresenting it as dependable, the two
// forms are left as comments rather than live code the reader would
// reasonably expect this file's own queries to match:
//
//   Collections.sort numbers   // function: dotted_identifier
//   dispatch['k'](1)           // function: index (reliable form: this one
//                              // — WITH parens, i.e. function_call not
//                              // juxt_function_call — is covered above and
//                              // does not share this fragility)

// ---------------------------------------------------------------------------
// decorations.scm: annotation, comment, groovy_doc (distinct extra node
// types — groovy_doc is NOT a subtype of comment)
// ---------------------------------------------------------------------------

// A plain line comment.

/**
 * A GroovyDoc comment.
 * @param n the input
 * @throws RuntimeException sometimes
 */
def documented(int n) { n }

@Deprecated
def annotated() {}

// ---------------------------------------------------------------------------
// refactor.scm: statement forms, including C-style for_loop (previously
// missing alongside for_in_loop)
// ---------------------------------------------------------------------------

def forLoopStatement() {
    for (int i = 0; i < 3; i++) {
        println i
    }
}

def forInLoopStatement() {
    for (i in [1, 2, 3]) {
        println i
    }
}

// ---------------------------------------------------------------------------
// types.scm: declared type variants (identifier, builtintype, array_type,
// type_with_generics / generics)
// ---------------------------------------------------------------------------

int builtinTyped = 1
PlainClass identifierTyped = null
int[] arrayTyped = [1, 2]
List<String> genericTyped = []
Map<String, Integer> nestedGenericTyped = [:]

// ---------------------------------------------------------------------------
// NEGATIVE cases — must NOT match / must error / must be absent.
// ---------------------------------------------------------------------------

// 1. `implements` is not supported by this grammar at all — the following
//    line, if uncommented, produces an ERROR node (verified via
//    `normalize syntax ast`). Left as a comment so this fixture itself
//    parses clean; documented here rather than fabricated as a query
//    pattern that would never match anything real.
//
//    class Bad implements PlainInterface {}

// 2. `trait X { ... }` and `enum X { ... }` are not modeled by this grammar
//    — they parse as unrelated bare identifiers followed by a closure, with
//    no ERROR and no recognizable definition node. There is nothing for
//    (class_definition ...) or any other query to match; commented out so
//    this fixture doesn't assert on ghost constructs:
//
//    trait NotATrait { }
//    enum NotAnEnum { RED, GREEN }

// 3. A bare (non-closure) declaration must NOT be picked up by the
//    closure-assigned-to-name @definition.function patterns.
def plainValueNotAFunction = 42

// 4. A bare reassignment to a non-closure value must NOT match either.
plainValueNotAFunction = 43

// 5. `handlers['k'](x)` as a bare top-level statement (no declaration/
//    assignment context) is ambiguously parsed by this grammar as TWO
//    separate statements (a juxt_function_call swallowing the bracket
//    literal as an argument-list list, then a separate parenthesized
//    expression) rather than one function_call with function: (index).
//    Confirmed via `normalize syntax ast`; not reproduced here as executable
//    code since it would silently produce a different tree shape than the
//    dispatch-table idiom above, not an ERROR — call.scm's `function: (index)`
//    clause is only expected to fire in the declaration/assignment-RHS forms
//    demonstrated above.

// 6. Chaining a call directly after a trailing (no-parens) closure argument
//    is a grammar limitation, not a query gap — see sample.groovy's comment
//    on `findAll { ... }.sum()`. Not reproduced here as it would introduce
//    an ERROR node into this fixture.
