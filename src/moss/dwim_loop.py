"""DWIM-Driven Agent Loop: LLM outputs terse intents, DWIM routes to tools.

Design principle: No tool schemas in LLM prompts. LLM outputs terse commands
like "skeleton foo.py" or "fix: add null check", DWIM interprets and routes.

See: docs/agentic-loop.md
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from datetime import UTC, datetime
from enum import Enum, auto
from typing import TYPE_CHECKING, Any

from moss.dwim import ToolMatch, analyze_intent, resolve_tool

if TYPE_CHECKING:
    from moss.moss_api import MossAPI

logger = logging.getLogger(__name__)


class LoopState(Enum):
    """State of the agent loop."""

    RUNNING = auto()
    DONE = auto()  # Agent signaled completion
    FAILED = auto()  # Unrecoverable error
    STALLED = auto()  # No progress
    MAX_TURNS = auto()  # Hit turn limit


@dataclass
class ParsedIntent:
    """Result of parsing a terse LLM intent."""

    verb: str  # Action: skeleton, expand, fix, validate, grep, done
    target: str | None  # File path, symbol name, or None
    content: str | None  # For "fix: ..." commands, the fix description
    raw: str  # Original input
    confidence: float = 1.0


@dataclass
class TurnResult:
    """Result of a single agent turn."""

    intent: ParsedIntent
    tool_match: ToolMatch | None
    tool_output: Any
    error: str | None = None
    duration_ms: int = 0


@dataclass
class LoopConfig:
    """Configuration for the DWIM loop."""

    max_turns: int = 50
    stall_threshold: int = 5  # Max turns without progress
    confidence_threshold: float = 0.3  # Below this, ask for clarification
    model: str = "gemini/gemini-2.5-flash-preview-05-20"
    temperature: float = 0.0
    system_prompt: str = ""


@dataclass
class LoopResult:
    """Result of running the DWIM loop."""

    state: LoopState
    turns: list[TurnResult]
    final_output: Any
    error: str | None = None
    total_duration_ms: int = 0


# Common action verbs and their canonical forms
ACTION_VERBS = {
    # Code exploration
    "skeleton": "skeleton",
    "skel": "skeleton",
    "structure": "skeleton",
    "outline": "skeleton",
    "expand": "expand",
    "show": "expand",
    "read": "expand",
    "view": "view",
    # Search
    "grep": "grep",
    "search": "search",
    "find": "find",
    "query": "query",
    # Code modification
    "fix": "fix",
    "patch": "patch",
    "edit": "edit",
    # Validation
    "validate": "validate",
    "check": "validate",
    "lint": "validate",
    # Dependencies
    "deps": "deps",
    "imports": "deps",
    "dependencies": "deps",
    # Call graph
    "callers": "callers",
    "callees": "callees",
    "calls": "callees",
    # Termination
    "done": "done",
    "finished": "done",
    "complete": "done",
}


def parse_intent(text: str) -> ParsedIntent:
    """Parse a terse LLM intent into structured form.

    Handles formats like:
    - "skeleton foo.py"
    - "expand Patch.apply"
    - "fix: add null check"
    - "grep 'def main' src/"
    - "done"

    Args:
        text: Raw LLM output

    Returns:
        ParsedIntent with verb, target, and optional content
    """
    text = text.strip()
    if not text:
        return ParsedIntent(verb="", target=None, content=None, raw=text, confidence=0.0)

    # Check for "fix: ..." format (verb with content)
    if ":" in text:
        parts = text.split(":", 1)
        verb_candidate = parts[0].strip().lower()
        if verb_candidate in ACTION_VERBS:
            return ParsedIntent(
                verb=ACTION_VERBS[verb_candidate],
                target=None,
                content=parts[1].strip(),
                raw=text,
            )

    # Split on whitespace
    parts = text.split(None, 1)
    first_word = parts[0].lower()

    # Check if first word is an action verb
    if first_word in ACTION_VERBS:
        verb = ACTION_VERBS[first_word]
        target = parts[1].strip() if len(parts) > 1 else None
        return ParsedIntent(verb=verb, target=target, content=None, raw=text)

    # Fallback: treat entire text as a natural language query
    # Let DWIM handle the routing
    return ParsedIntent(
        verb="query",
        target=None,
        content=text,
        raw=text,
        confidence=0.5,  # Lower confidence for unparsed input
    )


def build_tool_call(intent: ParsedIntent, api: MossAPI) -> tuple[str, dict[str, Any]]:
    """Build a tool call from parsed intent.

    Args:
        intent: Parsed intent from LLM
        api: MossAPI instance for context

    Returns:
        Tuple of (tool_name, parameters)
    """
    verb = intent.verb
    target = intent.target

    # Handle termination
    if verb == "done":
        return ("done", {})

    # Handle fix/edit (needs special treatment)
    if verb in ("fix", "patch", "edit"):
        return ("patch.apply", {"content": intent.content, "target": target})

    # Map verbs to tools
    verb_to_tool = {
        "skeleton": "skeleton.format",
        "expand": "skeleton.expand",
        "view": "view",
        "grep": "search.grep",
        "search": "search.find_symbols",
        "find": "search.find_definitions",
        "query": "query",
        "validate": "validation.validate",
        "deps": "dependencies.format",
        "callers": "callers",
        "callees": "callees",
    }

    tool_name = verb_to_tool.get(verb, verb)
    params: dict[str, Any] = {}

    if target:
        # Determine if target is a file path or symbol
        if "/" in target or target.endswith(".py"):
            params["path"] = target
        else:
            params["symbol"] = target

    return (tool_name, params)


class DWIMLoop:
    """DWIM-driven agent loop.

    The loop:
    1. Gets terse intent from LLM ("skeleton foo.py")
    2. Parses intent with parse_intent()
    3. Routes to tool with DWIM
    4. Executes tool and collects result
    5. Feeds result back to LLM
    6. Repeats until "done" or limit reached
    """

    def __init__(
        self,
        api: MossAPI,
        config: LoopConfig | None = None,
    ):
        self.api = api
        self.config = config or LoopConfig()
        self._turns: list[TurnResult] = []
        self._context: list[dict[str, str]] = []

    def _build_system_prompt(self) -> str:
        """Build the system prompt for terse agent mode."""
        if self.config.system_prompt:
            return self.config.system_prompt

        return """You are a code assistant. Output terse commands, one per line.

