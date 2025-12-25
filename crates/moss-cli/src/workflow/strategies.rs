//! Pluggable strategies for workflow execution.

use std::collections::HashMap;
use std::time::Duration;

/// Context management strategy.
pub trait ContextStrategy: Send + Sync {
    /// Add an item to the context.
    fn add(&mut self, key: &str, value: &str);

    /// Get the current context as a string (for LLM prompts).
    fn get_context(&self) -> String;

    /// Create a child context for nested execution.
    fn child(&self) -> Box<dyn ContextStrategy>;
}

/// Simple flat context - keeps last N items.
pub struct FlatContext {
    items: Vec<(String, String)>,
    max_items: usize,
}

impl FlatContext {
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }
}

impl ContextStrategy for FlatContext {
    fn add(&mut self, key: &str, value: &str) {
        self.items.push((key.to_string(), value.to_string()));
        if self.items.len() > self.max_items {
            self.items.remove(0);
        }
    }

    fn get_context(&self) -> String {
        self.items
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn child(&self) -> Box<dyn ContextStrategy> {
        Box::new(FlatContext::new(self.max_items))
    }
}

/// Cache strategy for action results.
pub trait CacheStrategy: Send + Sync {
    /// Get cached result for an action.
    fn get(&self, action: &str) -> Option<String>;

    /// Cache a result for an action.
    fn set(&mut self, action: &str, result: &str);

    /// Clear the cache.
    fn clear(&mut self);
}

/// No caching.
pub struct NoCache;

impl CacheStrategy for NoCache {
    fn get(&self, _action: &str) -> Option<String> {
        None
    }

    fn set(&mut self, _action: &str, _result: &str) {}

    fn clear(&mut self) {}
}

/// In-memory cache with optional result truncation.
pub struct InMemoryCache {
    cache: HashMap<String, String>,
    preview_length: Option<usize>,
}

impl InMemoryCache {
    pub fn new(preview_length: Option<usize>) -> Self {
        Self {
            cache: HashMap::new(),
            preview_length,
        }
    }
}

impl CacheStrategy for InMemoryCache {
    fn get(&self, action: &str) -> Option<String> {
        self.cache.get(action).cloned()
    }

    fn set(&mut self, action: &str, result: &str) {
        let value = match self.preview_length {
            Some(len) if result.len() > len => {
                format!("{}...(truncated)", &result[..len])
            }
            _ => result.to_string(),
        };
        self.cache.insert(action.to_string(), value);
    }

    fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Retry strategy for failed actions.
pub trait RetryStrategy: Send + Sync {
    /// Get the delay before the next retry attempt.
    /// Returns None if no more retries should be attempted.
    fn next_delay(&mut self, attempt: usize) -> Option<Duration>;

    /// Reset the retry state.
    fn reset(&mut self);
}

/// No retries.
pub struct NoRetry;

impl RetryStrategy for NoRetry {
    fn next_delay(&mut self, _attempt: usize) -> Option<Duration> {
        None
    }

    fn reset(&mut self) {}
}

/// Fixed delay retries.
pub struct FixedRetry {
    max_attempts: usize,
    delay: Duration,
}

impl FixedRetry {
    pub fn new(max_attempts: usize, delay_secs: f64) -> Self {
        Self {
            max_attempts,
            delay: Duration::from_secs_f64(delay_secs),
        }
    }
}

impl RetryStrategy for FixedRetry {
    fn next_delay(&mut self, attempt: usize) -> Option<Duration> {
        if attempt < self.max_attempts {
            Some(self.delay)
        } else {
            None
        }
    }

    fn reset(&mut self) {}
}

/// Exponential backoff retries.
pub struct ExponentialRetry {
    max_attempts: usize,
    base_delay: f64,
    max_delay: f64,
}

impl ExponentialRetry {
    pub fn new(max_attempts: usize, base_delay: f64, max_delay: Option<f64>) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay: max_delay.unwrap_or(60.0),
        }
    }
}

impl RetryStrategy for ExponentialRetry {
    fn next_delay(&mut self, attempt: usize) -> Option<Duration> {
        if attempt < self.max_attempts {
            let delay = self.base_delay * 2.0_f64.powi(attempt as i32);
            let delay = delay.min(self.max_delay);
            Some(Duration::from_secs_f64(delay))
        } else {
            None
        }
    }

    fn reset(&mut self) {}
}

/// Condition evaluator for state transitions.
pub fn evaluate_condition(condition: &str, _context: &str, result: &str) -> bool {
    match condition {
        "has_errors" => {
            result.to_lowercase().contains("error")
                || result.to_lowercase().contains("failed")
                || result.to_lowercase().contains("failure")
        }
        "success" => {
            !result.to_lowercase().contains("error")
                && !result.to_lowercase().contains("failed")
        }
        "empty" => result.trim().is_empty(),
        _ if condition.starts_with("contains:") => {
            let needle = condition.strip_prefix("contains:").unwrap();
            result.contains(needle)
        }
        _ => {
            // Unknown condition, default to false
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_context() {
        let mut ctx = FlatContext::new(3);
        ctx.add("a", "1");
        ctx.add("b", "2");
        ctx.add("c", "3");
        ctx.add("d", "4");

        let result = ctx.get_context();
        assert!(!result.contains("a: 1")); // Should be evicted
        assert!(result.contains("b: 2"));
        assert!(result.contains("d: 4"));
    }

    #[test]
    fn test_in_memory_cache() {
        let mut cache = InMemoryCache::new(Some(10));
        cache.set("action1", "short");
        cache.set("action2", "this is a very long result");

        assert_eq!(cache.get("action1"), Some("short".to_string()));
        assert!(cache.get("action2").unwrap().contains("truncated"));
    }

    #[test]
    fn test_evaluate_condition() {
        assert!(evaluate_condition("has_errors", "", "Error: something failed"));
        assert!(!evaluate_condition("has_errors", "", "All good"));
        assert!(evaluate_condition("success", "", "Completed successfully"));
        assert!(evaluate_condition("contains:TODO", "", "Found TODO in code"));
    }
}
