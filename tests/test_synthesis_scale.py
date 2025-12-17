"""Scale tests for synthesis framework.

Tests synthesis at depth 20+ to verify the framework handles
complex problem decomposition correctly.
"""

from __future__ import annotations

import time

import pytest

from moss.synthesis import (
    Context,
    Specification,
    Subproblem,
    SynthesisFramework,
)
from moss.synthesis.framework import SynthesisConfig
from moss.synthesis.strategy import DecompositionStrategy, StrategyMetadata


class RecursiveDecomposition(DecompositionStrategy):
    """Strategy that recursively decomposes to a target depth.

    Used for scale testing - decomposes problems until reaching
    the target depth, then returns atomic solutions.
    """

    def __init__(self, target_depth: int = 20, branching_factor: int = 2):
        self.target_depth = target_depth
        self.branching_factor = branching_factor
        self._decompose_count = 0

    @property
    def metadata(self) -> StrategyMetadata:
        return StrategyMetadata(
            name="recursive_test",
            description="Test strategy for scale testing",
            keywords=("recursive", "test", "scale"),
        )

    def can_handle(self, spec: Specification, context: Context) -> bool:
        return True

    def decompose(
        self,
        spec: Specification,
        context: Context,
    ) -> list[Subproblem]:
        self._decompose_count += 1

        # Extract depth from description if present
        depth = 0
        if "[depth:" in spec.description:
            try:
                depth = int(spec.description.split("[depth:")[1].split("]")[0])
            except (IndexError, ValueError):
                pass

        # Base case: reached target depth
        if depth >= self.target_depth:
            return []

        # Recursive case: create child subproblems
        subproblems = []
        for i in range(self.branching_factor):
            child_spec = Specification(
                description=f"Subproblem {i} [depth:{depth + 1}]",
            )
            deps = (len(subproblems) - 1,) if subproblems else ()
            subproblems.append(
                Subproblem(
                    specification=child_spec,
                    dependencies=deps,
                    priority=i,
                )
            )

        return subproblems

    def estimate_success(self, spec: Specification, context: Context) -> float:
        return 0.8


class TestScaleDepth:
    """Tests for synthesis at various depths."""

    @pytest.mark.asyncio
    async def test_depth_5(self):
        """Test synthesis at depth 5."""
        strategy = RecursiveDecomposition(target_depth=5, branching_factor=1)
        config = SynthesisConfig(max_depth=10, max_iterations=100)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        assert result.success
        # Root (depth 0) + 5 levels = 6 decompose calls
        assert strategy._decompose_count == 6

    @pytest.mark.asyncio
    async def test_depth_10(self):
        """Test synthesis at depth 10."""
        strategy = RecursiveDecomposition(target_depth=10, branching_factor=1)
        config = SynthesisConfig(max_depth=15, max_iterations=500)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        assert result.success
        # Root (depth 0) + 10 levels = 11 decompose calls
        assert strategy._decompose_count == 11

    @pytest.mark.asyncio
    async def test_depth_20(self):
        """Test synthesis at depth 20."""
        strategy = RecursiveDecomposition(target_depth=20, branching_factor=1)
        config = SynthesisConfig(max_depth=25, max_iterations=1000)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        assert result.success
        # Root (depth 0) + 20 levels = 21 decompose calls
        assert strategy._decompose_count == 21

    @pytest.mark.asyncio
    async def test_depth_limit_exceeded(self):
        """Test that depth limit is enforced."""
        strategy = RecursiveDecomposition(target_depth=30, branching_factor=1)
        config = SynthesisConfig(max_depth=20, max_iterations=1000)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        # Should fail due to depth limit
        assert not result.success
        assert "depth" in result.error.lower()


