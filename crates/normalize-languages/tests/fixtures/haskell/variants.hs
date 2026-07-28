-- Completeness-matrix fixture for haskell.{tags,calls,imports,complexity,types}.scm
--
-- One small, commented construct per node-type variant discovered by
-- cross-referencing each query's field constraints against arborium-haskell's
-- node-types.json (see docs/query-testing-methodology.md). A dedicated
-- NEGATIVE section at the bottom holds near-miss constructs that must NOT
-- match the patterns they resemble.
module Variants where

import qualified Data.Map as Map
-- import_name.variable: variable (plain function/value import)
-- import_name.variable: qualified, id: variable (a qualified name inside a list)
import Data.List (sort, nub)
-- import_name.type: name (a type/class name import)
import Data.Ord (Down)
-- import_name.operator: prefix_id (an operator import)
import Control.Applicative ((<|>))
-- hiding-imports reuse the same import_list/import_name shape as ordinary
-- named imports (no separate grammar node exists for "hiding" — see the
-- comment in haskell.imports.scm).
import Prelude hiding (lookup)

-- ============================================================================
-- tags.scm: function.name variants
-- ============================================================================

-- function.name: variable (the common case)
plainFunc :: Int -> Int
plainFunc x = x + 1

-- function.name: prefix_id (parenthesized custom operator definition)
(+++) :: [a] -> [a] -> [a]
(+++) xs ys = xs ++ ys

-- ============================================================================
-- tags.scm: data_type / newtype / type_synomym / class / instance name
-- variants (variable vs prefix_id form)
-- ============================================================================

-- data_type.name: name (the common case)
data Tree a = TLeaf | TNode a (Tree a) (Tree a)

-- data_type.name: prefix_id (parenthesized infix type constructor)
data (:+:) a b = TL a | TR b

-- newtype.name: name
newtype Count = Count Int

-- newtype.name: prefix_id
newtype (:*:) a = MkTimes a

-- type_synomym.name: name
type Name = String

-- type_synomym.name: prefix_id
type (:->) a b = a -> b

-- class.name: name
class Shape a where
  area :: a -> Double

-- class.name: prefix_id
class (:~:) a b where
  cast :: a -> b

-- instance.name: name (captured as @definition.module; name = class name)
instance Shape Tree where
  area _ = 0.0

-- instance.name: prefix_id
instance (:~:) Int Int where
  cast = id

-- ============================================================================
-- tags.scm: zero-argument / point-free top-level bindings (`bind` node,
-- distinct from `function` — requires no pattern arguments)
-- ============================================================================

-- bind.name: variable (point-free style — no arguments on the LHS)
doubleAll :: [Int] -> [Int]
doubleAll = map (* 2)

-- bind.name: prefix_id (point-free custom operator, defined without an
-- explicit argument pattern)
(<+>) :: Int -> Int -> Int
(<+>) = (+)

-- ============================================================================
-- calls.scm: apply.function variants
-- ============================================================================

-- apply.function: variable (plain local call)
callPlain :: Int
callPlain = plainFunc 1

-- apply.function: constructor (data constructor application)
callConstructor :: Tree Int
callConstructor = TNode 1 TLeaf TLeaf

-- apply.function: qualified, id: variable (qualified function call)
callQualifiedVariable :: Maybe Int
callQualifiedVariable = Map.lookup 1 Map.empty

-- apply.function: qualified, id: constructor (qualified constructor
-- application via the always-in-scope Prelude qualifier)
callQualifiedConstructor :: Maybe Int
callQualifiedConstructor = Prelude.Just (plainFunc 1)

-- apply.function: prefix_id, wrapping a bare operator (operator-section
-- used directly in prefix/function position — e.g. `($)` applied to a
-- function, a pervasive point-free idiom)
callOperatorSection :: Int
callOperatorSection = ($) plainFunc 1

-- apply.function: prefix_id, wrapping a qualified operator
callQualifiedOperatorSection :: Int
callQualifiedOperatorSection = (Prelude.+) 1 2

