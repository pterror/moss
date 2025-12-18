"""Project status and health reporting.

Combines insights from summarization, documentation checks, and TODO tracking
to provide a unified view of project health and what needs attention.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from moss.check_docs import DocChecker, DocCheckResult
from moss.check_todos import TodoChecker, TodoCheckResult, TodoStatus
from moss.summarize import DocSummarizer, Summarizer


@dataclass
class WeakSpot:
    """An area of the codebase that needs attention."""

    category: str  # "docs", "tests", "todos", "complexity"
    severity: str  # "high", "medium", "low"
    message: str
    suggestion: str | None = None
    file: Path | None = None


@dataclass
class NextAction:
    """A suggested next action."""

    priority: int  # 1 = highest
    category: str
    description: str
    source: str  # Where this came from (TODO.md, code, etc.)


@dataclass
class ProjectStatus:
    """Overall project status."""

    root: Path
    name: str

    # Code stats
    total_files: int = 0
    total_lines: int = 0
    total_modules: int = 0

    # Documentation stats
    doc_files: int = 0
    doc_words: int = 0
    doc_coverage: float = 0.0

    # TODO stats
    todos_pending: int = 0
    todos_done: int = 0
    todos_orphaned: int = 0

    # Issues
    weak_spots: list[WeakSpot] = field(default_factory=list)
    next_actions: list[NextAction] = field(default_factory=list)

    # Raw results for detailed access
    doc_check: DocCheckResult | None = None
    todo_check: TodoCheckResult | None = None

    @property
    def health_score(self) -> int:
        """Calculate overall health score (0-100)."""
        score = 100

        # Penalize for low doc coverage
        if self.doc_coverage < 0.5:
            score -= int((0.5 - self.doc_coverage) * 40)

        # Penalize for orphaned TODOs
        if self.todos_orphaned > 10:
            score -= min(20, self.todos_orphaned)
        elif self.todos_orphaned > 0:
            score -= self.todos_orphaned

        # Penalize for high-severity weak spots
        high_severity = sum(1 for w in self.weak_spots if w.severity == "high")
        score -= high_severity * 10

        # Penalize for doc check warnings
        if self.doc_check and self.doc_check.warning_count > 0:
            score -= min(15, self.doc_check.warning_count)

        return max(0, min(100, score))

    @property
    def health_grade(self) -> str:
        """Get letter grade for health."""
        score = self.health_score
        if score >= 90:
            return "A"
        if score >= 80:
            return "B"
        if score >= 70:
            return "C"
        if score >= 60:
            return "D"
        return "F"

    def to_markdown(self) -> str:
        """Render as markdown."""
        lines = [f"# Project Status: {self.name}", ""]

        # Health summary
        grade = self.health_grade
        score = self.health_score
        lines.append(f"**Health: {grade}** ({score}/100)")
        lines.append("")

        # Quick stats
        lines.append("## Overview")
        lines.append("")
        lines.append("| Metric | Value |")
        lines.append("|--------|-------|")
        lines.append(f"| Code | {self.total_files} files, {self.total_lines} lines |")
        lines.append(f"| Docs | {self.doc_files} files, {self.doc_words} words |")
        lines.append(f"| Doc Coverage | {self.doc_coverage:.0%} |")
        pct_done = (
            self.todos_done / (self.todos_done + self.todos_pending) * 100
            if (self.todos_done + self.todos_pending) > 0
            else 0
        )
        todo_str = f"{self.todos_done} done, {self.todos_pending} pending ({pct_done:.0f}%)"
        lines.append(f"| TODOs | {todo_str} |")
        if self.todos_orphaned > 0:
            lines.append(f"| Orphaned TODOs | {self.todos_orphaned} |")
        lines.append("")

        # Next actions
        if self.next_actions:
            lines.append("## Next Up")
            lines.append("")
            for action in sorted(self.next_actions, key=lambda a: a.priority)[:10]:
                icon = "!" if action.priority == 1 else "-"
                lines.append(f"{icon} **{action.category}**: {action.description}")
            lines.append("")

        # Weak spots
        if self.weak_spots:
            lines.append("## Areas Needing Attention")
            lines.append("")
            severity_order = {"high": 0, "medium": 1, "low": 2}
            severity_icons = {"high": "[!]", "medium": "[?]", "low": "[i]"}
            for spot in sorted(self.weak_spots, key=lambda s: severity_order[s.severity]):
                icon = severity_icons[spot.severity]
                lines.append(f"- {icon} **{spot.category}**: {spot.message}")
                if spot.suggestion:
                    lines.append(f"  - {spot.suggestion}")
            lines.append("")

        return "\n".join(lines)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON output."""
        return {
            "name": self.name,
            "root": str(self.root),
            "health": {
                "score": self.health_score,
                "grade": self.health_grade,
            },
            "stats": {
                "code": {
                    "files": self.total_files,
                    "lines": self.total_lines,
                    "modules": self.total_modules,
                },
                "docs": {
                    "files": self.doc_files,
                    "words": self.doc_words,
                    "coverage": self.doc_coverage,
                },
                "todos": {
                    "pending": self.todos_pending,
                    "done": self.todos_done,
                    "orphaned": self.todos_orphaned,
                },
            },
            "next_actions": [
                {
                    "priority": a.priority,
                    "category": a.category,
                    "description": a.description,
                    "source": a.source,
                }
                for a in sorted(self.next_actions, key=lambda a: a.priority)
            ],
            "weak_spots": [
                {
                    "category": w.category,
                    "severity": w.severity,
                    "message": w.message,
                    "suggestion": w.suggestion,
                    "file": str(w.file) if w.file else None,
                }
                for w in self.weak_spots
            ],
        }


