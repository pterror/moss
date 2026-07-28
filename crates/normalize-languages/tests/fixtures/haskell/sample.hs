{-# LANGUAGE ScopedTypeVariables #-}
module Main where

import Data.List (sort, nub)
import Data.Map (Map)
import qualified Data.Map as Map
import Prelude hiding (lookup)

-- | A simple data type for a tree
data Tree a = Leaf | Node a (Tree a) (Tree a)

-- | Newtype wrapper for a count
newtype Count = Count Int

-- | Type synonym
type Name = String

-- | A record type with named fields
data Rectangle = Rectangle
  { width :: Double
  , height :: Double
  }

-- | A typeclass (interface) for shapes
class Shape a where
  area :: a -> Double
  perimeter :: a -> Double

-- | Instance for the record type above
instance Shape Rectangle where
  area r = width r * height r
  perimeter r = 2 * (width r + height r)

-- | A second instance, to exercise multiple instances of one class
instance Shape Count where
  area (Count n) = fromIntegral n
  perimeter (Count n) = fromIntegral n * 4

-- | Custom infix operator, defined via parenthesized prefix syntax —
-- combining two rectangles into their bounding box.
(<+>) :: Rectangle -> Rectangle -> Rectangle
(<+>) a b = Rectangle (max (width a) (width b)) (max (height a) (height b))

-- | Insert a value into a BST
insert :: Ord a => a -> Tree a -> Tree a
insert x Leaf = Node x Leaf Leaf
insert x (Node y left right)
    | x < y    = Node y (insert x left) right
    | x > y    = Node y left (insert x right)
    | otherwise = Node y left right

-- | Check membership in a BST
member :: Ord a => a -> Tree a -> Bool
member _ Leaf = False
member x (Node y left right)
    | x == y = True
    | x < y  = member x left
    | otherwise = member x right

-- | Classify a number
classify :: Int -> String
classify n =
    if n < 0
        then "negative"
        else if n == 0
            then "zero"
            else "positive"

-- | Count unique elements in a list
countUnique :: Ord a => [a] -> Int
countUnique xs = length (nub (sort xs))

-- | Describe a number via a case expression with a guarded alternative
describe :: Int -> String
describe n = case n of
    0 -> "zero"
    x | x > 0 -> "positive"
      | otherwise -> "negative"

-- | Build frequency map (point-free / higher-order style — foldr partially
-- applied, with no explicit arguments named on the left-hand side)
frequencyMap :: Ord a => [a] -> Map a Int
frequencyMap = foldr (\x m -> Map.insertWith (+) x 1 m) Map.empty

-- | A helper that uses a where-clause: `bmiTier` is local to `describeBmi`
-- and must NOT be extracted as a top-level symbol.
describeBmi :: Double -> Double -> String
describeBmi weight height =
    bmiTier
  where
    bmi = weight / (height * height)
    bmiTier
      | bmi <= 18.5 = "underweight"
      | bmi <= 25.0 = "normal"
      | otherwise   = "overweight"

main :: IO ()
main = do
    let t = insert 3 (insert 1 (insert 2 Leaf))
    print (member 2 t)
    print (classify (-5))
    print (countUnique [1, 2, 1, 3, 2])
