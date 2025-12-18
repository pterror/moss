"""Canonical API surface for Moss.

This module provides the primary entry point for using Moss as a library.
Import MossAPI for organized access to all functionality.

Example:
    from moss import MossAPI

    # Create API instance
    api = MossAPI.for_project("/path/to/project")

    # Use various capabilities
    skeleton = api.skeleton.extract("src/main.py")
    deps = api.dependencies.analyze("src/")
    health = api.health.check()
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from moss.anchors import AnchorMatch
    from moss.cfg import ControlFlowGraph
    from moss.check_docs import DocCheckResult
    from moss.check_todos import TodoCheckResult
    from moss.context import CompiledContext, ContextHost
    from moss.dependencies import DependencyInfo
    from moss.dependency_analysis import DependencyAnalysis
    from moss.patches import Patch, PatchResult
    from moss.shadow_git import CommitHandle, ShadowGit
    from moss.skeleton import Symbol
    from moss.status import ProjectStatus
    from moss.structural_analysis import StructuralAnalysis
    from moss.summarize import ProjectSummary
    from moss.test_analysis import TestAnalysis
    from moss.validators import ValidationResult, Validator


@dataclass
class SkeletonAPI:
    """API for code skeleton extraction.

    Extracts structural summaries of code (classes, functions, signatures)
    without implementation details.
    """

    root: Path

    def extract(self, file_path: str | Path) -> list[Symbol]:
        """Extract skeleton from a Python file.

        Args:
            file_path: Path to the Python file (relative to root or absolute)

        Returns:
            List of Symbol objects representing the code structure
        """
        from moss.skeleton import extract_python_skeleton

        path = self._resolve_path(file_path)
        source = path.read_text()
        return extract_python_skeleton(source)

    def format(self, file_path: str | Path, show_bodies: bool = False) -> str:
        """Extract and format skeleton as readable text.

        Args:
            file_path: Path to the Python file
            show_bodies: Whether to include function/method bodies

        Returns:
            Formatted string representation of the skeleton
        """
        from moss.skeleton import format_skeleton

        symbols = self.extract(file_path)
        return format_skeleton(symbols, show_bodies=show_bodies)

    def _resolve_path(self, file_path: str | Path) -> Path:
        path = Path(file_path)
        if not path.is_absolute():
            path = self.root / path
        return path


@dataclass
class AnchorAPI:
    """API for finding code locations using fuzzy anchors.

    Anchors identify code elements (functions, classes, variables) by name
    and type, with fuzzy matching support.
    """

    root: Path

    def find(
        self,
        file_path: str | Path,
        name: str,
        anchor_type: str = "function",
    ) -> list[AnchorMatch]:
        """Find anchors matching a name in a file.

        Args:
            file_path: Path to search in
            name: Name to search for (supports fuzzy matching)
            anchor_type: Type filter - "function", "class", "variable", "method", "import"

        Returns:
            List of AnchorMatch objects with locations and confidence scores
        """
        from moss.anchors import Anchor, AnchorType, find_anchors

        path = self._resolve_path(file_path)
        source = path.read_text()

        type_map = {
            "function": AnchorType.FUNCTION,
            "class": AnchorType.CLASS,
            "variable": AnchorType.VARIABLE,
            "method": AnchorType.METHOD,
            "import": AnchorType.IMPORT,
        }
        anchor = Anchor(type=type_map.get(anchor_type, AnchorType.FUNCTION), name=name)
        return find_anchors(source, anchor)

    def resolve(
        self,
        file_path: str | Path,
        name: str,
        anchor_type: str = "function",
    ) -> AnchorMatch:
        """Resolve a single anchor (raises if ambiguous or not found).

        Args:
            file_path: Path to search in
            name: Name to search for
            anchor_type: Type filter

        Returns:
            Single best AnchorMatch

        Raises:
            AnchorNotFoundError: If no match found
            AmbiguousAnchorError: If multiple matches with equal confidence
        """
        from moss.anchors import Anchor, AnchorType, resolve_anchor

        path = self._resolve_path(file_path)
        source = path.read_text()

        type_map = {
            "function": AnchorType.FUNCTION,
            "class": AnchorType.CLASS,
            "variable": AnchorType.VARIABLE,
            "method": AnchorType.METHOD,
            "import": AnchorType.IMPORT,
        }
        anchor = Anchor(type=type_map.get(anchor_type, AnchorType.FUNCTION), name=name)
        return resolve_anchor(source, anchor)

    def _resolve_path(self, file_path: str | Path) -> Path:
        path = Path(file_path)
        if not path.is_absolute():
            path = self.root / path
        return path


@dataclass
class PatchAPI:
    """API for applying code patches.

    Supports AST-aware patching with automatic fallback to text-based
    patching when AST parsing fails.
    """

    root: Path

    def apply(
        self,
        file_path: str | Path,
        patch: Patch,
        write: bool = True,
    ) -> PatchResult:
        """Apply a patch to a file.

        Args:
            file_path: Path to the file to patch
            patch: Patch object describing the change
            write: Whether to write changes to disk

        Returns:
            PatchResult with success status and modified content
        """
        from moss.patches import apply_patch

        path = self._resolve_path(file_path)
        source = path.read_text()
        result = apply_patch(source, patch)

        if write and result.success:
            path.write_text(result.content)

        return result

    def apply_with_fallback(
        self,
        file_path: str | Path,
        patch: Patch,
        write: bool = True,
    ) -> PatchResult:
        """Apply a patch with automatic text fallback.

        First tries AST-aware patching, falls back to text-based
        if that fails.

        Args:
            file_path: Path to the file to patch
            patch: Patch object describing the change
            write: Whether to write changes to disk

        Returns:
            PatchResult with success status and modified content
        """
        from moss.patches import apply_patch_with_fallback

        path = self._resolve_path(file_path)
        source = path.read_text()
        result = apply_patch_with_fallback(source, patch)

        if write and result.success:
            path.write_text(result.content)

        return result

    def create(
        self,
        patch_type: str,
        anchor_name: str,
        content: str,
        **kwargs: Any,
    ) -> Patch:
        """Create a Patch object.

        Args:
            patch_type: Type of patch - "insert", "replace", "delete", "wrap"
            anchor_name: Name of the anchor to target
            content: New content for the patch
            **kwargs: Additional patch options (position, anchor_type, etc.)

        Returns:
            Patch object ready for application
        """
        from moss.anchors import AnchorType
        from moss.patches import Patch, PatchType

        type_map = {
            "insert": PatchType.INSERT,
            "replace": PatchType.REPLACE,
            "delete": PatchType.DELETE,
            "wrap": PatchType.WRAP,
        }

        anchor_type_map = {
            "function": AnchorType.FUNCTION,
            "class": AnchorType.CLASS,
            "variable": AnchorType.VARIABLE,
            "method": AnchorType.METHOD,
            "import": AnchorType.IMPORT,
        }

        return Patch(
            type=type_map[patch_type],
            anchor_name=anchor_name,
            content=content,
            anchor_type=anchor_type_map.get(
                kwargs.get("anchor_type", "function"), AnchorType.FUNCTION
            ),
            position=kwargs.get("position"),
        )

    def _resolve_path(self, file_path: str | Path) -> Path:
        path = Path(file_path)
        if not path.is_absolute():
            path = self.root / path
        return path


@dataclass
class DependencyAPI:
    """API for dependency analysis.

    Analyzes import/export relationships, detects circular dependencies,
    and provides coupling metrics.
    """

    root: Path

    def extract(self, file_path: str | Path) -> DependencyInfo:
        """Extract imports and exports from a file.

        Args:
            file_path: Path to analyze

        Returns:
            DependencyInfo with imports and exports
        """
        from moss.dependencies import extract_dependencies

        path = self._resolve_path(file_path)
        source = path.read_text()
        return extract_dependencies(source, str(path))

    def analyze(self) -> DependencyAnalysis:
        """Run full dependency analysis on the project.

        Returns:
            DependencyAnalysis with circular deps, god modules, orphans, etc.
        """
        from moss.dependency_analysis import DependencyAnalyzer

        analyzer = DependencyAnalyzer(self.root)
        return analyzer.analyze()

    def format(self, file_path: str | Path) -> str:
        """Extract and format dependencies as readable text.

        Args:
            file_path: Path to analyze

        Returns:
            Formatted string with imports and exports
        """
        from moss.dependencies import format_dependencies

        info = self.extract(file_path)
        return format_dependencies(info)

    def _resolve_path(self, file_path: str | Path) -> Path:
        path = Path(file_path)
        if not path.is_absolute():
            path = self.root / path
        return path


@dataclass
class CFGAPI:
    """API for control flow graph analysis.

    Builds control flow graphs showing execution paths through functions.
    """

    root: Path

    def build(self, file_path: str | Path) -> dict[str, ControlFlowGraph]:
        """Build CFGs for all functions in a file.

        Args:
            file_path: Path to the Python file

        Returns:
            Dict mapping function names to their ControlFlowGraph
        """
        from moss.cfg import build_cfg

        path = self._resolve_path(file_path)
        source = path.read_text()
        return build_cfg(source)

    def _resolve_path(self, file_path: str | Path) -> Path:
        path = Path(file_path)
        if not path.is_absolute():
            path = self.root / path
        return path


@dataclass
class ValidationAPI:
    """API for code validation.

    Runs validators (syntax, linting, tests) and reports issues.
    """

    root: Path

    def create_chain(self) -> Validator:
        """Create a standard Python validator chain.

        Returns:
            ValidatorChain configured for Python (syntax + ruff + pytest)
        """
        from moss.validators import create_python_validator_chain

        return create_python_validator_chain()

    def validate(self, file_path: str | Path) -> ValidationResult:
        """Validate a Python file with the default chain.

        Args:
            file_path: Path to validate

        Returns:
            ValidationResult with any issues found
        """
        chain = self.create_chain()
        path = self._resolve_path(file_path)
        source = path.read_text()
        return chain.validate(source, context={"file": str(path), "root": str(self.root)})

    def _resolve_path(self, file_path: str | Path) -> Path:
        path = Path(file_path)
        if not path.is_absolute():
            path = self.root / path
        return path


@dataclass
class GitAPI:
    """API for shadow git operations.

    Provides atomic commit/rollback operations for safe code modifications.
    """

    root: Path
    _shadow_git: ShadowGit | None = None

    def init(self) -> ShadowGit:
        """Initialize shadow git for the project.

        Returns:
            ShadowGit instance for managing branches
        """
        from moss.shadow_git import ShadowGit

        if self._shadow_git is None:
            self._shadow_git = ShadowGit(self.root)
        return self._shadow_git

    def create_branch(self, name: str | None = None) -> Any:
        """Create an isolated shadow branch for agent work.

        Args:
            name: Optional branch name (auto-generated if not provided)

        Returns:
            ShadowBranch context manager
        """
        git = self.init()
        return git.create_branch(name)

    def commit(self, message: str) -> CommitHandle:
        """Create a commit on the current shadow branch.

        Args:
            message: Commit message

        Returns:
            CommitHandle referencing the new commit
        """
        git = self.init()
        return git.commit(message)


@dataclass
class ContextAPI:
    """API for context compilation.

    Compiles code views (skeletons, CFGs, dependencies) into structured
    context for AI consumption.
    """

    root: Path
    _host: ContextHost | None = None

    def init(self) -> ContextHost:
        """Initialize the context host with default view providers.

        Returns:
            ContextHost instance
        """
        from moss.context import ContextHost
        from moss.views import create_default_registry

        if self._host is None:
            registry = create_default_registry()
            self._host = ContextHost(registry)
        return self._host

    def compile(
        self,
        file_paths: list[str | Path],
        view_types: list[str] | None = None,
    ) -> CompiledContext:
        """Compile context for the given files.

        Args:
            file_paths: Files to include in context
            view_types: View types to generate (default: skeleton, dependencies)

        Returns:
            CompiledContext with rendered views
        """
        from moss.views import Intent, ViewTarget, ViewType

        host = self.init()

        targets = []
        for path in file_paths:
            p = Path(path)
            if not p.is_absolute():
                p = self.root / p
            targets.append(ViewTarget(path=p))

        types = view_types or ["skeleton", "dependencies"]
        type_map = {
            "skeleton": ViewType.SKELETON,
            "dependencies": ViewType.DEPENDENCIES,
            "cfg": ViewType.CFG,
            "raw": ViewType.RAW,
        }
        view_type_enums = [type_map.get(t, ViewType.SKELETON) for t in types]

        intent = Intent(targets=targets, view_types=view_type_enums)
        return host.compile(intent)


@dataclass
class HealthAPI:
    """API for project health analysis.

    Provides comprehensive project health metrics and reports.
    """

    root: Path

    def check(self) -> ProjectStatus:
        """Run full health analysis on the project.

        Returns:
            ProjectStatus with health score, grade, and detailed metrics
        """
        from moss.status import StatusChecker

        checker = StatusChecker(self.root)
        return checker.check()

    def summarize(self) -> ProjectSummary:
        """Generate a project summary.

        Returns:
            ProjectSummary with module information
        """
        from moss.summarize import Summarizer

        summarizer = Summarizer(include_private=False, include_tests=False)
        return summarizer.summarize_project(self.root)

    def check_docs(self) -> DocCheckResult:
        """Check documentation health.

        Returns:
            DocCheckResult with coverage and issues
        """
        from moss.check_docs import DocChecker

        checker = DocChecker(self.root, check_links=True)
        return checker.check()

    def check_todos(self) -> TodoCheckResult:
        """Check TODO tracking health.

        Returns:
            TodoCheckResult with tracked and orphaned TODOs
        """
        from moss.check_todos import TodoChecker

        checker = TodoChecker(self.root)
        return checker.check()

    def analyze_structure(self) -> StructuralAnalysis:
        """Analyze structural code quality.

        Returns:
            StructuralAnalysis with hotspots and metrics
        """
        from moss.structural_analysis import StructuralAnalyzer

        analyzer = StructuralAnalyzer(self.root)
        return analyzer.analyze()

    def analyze_tests(self) -> TestAnalysis:
        """Analyze test coverage structure.

        Returns:
            TestAnalysis with module-test mappings
        """
        from moss.test_analysis import TestAnalyzer

        analyzer = TestAnalyzer(self.root)
        return analyzer.analyze()


@dataclass
class MossAPI:
    """Unified API for Moss functionality.

    Provides organized access to all Moss capabilities through
    domain-specific sub-APIs.

    Example:
        api = MossAPI.for_project("/path/to/project")

        # Extract code structure
        skeleton = api.skeleton.extract("src/main.py")

        # Analyze dependencies
        deps = api.dependencies.analyze()

        # Check project health
        health = api.health.check()
        print(f"Health grade: {health.health_grade}")
    """

    root: Path

    # Sub-APIs (initialized lazily)
    _skeleton: SkeletonAPI | None = None
    _anchor: AnchorAPI | None = None
    _patch: PatchAPI | None = None
    _dependencies: DependencyAPI | None = None
    _cfg: CFGAPI | None = None
    _validation: ValidationAPI | None = None
    _git: GitAPI | None = None
    _context: ContextAPI | None = None
    _health: HealthAPI | None = None

    @classmethod
    def for_project(cls, path: str | Path) -> MossAPI:
        """Create a MossAPI instance for a project directory.

        Args:
            path: Path to the project root

        Returns:
            MossAPI instance configured for the project
        """
        return cls(root=Path(path).resolve())

    @property
    def skeleton(self) -> SkeletonAPI:
        """Access skeleton extraction functionality."""
        if self._skeleton is None:
            self._skeleton = SkeletonAPI(root=self.root)
        return self._skeleton

    @property
    def anchor(self) -> AnchorAPI:
        """Access anchor finding functionality."""
        if self._anchor is None:
            self._anchor = AnchorAPI(root=self.root)
        return self._anchor

    @property
    def patch(self) -> PatchAPI:
        """Access patching functionality."""
        if self._patch is None:
            self._patch = PatchAPI(root=self.root)
        return self._patch

    @property
    def dependencies(self) -> DependencyAPI:
        """Access dependency analysis functionality."""
        if self._dependencies is None:
            self._dependencies = DependencyAPI(root=self.root)
        return self._dependencies

    @property
    def cfg(self) -> CFGAPI:
        """Access control flow graph functionality."""
        if self._cfg is None:
            self._cfg = CFGAPI(root=self.root)
        return self._cfg

    @property
    def validation(self) -> ValidationAPI:
        """Access validation functionality."""
        if self._validation is None:
            self._validation = ValidationAPI(root=self.root)
        return self._validation

    @property
    def git(self) -> GitAPI:
        """Access shadow git functionality."""
        if self._git is None:
            self._git = GitAPI(root=self.root)
        return self._git

    @property
    def context(self) -> ContextAPI:
        """Access context compilation functionality."""
        if self._context is None:
            self._context = ContextAPI(root=self.root)
        return self._context

    @property
    def health(self) -> HealthAPI:
        """Access health analysis functionality."""
        if self._health is None:
            self._health = HealthAPI(root=self.root)
        return self._health


# Convenience alias
API = MossAPI
