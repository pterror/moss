import gleam/io
import gleam/list
import gleam/int
import gleam/option.{type Option, Some as MySome}

// Type definition: a custom type
pub type Shape {
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
}

// Type alias
pub type Name = String

// Constant
pub const max_size = 100

/// Classify a number as negative, zero, or positive.
pub fn classify(n: Int) -> String {
  case n {
    _ if n < 0 -> "negative"
    0 -> "zero"
    _ -> "positive"
  }
}

// Sum even numbers in a list
pub fn sum_evens(values: List(Int)) -> Int {
  values
  |> list.filter(fn(x) { int.remainder(x, 2) == Ok(0) })
  |> list.fold(0, fn(acc, x) { acc + x })
}

// Point-free pipe target: `double` is invoked via the pipe operator with no
// parens/call syntax — idiomatic Gleam pipeline style.
pub fn double_all(values: List(Int)) -> List(Int) {
  values
  |> list.map(double)
}

fn double(x: Int) -> Int {
  x * 2
}

// Deprecation attribute on a public function.
@deprecated("use double instead")
pub fn old_double(x: Int) -> Int {
  x * 2
}

// Legacy FFI binding via `external fn`.
pub external fn native_abs(x: Int) -> Int =
  "erlang" "abs"

// Optional import usage exercising the aliased unqualified import above.
pub fn first_or_none(values: List(Int)) -> Option {
  case values {
    [x, ..] -> MySome(x)
    [] -> option.None
  }
}

// Compute factorial
pub fn factorial(n: Int) -> Int {
  case n {
    0 -> 1
    1 -> 1
    _ -> n * factorial(n - 1)
  }
}

// Greet a person
pub fn greet(name: String) -> String {
  "Hello, " <> name <> "!"
}

// Unreachable branch — exits via panic
pub fn unreachable(n: Int) -> String {
  case n {
    0 -> "zero"
    _ -> panic as "unexpected value"
  }
}

// Not yet implemented — exits via todo
pub fn not_done(n: Int) -> Int {
  todo
}

pub fn main() {
  io.println(classify(-3))
  io.println(greet("World"))
}
