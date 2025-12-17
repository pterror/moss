"""Moss: Headless agent orchestration layer for AI engineering."""

from moss.context import CompiledContext, ContextHost, StaticContext
from moss.dependencies import (
    DependencyInfo,
    Export,
    Import,
    PythonDependencyProvider,
    extract_dependencies,
    format_dependencies,
)
from moss.events import Event, EventBus, EventType
from moss.handles import (
    BinaryFileHandle,
    DeferredHandle,
    FileHandle,
    Handle,
    HandleMetadata,
    HandleRef,
    HandleRegistry,
    MemoryHandle,
)
from moss.shadow_git import CommitHandle, GitError, ShadowBranch, ShadowGit
from moss.skeleton import (
    PythonSkeletonProvider,
    Symbol,
    extract_python_skeleton,
    format_skeleton,
)
from moss.views import (
    Intent,
    RawViewProvider,
    View,
    ViewOptions,
    ViewProvider,
    ViewRegistry,
    ViewTarget,
    ViewType,
    create_default_registry,
)

__version__ = "0.1.0"

__all__ = [
    "BinaryFileHandle",
    "CommitHandle",
    "CompiledContext",
    "ContextHost",
    "DeferredHandle",
    "DependencyInfo",
    "Event",
    "EventBus",
    "EventType",
    "Export",
    "FileHandle",
    "GitError",
    "Handle",
    "HandleMetadata",
    "HandleRef",
    "HandleRegistry",
    "Import",
    "Intent",
    "MemoryHandle",
    "PythonDependencyProvider",
    "PythonSkeletonProvider",
    "RawViewProvider",
    "ShadowBranch",
    "ShadowGit",
    "StaticContext",
    "Symbol",
    "View",
    "ViewOptions",
    "ViewProvider",
    "ViewRegistry",
    "ViewTarget",
    "ViewType",
    "create_default_registry",
    "extract_dependencies",
    "extract_python_skeleton",
    "format_dependencies",
    "format_skeleton",
]
