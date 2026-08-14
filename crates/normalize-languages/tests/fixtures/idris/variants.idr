module Variants

-- Completeness-matrix fixture for idris query files. One small, commented
-- construct per node-type variant discovered while cross-referencing
-- idris.{calls,tags,imports,types,cfg,complexity}.scm against arborium's
-- node-types.json (see docs/query-testing-methodology.md).

import Data.List
import public Data.String

-- calls.scm: exp_name variants -----------------------------------------

-- (exp_name (loname)): simple name reference / call
plainCall : Int -> Int
plainCall x = identity x

-- (exp_name (qualified_loname)): qualified reference / call
qualCall : Int -> Int
qualCall x = Data.List.length [x]

-- (exp_name (caname)): constructor call
ctorCall : Maybe Int
ctorCall = Just 1

-- (exp_name (qualified_caname)): qualified constructor call
qualCtorCall : Maybe Int
qualCtorCall = Prelude.Just 1

-- (exp_name (operator)): operator used as a first-class function value
opAsValue : List Int -> Int
opAsValue xs = foldr (+) 0 xs

-- (exp_name (qualified_operator)): qualified operator used as a value
qualOpAsValue : Int
qualOpAsValue = Prelude.(+) 1 2

-- tags.scm: definition variants -----------------------------------------

-- (signature name: (loname)) @definition.function
sigFn : Int -> Int
sigFn n = n

-- (data name: (data_name)) @definition.class — flat/simple constructor form.
-- NOTE: the individual constructors (Circle/Rectangle) below are NOT
-- tagged as separate definitions — this grammar represents the simple
-- `data T = C1 x | C2 y` RHS as a flat sequence of `exp_name` applications
-- indistinguishable from any other type-level expression (see comment in
-- idris.tags.scm). This mirrors haskell.tags.scm's documented handling of
-- `data (:+:) a b = L a | R b`.
data VShape = VCircle Double
            | VRectangle Double Double

-- (record name: (record_name)) @definition.class, plus
-- (record_body (constructor name: (caname))) @definition.function
record VPoint where
  constructor MkVPoint
  vx : Double
  vy : Double

-- (interface (interface_head name: (interface_name))) @definition.interface,
-- plus (interface_body (signature name: (loname))) @definition.method
interface VShow a where
  vshow : a -> String

-- (data_body (signature name: (caname))) @definition.function — GADT-style
-- data declaration; constructors ARE structurally distinct signature nodes
-- here, unlike the flat form above.
data VExpr : Type where
  VLit : Int -> VExpr
  VAdd : VExpr -> VExpr -> VExpr

-- cfg.scm / complexity.scm: exp_if and exp_case ---------------------------

vclassify : Int -> String
vclassify n =
  if n < 0
    then "negative"
    else "non-negative"

vdescribe : VShape -> String
vdescribe shape =
  case shape of
       VCircle r => "circle"
       VRectangle w h => "rectangle"

-- NEGATIVE: bare field access is not a call (record_access, not exp_name-call)
fieldAccess : VPoint -> Double
fieldAccess p = p.vx

-- NEGATIVE: a lambda is not a call/definition site captured by tags/calls
lambdaExample : Int -> Int
lambdaExample = \n => n + 1
