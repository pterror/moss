import scala.collection.mutable.ArrayBuffer
import scala.math.abs
import scala.util.{Try, Success => S, Failure}

case class Point(x: Double, y: Double) {
  def distanceTo(other: Point): Double = {
    val dx = x - other.x
    val dy = y - other.y
    math.sqrt(dx * dx + dy * dy)
  }

  // Operator-method overloading (very common on case classes).
  def +(other: Point): Point = Point(x + other.x, y + other.y)
}

object Point {
  // Companion object: extra constructors/factories for the case class above.
  val origin: Point = Point(0.0, 0.0)
  def fromTuple(t: (Double, Double)): Point = Point(t._1, t._2)
}

class Stack[T] {
  private val items = ArrayBuffer.empty[T]

  def push(item: T): Unit = {
    items.append(item)
  }

  def pop(): Option[T] = {
    if (items.isEmpty) None
    else {
      val top = items.last
      items.remove(items.length - 1)
      Some(top)
    }
  }

  def peek(): Option[T] = items.lastOption

  def size: Int = items.length
}

// Classify a number
def classify(n: Int): String = {
  if (n < 0) "negative"
  else if (n == 0) "zero"
  else "positive"
}

def sumEvens(numbers: List[Int]): Int = {
  var total = 0
  for (n <- numbers) {
    if (n % 2 == 0) total += n
  }
  total
}

// Traits with mixins — the defining Scala idiom for composing behavior.
trait Named {
  def name: String
}
trait Aged {
  def age: Int
  def isAdult: Boolean = age >= 18
}
class Person(val name: String, val age: Int) extends Named with Aged

// Scala 3 enum — a headline replacement for the old sealed-trait ADT pattern.
enum Direction(val degrees: Int) {
  case North extends Direction(0)
  case East extends Direction(90)
  case South extends Direction(180)
  case West extends Direction(270)

  def opposite: Direction = this match {
    case Direction.North => Direction.South
    case Direction.South => Direction.North
    case Direction.East => Direction.West
    case Direction.West => Direction.East
  }
}

// Pattern matching with guards.
def describe(x: Any): String = x match {
  case n: Int if n > 0 => "positive int"
  case n: Int if n < 0 => "negative int"
  case 0 => "zero"
  case s: String if s.isEmpty => "empty string"
  case s: String => s"string: $s"
  case Point(px, py) if px == py => "diagonal point"
  case _ => "unknown"
}

// For-comprehension with a guard and a yield.
def pairs(xs: List[Int], ys: List[Int]): List[(Int, Int)] = for {
  x <- xs
  y <- ys
  if x != y
} yield (x, y)

// Extension method (Scala 3).
extension (p: Point) {
  def magnitude: Double = math.sqrt(p.x * p.x + p.y * p.y)
}

// Higher-kinded generic (a type constructor parameter, `F[_]`).
trait Functor[F[_]] {
  def map[A, B](fa: F[A])(f: A => B): F[B]
}

@main def run(): Unit = {
  val stack = new Stack[Int]()
  stack.push(1)
  stack.push(2)
  println(stack.pop())
  println(classify(-3))
  println(sumEvens(List(1, 2, 3, 4, 5)))
  val p1 = Point(0.0, 0.0)
  val p2 = Point(3.0, 4.0)
  println(p1.distanceTo(p2))
  println((p1 + p2).magnitude)
  println(Direction.North.opposite)
  println(describe(p1))
  println(pairs(List(1, 2), List(2, 3)))
}
