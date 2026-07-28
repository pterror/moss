// Completeness matrix for scala.{tags,calls,imports,complexity,types}.scm.
// Each construct below is commented with the field/variant it exercises, per
// docs/query-testing-methodology.md step 5. A dedicated NEGATIVE section at
// the bottom holds near-miss constructs that must NOT match.

// --- tags.scm: function_definition.name variants ---------------------------

// name: identifier (plain)
def plainFunc(): Int = 1

// name: operator_identifier (symbolic operator method)
class OpHolder(x: Int) {
  def +(other: OpHolder): Int = x
}

// --- tags.scm: definition.enum ---------------------------------------------

enum Color {
  case Red, Green, Blue
}

// --- tags.scm: @reference.call variants -------------------------------------

object CallVariants {
  def identity(x: Int): Int = x

  def run(): Unit = {
    // function: identifier (plain call)
    identity(1)

    // function: field_expression, field: identifier (method call)
    List(1, 2).map(identity)

    // function: field_expression, field: operator_identifier (explicit
    // operator-method call syntax)
    val a = OpHolder(1)
    val b = OpHolder(2)
    a.+(b)

    // function: generic_function wrapping identifier (turbofish-style call)
    identityGeneric[Int](1)

    // function: generic_function wrapping field_expression
    CallVariants.identityGeneric[Int](1)

    // function: parenthesized_expression (calling a parenthesized target)
    val f = (x: Int) => x + 1
    (f)(1)
  }

  def identityGeneric[T](x: T): T = x
}

// --- tags.scm: @reference.class (instance_expression) variants -------------

object NewVariants {
  def run(): Unit = {
    // plain type_identifier
    val a = new OpHolder(1)
    // generic_type(type_identifier)
    val b = new scala.collection.mutable.ArrayBuffer[Int]()
    // stable_type_identifier(type_identifier) — qualified, no generics
    val c = new java.util.Date()
    // generic_type(stable_type_identifier(type_identifier)) — qualified + generic
    val d = new java.util.HashMap[String, Int]()
  }
}

// --- tags.scm: @reference.implementation (extends_clause) variants ---------

trait TraitA
trait TraitB
trait TraitC[T]

// extends_clause.type field: only the *first* type after `extends` carries
// the `type` field in practice; subsequent `with` mixins are unfielded
// direct children (both must be captured — see scala.tags.scm's comment).
class MultiMixin extends TraitA with TraitB

// generic mixin
class GenericMixin extends TraitC[Int]

// qualified + generic mixin
class QualifiedMixin extends scala.collection.Iterable[Int]

// --- calls.scm / complexity.scm: control-flow and nesting ------------------

def matchWithGuard(x: Int): String = x match {
  case n if n > 0 => "pos"
  case n if n < 0 => "neg"
  case _ => "zero"
}

def forComprehension(xs: List[Int]): List[Int] = for {
  x <- xs
  if x > 0
} yield x * 2

// enum_definition nesting — methods inside an enum body.
enum Nested {
  case A, B
  def label: String = this match {
    case A => "a"
    case B => "b"
  }
}

// --- imports.scm / extract_imports: rename and wildcard variants -----------

import scala.collection.mutable.{Map => MutableMap}
import scala.util.{Try, Success => S, Failure}
import java.util.{List as JList, Map as JMap}
import scala.language.implicitConversions
import foo.bar.baz.*
import scala.collection.mutable.{_}

// --- NEGATIVE: constructs that must NOT match -------------------------------

object Negatives {
  // Bare field access/write is not a call.
  var counter: Int = 0
  def touchField(): Unit = {
    counter
    counter = 1
  }

  // A lambda binding is not a function/method definition.
  val lambdaBinding: Int => Int = x => x + 1

  // A method reference-like value (eta-expansion) is not itself a call.
  def rawIdentity(x: Int): Int = x
  val etaExpanded: Int => Int = rawIdentity
}
