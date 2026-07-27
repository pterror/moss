import groovy.transform.Immutable
import java.util.ArrayList

@Immutable
class Point {
    int x
    int y

    @Override
    double distanceTo(Point other) {
        Math.sqrt(Math.pow(other.x - x, 2) + Math.pow(other.y - y, 2))
    }
}

class MathUtils {
    static String classify(int n) {
        if (n < 0) {
            return "negative"
        } else if (n == 0) {
            return "zero"
        } else {
            return "positive"
        }
    }

    static int sumEvens(List<Integer> numbers) {
        return numbers.findAll { it % 2 == 0 }.sum() ?: 0
    }

    static int factorial(int n) {
        if (n <= 1) return 1
        return n * factorial(n - 1)
    }

    static String describe(int x) {
        switch (x) {
            case 0:
                return "zero"
            case 1:
                return "one"
            default:
                return "other"
        }
    }

    static int sumRange(List<Integer> xs) {
        int total = 0
        for (x in xs) {
            if (x < 0) {
                continue
            }
            if (x > 100) {
                break
            }
            total += x
        }
        return total
    }

    static int countDown(int n) {
        int i = n
        while (i > 0) {
            i--
        }
        return i
    }

    static int checkPositive(int n) {
        if (n < 0) {
            throw new IllegalArgumentException("negative")
        }
        return n
    }

    static void safeCall() {
        try {
            checkPositive(-1)
        } catch (IllegalArgumentException e) {
            println e
        } finally {
            println "done"
        }
    }
}

def greet(String name) {
    println "Hello, ${name}!"
}

def numbers = [1, 2, 3, 4, 5]
greet("World")
println MathUtils.classify(-5)
println MathUtils.sumEvens(numbers)
println MathUtils.factorial(5)
