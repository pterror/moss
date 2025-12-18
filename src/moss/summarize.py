"""Codebase summarization for documentation verification.

Provides hierarchical summaries of codebases:
- File level: Functions, classes, their purposes
- Module level: Module purpose, key exports
- Package level: Package structure, dependencies
- Project level: Architecture overview
"""

from __future__ import annotations

import ast
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from moss.skeleton import PythonSkeletonExtractor, Symbol


@dataclass
class FileSummary:
    """Summary of a single source file."""

    path: Path
    module_name: str
    docstring: str | None
    symbols: list[Symbol]
    imports: list[str]
    line_count: int

    @property
    def classes(self) -> list[Symbol]:
        """Get all class symbols."""
        return [s for s in self.symbols if s.kind == "class"]

    @property
    def functions(self) -> list[Symbol]:
        """Get all top-level function symbols."""
        return [s for s in self.symbols if s.kind == "function"]

    @property
    def public_symbols(self) -> list[Symbol]:
        """Get public (non-underscore) symbols."""
        return [s for s in self.symbols if not s.name.startswith("_")]

    def to_markdown(self, include_signatures: bool = True) -> str:
        """Render as markdown."""
        lines = [f"### `{self.module_name}`"]

        if self.docstring:
            # First line of docstring as description
            first_line = self.docstring.split("\n")[0].strip()
            lines.append(f"\n{first_line}")

        if self.classes:
            lines.append("\n**Classes:**")
            for cls in self.classes:
                desc = cls.docstring.split("\n")[0] if cls.docstring else ""
                if include_signatures:
                    lines.append(f"- `{cls.name}` - {desc}")
                else:
                    lines.append(f"- {cls.name}: {desc}")

        if self.functions:
            lines.append("\n**Functions:**")
            for fn in self.functions:
                desc = fn.docstring.split("\n")[0] if fn.docstring else ""
                if include_signatures:
                    lines.append(f"- `{fn.signature}` - {desc}")
                else:
                    lines.append(f"- {fn.name}: {desc}")

        return "\n".join(lines)


@dataclass
class PackageSummary:
    """Summary of a Python package (directory with __init__.py)."""

    path: Path
    name: str
    docstring: str | None  # From __init__.py
    files: list[FileSummary]
    subpackages: list[PackageSummary]

    @property
    def all_files(self) -> list[FileSummary]:
        """Get all files recursively."""
        result = list(self.files)
        for sub in self.subpackages:
            result.extend(sub.all_files)
        return result

    @property
    def total_lines(self) -> int:
        """Total lines of code."""
        return sum(f.line_count for f in self.all_files)

    @property
    def total_classes(self) -> int:
        """Total number of classes."""
        return sum(len(f.classes) for f in self.all_files)

    @property
    def total_functions(self) -> int:
        """Total number of functions."""
        return sum(len(f.functions) for f in self.all_files)

    def to_markdown(self, depth: int = 0) -> str:
        """Render as markdown."""
        indent = "#" * (depth + 2)
        lines = [f"{indent} `{self.name}/`"]

        if self.docstring:
            first_line = self.docstring.split("\n")[0].strip()
            lines.append(f"\n{first_line}")

        # Stats
        lines.append(f"\n*{len(self.files)} files, {self.total_lines} lines*")

        # Key modules
        if self.files:
            lines.append("\n**Modules:**")
            for f in sorted(self.files, key=lambda x: x.module_name):
                desc = f.docstring.split("\n")[0] if f.docstring else ""
                lines.append(f"- `{f.module_name}` - {desc}")

        # Subpackages
        for sub in sorted(self.subpackages, key=lambda x: x.name):
            lines.append("")
            lines.append(sub.to_markdown(depth + 1))

        return "\n".join(lines)


