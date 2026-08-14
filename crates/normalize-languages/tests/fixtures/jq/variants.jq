# variants.jq — completeness matrix for jq's query files.
#
# One small, clearly commented construct per node-type/token variant found
# by cross-referencing arborium-jq 2.17.0's node-types.json against
# jq.{tags,calls,imports,complexity,cfg}.scm, verified via
# `normalize syntax ast` / `normalize syntax query`. jq's grammar has no
# named fields at all (node-types.json shows an empty `fields: {}` for
# every node) — completeness here is about node-*type* and anonymous-
# *token* coverage, not field-variant coverage as in field-rich grammars.

# --- funcdef / @definition.function (jq.tags.scm) ------------------------

# Zero-arg funcdef: no funcdefargs node at all.
def zero_arg: .;

# Funcdef with identifier params.
def ident_params(a; b): a + b;

# Funcdef with $-variable params (value args, called as `f(1; 2)` but bound
# like `1 as $a`).
def var_params($a; $b): $a + $b;

# --- funcname / @call / @reference.call (jq.calls.scm, jq.tags.scm) ------

# Zero-arg call (no parens at all): funcname node is the entire call site.
def use_zero_arg: zero_arg;

# Call with args.
def use_ident_params: ident_params(1; 2);

# Nested call: outer/inner funcname both matched independently.
def use_nested: ident_params(var_params(1; 2); 3);

# Builtin call with no user funcdef (still a funcname node — calls.scm/
# tags.scm don't distinguish builtin vs. user-defined, correctly, since the
# grammar has no such distinction either).
def use_builtin: length;

# --- import_ / @import, @import.path, @import.alias (jq.imports.scm) -----
# (import_ statements must appear before any other definitions per jq's
# grammar, so these are declared at the top of the *real* import test in
# probe files rather than here — jq.imports.scm's completeness is instead
# covered directly against fixtures/jq/sample.jq's own
# `import "lib/utils" as utils;` line, which exercises the bare-identifier
# alias variant that was the actual bug found in this sweep.)

# --- if / elif / else / catch (jq.complexity.scm, jq.cfg.scm) ------------

# Bare if/then/end: no elif, no else. Exercises the ("if") @complexity /
# @cfg.branch fix (previously zero coverage for this, the single most
# common jq conditional form).
def bare_if: if . > 0 then "positive" end;

# if/then/else/end: no elif.
def if_else: if . > 0 then "positive" else "non-positive" end;

# if/then/elif/then/else/end: exercises the named `elif` node and its own
# @cfg.branch.condition/@cfg.branch.then sub-captures.
def if_elif_else:
  if . > 0 then "positive"
  elif . < 0 then "negative"
  else "zero"
  end;

# Multiple elif clauses (elif is `(multiple: true)` per node-types.json —
# verify more than one is matched, not just the first).
def if_elif_elif_else:
  if . == 1 then "one"
  elif . == 2 then "two"
  elif . == 3 then "three"
  else "other"
  end;

# --- reduce / foreach (jq.complexity.scm, jq.cfg.scm) ---------------------

# reduce: 2-part form (INIT; UPDATE).
def reduce_sum: reduce .[] as $x (0; . + $x);

# foreach: 3-part form (INIT; UPDATE; EXTRACT) — the form the CFG query
# deliberately does NOT decompose into loop.condition/loop.body (see
# jq.cfg.scm's header comment on why).
def foreach_running_sum: foreach .[] as $x (0; . + $x; .);

# --- try / catch (jq.complexity.scm, jq.cfg.scm) --------------------------

# try with catch — exercises the named `catch` node.
def try_catch_expr: try error("boom") catch .;

# try with NO catch — catch is optional; @cfg.try must still match without
# requiring a `catch` sibling.
def try_no_catch: try error("boom");

# --- and / or short-circuit (jq.complexity.scm) ---------------------------

def logical_and: (true and false);
def logical_or: (true or false);

# --- NEGATIVE cases: constructs that must NOT match -----------------------

# `//` (alternative operator) is intentionally NOT counted as @complexity —
# matches this codebase's convention (established for `??`/`?.` in
# javascript.complexity.scm/typescript.complexity.scm) of only counting
# actual short-circuit *boolean* operators (`and`/`or`, mirroring bash's
# `&&`/`||`), not fallback/optional-chaining operators.
def alternative_not_counted: (.a // .b);

# `?` (try-shorthand suffix, `expr?` == `try expr`) is intentionally NOT
# counted as @complexity for the same reason — it's a suppression operator,
# not a branch. (The full `try`/`catch` forms above ARE counted via `catch`
# for @complexity and via @cfg.try for CFG, since those genuinely branch.)
def optional_not_counted: (.a?);

# funcdefargs' own parameter identifiers must NOT be captured by
# `(funcdef (identifier) @name)` in jq.tags.scm — they're nested one level
# deeper inside `funcdefargs`, not a direct child of `funcdef`. Verified via
# `normalize syntax ast`: only `ident_params` itself (not `a`/`b`) is a
# direct-child identifier of its funcdef node above.

# A funcname that appears as a call *argument* (not as the query's own
# first child) must NOT be matched by jq.tags.scm's
# `(query . (funcname) @name) @reference.call` anchor pattern as if it were
# the outer call — each nested call gets its OWN match (see use_nested
# above), the anchor only prevents a non-call query from being
# mis-captured, not nested calls from being captured at all.
def negative_anchor_check: ident_params(use_builtin; 2);

# Main pipeline exercising the fixture's own definitions, so the whole file
# parses as one coherent program (not just a bag of unused defs).
.
| bare_if,
  if_else,
  if_elif_else,
  if_elif_elif_else,
  reduce_sum,
  foreach_running_sum,
  try_catch_expr,
  try_no_catch,
  logical_and,
  logical_or,
  alternative_not_counted,
  optional_not_counted,
  use_nested,
  negative_anchor_check
