"""Backward compatibility shim.

All synthesis plugin exports have moved to moss.synthesis.
Import from there instead:

    from moss.synthesis import (
        CodeGenerator,
        GeneratorMetadata,
        get_synthesis_registry,
        ...
    )
"""

from moss.synthesis import (
    Abstraction,
    CodeGenerator,
    GenerationCost,
    GenerationHints,
    GenerationResult,
    GeneratorMetadata,
    GeneratorRegistry,
    GeneratorType,
    LibraryMetadata,
    LibraryPlugin,
    LibraryRegistry,
    SynthesisRegistry,
    SynthesisValidator,
    ValidationResult,
    ValidatorMetadata,
    ValidatorRegistry,
    ValidatorType,
    get_synthesis_registry,
    reset_synthesis_registry,
)

# Re-export strategy items from strategy_registry module
from moss.synthesis.strategy_registry import (
    StrategyPlugin,
    StrategyRegistry,
    get_strategy_registry,
    reset_strategy_registry,
)

__all__ = [
    "Abstraction",
    "CodeGenerator",
    "GenerationCost",
    "GenerationHints",
    "GenerationResult",
    "GeneratorMetadata",
    "GeneratorRegistry",
    "GeneratorType",
    "LibraryMetadata",
    "LibraryPlugin",
    "LibraryRegistry",
    "StrategyPlugin",
    "StrategyRegistry",
    "SynthesisRegistry",
    "SynthesisValidator",
    "ValidationResult",
    "ValidatorMetadata",
    "ValidatorRegistry",
    "ValidatorType",
    "get_strategy_registry",
    "get_synthesis_registry",
    "reset_strategy_registry",
    "reset_synthesis_registry",
]