@dataclass
class ProjectSummary:
    """Summary of an entire project."""

    root: Path
    name: str
    packages: list[PackageSummary]
    standalone_files: list[FileSummary]
    metadata: dict[str, Any] = field(default_factory=dict)

    @property
    def total_lines(self) -> int:
        """Total lines of code."""
        pkg_lines = sum(p.total_lines for p in self.packages)
        file_lines = sum(f.line_count for f in self.standalone_files)
        return pkg_lines + file_lines

    @property
    def total_files(self) -> int:
        """Total number of files."""
        pkg_files = sum(len(p.all_files) for p in self.packages)
        return pkg_files + len(self.standalone_files)

    def to_markdown(self) -> str:
        """Render as markdown."""
        lines = [f"# {self.name}"]

        # Overview stats
        total_classes = sum(p.total_classes for p in self.packages)
        total_functions = sum(p.total_functions for p in self.packages)
        lines.append(
            f"\n**{self.total_files} files** | "
            f"**{self.total_lines} lines** | "
            f"**{total_classes} classes** | "
            f"**{total_functions} functions**"
        )

        # Packages
        for pkg in sorted(self.packages, key=lambda x: x.name):
            lines.append("")
            lines.append(pkg.to_markdown())

        # Standalone files
        if self.standalone_files:
            lines.append("\n## Standalone Files")
            for f in sorted(self.standalone_files, key=lambda x: x.module_name):
                lines.append(f.to_markdown())

        return "\n".join(lines)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON output."""
        return {
            "name": self.name,
            "root": str(self.root),
            "stats": {
                "total_files": self.total_files,
                "total_lines": self.total_lines,
            },
            "packages": [self._package_to_dict(p) for p in self.packages],
            "standalone_files": [self._file_to_dict(f) for f in self.standalone_files],
            "metadata": self.metadata,
        }

    def _package_to_dict(self, pkg: PackageSummary) -> dict[str, Any]:
        return {
            "name": pkg.name,
            "path": str(pkg.path),
            "docstring": pkg.docstring,
            "files": [self._file_to_dict(f) for f in pkg.files],
            "subpackages": [self._package_to_dict(s) for s in pkg.subpackages],
            "stats": {
                "total_lines": pkg.total_lines,
                "total_classes": pkg.total_classes,
                "total_functions": pkg.total_functions,
            },
        }

    def _file_to_dict(self, f: FileSummary) -> dict[str, Any]:
        return {
            "module": f.module_name,
            "path": str(f.path),
            "docstring": f.docstring,
            "line_count": f.line_count,
            "classes": [self._symbol_to_dict(s) for s in f.classes],
            "functions": [self._symbol_to_dict(s) for s in f.functions],
            "imports": f.imports,
        }

    def _symbol_to_dict(self, s: Symbol) -> dict[str, Any]:
        return {
            "name": s.name,
            "kind": s.kind,
            "signature": s.signature,
            "docstring": s.docstring,
            "line_count": s.line_count,
        }


class Summarizer:
    """Generates hierarchical summaries of codebases."""

    def __init__(
        self,
        include_private: bool = False,
        include_tests: bool = False,
    ):
        self.include_private = include_private
        self.include_tests = include_tests

    def summarize_project(self, root: Path) -> ProjectSummary:
        """Summarize an entire project."""
        root = root.resolve()
        name = root.name

        packages: list[PackageSummary] = []
        standalone_files: list[FileSummary] = []

        # Find src/ directory or use root
        src_dir = root / "src"
        if not src_dir.exists():
            src_dir = root

        # Find all Python packages
        for item in src_dir.iterdir():
            if item.is_dir() and (item / "__init__.py").exists():
                pkg = self.summarize_package(item)
                if pkg:
                    packages.append(pkg)
            elif item.is_file() and item.suffix == ".py":
                if self._should_include_file(item):
                    summary = self.summarize_file(item)
                    if summary:
                        standalone_files.append(summary)

        # Also check root for standalone files
        if src_dir != root:
            for item in root.iterdir():
                if item.is_file() and item.suffix == ".py":
                    if self._should_include_file(item):
                        summary = self.summarize_file(item)
                        if summary:
                            standalone_files.append(summary)

        return ProjectSummary(
            root=root,
            name=name,
            packages=packages,
            standalone_files=standalone_files,
        )

    def summarize_package(self, path: Path) -> PackageSummary | None:
        """Summarize a Python package."""
        if not path.is_dir():
            return None

        init_file = path / "__init__.py"
        if not init_file.exists():
            return None

        # Get package docstring from __init__.py
        docstring = None
        try:
            source = init_file.read_text()
            tree = ast.parse(source)
            docstring = ast.get_docstring(tree)
        except Exception:
            pass

        files: list[FileSummary] = []
        subpackages: list[PackageSummary] = []

        for item in sorted(path.iterdir()):
            if item.is_dir() and (item / "__init__.py").exists():
                sub = self.summarize_package(item)
                if sub:
                    subpackages.append(sub)
            elif item.is_file() and item.suffix == ".py":
                if self._should_include_file(item):
                    summary = self.summarize_file(item)
                    if summary:
                        files.append(summary)

        return PackageSummary(
            path=path,
            name=path.name,
            docstring=docstring,
            files=files,
            subpackages=subpackages,
        )

    def summarize_file(self, path: Path) -> FileSummary | None:
        """Summarize a single Python file."""
        if not path.is_file() or path.suffix != ".py":
            return None

        try:
            source = path.read_text()
        except Exception:
            return None

        # Parse AST for docstring and imports
        try:
            tree = ast.parse(source)
            docstring = ast.get_docstring(tree)
            imports = self._extract_imports(tree)
        except SyntaxError:
            docstring = None
            imports = []

        # Extract symbols using skeleton extractor
        extractor = PythonSkeletonExtractor(source, include_private=self.include_private)
        try:
            extractor.visit(ast.parse(source))
        except SyntaxError:
            pass

        return FileSummary(
            path=path,
            module_name=path.stem,
            docstring=docstring,
            symbols=extractor.symbols,
            imports=imports,
            line_count=len(source.splitlines()),
        )

    def _should_include_file(self, path: Path) -> bool:
        """Check if file should be included."""
        name = path.name

        # Skip test files unless requested
        if not self.include_tests:
            if name.startswith("test_") or name.endswith("_test.py"):
                return False
            if "tests" in path.parts:
                return False

        # Skip private files unless requested
        if not self.include_private:
            if name.startswith("_") and name != "__init__.py":
                return False

        return True

    def _extract_imports(self, tree: ast.AST) -> list[str]:
        """Extract import statements from AST."""
        imports = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append(alias.name)
            elif isinstance(node, ast.ImportFrom):
                module = node.module or ""
                imports.append(module)
        return imports