class StatusChecker:
    """Generate comprehensive project status."""

    def __init__(self, root: Path):
        self.root = root.resolve()

    def check(self) -> ProjectStatus:
        """Run all checks and compile status."""
        status = ProjectStatus(root=self.root, name=self.root.name)

        # Get code summary
        summarizer = Summarizer(include_private=False, include_tests=False)
        try:
            code_summary = summarizer.summarize_project(self.root)
            status.total_files = code_summary.total_files
            status.total_lines = code_summary.total_lines
            status.total_modules = len([f for p in code_summary.packages for f in p.all_files])
        except Exception:
            pass

        # Get doc summary
        doc_summarizer = DocSummarizer()
        try:
            doc_summary = doc_summarizer.summarize_docs(self.root)
            status.doc_files = len(doc_summary.files)
            status.doc_words = doc_summary.total_words
        except Exception:
            pass

        # Check docs
        doc_checker = DocChecker(self.root, check_links=True)
        try:
            doc_result = doc_checker.check()
            status.doc_check = doc_result
            status.doc_coverage = doc_result.coverage

            # Add weak spots from doc issues
            for issue in doc_result.issues:
                if issue.severity in ("error", "warning"):
                    status.weak_spots.append(
                        WeakSpot(
                            category="docs",
                            severity="high" if issue.severity == "error" else "medium",
                            message=issue.message,
                            suggestion=issue.suggestion,
                            file=issue.file,
                        )
                    )
        except Exception:
            pass

        # Check TODOs
        todo_checker = TodoChecker(self.root)
        try:
            todo_result = todo_checker.check()
            status.todo_check = todo_result
            status.todos_pending = todo_result.pending_count
            status.todos_done = todo_result.done_count
            status.todos_orphaned = todo_result.orphan_count

            # Add pending TODOs as next actions
            for item in todo_result.tracked_items:
                if item.status == TodoStatus.PENDING:
                    # Determine priority based on category
                    priority = 3  # Default
                    if item.category and "Phase" in item.category:
                        priority = 2
                    if item.category and "In Progress" in item.category:
                        priority = 1

                    status.next_actions.append(
                        NextAction(
                            priority=priority,
                            category=item.category or "Uncategorized",
                            description=item.text,
                            source="TODO.md",
                        )
                    )

            # Add orphaned TODOs as weak spots if there are many
            if todo_result.orphan_count > 5:
                status.weak_spots.append(
                    WeakSpot(
                        category="todos",
                        severity="medium",
                        message=f"{todo_result.orphan_count} TODOs in code not tracked in TODO.md",
                        suggestion="Add important TODOs to TODO.md or resolve them",
                    )
                )
        except Exception:
            pass

        # Identify additional weak spots
        self._identify_weak_spots(status)

        return status

    def _identify_weak_spots(self, status: ProjectStatus) -> None:
        """Identify additional weak spots based on analysis."""
        # Low doc coverage
        if status.doc_coverage < 0.3:
            status.weak_spots.append(
                WeakSpot(
                    category="docs",
                    severity="high",
                    message=f"Documentation coverage is only {status.doc_coverage:.0%}",
                    suggestion="Add documentation for undocumented modules",
                )
            )
        elif status.doc_coverage < 0.5:
            status.weak_spots.append(
                WeakSpot(
                    category="docs",
                    severity="medium",
                    message=f"Documentation coverage is {status.doc_coverage:.0%}",
                    suggestion="Consider documenting more modules",
                )
            )

        # Many pending TODOs
        if status.todos_pending > 20:
            status.weak_spots.append(
                WeakSpot(
                    category="todos",
                    severity="medium",
                    message=f"{status.todos_pending} pending TODOs",
                    suggestion="Consider prioritizing and completing some TODOs",
                )
            )

        # No documentation at all
        if status.doc_files == 0:
            status.weak_spots.append(
                WeakSpot(
                    category="docs",
                    severity="high",
                    message="No documentation files found",
                    suggestion="Add a README.md at minimum",
                )
            )
