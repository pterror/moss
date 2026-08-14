#!/usr/bin/env fish
# Completeness matrix for fish.{calls,tags,complexity,imports,cfg}.scm.
#
# `command.name` and `function_definition.name` both allow the same 12
# node-type variants per arborium-fish 2.17.0's node-types.json:
# brace_expansion, command_substitution, concatenation, double_quote_string,
# escape_sequence, float, glob, home_dir_expansion, integer,
# single_quote_string, variable_expansion, word.
# Every variant below is confirmed via `normalize syntax query` against a
# real probe before being added here — see fish.calls.scm/fish.tags.scm for
# the accompanying rationale comments.

# --- command.name variants (fish.calls.scm @call) --------------------------

# word — the common case, a bare command name.
echo word_variant

# variable_expansion — dispatch-by-variable, a real dotfile idiom
# (`$EDITOR file`, `$PAGER`, `$BROWSER url`).
set cmd echo
$cmd variable_expansion_variant

# concatenation — a quoted prefix immediately followed by a bare suffix
# with no space, e.g. `"$prefix"ho`.
set prefix ec
"$prefix"ho concatenation_variant

# command_substitution — the command name is itself the stdout of a
# subshell, a dynamic-command idiom.
(echo echo) command_substitution_variant

# double_quote_string — a fully quoted command name.
"echo" double_quote_string_variant

# single_quote_string — a fully single-quoted command name.
'echo' single_quote_string_variant

# brace_expansion — a bare brace-expansion command name (fish expands this
# to multiple command invocations at runtime).
{echo,echo} brace_expansion_variant

# glob — a bare glob command name.
* glob_variant

# home_dir_expansion — a bare `~` command name.
~ home_dir_expansion_variant

# escape_sequence — a bare escape-sequence command name.
\n escape_sequence_variant

# float — a bare numeric-looking command name (fish does not restrict
# command names to identifier-shaped words).
1.5 float_variant

# integer — a bare integer-looking command name.
42 integer_variant

# --- function_definition.name variants (fish.tags.scm @name) ---------------

# word — the common case.
function word_fn_name
    echo word_fn
end

# double_quote_string — quoting lets a function name contain spaces or
# other characters a bare word cannot.
function "quoted fn name"
    echo quoted_fn
end

# --- fish.complexity.scm: conditional_execution (and/or) -------------------

# `and`/`or` each wrap in their own `conditional_execution` node and are
# each a real decision point (mirrors bash's `&&`/`||` treatment).
test -f /tmp/nonexistent.txt
and echo and_variant
or echo or_variant

# --- fish.imports.scm: source / . (legacy alias) ----------------------------

source variants_sourced.fish

# `.` is a legacy alias for `source`, still a working builtin in fish 4.8.0
# (confirmed via `fish -c 'type .'`).
. variants_dot_sourced.fish

# `argument` is a `multiple` field on `command` — trailing words after the
# sourced path are passed as positional $argv to the sourced script, not
# additional paths. The `.` anchor in fish.imports.scm restricts the match
# to the first `argument` only; without it this line would spuriously
# produce two @import.path captures.
source variants_sourced_with_args.fish trailing_arg1 trailing_arg2

# ---------------------------------------------------------------------------
# NEGATIVE — constructs that must NOT match as imports/complexity/etc.
# ---------------------------------------------------------------------------

# `not test ...` is a `negated_statement`, a distinct node kind from
# `conditional_execution` (and/or). fish.complexity.scm intentionally does
# NOT count negation as a decision point (mirrors bash, which only counts
# `&&`/`||`, never plain `!`) — a single `not` doesn't add a branch, it
# inverts one predicate's truth value.
not test -f /tmp/nonexistent.txt

# `source` used as a plain argument value (not as the invoked command) must
# not be mistaken for an import — this is `echo`'s argument, not a sourced
# path.
echo source

# A command literally named "sourceish" must not false-positive-match the
# `#eq? @_cmd "source"` predicate (exact-text match, not prefix match).
sourceish some_file.fish

# A command literally named "dot" (not the single-character "." builtin)
# must not false-positive-match the `.` alias predicate.
dot some_file.fish
