package main

import (
	"fmt"
	"strings"
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
