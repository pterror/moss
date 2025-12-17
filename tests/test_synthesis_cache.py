"""Tests for synthesis caching."""

from __future__ import annotations

import time
from datetime import datetime, timedelta

from moss.synthesis import Specification
from moss.synthesis.cache import (
    CacheEntry,
    ExecutionResultCache,
    SolutionCache,
    StrategyCache,
    SynthesisCache,
    clear_all_caches,
    get_cache_stats,
    get_solution_cache,
    get_strategy_cache,
    get_test_cache,
)


class TestCacheEntry:
    """Tests for CacheEntry."""

    def test_entry_creation(self):
        entry = CacheEntry(value="test")
        assert entry.value == "test"
        assert entry.hit_count == 0
        assert entry.ttl_seconds is None

    def test_entry_not_expired_without_ttl(self):
        entry = CacheEntry(value="test")
        assert not entry.is_expired()

    def test_entry_not_expired_within_ttl(self):
        entry = CacheEntry(value="test", ttl_seconds=3600)
        assert not entry.is_expired()

    def test_entry_expired_after_ttl(self):
        entry = CacheEntry(
            value="test",
            created_at=datetime.now() - timedelta(seconds=10),
            ttl_seconds=5,
        )
        assert entry.is_expired()

    def test_access_increments_hit_count(self):
        entry = CacheEntry(value="test")
        entry.access()
        assert entry.hit_count == 1
        entry.access()
        assert entry.hit_count == 2

    def test_access_returns_value(self):
        entry = CacheEntry(value=42)
        result = entry.access()
        assert result == 42


class TestSynthesisCache:
    """Tests for SynthesisCache base class."""

    def test_set_and_get(self):
        cache = SynthesisCache()
        cache.set("key1", "value1")
        assert cache.get("key1") == "value1"

    def test_get_nonexistent(self):
        cache = SynthesisCache()
        assert cache.get("nonexistent") is None

    def test_expired_entry_returns_none(self):
        cache = SynthesisCache(default_ttl=1)
        cache.set("key1", "value1")
        # Manually expire the entry
        cache._cache["key1"].created_at = datetime.now() - timedelta(seconds=10)
        assert cache.get("key1") is None

    def test_hit_rate(self):
        cache = SynthesisCache()
        cache.set("key1", "value1")
        cache.get("key1")  # Hit
        cache.get("key1")  # Hit
        cache.get("missing")  # Miss
        assert cache.hit_rate == 2 / 3

    def test_lru_eviction(self):
        cache = SynthesisCache(max_size=10)
        for i in range(15):
            cache.set(f"key{i}", f"value{i}")

        # Should have evicted some entries
        assert len(cache._cache) <= 10

    def test_clear(self):
        cache = SynthesisCache()
        cache.set("key1", "value1")
        cache.get("key1")
        cache.clear()
        assert cache.get("key1") is None
        assert cache._hits == 0

    def test_stats(self):
        cache = SynthesisCache(max_size=100)
        cache.set("key1", "value1")
        cache.get("key1")
        cache.get("missing")

        stats = cache.stats
        assert stats["size"] == 1
        assert stats["max_size"] == 100
        assert stats["hits"] == 1
        assert stats["misses"] == 1


class TestExecutionResultCache:
    """Tests for ExecutionResultCache."""

    def test_cache_test_result(self):
        cache = ExecutionResultCache()
        cache.cache_test_result("def test_x(): pass", "solution", True)
        result = cache.get_test_result("def test_x(): pass", "solution")
        assert result is True

    def test_cache_failed_test(self):
        cache = ExecutionResultCache()
        cache.cache_test_result("def test_fail(): 1/0", "bad", False)
        result = cache.get_test_result("def test_fail(): 1/0", "bad")
        assert result is False

    def test_different_solutions_different_keys(self):
        cache = ExecutionResultCache()
        cache.cache_test_result("test", "solution1", True)
        cache.cache_test_result("test", "solution2", False)

        assert cache.get_test_result("test", "solution1") is True
        assert cache.get_test_result("test", "solution2") is False

    def test_python_version_affects_key(self):
        cache = ExecutionResultCache()
        cache.cache_test_result("test", "solution", True, python_version="3.11")
        # Different python version should miss
        assert cache.get_test_result("test", "solution", python_version="3.12") is None


