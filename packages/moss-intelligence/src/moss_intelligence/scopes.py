"""Scope Analysis: public/private symbol statistics.

This module provides analysis of symbol visibility across a codebase:
- Count public vs private symbols per file/project
- Break down by symbol kind (class, function, method, variable)
- Identify visibility patterns and anomalies

Example:
    report = analyze_file_scopes(Path("mymodule.py"))
    print(f"Public: {report.public_count}, Private: {report.private_count}")
"""

from __future__ import annotations

import ast
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class ScopeStats:
    """Statistics for a single scope category."""

    public: int = 0
    private: int = 0
    dunder: int = 0  # __dunder__ methods

    @property
    def total(self) -> int:
        """Total symbols in this category."""
        return self.public + self.private + self.dunder

    @property
    def public_ratio(self) -> float:
        """Ratio of public symbols (0.0 to 1.0)."""
        if self.total == 0:
            return 1.0
        return self.public / self.total


@dataclass
class FileScopeReport:
    """Scope analysis report for a single file."""

    path: Path
    classes: ScopeStats = field(default_factory=ScopeStats)
    functions: ScopeStats = field(default_factory=ScopeStats)
    methods: ScopeStats = field(default_factory=ScopeStats)
    variables: ScopeStats = field(default_factory=ScopeStats)

    @property
    def total_public(self) -> int:
        """Total public symbols."""
        return (
            self.classes.public
            + self.functions.public
            + self.methods.public
            + self.variables.public
        )

    @property
    def total_private(self) -> int:
        """Total private symbols."""
        return (
            self.classes.private
            + self.functions.private
            + self.methods.private
            + self.variables.private
        )

    @property
    def total_dunder(self) -> int:
        """Total dunder symbols."""
        return (
            self.classes.dunder
            + self.functions.dunder
            + self.methods.dunder
            + self.variables.dunder
        )

    @property
    def total(self) -> int:
        """Total symbols."""
        return self.total_public + self.total_private + self.total_dunder

    @property
    def public_ratio(self) -> float:
        """Ratio of public symbols."""
        if self.total == 0:
            return 1.0
        return self.total_public / self.total

    def to_dict(self) -> dict:
        """Convert to serializable dictionary."""
        return {
            "path": str(self.path),
            "classes": {
                "public": self.classes.public,
                "private": self.classes.private,
                "dunder": self.classes.dunder,
            },
            "functions": {
                "public": self.functions.public,
                "private": self.functions.private,
                "dunder": self.functions.dunder,
            },
            "methods": {
                "public": self.methods.public,
                "private": self.methods.private,
                "dunder": self.methods.dunder,
            },
            "variables": {
                "public": self.variables.public,
                "private": self.variables.private,
                "dunder": self.variables.dunder,
            },
            "totals": {
                "public": self.total_public,
                "private": self.total_private,
                "dunder": self.total_dunder,
                "total": self.total,
                "public_ratio": self.public_ratio,
            },
        }

    def to_compact(self) -> str:
        """Format as compact string."""
        lines = [f"# {self.path.name}"]
        lines.append(
            f"Total: {self.total} symbols "
            f"({self.total_public} public, {self.total_private} private, "
            f"{self.total_dunder} dunder)"
        )
        lines.append(f"Public ratio: {self.public_ratio:.1%}")

        if self.classes.total > 0:
            lines.append(f"  Classes: {self.classes.public}/{self.classes.total} public")
        if self.functions.total > 0:
            lines.append(f"  Functions: {self.functions.public}/{self.functions.total} public")
        if self.methods.total > 0:
            lines.append(f"  Methods: {self.methods.public}/{self.methods.total} public")

        return "\n".join(lines)


