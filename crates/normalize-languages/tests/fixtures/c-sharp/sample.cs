using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace SampleApp
{
    public class Stack<T> : IEnumerable<T>, System.IDisposable
    {
        private List<T> items = new List<T>();

        public void Push(T item)
        {
            items.Add(item);
        }

        public T Pop()
        {
            if (items.Count == 0)
            {
                throw new InvalidOperationException("Stack is empty");
            }
            T top = items[items.Count - 1];
            items.RemoveAt(items.Count - 1);
            return top;
        }

        public T Peek()
        {
            if (items.Count == 0)
            {
                throw new InvalidOperationException("Stack is empty");
            }
            return items[items.Count - 1];
        }

        public bool IsEmpty => items.Count == 0;
        public int Count => items.Count;

        // Real-world idiom: LINQ pipeline over a generic container.
        public IEnumerable<T> EvensByIndex()
        {
            return items.Where((_, idx) => idx % 2 == 0).Select(x => x).ToList();
        }

        public IEnumerator<T> GetEnumerator() => items.GetEnumerator();

        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator() => GetEnumerator();

        public void Dispose() => items.Clear();
    }

    // Nested class delegating to a base-class constructor and a sibling
    // overload via base(...)/this(...) — explicit_constructor_invocation-
    // equivalent (constructor_initializer) idiom.
    public class BoundedStack<T> : Stack<T>
    {
        public int Capacity { get; }

        public BoundedStack() : this(16)
        {
        }

        public BoundedStack(int capacity) : base()
        {
            Capacity = capacity;
        }
    }

    // Record (C# 9+): a real-world immutable data-carrier idiom, with a
    // primary-constructor base type (record inheritance).
    public record Point(int X, int Y)
    {
        public double Distance() => Math.Sqrt(X * X + Y * Y);
    }

    public record Point3D(int X, int Y, int Z) : Point(X, Y);

    // Extension methods (static class + `this` modifier on first parameter) —
    // a near-ubiquitous modern C# idiom, especially paired with LINQ.
    public static class StringExtensions
    {
        public static bool IsBlank(this string? value) => string.IsNullOrWhiteSpace(value);
    }

    // async/await idiom.
    public static class Fetcher
    {
        public static async Task<int> FetchLengthAsync(string url)
        {
            using var client = new System.Net.Http.HttpClient();
            string result = await client.GetStringAsync(url);
            return result.Length;
        }
    }

    /// <summary>Utility math functions.</summary>
    [Obsolete("Use MathHelper instead")]
    public static class MathUtils
    {
        public static string Classify(int n)
        {
            if (n < 0)
                return "negative";
            else if (n == 0)
                return "zero";
            else
                return "positive";
        }

        public static int SumEvens(IEnumerable<int> numbers)
        {
            int total = 0;
            foreach (int n in numbers)
            {
                if (n % 2 == 0)
                    total += n;
            }
            return total;
        }
    }

    public static class SwitchDemo
    {
        public static string Describe(int n)
        {
            switch (n)
            {
                case 0:
                    return "zero";
                case 1:
                    return "one";
                default:
                    break;
            }

            string label = n switch
            {
                < 0 => "negative",
                0 => "zero",
                _ => "positive",
            };

            int i = 0;
            while (i < n)
            {
                i++;
                if (i == 3)
                {
                    continue;
                }
            }

            try
            {
                throw new InvalidOperationException("boom");
            }
            catch (InvalidOperationException ex)
            {
                label += ex.Message;
            }
            finally
            {
                label += "!";
            }

            return label;
        }
    }

    class Program
    {
        static void Main(string[] args)
        {
            var stack = new Stack<int>();
            stack.Push(1);
            stack.Push(2);
            Console.WriteLine(stack.Pop());
            Console.WriteLine(MathUtils.Classify(-5));
            Console.WriteLine(MathUtils.SumEvens(new[] { 1, 2, 3, 4, 5 }));

            // Generic method call (unqualified): Bar<int>()-shaped idiom.
            var identityResult = Identity<int>(42);

            // Qualified generic call chain (LINQ): Enumerable.Range(...).Where(...).ToList().
            var evens = Enumerable.Range(0, 10).Where(x => x % 2 == 0).ToList();

            // Null-conditional invocation chain.
            string? maybeNull = null;
            int? maybeLength = maybeNull?.Trim()?.Length;

            var bounded = new BoundedStack<string>(4);
            var point = new Point3D(1, 2, 3);
            Console.WriteLine(point.Distance());

            string? blank = "  ";
            Console.WriteLine(blank.IsBlank());
        }

        static T Identity<T>(T value) => value;
    }
}
