// variants.go is a completeness matrix: one small, commented construct per
// grammar-legal node-type variant found while applying
// docs/query-testing-methodology.md to go.{tags,calls,imports,complexity,types}.scm.
// Every construct here is annotated with which field/variant it exercises,
// verified against real parse output via `normalize syntax query`/
// `normalize syntax ast` (node-types.json for arborium-go 2.17.0), not
// assumed. A NEGATIVE section at the bottom holds constructs that must
// NOT match, so regressions in either direction are caught.
package variants

import (
	"fmt"

	// import_spec.name: dot — dot import (blanket-imports strings' exports).
	. "strings"
	// import_spec.name: blank_identifier — side-effect-only import.
	_ "os"
	// import_spec.name: package_identifier — aliased import.
	f "fmt"
	// import_spec.path: raw_string_literal — grammar-legal but vanishingly
	// rare in practice (gofmt never emits it); this is the only realistic
	// way to exercise it.
	`errors`
)

// --- calls.scm / tags.scm: call_expression.function variants ---------------

// plain_call — function: identifier
func identity(x int) int {
	return x
}

// scoped_call — function: selector_expression, package-qualified
func sizeOf() int {
	return 0
}

type holder struct {
	value int
}

// method_call — function: selector_expression, method on a value
func (h holder) len() int {
	return h.value
}

func callVariants() {
	_ = identity(1)                  // plain_call
	_ = (identity)(1)                // parenthesized_expression wrapping identifier
	h := holder{value: 5}
	_ = h.len()                      // method_call: selector_expression
	_ = (h.len)()                    // parenthesized_expression wrapping selector_expression
	_ = fmt.Sprintf("%d", sizeOf())  // scoped_call: pkg.Func()
	_ = f.Sprintf("%d", 1)           // scoped_call via aliased import
	_ = Repeat("x", 2)               // dot-imported call: strings.Repeat via `. "strings"`
	err := errors.New("x")
	_ = err
}

// --- calls.scm write-context: @call vs @call.write --------------------------

func writeContextCall() {
	var result int
	result = identity(1)  // @call.write: assignment RHS
	result += identity(2) // @call.write: compound-assignment RHS
	_read := identity(3)  // @call: let-bound (read context, not write)
	_ = result
	_ = _read
}

// --- tags.scm/types.scm: type_spec vs type_alias ----------------------------

// type_spec — a named type definition (struct)
type Plain struct {
	X int
}

// type_alias — `type X = Y`, a distinct node type from type_spec (no `=`
// token appears in a type_spec). Silently unhandled before this fix.
type PlainAlias = Plain

// Generic type_spec: type_parameters is populated but the node is still
// type_spec, so this must be found the same way as Plain.
type Generic[T any] struct {
	Value T
}

// --- tags.scm/types.scm: qualified_type reference ---------------------------

type qualifiedRef struct {
	// qualified_type: package + name fields, both single-variant (fully
	// covered already — included here for completeness-matrix documentation).
	W fmt.Stringer
}

// --- tags.scm: const_spec multi-name completeness ---------------------------

const (
	// const_spec.name field: multiple=true, and per real parse output
	// (verified: only the FIRST identifier in a comma list is actually
	// tagged with the `name` field by tree-sitter-go; later names in the
	// same spec are unfielded children). A field-constrained pattern drops
	// every name after the first; a positional pattern catches all of them.
	SingleConst = 1
	MultiA, MultiB = 2, 3 // both MultiA and MultiB must be found
)

// --- calls.scm/tags.scm: deliberately-excluded call_expression.function ----
// variants (documented negative cases, not bugs — see go.calls.scm/
// go.tags.scm's own comments for the reasoning: none of these have a
// stable, nameable callee).

func higherOrder() func(int) int {
	return func(delta int) int {
		return delta
	}
}

var dispatch = []func(){
	func() { fmt.Println("zero") },
}

func negativeCallVariants() {
	// NEGATIVE: function: call_expression (curried call) — must not
	// produce any @call/@name capture for the outer call.
	_ = higherOrder()(1)

	// NEGATIVE: function: func_literal (immediately-invoked closure) —
	// must not produce any @call/@name capture. This is the idiomatic
	// goroutine/defer shape; excluded ONLY when called inline like this,
	// not when assigned to a name first (see dispatch above, called below).
	func() {
		fmt.Println("iife")
	}()

	// NEGATIVE: function: index_expression (dispatch-table call) — must
	// not produce any @call/@name capture.
	dispatch[0]()
}
