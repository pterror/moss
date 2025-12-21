"""Custom analysis rules for user-defined patterns.

This module allows users to define custom rules to detect patterns
in their codebase. Rules can match:
- Function/class names
- Import patterns
- Code patterns (regex)
- Symbol relationships

Usage:
    from moss.rules import RuleEngine, Rule, load_rules_from_toml

    # Define rules
    rules = [
        Rule(
            name="no-print",
            pattern=r"\\bprint\\s*\\(",
            message="Use logging instead of print",
            severity="warning",
        ),
    ]

    # Run analysis
    engine = RuleEngine(rules)
    violations = engine.check_file(Path("myfile.py"))
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class Rule:
    """A custom analysis rule."""

    name: str
    pattern: str  # Regex pattern
    message: str
    severity: str = "warning"  # info, warning, error
    category: str = "custom"
    file_pattern: str = "*.py"  # Glob pattern for applicable files
    enabled: bool = True

    # Optional metadata
    fix: str | None = None  # Suggested fix
    documentation: str | None = None


@dataclass
class Violation:
    """A rule violation found in code."""

    rule: Rule
    file_path: Path
    line: int
    column: int
    match_text: str
    context: str = ""  # Surrounding code context


@dataclass
class RuleResult:
    """Result of running rules on a codebase."""

    violations: list[Violation] = field(default_factory=list)
    files_checked: int = 0
    rules_applied: int = 0

    def by_severity(self, severity: str) -> list[Violation]:
        """Get violations by severity."""
        return [v for v in self.violations if v.rule.severity == severity]

    def by_rule(self, rule_name: str) -> list[Violation]:
        """Get violations by rule name."""
        return [v for v in self.violations if v.rule.name == rule_name]

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "files_checked": self.files_checked,
            "rules_applied": self.rules_applied,
            "total_violations": len(self.violations),
            "by_severity": {
                "error": len(self.by_severity("error")),
                "warning": len(self.by_severity("warning")),
                "info": len(self.by_severity("info")),
            },
            "violations": [
                {
                    "rule": v.rule.name,
                    "file": str(v.file_path),
                    "line": v.line,
                    "column": v.column,
                    "message": v.rule.message,
                    "severity": v.rule.severity,
                    "match": v.match_text,
                }
                for v in self.violations
            ],
        }


class RuleEngine:
    """Engine for checking custom rules against code."""

    def __init__(self, rules: list[Rule] | None = None) -> None:
        """Initialize rule engine.

        Args:
            rules: List of rules to apply
        """
        self.rules = rules or []
        self._compiled: dict[str, re.Pattern[str]] = {}

    def add_rule(self, rule: Rule) -> None:
        """Add a rule to the engine."""
        self.rules.append(rule)
        # Clear compiled cache
        if rule.name in self._compiled:
            del self._compiled[rule.name]

    def remove_rule(self, name: str) -> bool:
        """Remove a rule by name."""
        for i, rule in enumerate(self.rules):
            if rule.name == name:
                del self.rules[i]
                if name in self._compiled:
                    del self._compiled[name]
                return True
        return False

    def _get_pattern(self, rule: Rule) -> re.Pattern[str]:
        """Get compiled regex pattern for a rule."""
        if rule.name not in self._compiled:
            self._compiled[rule.name] = re.compile(rule.pattern)
        return self._compiled[rule.name]

    def check_file(self, file_path: Path) -> list[Violation]:
        """Check a single file against all rules.

        Args:
            file_path: Path to file to check

        Returns:
            List of violations found
        """
        violations: list[Violation] = []

        try:
            content = file_path.read_text()
        except (OSError, UnicodeDecodeError):
            return violations

        lines = content.splitlines()

        for rule in self.rules:
            if not rule.enabled:
                continue

            # Check file pattern
            if not file_path.match(rule.file_pattern):
                continue

            pattern = self._get_pattern(rule)

            for line_num, line in enumerate(lines, 1):
                for match in pattern.finditer(line):
                    # Get context (surrounding lines)
                    ctx_start = max(0, line_num - 2)
                    ctx_end = min(len(lines), line_num + 1)
                    context_lines = lines[ctx_start:ctx_end]
                    context = "\n".join(context_lines)

                    violations.append(
                        Violation(
                            rule=rule,
                            file_path=file_path,
                            line=line_num,
                            column=match.start() + 1,
                            match_text=match.group(),
                            context=context,
                        )
                    )

        return violations

    def check_directory(
        self,
        directory: Path,
        pattern: str = "**/*.py",
        exclude_patterns: list[str] | None = None,
    ) -> RuleResult:
        """Check all files in a directory.

        Args:
            directory: Directory to check
            pattern: Glob pattern for files
            exclude_patterns: Patterns to exclude

        Returns:
            RuleResult with all violations
        """
        directory = Path(directory).resolve()
        exclude_patterns = exclude_patterns or ["**/__pycache__/*", "**/.venv/*"]

        result = RuleResult()
        result.rules_applied = sum(1 for r in self.rules if r.enabled)

        # Collect files
        files = list(directory.glob(pattern))

        # Filter excluded files
        for exclude in exclude_patterns:
            excluded = set(directory.glob(exclude))
            files = [f for f in files if f not in excluded]

        # Check each file
        for file_path in files:
            violations = self.check_file(file_path)
            result.violations.extend(violations)
            result.files_checked += 1

        return result


# Built-in rules
BUILTIN_RULES = [
    Rule(
        name="no-print",
        pattern=r"\bprint\s*\(",
        message="Consider using logging instead of print statements",
        severity="info",
        category="best-practice",
    ),
    Rule(
        name="no-breakpoint",
        pattern=r"\bbreakpoint\s*\(",
        message="Remove breakpoint() call before committing",
        severity="warning",
        category="debug",
    ),
    Rule(
        name="no-todo",
        pattern=r"#\s*TODO[:\s]",
        message="TODO comment found",
        severity="info",
        category="documentation",
    ),
    Rule(
        name="no-fixme",
        pattern=r"#\s*FIXME[:\s]",
        message="FIXME comment found - needs attention",
        severity="warning",
        category="documentation",
    ),
    Rule(
        name="no-bare-except",
        pattern=r"except\s*:",
        message="Avoid bare except clauses",
        severity="warning",
        category="error-handling",
    ),
    Rule(
        name="no-pass-except",
        pattern=r"except.*:\s*\n\s*pass",
        message="Avoid silently swallowing exceptions",
        severity="warning",
        category="error-handling",
    ),
]


def load_rules_from_toml(path: Path) -> list[Rule]:
    """Load custom rules from a TOML file.

    Expected format:
    ```toml
    [[rules]]
    name = "no-debug"
    pattern = "import pdb"
    message = "Remove debug imports"
    severity = "warning"

    [[rules]]
    name = "no-star-import"
    pattern = "from .* import \\\\*"
    message = "Avoid star imports"
    severity = "info"
    ```

    Args:
        path: Path to TOML file

    Returns:
        List of Rule objects
    """
    import tomllib

    content = path.read_text()
    data = tomllib.loads(content)

    rules: list[Rule] = []
    for rule_data in data.get("rules", []):
        rules.append(
            Rule(
                name=rule_data["name"],
                pattern=rule_data["pattern"],
                message=rule_data["message"],
                severity=rule_data.get("severity", "warning"),
                category=rule_data.get("category", "custom"),
                file_pattern=rule_data.get("file_pattern", "*.py"),
                enabled=rule_data.get("enabled", True),
                fix=rule_data.get("fix"),
                documentation=rule_data.get("documentation"),
            )
        )

    return rules


def load_rules_from_config(directory: Path) -> list[Rule]:
    """Load rules from configuration files.

    Looks for rules in:
    1. moss.toml [rules] section
    2. .moss/rules.toml
    3. pyproject.toml [tool.moss.rules] section

    Args:
        directory: Project directory

    Returns:
        List of Rule objects
    """
    import tomllib

    rules: list[Rule] = []
    directory = Path(directory).resolve()

    # Check moss.toml
    moss_toml = directory / "moss.toml"
    if moss_toml.exists():
        try:
            data = tomllib.loads(moss_toml.read_text())
            for rule_data in data.get("rules", []):
                rules.append(_rule_from_dict(rule_data))
        except (OSError, tomllib.TOMLDecodeError, KeyError, TypeError):
            pass

    # Check .moss/rules.toml
    rules_toml = directory / ".moss" / "rules.toml"
    if rules_toml.exists():
        try:
            rules.extend(load_rules_from_toml(rules_toml))
        except (OSError, tomllib.TOMLDecodeError, KeyError, TypeError):
            pass

    # Check pyproject.toml
    pyproject = directory / "pyproject.toml"
    if pyproject.exists():
        try:
            data = tomllib.loads(pyproject.read_text())
            tool_moss = data.get("tool", {}).get("moss", {})
            for rule_data in tool_moss.get("rules", []):
                rules.append(_rule_from_dict(rule_data))
        except (OSError, tomllib.TOMLDecodeError, KeyError, TypeError):
            pass

    return rules


def _rule_from_dict(data: dict[str, Any]) -> Rule:
    """Create a Rule from a dictionary."""
    return Rule(
        name=data["name"],
        pattern=data["pattern"],
        message=data["message"],
        severity=data.get("severity", "warning"),
        category=data.get("category", "custom"),
        file_pattern=data.get("file_pattern", "*.py"),
        enabled=data.get("enabled", True),
        fix=data.get("fix"),
        documentation=data.get("documentation"),
    )


def create_engine_with_builtins(
    include_builtins: bool = True,
    custom_rules: list[Rule] | None = None,
) -> RuleEngine:
    """Create a rule engine with optional built-in rules.

    Args:
        include_builtins: Include built-in rules
        custom_rules: Additional custom rules

    Returns:
        Configured RuleEngine
    """
    rules: list[Rule] = []

    if include_builtins:
        rules.extend(BUILTIN_RULES)

    if custom_rules:
        rules.extend(custom_rules)

    return RuleEngine(rules)
