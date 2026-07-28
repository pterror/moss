package main

import (
	"fmt"
	"strings"

	io "io"
)

// Stack is a generic LIFO structure.
type Stack struct {
	items []string
}

func NewStack() *Stack {
	return &Stack{}
}

func (s *Stack) Push(item string) {
	s.items = append(s.items, item)
}

func (s *Stack) Pop() (string, bool) {
	if len(s.items) == 0 {
		return "", false
	}
	last := s.items[len(s.items)-1]
	s.items = s.items[:len(s.items)-1]
	return last, true
}

func Classify(n int) string {
	if n < 0 {
		return "negative"
	} else if n == 0 {
		return "zero"
	}
	return "positive"
}

func JoinWords(words []string, sep string) string {
	result := strings.Join(words, sep)
	fmt.Println(result)
	return result
}

// Describe classifies a number via a switch statement.
func Describe(x int) string {
	switch x {
	case 0:
		return "zero"
	case 1, 2:
		return "small"
	default:
		return "big"
	}
}

// SumRange totals a slice via a for-range loop with break/continue.
func SumRange(xs []int) int {
	total := 0
	for i, x := range xs {
		if x < 0 {
			continue
		}
		if i > 100 {
			break
		}
		total += x
	}
	return total
}

// WithDefer demonstrates a deferred cleanup call.
func WithDefer() {
	defer fmt.Println("done")
	fmt.Println("start")
}

// WithGoroutine demonstrates goroutine spawn plus channel send/receive.
func WithGoroutine(ch chan int) int {
	go func() {
		ch <- 1
	}()
	return <-ch
}

// MyInt is a type alias (not a new named type) for int.
type MyInt = int

// Ordered constrains Number to comparable-ordered numeric types (idiomatic
// Go generics: a constraint interface used as a type parameter bound).
type Ordered interface {
	~int | ~float64
}

// Max is a generic function relying on type inference at call sites.
func Max[T Ordered](a, b T) T {
	if a > b {
		return a
	}
	return b
}

// Box is a generic container type.
type Box[T any] struct {
	Value T
}

// Get is a method on a generic receiver.
func (b Box[T]) Get() T {
	return b.Value
}

// Base and Derived demonstrate Go's struct-embedding idiom (composition
// over inheritance).
type Base struct {
	ID int
}

type Derived struct {
	Base
	Name string `json:"name"`
}

// Named color constants via an iota block, including a multi-name spec
// (comma-separated identifiers sharing one initializer expression list).
type Color int

const (
	Red Color = iota
	Green
	Blue
	minColor, maxColor = 0, 2
)

// Reader/ReadCloser demonstrate interface embedding.
type Reader interface {
	Read(p []byte) (n int, err error)
}

type ReadCloser interface {
	Reader
	Close() error
}

// adder returns a closure — the idiomatic higher-order-function pattern.
func adder(base int) func(int) int {
	return func(delta int) int {
		return base + delta
	}
}

// sumEvens demonstrates an iterator-pipeline-shaped loop plus a package-
// qualified call (io.Discard reference via the aliased "io" import above).
func sumEvens(nums []int) int {
	total := 0
	for _, n := range nums {
		if n%2 == 0 {
			total += n
		}
	}
	fmt.Fprintln(io.Discard, total)
	return total
}

func main() {
	s := NewStack()
	s.Push("a")
	fmt.Println(Classify(-1))
	fmt.Println(JoinWords([]string{"a", "b"}, ","))
	fmt.Println(Max(3, 4))
	b := Box[int]{Value: 5}
	fmt.Println(b.Get())
	d := Derived{Base: Base{ID: 1}, Name: "x"}
	fmt.Println(d.ID, d.Name)

	add5 := adder(5)
	fmt.Println(add5(2))

	// Immediately-invoked closures: the idiomatic goroutine/defer pattern.
	go func() {
		fmt.Println("goroutine")
	}()
	defer func() {
		fmt.Println("deferred")
	}()

	ch := make(chan int, 1)
	ch <- 1
	fmt.Println(<-ch)

	fmt.Println(sumEvens([]int{1, 2, 3, 4}))
	var m MyInt = 5
	fmt.Println(m)

	fmt.Println(Describe(1))
	fmt.Println(SumRange([]int{1, -1, 2}))
	WithDefer()
	fmt.Println(WithGoroutine(make(chan int, 1)))
}