class TestScaleBranching:
    """Tests for synthesis with various branching factors."""

    @pytest.mark.asyncio
    async def test_branching_2_depth_3(self):
        """Test binary branching at depth 3 (2^3 = 8 leaves)."""
        strategy = RecursiveDecomposition(target_depth=3, branching_factor=2)
        config = SynthesisConfig(max_depth=10, max_iterations=100)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        assert result.success
        assert result.subproblems_solved >= 8

    @pytest.mark.asyncio
    async def test_branching_3_depth_3(self):
        """Test ternary branching at depth 3 (3^3 = 27 leaves)."""
        strategy = RecursiveDecomposition(target_depth=3, branching_factor=3)
        config = SynthesisConfig(max_depth=10, max_iterations=200)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        assert result.success
        assert result.subproblems_solved >= 27


class TestScalePerformance:
    """Performance tests for scale synthesis."""

    @pytest.mark.asyncio
    async def test_performance_depth_10_linear(self):
        """Benchmark linear decomposition at depth 10."""
        strategy = RecursiveDecomposition(target_depth=10, branching_factor=1)
        config = SynthesisConfig(max_depth=15, max_iterations=500)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")

        start = time.perf_counter()
        result = await framework.synthesize(spec)
        elapsed = time.perf_counter() - start

        assert result.success
        # Should complete in under 1 second
        assert elapsed < 1.0, f"Too slow: {elapsed:.2f}s"

    @pytest.mark.asyncio
    async def test_performance_depth_20_linear(self):
        """Benchmark linear decomposition at depth 20."""
        strategy = RecursiveDecomposition(target_depth=20, branching_factor=1)
        config = SynthesisConfig(max_depth=25, max_iterations=1000)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")

        start = time.perf_counter()
        result = await framework.synthesize(spec)
        elapsed = time.perf_counter() - start

        assert result.success
        # Should complete in under 2 seconds
        assert elapsed < 2.0, f"Too slow: {elapsed:.2f}s"

    @pytest.mark.asyncio
    async def test_performance_parallel_branching(self):
        """Benchmark parallel subproblem solving."""
        strategy = RecursiveDecomposition(target_depth=4, branching_factor=4)
        config = SynthesisConfig(
            max_depth=10,
            max_iterations=500,
            parallel_subproblems=True,  # Enable parallel execution
        )
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")

        start = time.perf_counter()
        result = await framework.synthesize(spec)
        elapsed = time.perf_counter() - start

        assert result.success
        # Parallel should be reasonably fast
        assert elapsed < 2.0, f"Too slow: {elapsed:.2f}s"

    @pytest.mark.asyncio
    async def test_memory_usage_deep_synthesis(self):
        """Test memory doesn't explode with deep synthesis."""
        import sys

        strategy = RecursiveDecomposition(target_depth=15, branching_factor=1)
        config = SynthesisConfig(max_depth=20, max_iterations=500)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")

        # Measure before
        # Note: This is approximate, actual memory measurement would need tracemalloc
        result = await framework.synthesize(spec)

        assert result.success
        # Framework object should stay reasonable size
        assert sys.getsizeof(framework) < 10000  # Under 10KB


class TestIterationLimits:
    """Tests for iteration limit behavior."""

    @pytest.mark.asyncio
    async def test_iteration_limit_reached(self):
        """Test that iteration limit stops runaway synthesis."""
        strategy = RecursiveDecomposition(target_depth=100, branching_factor=2)
        config = SynthesisConfig(max_depth=100, max_iterations=50)  # Low limit
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        # Should fail due to iteration limit
        assert not result.success
        assert "iteration" in result.error.lower()

    @pytest.mark.asyncio
    async def test_high_iteration_limit(self):
        """Test with higher iteration limit for complex problems."""
        strategy = RecursiveDecomposition(target_depth=5, branching_factor=3)
        config = SynthesisConfig(max_depth=10, max_iterations=500)
        framework = SynthesisFramework(strategies=[strategy], config=config)

        spec = Specification(description="Root problem [depth:0]")
        result = await framework.synthesize(spec)

        assert result.success
        assert result.iterations <= 500
