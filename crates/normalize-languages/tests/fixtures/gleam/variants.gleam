// Completeness fixture: one construct per grammar-legal variant of each field
// the gleam.{tags,calls,imports,decorations}.scm queries constrain,
// cross-referenced against arborium-gleam 2.17.0's node-types.json.
// Every construct here is *expected to be captured*; see query_fixtures.rs
// `gleam_*_completeness_*` tests for the matrix.
//
// This file also carries a NEGATIVE section (near-miss constructs that must
// NOT be captured) to guard against over-broad patterns.

import gleam/list.{type Option, Some as MySome} // unqualified_import with alias: import.alias = "MySome"
import gleam/result as res // whole-import alias: import.alias = "res"

// --- function definitions ---------------------------------------------------

pub fn plain_function(x: Int) -> Int {
  x
}

// external_function: distinct node type from `function`, legacy FFI-binding
// syntax; still parses without error.
pub external fn native_add(a: Int, b: Int) -> Int =
  "erlang" "+"

// --- function_call.function variants ---------------------------------------

fn plain_call() {
  identity(1) // function: identifier
}

fn qualified_call() {
  list.length([1, 2]) // function: field_access (record: identifier, field: label)
}

fn identity(x: Int) -> Int {
  x
}

// Point-free pipe target: `identity` invoked with no call parens via `|>`.
// binary_expression(operator: "|>", right: identifier) — NOT wrapped in a
// function_call node, verified via `normalize syntax ast`.
pub fn pipe_bare_identifier(x: Int) -> Int {
  x |> identity
}

// Qualified pipe target (already covered structurally: the right side is a
// function_call whose function is a field_access, matched by the existing
// qualified_call pattern above — included here for documentation, not a
// distinct capture path).
pub fn pipe_qualified(values: List(Int)) -> Int {
  values |> list.length
}

// --- type definitions --------------------------------------------------------

pub type Color {
  Red
  Green
  Blue
}

pub type Meters =
  Float

// --- decorations: attributes -------------------------------------------------

@deprecated("use plain_function instead")
pub fn legacy_function(x: Int) -> Int {
  x
}

@external(erlang, "math", "sqrt")
pub fn sqrt(x: Float) -> Float

// --- NEGATIVE cases: must not be captured -----------------------------------

pub fn negative_cases(x: Int) -> Int {
  // A bare field access with no call parens must never appear in @call.
  let holder = Color
  let _tag = holder

  // An anonymous_function (closure) definition site must never appear as a
  // @definition.function tag — only named `function`/`external_function`
  // nodes should.
  let add_one = fn(n: Int) -> Int { n + 1 }

  // Calling the closure IS a plain call (function: identifier), captured
  // normally — included so the negative test can assert the closure's
  // *definition* site is absent while its *call* site is present.
  add_one(x)
}