class TestSolutionCache:
    """Tests for SolutionCache."""

    def test_cache_solution(self):
        cache = SolutionCache()
        spec = Specification(description="add two numbers")
        cache.cache_solution(spec, "def add(a, b): return a + b")
        result = cache.get_solution(spec)
        assert result == "def add(a, b): return a + b"

    def test_type_signature_affects_key(self):
        cache = SolutionCache()
        spec1 = Specification(description="convert", type_signature="int -> str")
        spec2 = Specification(description="convert", type_signature="str -> int")

        cache.cache_solution(spec1, "solution1")
        cache.cache_solution(spec2, "solution2")

        assert cache.get_solution(spec1) == "solution1"
        assert cache.get_solution(spec2) == "solution2"

    def test_context_hash_affects_key(self):
        cache = SolutionCache()
        spec = Specification(description="compute")

        cache.cache_solution(spec, "solution1", context_hash="ctx1")
        cache.cache_solution(spec, "solution2", context_hash="ctx2")

        assert cache.get_solution(spec, context_hash="ctx1") == "solution1"
        assert cache.get_solution(spec, context_hash="ctx2") == "solution2"


class TestStrategyCache:
    """Tests for StrategyCache."""

    def test_cache_strategy(self):
        cache = StrategyCache()
        spec = Specification(description="build REST API")
        cache.cache_strategy(spec, "pattern_based")
        assert cache.get_strategy(spec) == "pattern_based"

    def test_constraints_affect_key(self):
        cache = StrategyCache()
        spec1 = Specification(description="sort", constraints=("stable",))
        spec2 = Specification(description="sort", constraints=("fast",))

        cache.cache_strategy(spec1, "strategy1")
        cache.cache_strategy(spec2, "strategy2")

        assert cache.get_strategy(spec1) == "strategy1"
        assert cache.get_strategy(spec2) == "strategy2"


class TestGlobalCaches:
    """Tests for global cache instances."""

    def test_get_test_cache(self):
        cache1 = get_test_cache()
        cache2 = get_test_cache()
        assert cache1 is cache2  # Same instance

    def test_get_solution_cache(self):
        cache1 = get_solution_cache()
        cache2 = get_solution_cache()
        assert cache1 is cache2

    def test_get_strategy_cache(self):
        cache1 = get_strategy_cache()
        cache2 = get_strategy_cache()
        assert cache1 is cache2

    def test_clear_all_caches(self):
        test_cache = get_test_cache()
        test_cache.set("key", "value")
        clear_all_caches()
        assert test_cache.get("key") is None

    def test_get_cache_stats(self):
        stats = get_cache_stats()
        assert "test_cache" in stats
        assert "solution_cache" in stats
        assert "strategy_cache" in stats
        assert "hit_rate" in stats["test_cache"]


# =============================================================================
# Performance Benchmarks
# =============================================================================


class TestCachePerformance:
    """Performance benchmarks for caches."""

    def test_cache_set_performance(self):
        """Benchmark cache set operations."""
        cache = SynthesisCache(max_size=100000)
        start = time.perf_counter()

        for i in range(10000):
            cache.set(f"key{i}", f"value{i}")

        elapsed = time.perf_counter() - start
        ops_per_sec = 10000 / elapsed

        # Should be able to do at least 50k sets/sec
        assert ops_per_sec > 50000, f"Too slow: {ops_per_sec:.0f} ops/sec"

    def test_cache_get_performance(self):
        """Benchmark cache get operations."""
        cache = SynthesisCache(max_size=100000)

        # Pre-populate
        for i in range(10000):
            cache.set(f"key{i}", f"value{i}")

        start = time.perf_counter()

        for i in range(10000):
            cache.get(f"key{i}")

        elapsed = time.perf_counter() - start
        ops_per_sec = 10000 / elapsed

        # Should be able to do at least 100k gets/sec
        assert ops_per_sec > 100000, f"Too slow: {ops_per_sec:.0f} ops/sec"

    def test_cache_hit_rate_under_load(self):
        """Test cache hit rate with realistic workload."""
        cache = SynthesisCache(max_size=1000)

        # Simulate workload with some repeated accesses
        for i in range(5000):
            # 70% of accesses are to 20% of keys (Pareto)
            if i % 10 < 7:
                key = f"hot{i % 100}"  # Hot keys
            else:
                key = f"cold{i}"  # Cold keys

            if cache.get(key) is None:
                cache.set(key, f"value{i}")

        # Should have reasonable hit rate
        assert cache.hit_rate > 0.3, f"Hit rate too low: {cache.hit_rate:.2%}"

    def test_lru_eviction_performance(self):
        """Test LRU eviction doesn't cause major slowdown."""
        cache = SynthesisCache(max_size=100)

        start = time.perf_counter()

        # Force many evictions
        for i in range(10000):
            cache.set(f"key{i}", f"value{i}")

        elapsed = time.perf_counter() - start
        ops_per_sec = 10000 / elapsed

        # Should still be reasonably fast
        assert ops_per_sec > 10000, f"Too slow with eviction: {ops_per_sec:.0f} ops/sec"