-- apply.function: parens, expression: variable (redundant parens around a
-- plain identifier used as the call target)
callParenVariable :: Int
callParenVariable = (plainFunc) 1

-- apply.function: parens, expression: qualified (redundant parens around a
-- qualified identifier used as the call target)
callParenQualified :: Maybe Int
callParenQualified = (Map.lookup) 1 Map.empty

-- ============================================================================
-- imports.scm: import_name variants (see the `import ... hiding (...)` /
-- `import ... (...)` forms in this file's own header for a plain import;
-- named-import variants are exercised directly below via probe-only
-- declarations, since GHC would reject genuinely-unused imports as warnings
-- but not as parse errors — these are left as import lines exercising the
-- shapes, not as functioning code).
-- ============================================================================

-- import_name.variable: variable / import_name.type: name / import_name.operator: prefix_id
-- (grouped into one import line to keep the fixture compact — see the
-- completeness test for per-name assertions)

-- ============================================================================
-- complexity.scm: branching-construct variants
-- ============================================================================

-- conditional (if/then/else)
ifExpr :: Int -> String
ifExpr n = if n > 0 then "pos" else "neg"

-- case with plain (unguarded) alternatives — each `alternative` is its own
-- decision point
caseExpr :: Int -> String
caseExpr n = case n of
  0 -> "zero"
  1 -> "one"
  _ -> "many"

-- guard (per-clause guarded equation)
guardExpr :: Int -> String
guardExpr n
  | n > 0 = "pos"
  | otherwise = "non-pos"

-- lambda
lambdaExpr :: Int -> Int
lambdaExpr = \x -> x + 1

-- multi_way_if (MultiWayIf extension — GHC common idiom, replaces nested
-- if/then/else chains)
multiWayIfExpr :: Int -> String
multiWayIfExpr n = if
  | n < 0 -> "neg"
  | n == 0 -> "zero"
  | otherwise -> "pos"

-- lambda_case (LambdaCase extension — `\case` pattern-matching lambda)
lambdaCaseExpr :: Maybe Int -> String
lambdaCaseExpr = \case
  Nothing -> "none"
  Just _ -> "some"

-- ============================================================================
-- types.scm: type.reference variants
-- ============================================================================

-- plain type name
plainType :: Int -> Int
plainType x = x

-- qualified type name (Map.Map)
qualifiedType :: Map.Map Int Int -> Int
qualifiedType m = plainFunc 1

-- generic/applied type (Maybe Int)
genericType :: Maybe Int -> Bool
genericType (Just _) = True
genericType Nothing = False

-- ============================================================================
-- NEGATIVE: constructs that must NOT match the patterns above
-- ============================================================================

-- NEGATIVE (tags): where-bound local helper functions must NOT be tagged as
-- top-level symbols — `function` is also the node type used for local
-- helpers, and only top-level `(declarations (function ...))` children
-- should be tagged.
negOuter :: Int -> Int
negOuter n = negHelper n
  where
    negHelper x = x + 1

-- NEGATIVE (tags): `let`-bound local variables inside a `do` block must NOT
-- be tagged as top-level symbols — `bind` is also the node type for local
-- let-bindings, and only top-level `(declarations (bind ...))` children
-- should be tagged.
negMain :: IO ()
negMain = do
  let negLocal = plainFunc 1
  print negLocal

-- NEGATIVE (calls): `(f . g) x` — point-free composition applied directly.
-- `apply.function` is `parens` wrapping an `infix` expression here, not a
-- single nameable identifier; deliberately NOT matched (composing two named
-- functions is not itself a single "call" to either).
negComposed :: Int -> Int
negComposed = (plainFunc . plainFunc) 1

-- NEGATIVE (complexity): a function with zero branches must report base
-- complexity only — no `match`/equation-wrapper node should ever contribute
-- to @complexity (this was the root-cause bug: every function's own
-- equation body was previously miscounted as a decision point).
negTrivial :: Int -> Int
negTrivial x = x + 1