@dataclass
class ProjectScopeReport:
    """Scope analysis report for a project."""

    root: Path
    files: list[FileScopeReport] = field(default_factory=list)

    @property
    def total_public(self) -> int:
        """Total public symbols across all files."""
        return sum(f.total_public for f in self.files)

    @property
    def total_private(self) -> int:
        """Total private symbols across all files."""
        return sum(f.total_private for f in self.files)

    @property
    def total_dunder(self) -> int:
        """Total dunder symbols across all files."""
        return sum(f.total_dunder for f in self.files)

    @property
    def total(self) -> int:
        """Total symbols across all files."""
        return sum(f.total for f in self.files)

    @property
    def public_ratio(self) -> float:
        """Overall public symbol ratio."""
        if self.total == 0:
            return 1.0
        return self.total_public / self.total

    @property
    def file_count(self) -> int:
        """Number of files analyzed."""
        return len(self.files)

    def to_dict(self) -> dict:
        """Convert to serializable dictionary."""
        return {
            "root": str(self.root),
            "file_count": self.file_count,
            "totals": {
                "public": self.total_public,
                "private": self.total_private,
                "dunder": self.total_dunder,
                "total": self.total,
                "public_ratio": self.public_ratio,
            },
            "files": [f.to_dict() for f in self.files],
        }

    def to_compact(self) -> str:
        """Format as compact summary."""
        lines = [f"# Project Scope Analysis: {self.root.name}"]
        lines.append(f"Files analyzed: {self.file_count}")
        lines.append(
            f"Total: {self.total} symbols "
            f"({self.total_public} public, {self.total_private} private)"
        )
        lines.append(f"Public ratio: {self.public_ratio:.1%}")

        # Show top 5 most private files
        private_files = sorted(self.files, key=lambda f: f.total_private, reverse=True)[:5]
        if private_files and any(f.total_private > 0 for f in private_files):
            lines.append("\nMost private symbols:")
            for f in private_files:
                if f.total_private > 0:
                    lines.append(f"  {f.path.name}: {f.total_private} private")

        return "\n".join(lines)


def _classify_name(name: str) -> str:
    """Classify a name as public, private, or dunder."""
    if name.startswith("__") and name.endswith("__"):
        return "dunder"
    elif name.startswith("_"):
        return "private"
    return "public"


def _increment_stats(stats: ScopeStats, name: str) -> None:
    """Increment the appropriate counter based on name classification."""
    classification = _classify_name(name)
    if classification == "dunder":
        stats.dunder += 1
    elif classification == "private":
        stats.private += 1
    else:
        stats.public += 1


class ScopeVisitor(ast.NodeVisitor):
    """AST visitor that collects scope statistics."""

    def __init__(self):
        self.report = FileScopeReport(path=Path("."))
        self._in_class = False

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        _increment_stats(self.report.classes, node.name)

        # Visit methods inside the class
        old_in_class = self._in_class
        self._in_class = True
        self.generic_visit(node)
        self._in_class = old_in_class

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        if self._in_class:
            _increment_stats(self.report.methods, node.name)
        else:
            _increment_stats(self.report.functions, node.name)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        if self._in_class:
            _increment_stats(self.report.methods, node.name)
        else:
            _increment_stats(self.report.functions, node.name)

    def visit_Assign(self, node: ast.Assign) -> None:
        # Only count module-level assignments
        if not self._in_class:
            for target in node.targets:
                if isinstance(target, ast.Name):
                    _increment_stats(self.report.variables, target.id)

    def visit_AnnAssign(self, node: ast.AnnAssign) -> None:
        # Only count module-level annotated assignments
        if not self._in_class:
            if isinstance(node.target, ast.Name):
                _increment_stats(self.report.variables, node.target.id)


def analyze_file_scopes(path: Path) -> FileScopeReport:
    """Analyze scope statistics for a single Python file.

    Args:
        path: Path to the Python file

    Returns:
        FileScopeReport with public/private statistics
    """
    source = path.read_text()

    try:
        tree = ast.parse(source)
    except SyntaxError:
        return FileScopeReport(path=path)

    visitor = ScopeVisitor()
    visitor.report.path = path
    visitor.visit(tree)

    return visitor.report


def analyze_project_scopes(
    root: Path,
    pattern: str = "**/*.py",
) -> ProjectScopeReport:
    """Analyze scope statistics for a project.

    Args:
        root: Project root directory
        pattern: Glob pattern for Python files

    Returns:
        ProjectScopeReport with aggregated statistics
    """
    report = ProjectScopeReport(root=root)

    # Skip common non-source directories
    skip_dirs = {
        ".git",
        ".venv",
        "venv",
        "node_modules",
        "__pycache__",
        ".mypy_cache",
        ".pytest_cache",
        "build",
        "dist",
    }

    for file_path in root.glob(pattern):
        if not file_path.is_file():
            continue

        # Skip files in excluded directories
        parts = file_path.relative_to(root).parts
        if any(part in skip_dirs for part in parts):
            continue

        try:
            file_report = analyze_file_scopes(file_path)
            if file_report.total > 0:
                report.files.append(file_report)
        except (OSError, UnicodeDecodeError):
            continue

    return report


def format_scope_report(report: FileScopeReport | ProjectScopeReport) -> str:
    """Format a scope report as human-readable text.

    Args:
        report: File or project scope report

    Returns:
        Formatted string
    """
    return report.to_compact()
