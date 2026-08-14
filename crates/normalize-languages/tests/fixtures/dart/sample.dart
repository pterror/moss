import 'dart:collection';
import 'dart:math' as math;

class Point {
  final double x;
  final double y;

  const Point(this.x, this.y);

  double distanceTo(Point other) {
    final dx = x - other.x;
    final dy = y - other.y;
    return math.sqrt(dx * dx + dy * dy);
  }
}

class Stack<T> {
  final Queue<T> _items = Queue<T>();

  void push(T item) {
    _items.addLast(item);
  }

  T? pop() {
    if (_items.isEmpty) return null;
    return _items.removeLast();
  }

  T? peek() {
    if (_items.isEmpty) return null;
    return _items.last;
  }

  bool get isEmpty => _items.isEmpty;
  int get length => _items.length;
}

class Rectangle {
  final double width;
  final double height;

  Rectangle(this.width, this.height);

  Rectangle.square(double side) : width = side, height = side;

  factory Rectangle.fromPoints(Point a, Point b) {
    return Rectangle((b.x - a.x).abs(), (b.y - a.y).abs());
  }

  double get area => width * height;

  Rectangle operator +(Rectangle other) =>
      Rectangle(width + other.width, height + other.height);
}

mixin Loggable {
  void log(String message) {
    print(message);
  }
}

extension StringExtras on String {
  String shout() => toUpperCase();
}

String describeSize(double area) {
  return switch (area) {
    0 => 'empty',
    < 10 => 'small',
    _ => 'large',
  };
}

int firstOrDefault(List<int>? values) {
  return values?.first ?? -1;
}

/// Classify a number as negative, zero, or positive.
@pragma('vm:prefer-inline')
String classify(int n) {
  if (n < 0) {
    return 'negative';
  } else if (n == 0) {
    return 'zero';
  } else {
    return 'positive';
  }
}

int sumEvens(List<int> numbers) {
  var total = 0;
  for (final n in numbers) {
    if (n % 2 == 0) total += n;
  }
  return total;
}

String describe(int n) {
  switch (n) {
    case 0:
      return 'zero';
    case 1:
      return 'one';
    default:
      break;
  }

  for (var i = 0; i < n; i++) {
    if (i == 2) {
      continue;
    }
  }

  var j = 0;
  while (j < n) {
    j++;
  }

  do {
    j--;
  } while (j > 0);

  try {
    throw Exception('boom');
  } catch (e) {
    j = -1;
  } finally {
    j += 1;
  }

  return j.toString();
}

void main() {
  final stack = Stack<int>();
  stack.push(10);
  stack.push(20);
  print(stack.pop());
  print(classify(-3));
  print(sumEvens([1, 2, 3, 4, 5]));
  final p1 = Point(0.0, 0.0);
  final p2 = Point(3.0, 4.0);
  print(p1.distanceTo(p2));

  final rect = Rectangle.fromPoints(p1, p2);
  print(rect.area);
  print(describeSize(rect.area));
  print(firstOrDefault([1, 2, 3]));
  print('shout'.shout());
  print([3, 1, 2].map((x) => x * 2).toList());
}
