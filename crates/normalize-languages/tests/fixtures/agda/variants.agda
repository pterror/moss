-- Completeness matrix for Agda query files.
-- Each section is commented with the field/variant/shape it exercises, per
-- docs/query-testing-methodology.md step 5. Verified against arborium-agda
-- 2.17.0's node-types.json AND real parse output (`normalize syntax ast` /
-- `normalize syntax query --show-source`) — this grammar's node-types.json
-- has no `fields` at all (everything is positional children), and several
-- declared node types (`type_signature`) are never actually produced, so
-- CST verification mattered more here than for field-based grammars.
module Variants where

-- imports.scm: plain `import` statement.
import Data.List

-- imports.scm: `open import` — combined open+import (open > import >
-- module_name). Must NOT double-count as two @import entries for one
-- statement (regression case for a real duplicate-capture bug).
open import Data.Maybe using (Maybe)

-- imports.scm: bare `open ModuleName` — glob-opening an already-imported
-- module (open > module_name directly, no nested `import`).
open Data.List

-- tags.scm/types.scm: `function > lhs > function_name`, rhs led by `:`
-- (type signature form) immediately followed by rhs led by `=` (defining
-- equation form) — the two forms share one node type, distinguished only
-- by the leading rhs token.
identity : Int → Int
identity n = n

-- tags.scm: `function > lhs`'s name-bearing first child is a BARE `atom`
-- (no `function_name` wrapper) in the defining-equation form — previously
-- entirely unmatched, meaning tags.scm only ever captured a function via
-- its type signature, never its actual `= ...` equation. `pointfree` has
-- no signature at all (legal, and common during incremental Agda
-- development), so it was completely invisible to tags before this fix.
pointfree : Int
pointfree = 42

-- tags.scm negative: `where`-bound local helpers must NOT be captured as
-- top-level definitions — regression test for a real leak. `function` is
-- also the node type for `where` clauses (`hasHelper n = helper n where
-- helper x = x` parses as `function > where > function`, the helper never
-- a direct child of source_file/module/private/abstract/instance/mutual),
-- and an earlier, unscoped version of this query captured `helper`'s
-- signature as if it were a top-level symbol (confirmed via
-- `normalize syntax ast` + `normalize syntax query` on a `where` probe).
hasHelper : Int → Int
hasHelper n = helper n
  where
    helper : Int → Int
    helper x = x

-- tags.scm: data constructor declarations parse with the identical
-- `function > lhs > function_name` shape as identity's signature above —
-- the grammar has no distinct "constructor" node.
data Color : Set where
  Red   : Color
  Green : Color
  Blue  : Color

-- types.scm: record field signature (`signature > field_name : expr`,
-- NOT the double-nested `signature > signature > function_name` shape an
-- earlier, dead pattern assumed).
record Box : Set where
  field
    contents : Int

-- calls.scm: application at the head of a defining equation
-- (`rhs "=" . (expr . (atom . (qid)) . (atom))`), with a second argument
-- atom present so it isn't mistaken for a bare reference.
addOne : Int → Int
addOne n = suc n

-- calls.scm: nested/parenthesized application — only the OUTER call
-- (`addOne`) is captured; the inner `addOne n` is not reachable by the
-- rhs-anchored pattern (documented coverage limit, not a false positive).
addTwo : Int → Int
addTwo n = addOne (addOne n)

-- NEGATIVE: a bare single-atom rhs (no second argument atom) must NOT be
-- captured as a call — this used to false-positive because numeric
-- literals parse as `qid` atoms in this grammar (verified via
-- `normalize syntax ast`: `answer = 42`'s rhs expr is `atom(qid "42")`,
-- structurally identical to a bare identifier reference).
answer : Int
answer = 42

-- NEGATIVE: infix operator use is a KNOWN, documented residual false
-- positive (not something this fixture asserts is clean) — `m` here would
-- still be captured as a bogus @call under the current query because the
-- grammar cannot distinguish `f x` from `m + m` without fixity
-- resolution. Kept here as a marked, honest limitation rather than
-- silently working around it with an unverifiable heuristic.
addSelf : Int → Int
addSelf m = m + m
