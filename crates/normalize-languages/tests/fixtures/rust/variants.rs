// Completeness fixture: one construct per grammar-legal variant of each field
// the rust.{tags,calls,imports}.scm queries constrain, cross-referenced against
// arborium-rust's node-types.json. Every construct here is *expected to be
// captured*; see query_fixtures.rs `rust_completeness_*` tests for the matrix.
//
// This file also carries a handful of deliberate near-miss constructs (marked
// NEGATIVE) that must NOT be captured by the query under test, to guard
// against over-broad patterns.

use std::io::{self, Read, Write}; // self-import: import.name = "self"
use std::io::{self as io_alias};  // aliased self-import
use std::collections::HashMap; // plain multi-segment import
use std::collections::{HashSet, BTreeMap as Tree}; // multi-name, with an aliased member
use std::env::*; // wildcard import
pub use std::fmt::Debug as DebugTrait; // aliased re-export

// --- call_expression.function variants -------------------------------------

fn plain_call() {
    identity(1); // function: identifier
}

fn scoped_call() {
    std::mem::drop(1); // function: scoped_identifier
}

fn method_call() {
    let v: Vec<i32> = Vec::new();
    v.len(); // function: field_expression
}

fn turbofish_plain_call() {
    identity::<i32>(1); // function: generic_function -> function: identifier
}

fn turbofish_scoped_call() {
    std::mem::size_of::<i32>(); // function: generic_function -> function: scoped_identifier
}

fn turbofish_method_call() {
    "5".parse::<i32>().unwrap(); // function: generic_function -> function: field_expression
}

fn identity(x: i32) -> i32 {
    x
}

// --- write-context call variants (@call.write vs plain @call) --------------

fn write_context_call() {
    let mut result;
    result = identity(1); // assignment RHS: @call.write, function: identifier
    result += identity(2); // compound assignment RHS: @call.write
    let _read = identity(3); // `let` RHS is NOT an assignment_expression: plain @call, never @call.write
    let _ = result;
}

// --- impl_item.type / impl_item.trait variants ------------------------------

struct Plain;
struct Generic<T>(T);

impl Plain {} // type: type_identifier

impl<T> Generic<T> {} // type: generic_type -> type: type_identifier

impl std::fmt::Debug for Plain {
    // trait: scoped_type_identifier
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plain")
    }
}

impl From<i32> for Plain {
    // trait: generic_type -> type: type_identifier
    fn from(_: i32) -> Self {
        Plain
    }
}

impl std::ops::Add<i32> for Plain {
    // trait: generic_type -> type: scoped_type_identifier
    type Output = Plain;
    fn add(self, _: i32) -> Plain {
        self
    }
}

// --- NEGATIVE cases: must not be captured as calls/definitions -------------

struct NegativeHolder {
    field: i32,
}

fn negative_cases(holder: &NegativeHolder) -> i32 {
    // A closure literal is not a `function_item`. It must never appear in
    // tags @definition.function / @definition.method captures.
    let add_one = |x: i32| x + 1;

    // A bare field access with no call parens must never appear in any
    // @call capture (regression guard against over-eager field_expression
    // patterns matching plain member access).
    let _read = holder.field;

    // `let`-bound call results are plain reads (@call), never @call.write —
    // @call.write is reserved for assignment_expression/compound_assignment_expr
    // RHS, which `let` is not.
    let _bound = add_one(1);

    // Known grammar-level ambiguity (documented, not asserted against):
    // tuple-struct construction `Generic(1)` parses identically to a plain
    // call_expression(function: identifier) and WILL be captured by @call
    // even though it is not a function call. This is a tree-sitter/CST
    // limitation shared with every "identifier(...)" pattern, not a bug in
    // our queries — see docs/rust-query-completeness-methodology.md.
    let _ctor = Generic(1);

    _bound
}
