import Foundation
import Swift

struct Point {
    let x: Double
    let y: Double

    func distanceTo(_ other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return (dx * dx + dy * dy).squareRoot()
    }
}

class Stack<T> {
    private var items: [T] = []

    func push(_ item: T) {
        items.append(item)
    }

    func pop() -> T? {
        if items.isEmpty {
            return nil
        }
        return items.removeLast()
    }

    func peek() -> T? {
        return items.last
    }

    var isEmpty: Bool {
        return items.isEmpty
    }

    var count: Int {
        return items.count
    }
}

/// Classify a number as negative, zero, or positive.
@discardableResult
func classify(_ n: Int) -> String {
    if n < 0 {
        return "negative"
    } else if n == 0 {
        return "zero"
    } else {
        return "positive"
    }
}

func sumEvens(_ numbers: [Int]) -> Int {
    var total = 0
    for n in numbers {
        if n % 2 == 0 {
            total += n
        }
    }
    return total
}

let stack = Stack<Int>()
stack.push(10)
stack.push(20)
print(stack.pop() ?? 0)
print(classify(-5))
print(sumEvens([1, 2, 3, 4, 5]))

// --- Protocols / protocol extensions -----------------------------------

protocol Greetable {
    var name: String { get set }
    func greet() -> String
    associatedtype Payload
}

extension Greetable {
    func greet() -> String {
        return "Hello, \(name)!"
    }
}

// --- Generics with constraints ------------------------------------------

func largest<T: Comparable>(_ items: [T]) -> T? {
    guard var best = items.first else {
        return nil
    }
    for item in items {
        if item > best {
            best = item
        }
    }
    return best
}

// --- Enums with associated values ----------------------------------------

enum NetworkResult {
    case success(String)
    case failure(Error)
    case pending, cancelled
}

func describe(_ result: NetworkResult) -> String {
    switch result {
    case .success(let body):
        return "ok: \(body)"
    case .failure(let error):
        return "error: \(error)"
    case .pending, .cancelled:
        return "no result"
    }
}

// --- Extensions (protocol conformance + computed properties) -------------

class Coordinate {
    var x: Int
    var y: Int

    init(x: Int, y: Int) {
        self.x = x
        self.y = y
    }
}

extension Coordinate {
    var magnitude: Double {
        return Double(x * x + y * y).squareRoot()
    }

    static func == (lhs: Coordinate, rhs: Coordinate) -> Bool {
        return lhs.x == rhs.x && lhs.y == rhs.y
    }
}

extension Array where Element == Int {
    var doubled: [Int] {
        return self.map { $0 * 2 }
    }
}

// --- Closures / trailing closures / optional chaining --------------------

final class Downloader {
    var onComplete: (() -> Void)?

    func run(completion: () -> Void) {
        completion()
    }

    func finish() {
        onComplete?()
        onComplete!()
    }
}

func runWithTrailingClosure() {
    let numbers = [1, 2, 3, 4, 5]
    let doubled = numbers.map { $0 * 2 }
    let sum = doubled.reduce(0) { partial, next in partial + next }
    print(doubled, sum)

    let downloader = Downloader()
    downloader.run {
        print("done")
    }
}