Available commands:
- skeleton <file> - show file structure
- expand <symbol> - show full source of symbol
- grep <pattern> <path> - search for pattern
- deps <file> - show imports/dependencies
- callers <symbol> - who calls this
- callees <symbol> - what does this call
- validate - run linters/checks
- fix: <description> - describe the fix to make
- done - signal completion

Be terse. No prose. Just commands."""

    async def _get_llm_response(self, user_message: str) -> str:
        """Get next intent from LLM.

        Args:
            user_message: Result from previous tool or user input

        Returns:
            LLM's terse response
        """
        try:
            import litellm
        except ImportError as e:
            msg = "litellm required for DWIMLoop. Install with: pip install litellm"
            raise ImportError(msg) from e

        # Add user message to context
        self._context.append({"role": "user", "content": user_message})

        # Build messages
        messages = [{"role": "system", "content": self._build_system_prompt()}]
        messages.extend(self._context)

        # Call LLM
        response = await litellm.acompletion(
            model=self.config.model,
            messages=messages,
            temperature=self.config.temperature,
        )

        # Extract response
        assistant_msg = response.choices[0].message.content or ""

        # Add to context
        self._context.append({"role": "assistant", "content": assistant_msg})

        return assistant_msg.strip()

    async def _execute_tool(self, tool_name: str, params: dict[str, Any]) -> Any:
        """Execute a tool and return result.

        Args:
            tool_name: Name of tool to execute
            params: Tool parameters

        Returns:
            Tool output (string or structured data)
        """
        # Handle termination
        if tool_name == "done":
            return None

        # Route through MossAPI
        parts = tool_name.split(".")
        if len(parts) == 2:
            api_name, method_name = parts
            sub_api = getattr(self.api, api_name, None)
            if sub_api:
                method = getattr(sub_api, method_name, None)
                if method:
                    # Handle async methods
                    result = method(**params)
                    if hasattr(result, "__await__"):
                        result = await result
                    return result

        # Fallback: try DWIM routing
        matches = analyze_intent(tool_name)
        if matches and matches[0].confidence > self.config.confidence_threshold:
            best = matches[0]
            # Recursively try the matched tool
            return await self._execute_tool(best.tool, params)

        return f"Unknown tool: {tool_name}"

    async def run(self, task: str) -> LoopResult:
        """Run the DWIM loop on a task.

        Args:
            task: Initial task description

        Returns:
            LoopResult with final state and all turns
        """
        self._turns = []
        self._context = []
        start_time = datetime.now(UTC)

        # Initial prompt
        current_input = f"Task: {task}"

        try:
            for _turn_num in range(self.config.max_turns):
                turn_start = datetime.now(UTC)

                # Get LLM response
                llm_response = await self._get_llm_response(current_input)

                # Parse intent
                intent = parse_intent(llm_response)

                # Check for completion
                if intent.verb == "done":
                    duration = int((datetime.now(UTC) - turn_start).total_seconds() * 1000)
                    self._turns.append(
                        TurnResult(
                            intent=intent,
                            tool_match=None,
                            tool_output=None,
                            duration_ms=duration,
                        )
                    )
                    total_duration = int((datetime.now(UTC) - start_time).total_seconds() * 1000)
                    return LoopResult(
                        state=LoopState.DONE,
                        turns=self._turns,
                        final_output=self._turns[-2].tool_output if len(self._turns) > 1 else None,
                        total_duration_ms=total_duration,
                    )

                # Build and execute tool call
                tool_name, params = build_tool_call(intent, self.api)
                tool_match = resolve_tool(tool_name) if tool_name != "done" else None

                try:
                    output = await self._execute_tool(tool_name, params)
                    error = None
                except Exception as e:
                    output = None
                    error = str(e)

                # Record turn
                duration = int((datetime.now(UTC) - turn_start).total_seconds() * 1000)
                self._turns.append(
                    TurnResult(
                        intent=intent,
                        tool_match=tool_match,
                        tool_output=output,
                        error=error,
                        duration_ms=duration,
                    )
                )

                # Format result for next turn
                if error:
                    current_input = f"Error: {error}"
                elif output is None:
                    current_input = "(no output)"
                elif isinstance(output, str):
                    current_input = output[:2000]  # Truncate large outputs
                else:
                    current_input = str(output)[:2000]

            # Max turns reached
            total_duration = int((datetime.now(UTC) - start_time).total_seconds() * 1000)
            return LoopResult(
                state=LoopState.MAX_TURNS,
                turns=self._turns,
                final_output=self._turns[-1].tool_output if self._turns else None,
                error=f"Max turns ({self.config.max_turns}) reached",
                total_duration_ms=total_duration,
            )

        except Exception as e:
            total_duration = int((datetime.now(UTC) - start_time).total_seconds() * 1000)
            return LoopResult(
                state=LoopState.FAILED,
                turns=self._turns,
                final_output=None,
                error=str(e),
                total_duration_ms=total_duration,
            )


__all__ = [
    "DWIMLoop",
    "LoopConfig",
    "LoopResult",
    "LoopState",
    "ParsedIntent",
    "TurnResult",
    "parse_intent",
]
