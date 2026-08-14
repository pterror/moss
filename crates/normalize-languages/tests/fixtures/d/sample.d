import std.stdio;
import std.math : sqrt, pow;
import std.algorithm : filter, reduce;
import io = std.stdio;

interface Drawable {
    void draw();
}

class Stack(T) : Drawable {
    private T[] items;

    void push(T item) {
        items ~= item;
    }

    T pop() {
        auto last = items[$ - 1];
        items = items[0 .. $ - 1];
        return last;
    }

    override void draw() {
        writeln(items);
    }
}

struct Pair(K, V) {
    K key;
    V value;
}

union Variant(T) {
    T value;
    ubyte[T.sizeof] bytes;
}

auto square(int x) {
    return x * x;
}

struct Point {
    double x;
    double y;
}

class Shape {
    string name;

    this(string name) {
        this.name = name;
    }

    double area() {
        return 0.0;
    }
}

class Circle : Shape {
    double radius;

    this(double r) {
        super("circle");
        this.radius = r;
    }

    override double area() {
        return 3.14159 * radius * radius;
    }
}

double distance(Point a, Point b) {
    double dx = b.x - a.x;
    double dy = b.y - a.y;
    return sqrt(dx * dx + dy * dy);
}

string classify(int n) {
    if (n < 0) {
        return "negative";
    } else if (n == 0) {
        return "zero";
    } else {
        return "positive";
    }
}

int sumEvens(int[] values) {
    int total = 0;
    foreach (v; values) {
        if (v % 2 == 0) {
            total += v;
        }
    }
    return total;
}

enum Direction { North, South, East, West }

int describeDirection(Direction d) {
    final switch (d) {
        case Direction.North: return 0;
        case Direction.South: return 1;
        case Direction.East: return 2;
        case Direction.West: return 3;
    }
}

int describe(int n) {
    switch (n) {
        case 0:
            return 0;
        case 1:
            return 1;
        default:
            break;
    }

    for (int i = 0; i < n; i++) {
        if (i == 2) {
            continue;
        }
    }

    int j = 0;
    while (j < n) {
        j++;
    }

    do {
        j--;
    } while (j > 0);

    try {
        throw new Exception("boom");
    } catch (Exception e) {
        j = -1;
    } finally {
        j += 1;
    }

    return j;
}

void main() {
    auto p1 = Point(3.0, 4.0);
    auto p2 = Point(0.0, 0.0);
    writeln(distance(p1, p2));

    auto c = new Circle(5.0);
    writeln(c.area());
    writeln(classify(-3));
    writeln(sumEvens([1, 2, 3, 4, 5, 6]));
    writeln(square(4));
    writeln(describeDirection(Direction.East));

    auto stack = new Stack!int();
    stack.push(1);
    stack.push(2);
    stack.draw();

    Pair!(string, int) pair;
    pair.key = "answer";
    pair.value = 42;
}
