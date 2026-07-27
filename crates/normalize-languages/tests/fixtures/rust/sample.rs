use std::collections::HashMap;
use std::fmt::{self, Display};
use std::ops::Add;

#[derive(Debug, Clone)]
pub struct Counter {
    counts: HashMap<String, usize>,
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, key: &str) {
        let entry = self.counts.entry(key.to_string()).or_insert(0);
        *entry += 1;
    }

    pub fn get(&self, key: &str) -> usize {
        *self.counts.get(key).unwrap_or(&0)
    }

    /// Iterator-chain idiom: closures + turbofish collect.
    pub fn top_keys(&self, limit: usize) -> Vec<String> {
        let mut pairs: Vec<_> = self
            .counts
            .iter()
            .filter(|(_, v)| **v > 0)
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>();
        pairs.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
        pairs.into_iter().take(limit).map(|(k, _)| k).collect()
    }
}

impl Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.counts {
            writeln!(f, "{k}: {v}")?;
        }
        Ok(())
    }
}

// Path-qualified trait impl: `trait:` field is `scoped_type_identifier`, not
// the plain `type_identifier` most queries assume.
impl std::cmp::PartialEq for Counter {
    fn eq(&self, other: &Self) -> bool {
        self.counts.len() == other.counts.len()
    }
}

/// Classify a number
pub fn classify(n: i32) -> &'static str {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else {
        "positive"
    }
}

pub fn sum_evens(values: &[i32]) -> i32 {
    let mut total = 0;
    for v in values {
        if v % 2 == 0 {
            total += v;
        }
    }
    total
}

/// Generic struct + generic impl block: `impl_item.type` is `generic_type`,
/// not `type_identifier`. This is a very common idiom that a naive tags query
/// (matching only the plain form) silently drops the container for.
pub struct Point<T> {
    x: T,
    y: T,
}

impl<T: Add<Output = T> + Copy> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    pub fn sum(&self) -> T {
        self.x + self.y
    }
}

// Generic trait impl: `impl From<u32> for Point<u32>` — `trait:` field is
// `generic_type`, and its inner `type:` is a plain `type_identifier`.
impl From<u32> for Point<u32> {
    fn from(v: u32) -> Self {
        Point { x: v, y: v }
    }
}

pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
        }
    }
}

/// Nested module — checks that container nesting (mod > impl > fn) survives
/// generic and path-qualified forms too.
pub mod shapes {
    use super::Shape;

    pub fn describe(shape: &Shape) -> String {
        match shape {
            Shape::Circle(_) => "circle".to_string(),
            Shape::Rectangle(_, _) => "rectangle".to_string(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn describes_circle() {
            assert_eq!(describe(&Shape::Circle(1.0)), "circle");
        }
    }
}

/// Turbofish calls in ordinary (non-method) position.
pub fn parse_all(values: &[&str]) -> Vec<i32> {
    values
        .iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect::<Vec<i32>>()
}

/// Closures assigned to bindings — closures are values, not `function_item`,
/// and must not be picked up by function/method-definition queries.
pub fn make_adder(base: i32) -> impl Fn(i32) -> i32 {
    move |x| base + x
}

macro_rules! double {
    ($x:expr) => {
        $x * 2
    };
}

pub fn use_macro(n: i32) -> i32 {
    double!(n)
}
