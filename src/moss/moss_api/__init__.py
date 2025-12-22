"""MossAPI and sub-APIs for code intelligence.

This package provides the main MossAPI class and specialized sub-APIs.

Structure:
- _main.py: All API classes (4148 lines, to be split incrementally)

Planned splits (not yet done):
- skeleton.py: SkeletonAPI
- tree.py: TreeAPI
- edit.py: EditAPI
- analysis.py: ComplexityAPI, ClonesAPI, SecurityAPI, WeaknessesAPI, etc.
- git.py: GitAPI, ShadowGitAPI, GitHotspotsAPI
- context.py: ContextAPI, CFGAPI, DependencyAPI
"""

# Re-export everything from _main for backwards compatibility
from moss.moss_api._main import (
    API,
    CFGAPI,
    RAGAPI,
    AnchorAPI,
    ClonesAPI,
    ComplexityAPI,
    ContextAPI,
    DependencyAPI,
    EditAPI,
    ExternalDepsAPI,
    FileMatch,
    GitAPI,
    GitHotspotsAPI,
    GrepMatch,
    GrepResult,
    GuessabilityAPI,
    HealthAPI,
    Lesson,
    LessonsAPI,
    ModuleSummary,
    MossAPI,
    PatchAPI,
    PathResolvingMixin,
    PatternsAPI,
    RefCheckAPI,
    RelatedFile,
    RelatedFilesResult,
    SecurityAPI,
    ShadowGitAPI,
    SkeletonAPI,
    SymbolExplanation,
    SymbolMatch,
    SymbolReference,
    TelemetryAPI,
    TodoAPI,
    TodoListResult,
    TodoSearchResult,
    TomlAPI,
    TreeAPI,
    ValidationAPI,
    WeaknessesAPI,
    WebAPI,
)

__all__ = [
    "API",
    "CFGAPI",
    "RAGAPI",
    "AnchorAPI",
    "ClonesAPI",
    "ComplexityAPI",
    "ContextAPI",
    "DependencyAPI",
    "EditAPI",
    "ExternalDepsAPI",
    "FileMatch",
    "GitAPI",
    "GitHotspotsAPI",
    "GrepMatch",
    "GrepResult",
    "GuessabilityAPI",
    "HealthAPI",
    "Lesson",
    "LessonsAPI",
    "ModuleSummary",
    "MossAPI",
    "PatchAPI",
    "PathResolvingMixin",
    "PatternsAPI",
    "RefCheckAPI",
    "RelatedFile",
    "RelatedFilesResult",
    "SecurityAPI",
    "ShadowGitAPI",
    "SkeletonAPI",
    "SymbolExplanation",
    "SymbolMatch",
    "SymbolReference",
    "TelemetryAPI",
    "TodoAPI",
    "TodoListResult",
    "TodoSearchResult",
    "TomlAPI",
    "TreeAPI",
    "ValidationAPI",
    "WeaknessesAPI",
    "WebAPI",
]
