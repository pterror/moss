#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_CAPACITY 1024
#define CLAMP(v, lo, hi) ((v) < (lo) ? (lo) : ((v) > (hi) ? (hi) : (v)))

#ifdef DEBUG_STACK
#define STACK_LOG(msg) printf("stack: %s\n", (msg))
#else
#define STACK_LOG(msg)
#endif

typedef struct {
    int *data;
    int top;
    int capacity;
} Stack;

/* Comparator callback type used by generic sort helpers — the ubiquitous
 * C idiom for parameterizing behavior without closures (qsort, bsearch). */
typedef int (*Comparator)(const void *, const void *);

union Cell {
    int as_int;
    float as_float;
    void *as_ptr;
}; // real-world idiom: tagged-union-style value cell

/* Creates a new stack with the given capacity. */
Stack *stack_new(int capacity) {
    capacity = CLAMP(capacity, 1, MAX_CAPACITY);
    STACK_LOG("allocating stack");
    Stack *s = malloc(sizeof(Stack));
    s->data = malloc(sizeof(int) * capacity);
    s->top = -1;
    s->capacity = capacity;
    return s;
}

int compare_ints(const void *a, const void *b) {
    int ia = *(const int *)a;
    int ib = *(const int *)b;
    return ia - ib;
}

/* Real-world callback idiom: a Comparator-typed function pointer, sorted
 * with qsort and invoked directly through the variable. */
void sort_with(int *arr, int len, Comparator cmp) {
    qsort(arr, (size_t)len, sizeof(int), cmp);
    if (cmp(&arr[0], &arr[1]) > 0) {
        STACK_LOG("first two out of order after sort");
    }
}

int stack_push(Stack *s, int value) {
    if (s->top >= s->capacity - 1) {
        return 0;
    }
    s->data[++(s->top)] = value;
    return 1;
}

int stack_pop(Stack *s, int *out) {
    if (s->top < 0) {
        return 0;
    }
    *out = s->data[(s->top)--];
    return 1;
}

void stack_free(Stack *s) {
    free(s->data);
    free(s);
}

const char *classify(int n) {
    if (n < 0) {
        return "negative";
    } else if (n == 0) {
        return "zero";
    } else {
        return "positive";
    }
}

int sum_evens(int *arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        if (arr[i] % 2 == 0) {
            total += arr[i];
        }
    }
    return total;
}

int main(void) {
    Stack *s = stack_new(10);
    stack_push(s, 42);
    int val = 0;
    stack_pop(s, &val);
    printf("popped: %d\n", val);
    printf("classify(-5): %s\n", classify(-5));
    int nums[] = {1, 2, 3, 4, 5};
    printf("sum_evens: %d\n", sum_evens(nums, 5));
    sort_with(nums, 5, compare_ints);
    union Cell cell;
    cell.as_int = 42;
    stack_free(s);
    return 0;
}
