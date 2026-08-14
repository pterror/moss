library(stats)
library(utils)

# Classify a number as negative, zero, or positive
classify <- function(n) {
  if (n < 0) {
    return("negative")
  } else if (n == 0) {
    return("zero")
  } else {
    return("positive")
  }
}

# Sum even numbers in a vector
sum_evens <- function(values) {
  total <- 0
  for (v in values) {
    if (v %% 2 == 0) {
      total <- total + v
    }
  }
  return(total)
}

# Count occurrences of each unique value
count_occurrences <- function(values) {
  counts <- list()
  for (v in values) {
    key <- as.character(v)
    if (is.null(counts[[key]])) {
      counts[[key]] <- 1
    } else {
      counts[[key]] <- counts[[key]] + 1
    }
  }
  return(counts)
}

# Compute factorial recursively
factorial_r <- function(n) {
  if (n <= 1) {
    return(1)
  }
  return(n * factorial_r(n - 1))
}

print(classify(-3))
print(sum_evens(1:10))
print(stats::median(1:10))
print(factorial_r(5))

# Closure: make_counter returns a function that closes over `i`.
make_counter <- function() {
  i <- 0
  function() {
    i <<- i + 1
    i
  }
}
counter <- make_counter()
counter()

# apply-family / vectorized idiom: closure passed as an argument.
squares <- sapply(1:5, function(v) v^2)

# Environment-based ("R6-lite") object: methods assigned via `$` onto a
# list/environment, the idiomatic pre-R6-package OOP pattern.
make_stack <- function() {
  self <- new.env()
  self$items <- list()
  self$push <- function(x) {
    self$items[[length(self$items) + 1]] <- x
  }
  self$pop <- function() {
    n <- length(self$items)
    self$items[[n]]
  }
  self
}
stack <- make_stack()
stack$push(42)
stack$pop()

# Native pipe (R >= 4.1) and magrittr-style infix pipe.
piped_sum <- 1:10 |> sum()
piped_mean <- c(1, 2, 3) %>% mean()

# Right-assignment.
(function(x) x * x) -> square
