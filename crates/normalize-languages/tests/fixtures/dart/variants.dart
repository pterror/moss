// Completeness matrix for Dart's tags/calls/imports/complexity/cfg/refactor
// queries. One small, commented construct per node-type variant found via
// node-types.json (arborium-dart 2.17.0), plus a NEGATIVE section of
// near-miss constructs that must NOT produce the capture they superficially
// resemble. See crates/normalize-languages/src/queries/dart.*.scm.

library variants_library;

import 'dart:collection';
import 'dart:math' as math;
import 'dart:collection' show Queue, ListBase hide LinkedList; // combinators

part 'variants_part.dart'; // part_directive (@import.path)

// --- tags: constructor completeness (method_signature's children are not
// just function_signature/getter_signature/setter_signature) ------------

class Widget {
  int id;

  Widget(this.id); // constructor_signature, unnamed

  Widget.named(this.id); // constructor_signature, named

  factory Widget.fromId(int id) { // factory_constructor_signature, named
    return Widget(id);
  }

  factory Widget.zero() => Widget(0); // factory_constructor_signature, named (arrow body)

  const Widget.constUnnamed(int x) : id = x; // constant_constructor_signature, named

  int operator +(Widget other) => id + other.id; // operator_signature (binary_operator)
  int operator -() => -id; // operator_signature (unary, still binary_operator node)
  bool operator ==(Object other) => other is Widget && other.id == id; // operator_signature
  int operator [](int i) => id + i; // operator_signature ([] index get)
  void operator []=(int i, int v) { id = v; } // operator_signature ([]= index set)
}

class RedirectingWidget implements Widget {
  @override
  int id;
  RedirectingWidget(this.id);
  factory RedirectingWidget.make(int id) = RedirectingWidget; // redirecting_factory_constructor_signature
}

// --- tags: class/enum/mixin/extension/function/getter/setter -------------

enum Direction { north, south, east, west } // enum_declaration

mixin Flying { // mixin_declaration
  void fly() {}
}

extension IntExtras on int { // extension_declaration
  int doubled() => this * 2;
}

int addOne(int x) => x + 1; // function_signature (top-level)

class Counter {
  int _value = 0;
  int get value => _value; // getter_signature
  set value(int v) { _value = v; } // setter_signature
}

// --- calls: bare / method / null-aware / chained / generic ---------------

void callSites() {
  addOne(1); // bare identifier call
  math.sqrt(4); // qualified/member call
  final list = <int>[1, 2, 3];
  list.map((x) => x).toList(); // chained method calls
  list.sort<int>(); // generic method call (type_arguments in argument_part)
  Widget.fromId(2); // named-constructor-style call (factory)
  int? maybe = 1;
  maybe?.toString(); // null-aware method call
}

// --- complexity / cfg: switch_expression is distinct from switch_statement --

String sizeOf(int n) {
  return switch (n) { // switch_expression (not switch_statement)
    0 => 'zero',
    1 => 'one',
    _ => 'many',
  };
}

int sizeOfStatement(int n) {
  switch (n) { // switch_statement, for contrast
    case 0:
      return 0;
    default:
      return 1;
  }
}

int coalesce(int? x) {
  return x ?? 0; // if_null_expression — short-circuit branch, same category as && / ||
}

// --- NEGATIVE: constructs that must NOT match -----------------------------

void negatives() {
  // A plain field/property access with no call selector must not produce
  // a @call capture — only identifier-immediately-followed-by-a-call-
  // selector should match.
  final w = Widget(1);
  final x = w.id; // property read, not a call
  print(x);
}
