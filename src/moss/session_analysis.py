"""Claude Code session log analysis.

Parses Claude Code JSONL logs to extract:
- Tool call frequency and success rates
- Error patterns and retry loops
- Token usage and context growth
- Parallelization opportunities
"""

from __future__ import annotations

import json
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class ToolStats:
    """Statistics for a single tool."""

    name: str
    calls: int = 0
    errors: int = 0

    @property
    def success_rate(self) -> float:
        if self.calls == 0:
            return 0.0
        return (self.calls - self.errors) / self.calls

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "calls": self.calls,
            "errors": self.errors,
            "success_rate": round(self.success_rate * 100, 1),
        }


@dataclass
class TokenStats:
    """Token usage statistics."""

    total_input: int = 0
    total_output: int = 0
    cache_read: int = 0
    cache_create: int = 0
    min_context: int = 0
    max_context: int = 0
    api_calls: int = 0

    @property
    def avg_context(self) -> int:
        if self.api_calls == 0:
            return 0
        # Context = new input + cache read
        total_context = self.total_input + self.cache_read
        return total_context // self.api_calls

    def to_dict(self) -> dict[str, Any]:
        return {
            "total_input": self.total_input,
            "total_output": self.total_output,
            "cache_read": self.cache_read,
            "cache_create": self.cache_create,
            "min_context": self.min_context,
            "max_context": self.max_context,
            "avg_context": self.avg_context,
            "api_calls": self.api_calls,
        }


@dataclass
class ErrorPattern:
    """A recurring error pattern."""

    category: str
    count: int
    examples: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "category": self.category,
            "count": self.count,
            "examples": self.examples[:3],
        }


@dataclass
class SessionAnalysis:
    """Complete analysis of a Claude Code session."""

    session_path: Path
    message_counts: dict[str, int] = field(default_factory=dict)
    tool_stats: dict[str, ToolStats] = field(default_factory=dict)
    token_stats: TokenStats = field(default_factory=TokenStats)
    error_patterns: list[ErrorPattern] = field(default_factory=list)
    parallel_opportunities: int = 0
    total_turns: int = 0

    @property
    def total_tool_calls(self) -> int:
        return sum(t.calls for t in self.tool_stats.values())

    @property
    def total_errors(self) -> int:
        return sum(t.errors for t in self.tool_stats.values())

    @property
    def overall_success_rate(self) -> float:
        if self.total_tool_calls == 0:
            return 0.0
        return (self.total_tool_calls - self.total_errors) / self.total_tool_calls

    def to_dict(self) -> dict[str, Any]:
        return {
            "session_path": str(self.session_path),
            "message_counts": self.message_counts,
            "tool_stats": {k: v.to_dict() for k, v in self.tool_stats.items()},
            "token_stats": self.token_stats.to_dict(),
            "error_patterns": [e.to_dict() for e in self.error_patterns],
            "summary": {
                "total_tool_calls": self.total_tool_calls,
                "total_errors": self.total_errors,
                "success_rate": round(self.overall_success_rate * 100, 1),
                "total_turns": self.total_turns,
                "parallel_opportunities": self.parallel_opportunities,
            },
        }

    def to_compact(self) -> str:
        """Format as compact summary."""
        lines = []
        lines.append(
            f"session: {self.total_tool_calls} tool calls, {self.overall_success_rate:.0%} success"
        )

        # Top tools
        top_tools = sorted(self.tool_stats.values(), key=lambda t: t.calls, reverse=True)[:5]
        tool_summary = ", ".join(f"{t.name}:{t.calls}" for t in top_tools)
        lines.append(f"tools: {tool_summary}")

        # Errors
        if self.total_errors:
            lines.append(f"errors: {self.total_errors}")

        # Token stats
        if self.token_stats.api_calls:
            ctx_k = self.token_stats.avg_context / 1000
            lines.append(f"context: avg {ctx_k:.0f}K tokens")

        return "\n".join(lines)

    def to_markdown(self) -> str:
        """Format as markdown report."""
        lines = ["# Session Analysis", ""]

        # Summary
        lines.append("## Summary")
        lines.append("")
        lines.append(f"- **Tool calls**: {self.total_tool_calls}")
        lines.append(f"- **Success rate**: {self.overall_success_rate:.1%}")
        lines.append(f"- **Total turns**: {self.total_turns}")
        lines.append(f"- **Parallel opportunities**: {self.parallel_opportunities}")
        lines.append("")

        # Message types
        if self.message_counts:
            lines.append("## Message Types")
            lines.append("")
            lines.append("| Type | Count |")
            lines.append("|------|-------|")
            for msg_type, count in sorted(self.message_counts.items(), key=lambda x: -x[1]):
                lines.append(f"| {msg_type} | {count} |")
            lines.append("")

        # Tool usage
        if self.tool_stats:
            lines.append("## Tool Usage")
            lines.append("")
            lines.append("| Tool | Calls | Errors | Success Rate |")
            lines.append("|------|-------|--------|--------------|")
            for tool in sorted(self.tool_stats.values(), key=lambda t: t.calls, reverse=True):
                lines.append(
                    f"| {tool.name} | {tool.calls} | {tool.errors} | {tool.success_rate:.0%} |"
                )
            lines.append("")

        # Token usage
        if self.token_stats.api_calls:
            ts = self.token_stats
            lines.append("## Token Usage")
            lines.append("")
            lines.append(f"- **API calls**: {ts.api_calls}")
            lines.append(f"- **Avg context**: {ts.avg_context:,} tokens")
            lines.append(f"- **Context range**: {ts.min_context:,} - {ts.max_context:,}")
            if ts.cache_read:
                lines.append(f"- **Cache read**: {ts.cache_read:,} tokens")
            if ts.cache_create:
                lines.append(f"- **Cache create**: {ts.cache_create:,} tokens")
            lines.append("")

        # Error patterns
        if self.error_patterns:
            lines.append("## Error Patterns")
            lines.append("")
            for pattern in self.error_patterns:
                lines.append(f"### {pattern.category} ({pattern.count})")
                for ex in pattern.examples[:3]:
                    lines.append(f"- {ex}")
                lines.append("")

        return "\n".join(lines)


