"""Backward compatibility shim.

Strategy plugin exports have moved to moss.synthesis.strategy_registry.
"""

from moss.synthesis.strategy_registry import (
    StrategyPlugin,
    StrategyRegistry,
    get_strategy_registry,
    reset_strategy_registry,
)

__all__ = [
    "StrategyPlugin",
    "StrategyRegistry",
    "get_strategy_registry",
    "reset_strategy_registry",
]
