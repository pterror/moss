// Completeness matrix for kotlin.{tags,calls,imports,types}.scm.
// Each section is commented with which field/variant it exercises so this
// file doubles as documentation of the matrix (see
// docs/query-testing-methodology.md step 5).

package variants

// --- imports.scm --------------------------------------------------------
//
// import_header: plain identifier path (anchored so it does not also match
// the aliased/wildcard forms below), identifier + import_alias
// (type_identifier), and identifier + wildcard_import.
// A same-line trailing comment on the plain import below exercises a real
// grammar quirk: `line_comment` is declared `extra` in this grammar but a
// trailing anchor alone does not skip it — the comment attaches as a
// literal trailing child of `import_header`, so the plain-import pattern
// needs a dedicated "identifier, then a comment, then nothing" variant.

import java.util.ArrayList // trailing same-line comment variant
import java.util.HashMap as JHashMap
import kotlin.math.*

// --- tags.scm / types.scm: type-defining declarations --------------------

// class_declaration: plain class, direct type_identifier child of
// class_declaration itself -> @definition.class (tags) / @definition.type
// (types).
class PlainClass

// object_declaration: singleton object -> @definition.class /
// @definition.type.
object PlainObject

// type_alias: type_identifier child -> @definition.type (tags + types).
typealias PlainAlias = List<Int>

// interface: same node kind as class_declaration (class_identifier),
// distinguished only by the "interface" keyword child — kotlin.rs's
// refine_kind reads that, tags.scm itself makes no distinction.
interface PlainInterface {
    fun requirement()
}

// enum class + enum_entry: (simple_identifier) child of enum_entry ->
// @definition.constant.
enum class Direction {
    NORTH, SOUTH
}

// sealed class: same node kind as class_declaration, no separate variant.
sealed class SealedBase

// --- tags.scm: delegation_specifier variants ------------------------------

// delegation_specifier -> constructor_invocation -> user_type ->
// type_identifier: superclass call WITH parens/args -> @reference.class.
open class OpenBase(val tag: String)

class ConstructorInvocationVariant : OpenBase("x")

// delegation_specifier -> user_type -> type_identifier directly (no
// invocation): implementing an interface with the bare `: Type` form, by
// far the most common Kotlin idiom -> @reference.class.
class PlainDelegationVariant : PlainInterface {
    override fun requirement() {}
}

// delegation_specifier -> explicit_delegation -> user_type ->
// type_identifier: interface delegation via `by` -> @reference.class.
class ExplicitDelegationVariant(impl: PlainInterface) : PlainInterface by impl

// --- tags.scm / calls.scm: constructor_delegation_call --------------------

// constructor_delegation_call wrapping the "this" keyword: secondary
// constructor delegating to the primary constructor -> @reference.call /
// @call with name "this".
class ThisDelegationVariant(val a: Int, val b: Int) {
    constructor(a: Int) : this(a, 0)
}

// constructor_delegation_call wrapping the "super" keyword: secondary
// constructor delegating to the superclass's constructor -> @reference.call
// / @call with name "super".
open class SuperBase {
    constructor(x: Int)
}

class SuperDelegationVariant : SuperBase {
    constructor(x: Int) : super(x)
}

// --- tags.scm / calls.scm: call_expression variants -----------------------

fun plainCall() {
    // call_expression -> simple_identifier directly (no navigation):
    // top-level/local function call -> @reference.call / @call.
    println("plain")
}

fun navigationCall() {
    val list = ArrayList<Int>()
    // call_expression -> navigation_expression -> (_) qualifier +
    // navigation_suffix -> simple_identifier: method call via `.` ->
    // @reference.call / @call, with @call.qualifier for the receiver.
    list.add(1)
}

fun trailingLambdaCall() {
    // Trailing-lambda call: call_expression's callee shape (navigation to
    // "map") is unaffected by whether the call carries value_arguments or
    // only an annotated_lambda call_suffix — verifies the existing pattern
    // already covers this without a dedicated new clause.
    listOf(1, 2).map { it * 2 }
}

// --- types.scm: type.reference variants -----------------------------------

// Plain, unqualified type_identifier in a type position.
val plainType: PlainClass? = null

// user_type wrapping a generic type argument: List<PlainClass> — the
// generic argument's type_identifier ("PlainClass") is nested inside a
// user_type, exercising the "already covered by the blanket pattern,
// don't double-count" fix.
val genericType: List<PlainClass> = emptyList()

// --- NEGATIVE: constructs that must NOT match -----------------------------

// callable_reference (`::foo` / `Type::method`) is a distinct node kind
// from call_expression and must never be treated as a call.
val methodReferenceNegative = PlainClass::hashCode

// annotation usage WITH constructor args: `constructor_invocation` is also
// a legal child of `annotation` (not just `delegation_specifier`), and
// tags.scm's superclass-call pattern is deliberately scoped to
// `delegation_specifier` so this must NOT produce a @reference.class.
@Deprecated("use PlainClass instead")
class AnnotationArgsNegative

// property_declaration (val/var, class-level or local): intentionally
// excluded from tags.scm (see kotlin.tags.scm's comment) because the
// grammar reuses the same node kind for local variable declarations, which
// have no reliable ancestor-free way to distinguish. Must NOT produce a
// @definition.* capture.
val topLevelPropertyNegative = 42

// unnamed companion object: has no type_identifier child at all, so it is
// architecturally unable to produce a @name capture without fabricating
// text the grammar doesn't provide — documented absence, not a bug.
class UnnamedCompanionNegative {
    companion object {
        const val VALUE = 1
    }
}
