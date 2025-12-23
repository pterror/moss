"""moss-intelligence: Code understanding and analysis.

Stateless, pure code intelligence. No LLM, no memory, no side effects.

Example:
    from moss_intelligence import Intelligence

    intel = Intelligence("/path/to/project")

    # Views
    skeleton = intel.skeleton("src/main.py")
    tree = intel.tree("src/", depth=2)

    # Analysis
    complexity = intel.complexity("src/")
    deps = intel.dependencies("src/main.py")
"""

from pathlib import Path
from typing import Protocol, runtime_checkable


@runtime_checkable
class ViewResult(Protocol):
    """Protocol for view results."""
    content: str
    path: Path


class Intelligence:
    """Main entry point for code intelligence.

    Provides stateless, pure code understanding:
    - Views: skeleton, tree, source
    - Analysis: complexity, security, dependencies
    - Search: symbols, references
    """

    def __init__(self, root: str | Path):
        """Initialize intelligence for a project.

        Args:
            root: Path to project root directory
        """
        self.root = Path(root).resolve()
        if not self.root.is_dir():
            raise ValueError(f"Not a directory: {self.root}")

    # === Views ===

    def view(self, locator: str) -> str:
        """View code at a locator (path, symbol, or pattern).

        Args:
            locator: File path, symbol name, or pattern

        Returns:
            Formatted view of the code
        """
        raise NotImplementedError("TODO: implement view")

    def skeleton(self, path: str | Path) -> list:
        """Extract skeleton (signatures only) from a file.

        Args:
            path: Path to file (relative to root or absolute)

        Returns:
            List of Symbol objects
        """
        raise NotImplementedError("TODO: implement skeleton")

    def tree(self, path: str | Path = ".", depth: int = 2) -> str:
        """Get tree view of directory structure.

        Args:
            path: Directory path
            depth: Maximum depth to traverse

        Returns:
            Formatted tree string
        """
        raise NotImplementedError("TODO: implement tree")

    # === Analysis ===

    def analyze(self, target: str | Path) -> dict:
        """Run comprehensive analysis on target.

        Args:
            target: File or directory to analyze

        Returns:
            Analysis results dict
        """
        raise NotImplementedError("TODO: implement analyze")

    def complexity(self, path: str | Path) -> dict:
        """Analyze cyclomatic complexity.

        Args:
            path: File or directory to analyze

        Returns:
            Complexity report
        """
        raise NotImplementedError("TODO: implement complexity")

    def security(self, path: str | Path) -> dict:
        """Run security analysis.

        Args:
            path: File or directory to analyze

        Returns:
            Security findings
        """
        raise NotImplementedError("TODO: implement security")

    def dependencies(self, path: str | Path) -> dict:
        """Analyze dependencies and imports.

        Args:
            path: File or directory to analyze

        Returns:
            Dependency graph
        """
        raise NotImplementedError("TODO: implement dependencies")

    # === Search ===

    def symbols(self, pattern: str) -> list:
        """Search for symbols matching pattern.

        Args:
            pattern: Glob or regex pattern

        Returns:
            List of matching symbols
        """
        raise NotImplementedError("TODO: implement symbols")

    def references(self, symbol: str) -> list:
        """Find references to a symbol.

        Args:
            symbol: Fully qualified symbol name

        Returns:
            List of reference locations
        """
        raise NotImplementedError("TODO: implement references")

    # === Edit (structural, no LLM) ===

    def edit(self, path: str | Path, changes: dict) -> dict:
        """Apply structural edits to a file.

        Args:
            path: File to edit
            changes: Structured change specification

        Returns:
            Edit result with diff
        """
        raise NotImplementedError("TODO: implement edit")


__all__ = ["Intelligence", "ViewResult"]
