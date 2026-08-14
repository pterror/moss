// Completeness matrix for fsharp.{tags,calls,imports,types,complexity}.scm.
// One construct per node-type/field variant found in arborium-fsharp
// 2.17.0's node-types.json, plus a NEGATIVE section for constructs that
// must not match certain captures.
module VariantsModule

open System

// --- imports.scm: plain (dotted) namespace ---
open System.Text

// --- tags.scm: plain function definition ---
let variantFunction x = x + 1

// --- tags.scm: named_module — dotted module name must be captured as ONE
// @name ("Outer.Inner"), not one capture per path component. Exercised via
// a nested module below since a file-level `module A.B` can only appear
// once per file.
module Nested =
    module Deep =
        let inner () = 1

// --- tags.scm: type_name as plain identifier ---
type SimpleUnion =
    | CaseA
    | CaseB of int

// --- tags.scm: member_defn / method_or_prop_defn — instance method,
// instance property, static factory method. `property_or_ident`'s name is
// nested (this.Name has TWO identifier children; a static member has ONE)
// — see fsharp.tags.scm's header comment on this pattern.
type VariantClass(seed: int) =
    let mutable state = seed
    member this.Increment(step: int) =
        state <- state + step
        state
    member this.State = state
    static member Zero() = VariantClass(0)

// --- calls.scm: plain function application ---
let variantCallResult = variantFunction 1

// --- calls.scm: qualified call (2-component long_identifier) ---
let variantQualifiedCall = Math.Sqrt 4.0

// --- calls.scm: dot_expression method call (obj.Method(args)) ---
let variantMethodCall =
    let c = VariantClass.Zero()
    c.Increment(5)

// --- calls.scm: dot_expression with multi-component field (grammar splits
// a 4-component dotted call target 2-and-2: base "System.Collections",
// field "Generic.List") ---
let variantDeepDottedCall () = System.Collections.Generic.List<int>()

// --- types.scm: simple_type ---
let variantSimpleType (n: int) = n

// --- types.scm: generic_type ---
let variantGenericType (xs: List<int>) = xs

// --- types.scm: atomic_type via a `:?` type-test pattern ---
let variantAtomicType (o: obj) =
    match o with
    | :? System.String -> "string"
    | _ -> "other"

// --- complexity.scm: if / elif / else (branch) ---
let variantIfElif n =
    if n < 0 then "neg"
    elif n = 0 then "zero"
    else "pos"

// --- complexity.scm: rule (match arm) ---
let variantMatchRules n =
    match n with
    | 0 -> "zero"
    | 1 -> "one"
    | _ -> "many"

// --- complexity.scm: for_expression (loop) ---
let variantForLoop xs =
    let mutable total = 0
    for x in xs do
        total <- total + x
    total

// --- complexity.scm: while_expression (loop) ---
let variantWhileLoop n =
    let mutable i = n
    while i > 0 do
        i <- i - 1
    i

// --- complexity.scm: try_expression ---
let variantTry () =
    try
        1
    with
    | _ -> 0

// --- complexity.scm: infix_expression with && / || (boolean, must count) ---
let variantBooleanInfix a b = a > 0 && (b > 0 || a = b)

// ---------------------------------------------------------------------------
// NEGATIVE section
// ---------------------------------------------------------------------------

// complexity.scm: arithmetic/comparison infix operators must NOT count as
// complexity — only && / || should. This function has zero branches and
// zero boolean operators, so it must score minimal complexity.
let negativeArithmeticInfix a b = a + b - a * b / 2

// tags.scm: a plain top-level `let` value binding (no lambda/case-lambda
// value) must be @definition.function via function_declaration_left, not
// spuriously duplicated — it has no `(define name value)`-style ambiguity
// in this grammar (F# always uses function_declaration_left for `let`
// bindings, unlike Scheme's `define`), included here as a completeness
// check rather than a true near-miss.
let negativePlainLet = 42
