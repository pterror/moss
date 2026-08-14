# variants.r — R query-completeness matrix.
#
# One small, commented construct per node-type variant identified by
# cross-referencing r.{tags,calls,complexity,imports}.scm against
# node-types.json (arborium-r 2.17.0). See docs/query-testing-methodology.md.
# A dedicated NEGATIVE section at the bottom lists near-miss constructs that
# must NOT be captured by the relevant query.

# --- tags: @definition.function, all binary_operator forms -----------------

# lhs: (identifier), operator "<-"
left_arrow_fn <- function(x) x

# lhs: (identifier), operator "="
equals_fn = function(x) x

# lhs: (identifier), operator "<<-" (global/superassignment)
outer_env_fn <- function() {
  inner <<- function(x) x
}

# lhs: (extract_operator) via `$` — R6/environment-style method definition,
# operator "<-"
obj_env <- new.env()
obj_env$method_dollar <- function(x) x

# lhs: (extract_operator) via `$`, operator "="
obj_env$method_dollar_eq = function(x) x

# Right-assignment: (parenthesized_expression (function_definition)) -> name.
# The function *must* be parenthesized — R's `->`/`->>` bind looser than a
# bare `function(...) body`, so an unparenthesized function swallows the
# arrow into its own body instead of naming the whole definition.
(function(x) x + 1) -> right_arrow_fn

# Right-assignment with "->>" (global right-assign)
(function(x) x * 2) ->> right_arrow_global_fn

# Lambda shorthand (R >= 4.1): function_definition.name == "\" instead of
# "function" — same node type, no separate query clause needed. Included
# here to document that the tags query's `rhs: (function_definition)` field
# constraint does not need to (and cannot, since name isn't a captured
# field here) distinguish the two spellings.
lambda_fn <- \(x) x + 1

# --- calls: @call / @call.qualifier, all call.function variants ------------

# function: (identifier) — plain call
plain_call_target <- function() 1
plain_call_target()

# function: (namespace_operator) — pkg::fn() / pkg:::fn()
stats_call <- stats::median(1:3)
internal_call <- base:::.Internal

# function: (extract_operator) via `$` — method-style call on a
# list/environment (R6-lite / Reference Class idiom).
receiver_env <- new.env()
receiver_env$run <- function() 1
receiver_env$run()

# function: (extract_operator) via `@` (S4 slot access) is the same node
# type/field shape as `$` per node-types.json (extract_operator.operator
# allows both "$" and "@"); no separate query clause needed. Not
# independently exercised here since S4 classes require setClass()
# scaffolding this fixture doesn't otherwise use.

# function: (subset2) — obj[["name"]]() call via double-bracket subsetting.
# The callee name lives inside a string argument, not a named field, so
# only the call site + qualifier are captured (no resolvable call name).
bracket2_holder <- list(fn = function() 1)
bracket2_holder[["fn"]]()

# function: (subset) — obj["name"]() call via single-bracket subsetting.
bracket1_holder <- list(fn = function() 1)
bracket1_holder["fn"]()

# --- imports: @import / @import.path ----------------------------------------

# library(pkg) — bareword
library(utils)

# library("pkg") — quoted string
library("tools")

# library(pkg, character.only = TRUE) — positional arg only, the trailing
# named argument must NOT be captured as a second import.path.
library(utils, character.only = TRUE)

# library(package = "pkg") — named argument; import.path must resolve to
# the string value, not the whole "package = ..." text.
library(package = "methods")

# require(pkg)
require(grDevices)

# requireNamespace("pkg") — soft/conditional dependency check.
requireNamespace("grid")

# --- complexity: @complexity / @nesting, all branch/loop/logical variants --

if (TRUE) {
  1
}

if (TRUE) {
  1
} else {
  2
}

for (i in 1:3) {
  i
}

while (FALSE) {
  1
}

repeat {
  break
}

# "&&" / "||" short-circuit logical operators add complexity but not nesting.
if (TRUE && FALSE) {
  1
}
if (TRUE || FALSE) {
  1
}

# --- NEGATIVE cases: must NOT match ------------------------------------------

# A bare function literal (not assigned to anything) must not be tagged as
# @definition.function — it has no binary_operator wrapper at all.
(function(x) x)(1)

# A plain (non-`->`) binary_operator whose rhs is a function_definition but
# whose operator is something other than <-, =, <<-, ->, ->> (e.g. `==`)
# must not be tagged. `==` doesn't type-check against a function on the
# rhs in real R, but the grammar itself is permissive — verifying the
# operator field constraint, not R's runtime semantics, is what matters
# here.
some_flag <- FALSE
# (left deliberately not constructed with `==` + function rhs: R's parser
#  accepts it, but no real code does this, and it adds no coverage beyond
#  the operator field-list check already done against node-types.json.)

# A `$`-access that is only a field *read*, not a call, must not appear in
# @call captures — only `receiver_env$run()` above (the *called* form)
# should produce a @call/@call.qualifier pair.
negative_field_read <- receiver_env$run

# A `[[`/`[`-access that is only a subscript *read*, not a call, must not
# appear in @call captures.
negative_bracket_read <- bracket2_holder[["fn"]]
negative_bracket1_read <- bracket1_holder["fn"]

# `library` / `require` / `requireNamespace` used as plain identifiers
# (not called) must not produce an @import.
library_as_value <- library

# A call to an unrelated function sharing no name with library/require/
# requireNamespace must not be captured as an import.
loadNamespace("stats")
