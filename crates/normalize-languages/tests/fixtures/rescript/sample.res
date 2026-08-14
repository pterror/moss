open Belt
open Belt.Array
include Belt.Result

type point = {
  x: float,
  y: float,
}

type shape =
  | Circle(float)
  | Rectangle(float, float)

let square = (n: float) => n *. n

let distance = (p1: point, p2: point) => {
  let dx = p2.x -. p1.x
  let dy = p2.y -. p1.y
  Js.Math.sqrt(square(dx) +. square(dy))
}

let area = (s: shape) =>
  switch s {
  | Circle(r) => Js.Math._PI *. r *. r
  | Rectangle(w, h) => w *. h
  }

/** Classify a number as negative, zero, or positive. */
@inline
let classify = (n: int) =>
  if n < 0 {
    "negative"
  } else if n == 0 {
    "zero"
  } else {
    "positive"
  }

let sumEvens = (xs: array<int>) =>
  Array.reduce(xs, 0, (acc, x) =>
    if mod_float(float_of_int(x), 2.0) == 0.0 {
      acc + x
    } else {
      acc
    }
  )

type accumulator = {mutable total: int}

let sumViaLoop = (xs: array<int>) => {
  let acc = {total: 0}
  for i in 0 to Array.length(xs) - 1 {
    acc.total = acc.total + xs[i]->Belt.Option.getWithDefault(0)
  }
  acc.total
}

let sumViaWhile = (xs: list<int>) => {
  let remaining = ref(xs)
  let total = ref(0)
  while remaining.contents != list{} {
    switch remaining.contents {
    | list{h, ...t} =>
      total := total.contents + h
      remaining := t
    | list{} => ()
    }
  }
  total.contents
}

let safeDivide = (a: int, b: int) =>
  try {
    a / b
  } catch {
  | Division_by_zero => 0
  }

let main = () => {
  let p1 = {x: 3.0, y: 4.0}
  let p2 = {x: 0.0, y: 0.0}
  Js.log(distance(p1, p2))
  Js.log(classify(-3))
  Js.log(sumEvens([1, 2, 3, 4, 5, 6]))
  // pipe-first call chained into another module-qualified call
  Js.log(Belt.Array.length([1, 2, 3]->Belt.Array.map(x => x * 2)))
  Js.log(sumViaLoop([1, 2, 3]))
  Js.log(sumViaWhile(list{1, 2, 3}))
  Js.log(safeDivide(10, 0))
}
