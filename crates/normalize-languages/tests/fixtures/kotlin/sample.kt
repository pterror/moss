package com.example.sample

import java.util.LinkedList
import kotlin.math.abs
import kotlin.math.max as mathMax

data class Point(val x: Double, val y: Double) {
    fun distanceTo(other: Point): Double {
        val dx = x - other.x
        val dy = y - other.y
        return Math.sqrt(dx * dx + dy * dy)
    }
}

interface Shape {
    fun area(): Double
}

sealed class Figure {
    class Circle(val r: Double) : Shape {
        override fun area(): Double = Math.PI * r * r
    }

    object Empty : Figure()
}

class Queue<T> {
    private val items = LinkedList<T>()

    fun enqueue(item: T) {
        items.addLast(item)
    }

    fun dequeue(): T? {
        return if (items.isEmpty()) null else items.removeFirst()
    }

    fun peek(): T? = items.peekFirst()

    val size: Int get() = items.size

    companion object {
        fun <T> of(vararg elements: T): Queue<T> {
            val q = Queue<T>()
            for (e in elements) q.enqueue(e)
            return q
        }
    }
}

// A repository with a secondary constructor that delegates to the primary.
class Repository(val name: String, val capacity: Int) {
    constructor(name: String) : this(name, 16)

    fun describe(): String = "$name ($capacity)"
}

// Extension function on a builtin type.
fun String.shout(): String = this.uppercase() + "!"

// Suspend function (coroutines).
suspend fun fetchData(id: Int): String {
    return "data-$id"
}

// Classify a number
@JvmStatic
fun classify(n: Int): String {
    return when {
        n < 0 -> "negative"
        n == 0 -> "zero"
        else -> "positive"
    }
}

fun sumEvens(numbers: List<Int>): Int {
    var total = 0
    for (n in numbers) {
        if (n % 2 == 0) {
            total += n
        }
    }
    return total
}

fun main() {
    val q = Queue<Int>()
    q.enqueue(1)
    q.enqueue(2)
    println(q.dequeue())
    println(classify(-3))
    println(sumEvens(listOf(1, 2, 3, 4, 5)))
    val p1 = Point(0.0, 0.0)
    val p2 = Point(3.0, 4.0)
    println(p1.distanceTo(p2))
    println(abs(-5))
    println(mathMax(1, 2))

    // Trailing-lambda call and lambda-with-arrow.
    val doubled = listOf(1, 2, 3).map { it * 2 }
    val evens = doubled.filter { x -> x % 2 == 0 }
    println(doubled)
    println(evens)

    println("hello".shout())

    val repo = Repository("cache")
    println(repo.describe())

    val figures: List<Figure> = listOf(Figure.Circle(1.0), Figure.Empty)
    for (figure in figures) {
        when (figure) {
            is Figure.Circle -> println(figure.r)
            is Figure.Empty -> println("empty")
        }
    }

    try {
        println(Repository("temp").describe())
    } catch (e: Exception) {
        println(e)
    } finally {
        println("done")
    }
}
