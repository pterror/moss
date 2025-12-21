"""Heuristic Guardrails: Fast, structural mistake detection.

Heuristics provide a fast feedback loop for obvious mistakes (anti-patterns,
common hallucinations, convention violations) before reaching expensive
domain-specific validators.
"""

from __future__ import annotations

import ast
import re
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from pathlib import Path

from moss.validators import ValidationIssue, ValidationResult, ValidationSeverity


@dataclass
class HeuristicResult:
    """Result of a heuristic check."""

    success: bool
    issues: list[ValidationIssue] = field(default_factory=list)


class HeuristicRule(ABC):
    """Abstract base for heuristic rules."""

    @property
    @abstractmethod
    def name(self) -> str:
        """Rule name."""
        ...

    @abstractmethod
    def check(self, path: Path, source: str | None = None) -> HeuristicResult:
        """Run the heuristic check."""
        ...


class UnusedImportHeuristic(HeuristicRule):
    """Detect obviously unused imports via simple string matching."""

    @property
    def name(self) -> str:
        return "unused_import"

    def check(self, path: Path, source: str | None = None) -> HeuristicResult:
        if source is None:
            source = path.read_text()

        issues = []
        try:
            tree = ast.parse(source)
            imports = []
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports.append((alias.asname or alias.name, node.lineno))
                elif isinstance(node, ast.ImportFrom):
                    for alias in node.names:
                        imports.append((alias.asname or alias.name, node.lineno))

            # Simple check: is the name used anywhere else in the file?
            # Note: This is a heuristic, real linter is better.
            for name, lineno in imports:
                # Find occurrences of name not in import lines
                pattern = rf"\b{name}\b"
                matches = re.findall(pattern, source)
                # If only one match, it's just the import itself
                if len(matches) <= 1:
                    issues.append(
                        ValidationIssue(
                            message=f"Import '{name}' appears unused",
                            severity=ValidationSeverity.WARNING,
                            file=path,
                            line=lineno,
                            source="heuristic:unused_import",
                        )
                    )
        except Exception:
            pass

        return HeuristicResult(success=len(issues) == 0, issues=issues)


class LargeFunctionHeuristic(HeuristicRule):
    """Detect functions that are obviously too large/complex."""

    def __init__(self, max_lines: int = 50):
        self.max_lines = max_lines

    @property
    def name(self) -> str:
        return "large_function"

    def check(self, path: Path, source: str | None = None) -> HeuristicResult:
        if source is None:
            source = path.read_text()

        issues = []
        try:
            tree = ast.parse(source)
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    lines = node.end_lineno - node.lineno if node.end_lineno else 0
                    if lines > self.max_lines:
                        issues.append(
                            ValidationIssue(
                                message=f"Function '{node.name}' is very large ({lines} lines)",
                                severity=ValidationSeverity.INFO,
                                file=path,
                                line=node.lineno,
                                source="heuristic:large_function",
                            )
                        )
        except Exception:
            pass

        return HeuristicResult(success=len(issues) == 0, issues=issues)


class HeuristicEngine:
    """Orchestrates multiple heuristic rules."""

    def __init__(self, rules: list[HeuristicRule] | None = None):
        self.rules = rules or [UnusedImportHeuristic(), LargeFunctionHeuristic()]

    def check(self, path: Path) -> ValidationResult:
        """Run all heuristics and return a ValidationResult."""
        all_issues = []
        try:
            source = path.read_text()
            for rule in self.rules:
                result = rule.check(path, source)
                all_issues.extend(result.issues)
        except Exception as e:
            all_issues.append(
                ValidationIssue(
                    message=f"Heuristic engine failed: {e}",
                    severity=ValidationSeverity.WARNING,
                    file=path,
                    source="heuristic_engine",
                )
            )

        return ValidationResult(
            success=True,  # Heuristics are advisory, don't block by default
            issues=all_issues,
            metadata={"source": "heuristic_engine"},
        )
