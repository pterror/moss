#!/usr/bin/env bash
# Completeness matrix for bash.{tags,calls,imports,complexity}.scm.
# One small, commented construct per node-type variant found by
# cross-referencing arborium-bash 2.17.0's node-types.json
# (docs/query-testing-methodology.md step 2), plus a NEGATIVE section of
# near-miss constructs that must not match.

# ---------------------------------------------------------------------------
# tags.scm: function_definition.name is always `word`, but there are two
# distinct *syntactic* forms of function_definition that must both parse to
# that same shape.
# ---------------------------------------------------------------------------

# Variant: `function NAME { ... }` — keyword form, no parens.
function fn_keyword_no_parens {
    echo "a"
}

# Variant: `function NAME() { ... }` — keyword form with parens.
function fn_keyword_with_parens() {
    echo "b"
}

# Variant: `NAME() { ... }` — POSIX form, no keyword (the common form).
fn_posix_form() {
    echo "c"
}

# Variant: function body is `if_statement` rather than `compound_statement`
# (function_definition.body allows if_statement/subshell/test_command too,
# not just `{ }`).
function fn_body_if_statement
if [[ -z "$1" ]]; then
    echo "empty"
else
    echo "nonempty"
fi

# ---------------------------------------------------------------------------
# calls.scm: (command name: (command_name) @call) — command_name always
# wraps exactly one child, but that child varies by _primary_expression
# variant (and the concatenation supertype). Enumerate the ones bash scripts
# actually produce.
# ---------------------------------------------------------------------------

# Variant: bare word command name (the overwhelmingly common case).
plain_command_call() {
    ls
}

# Variant: relative-path command name.
relative_path_call() {
    ./script.sh
}

# Variant: quoted command name (string wrapping command_name).
quoted_command_call() {
    "ls" -la
}

# Variant: command name via simple variable expansion.
variable_command_call() {
    local cmd="ls"
    $cmd -la
}

# Variant: command name via braced expansion.
braced_variable_command_call() {
    local cmd="ls"
    ${cmd} -la
}

# Variant: command inside a pipeline — each pipeline stage is its own
# `command` node, independently captured.
pipeline_call() {
    ls | grep foo | sort
}

# Variant: command inside a subshell.
subshell_call() {
    (ls)
}

# Variant: command inside `$( )` command substitution.
command_substitution_call() {
    local out
    out="$(ls -la)"
    echo "$out"
}

# Variant: command inside a negated_command (`! cmd`).
negated_call() {
    ! ls
}

# ---------------------------------------------------------------------------
# imports.scm: `source`/`.` with the `.` field anchor restricting to the
# FIRST argument only (a `source file.sh arg1 arg2` command passes arg1/arg2
# as positional params to the sourced script — they are not import paths).
# ---------------------------------------------------------------------------

# Variant: `source path` — bare word path.
source ./plain_path.sh

# Variant: `. path` — POSIX dot-command form.
. ./dot_path.sh

# Variant: `source "path"` — quoted string path.
source "./quoted_path.sh"

# Variant: `source $var` — simple-expansion path.
lib_path="./lib.sh"
source $lib_path

# Variant: `source "$var/sub.sh"` — string containing an expansion.
source "$lib_path/sub.sh"

# Variant: source with trailing positional args passed to the sourced
# script — only the FIRST argument (the path) must be captured as
# @import.path; `arg1`/`arg2` must not.
source ./with_args.sh arg1 arg2

# ---------------------------------------------------------------------------
# complexity.scm variants.
# ---------------------------------------------------------------------------

# Variant: if_statement + elif_clause (flat siblings, not nested — each
# elif is one complexity point, not compounding).
complexity_if_elif() {
    if [[ "$1" == "a" ]]; then
        echo a
    elif [[ "$1" == "b" ]]; then
        echo b
    elif [[ "$1" == "c" ]]; then
        echo c
    else
        echo d
    fi
}

# Variant: for_statement (word-list form).
complexity_for() {
    for x in a b c; do
        echo "$x"
    done
}

# Variant: c_style_for_statement — distinct node type from for_statement.
complexity_c_style_for() {
    local i
    for (( i = 0; i < 10; i++ )); do
        echo "$i"
    done
}

# Variant: while_statement.
complexity_while() {
    local i=0
    while (( i < 3 )); do
        (( i++ ))
    done
}

# Variant: while_statement with `until` keyword — same node type
# (while_statement), just an anonymous "until" token instead of "while".
complexity_until() {
    local i=0
    until (( i >= 3 )); do
        (( i++ ))
    done
}

# Variant: case_statement + case_item (one complexity point per case_item,
# regardless of how many `|`-separated patterns share that item).
complexity_case() {
    case "$1" in
        a|b) echo ab ;;
        c) echo c ;;
        *) echo default ;;
    esac
}

# Variant: pipeline.
complexity_pipeline() {
    ls | grep foo
}

# Variant: list (&&/|| chain at the statement level).
complexity_list_and() {
    true && echo yes
}

complexity_list_or() {
    false || echo fallback
}

# Variant: binary_expression with && inside a test/arithmetic condition
# (logical short-circuit — a genuine decision point).
complexity_binary_and() {
    if [[ -n "$1" && -n "$2" ]]; then
        echo both
    fi
}

# Variant: binary_expression with || inside a test condition.
complexity_binary_or() {
    if [[ -z "$1" || -z "$2" ]]; then
        echo either
    fi
}

# Variant: ternary_expression (arithmetic-context `? :`).
complexity_ternary() {
    local -i n="$1"
    local -i sign=$(( n > 0 ? 1 : -1 ))
    echo "$sign"
}

# ---------------------------------------------------------------------------
# NEGATIVE section — constructs that must NOT match.
# ---------------------------------------------------------------------------

# Negative: a variable READ ($VAR) must not be mistaken for a function tag,
# call, or import — it produces no command/function_definition/source node.
negative_variable_read() {
    local VAR="x"
    echo "$VAR"
}

# Negative: a plain assignment (VAR=value, no command substitution) must not
# be captured as a @call — variable_assignment is a distinct node type from
# command, with no command_name field.
negative_plain_assignment() {
    local PLAIN_VAR=42
}

# Negative: a string that merely CONTAINS command-like or source-like text
# must not produce any tags/calls/imports/complexity captures — it's a
# string_content leaf, not an executed command or a control-flow node.
negative_string_looks_like_command() {
    echo "source ./not_actually_sourced.sh"
    echo "function not_a_real_function() { echo hi; }"
    echo "if this were real it would break; then echo would; fi"
}

# Negative: a comment containing shell-like syntax must not match anything —
# it's a `comment` leaf node, invisible to every query in this file.
# source ./commented_out_import.sh
# function commented_out_fn() { echo no; }
# if false; then echo no; fi

# Negative: a function name appearing inside a string (not as an actual
# function_definition) must not produce a tags @name capture.
negative_function_name_in_string() {
    local msg="calling function fn_posix_form now"
    echo "$msg"
}

# Negative: plain arithmetic inside `(( ))` must NOT contribute complexity —
# binary_expression is only counted when its operator is literally "&&" or
# "||"; ordinary comparison/arithmetic operators (<, >, +=, ...) must not
# match the complexity query at all.
negative_arithmetic_not_complexity() {
    local -i total=0
    (( total += 1 ))
    (( total < 100 ))
}
