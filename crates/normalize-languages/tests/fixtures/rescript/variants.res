// ReScript completeness matrix — one small construct per node-type variant
// cross-referenced against arborium-rescript-2.17.0's node-types.json, plus
// a NEGATIVE section for constructs that must NOT match.

// --- calls.scm: call_expression.function variants -------------------------

// value_identifier: simple call
let simpleCall = () => plainCall()

// value_identifier_path (2-segment): module-qualified call
let twoSegmentCall = (xs: list<int>) => List.map(xs, x => x)

// value_identifier_path (multi-segment, nested module_identifier_path):
// module-qualified call with @call.qualifier = "Belt.Array"
let qualifiedCall = () => Belt.Array.map([1, 2, 3], x => x)

// member_expression: record/object field call
type callable = {method_: unit => int}
let memberCall = (o: callable) => o.method_()

// pipe-first call: arr->Belt.Array.map(f) — the call_expression is the
// pipe's right operand; already covered by the value_identifier_path
// pattern above once inside the call_expression, this line exercises the
// pipe-expression wrapper specifically.
let pipeCall = (arr: array<int>) => arr->Belt.Array.map(x => x)

// --- complexity.scm / cfg.scm: loops and try/catch -------------------------

let loopFor = () => {
  for i in 0 to 10 {
    Js.log(i)
  }
}

let loopWhile = (r: ref<int>) => {
  while r.contents < 10 {
    r := r.contents + 1
  }
}

let tryCatch = () => {
  try {
    plainCall()
  } catch {
  | Not_found => 0
  }
}

// --- imports.scm: open vs include ------------------------------------------

open Belt
include Belt

// --- tags.scm: definition variants ------------------------------------------

let namedBinding = 5

external externalBinding: int = "externalBinding"

type variantType =
  | VariantA
  | VariantB(int)

module NestedModule = {
  let inner = 1
}

// --- NEGATIVE cases: must NOT match as @call -------------------------------

// Bare field read (no call parens) must never be captured as a call.
let negativeFieldRead = (o: callable) => o.method_

// A value_identifier used only as a call *argument* must never itself be
// captured as @call (only `plainCall` — the function position — should be).
let negativeArgument = () => plainCall(namedBinding)