class SessionAnalyzer:
    """Analyze Claude Code session logs."""

    def __init__(self, session_path: Path):
        self.session_path = Path(session_path)

    def analyze(self) -> SessionAnalysis:
        """Parse and analyze the session log."""
        result = SessionAnalysis(session_path=self.session_path)

        if not self.session_path.exists():
            return result

        # Read JSONL file
        entries = self._read_entries()

        # Count message types
        result.message_counts = self._count_message_types(entries)

        # Analyze tool usage
        result.tool_stats = self._analyze_tools(entries)

        # Analyze tokens
        result.token_stats = self._analyze_tokens(entries)

        # Find error patterns
        result.error_patterns = self._find_error_patterns(entries)

        # Count turns and parallel opportunities
        result.total_turns = self._count_turns(entries)
        result.parallel_opportunities = self._find_parallel_opportunities(entries)

        return result

    def _read_entries(self) -> list[dict[str, Any]]:
        """Read all JSONL entries."""
        entries = []
        try:
            with open(self.session_path, encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line:
                        try:
                            entries.append(json.loads(line))
                        except json.JSONDecodeError:
                            continue
        except OSError:
            pass
        return entries

    def _count_message_types(self, entries: list[dict]) -> dict[str, int]:
        """Count occurrences of each message type."""
        counter: Counter[str] = Counter()
        for entry in entries:
            msg_type = entry.get("type", "unknown")
            counter[msg_type] += 1
        return dict(counter)

    def _analyze_tools(self, entries: list[dict]) -> dict[str, ToolStats]:
        """Analyze tool call frequency and success rates."""
        stats: dict[str, ToolStats] = {}

        for entry in entries:
            if entry.get("type") != "assistant":
                continue

            message = entry.get("message", {})
            content = message.get("content", [])

            if not isinstance(content, list):
                continue

            for block in content:
                if not isinstance(block, dict):
                    continue

                # Tool use blocks
                if block.get("type") == "tool_use":
                    tool_name = block.get("name", "unknown")
                    if tool_name not in stats:
                        stats[tool_name] = ToolStats(name=tool_name)
                    stats[tool_name].calls += 1

                # Tool result blocks (look for errors)
                if block.get("type") == "tool_result":
                    # Check if this is an error
                    is_error = block.get("is_error", False)
                    content_text = block.get("content", "")
                    if is_error or (
                        isinstance(content_text, str)
                        and any(
                            err in content_text.lower() for err in ["error", "failed", "exception"]
                        )
                    ):
                        # Tool error detected - would need to track tool_ids to match
                        # For now, errors are counted in _find_error_patterns
                        pass

        return stats

    def _analyze_tokens(self, entries: list[dict]) -> TokenStats:
        """Analyze token usage from API calls."""
        stats = TokenStats()
        request_data: dict[str, dict] = {}

        for entry in entries:
            if entry.get("type") != "assistant":
                continue

            message = entry.get("message", {})
            usage = message.get("usage", {})

            if not usage:
                continue

            # Deduplicate by request ID and take max values (streaming updates)
            request_id = entry.get("requestId", str(id(entry)))
            input_tokens = usage.get("input_tokens", 0)
            output_tokens = usage.get("output_tokens", 0)
            cache_read = usage.get("cache_read_input_tokens", 0)
            cache_create = usage.get("cache_creation_input_tokens", 0)

            if request_id not in request_data:
                request_data[request_id] = {
                    "input": 0,
                    "output": 0,
                    "cache_read": 0,
                    "cache_create": 0,
                }

            # Take max values for this request (streaming updates progressively)
            rd = request_data[request_id]
            rd["input"] = max(rd["input"], input_tokens)
            rd["output"] = max(rd["output"], output_tokens)
            rd["cache_read"] = max(rd["cache_read"], cache_read)
            rd["cache_create"] = max(rd["cache_create"], cache_create)

        # Aggregate
        for rd in request_data.values():
            if rd["input"] > 0 or rd["cache_read"] > 0:
                stats.api_calls += 1
                stats.total_input += rd["input"]
                stats.total_output += rd["output"]
                stats.cache_read += rd["cache_read"]
                stats.cache_create += rd["cache_create"]

                # Context = new input + cache read
                context_size = rd["input"] + rd["cache_read"]
                if stats.min_context == 0 or context_size < stats.min_context:
                    stats.min_context = context_size
                if context_size > stats.max_context:
                    stats.max_context = context_size

        return stats

    def _find_error_patterns(self, entries: list[dict]) -> list[ErrorPattern]:
        """Identify recurring error patterns."""
        error_categories: dict[str, list[str]] = {}

        for entry in entries:
            if entry.get("type") != "assistant":
                continue

            message = entry.get("message", {})
            content = message.get("content", [])

            if not isinstance(content, list):
                continue

            for block in content:
                if not isinstance(block, dict):
                    continue

                if block.get("type") == "tool_result" and block.get("is_error"):
                    error_text = str(block.get("content", ""))[:100]
                    category = self._categorize_error(error_text)
                    if category not in error_categories:
                        error_categories[category] = []
                    error_categories[category].append(error_text)

        patterns = []
        for category, examples in sorted(error_categories.items(), key=lambda x: -len(x[1])):
            patterns.append(ErrorPattern(category=category, count=len(examples), examples=examples))

        return patterns

    def _categorize_error(self, error_text: str) -> str:
        """Categorize an error by its content."""
        text = error_text.lower()
        if "exit code" in text:
            return "Command failure"
        if "not found" in text:
            return "File not found"
        if "permission" in text:
            return "Permission error"
        if "timeout" in text:
            return "Timeout"
        if "syntax" in text:
            return "Syntax error"
        if "import" in text:
            return "Import error"
        return "Other"

    def _count_turns(self, entries: list[dict]) -> int:
        """Count assistant turns."""
        seen_request_ids: set[str] = set()
        for entry in entries:
            if entry.get("type") == "assistant":
                request_id = entry.get("requestId", str(id(entry)))
                seen_request_ids.add(request_id)
        return len(seen_request_ids)

    def _find_parallel_opportunities(self, entries: list[dict]) -> int:
        """Count turns with only 1 tool call that could have parallelized."""
        single_tool_turns = 0

        for entry in entries:
            if entry.get("type") != "assistant":
                continue

            message = entry.get("message", {})
            content = message.get("content", [])

            if not isinstance(content, list):
                continue

            tool_uses = [b for b in content if isinstance(b, dict) and b.get("type") == "tool_use"]

            # If exactly 1 tool use, it's a potential parallel opportunity
            if len(tool_uses) == 1:
                single_tool_turns += 1

        return single_tool_turns


def analyze_session(path: str | Path) -> SessionAnalysis:
    """Convenience function to analyze a session log.

    Args:
        path: Path to the JSONL session file

    Returns:
        SessionAnalysis with all statistics
    """
    analyzer = SessionAnalyzer(Path(path))
    return analyzer.analyze()
