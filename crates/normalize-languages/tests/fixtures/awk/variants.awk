#!/usr/bin/gawk -f
# Completeness matrix for AWK/gawk query files.
# Each section is commented with the field/variant it exercises, per
# docs/query-testing-methodology.md step 5. Verified against arborium-awk
# 2.17.0's node-types.json and real parse output (`normalize syntax ast` /
# `normalize syntax query --show-source`).

# imports.scm: @include directive (must produce @import.path = "lib.awk").
@include "lib.awk"

# imports.scm: @load directive (must produce @import.path = "ext").
@load "ext"

# imports.scm NEGATIVE: @namespace is NOT an import — must not be
# captured, even though it shares the identical `directive > string`
# shape as @include/@load (distinguished only by the anonymous keyword
# token, which the old unscoped pattern ignored).
@namespace "mylib"

# tags.scm: func_def.name = identifier (plain function definition).
function plain_fn(x) {
    return x
}

# tags.scm: func_def.name = ns_qualified_name (gawk namespace-qualified
# function definition) — previously entirely unmatched.
function mylib::qualified_fn(x, y) {
    return x + y
}

# calls.scm: func_call.name = identifier (already-handled baseline).
function calls_plain() {
    return plain_fn(1)
}

# calls.scm: func_call.name = ns_qualified_name (already-handled
# baseline — calls.scm had this before tags.scm's definition-side gap was
# found).
function calls_qualified() {
    return mylib::qualified_fn(1, 2)
}

# calls.scm: indirect call via a variable (`@f(...)`) — the wrapped
# func_call node still has its `name` field populated with the variable
# identifier (verified via `normalize syntax ast`: func_call's `name`
# field is NOT required per node-types.json specifically to allow this
# case), so the existing plain-identifier call pattern already covers it
# with no query change needed.
function calls_indirect(    f) {
    f = "plain_fn"
    return @f(1)
}

# complexity.scm/cfg.scm: switch/case (gawk extension) — previously
# entirely absent from both files. switch_statement has NO fields at all
# (not even an unfielded-body situation like if/while/for — the scrutinee
# is a bare positional first child before switch_body).
function switch_demo(n,    result) {
    switch (n) {
        case 0:
            result = "zero"
            break
        case 1:
        case 2:
            result = "small"
            break
        default:
            result = "large"
    }
    return result
}
