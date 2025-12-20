"""Vanilla Agent Loop: Minimal agent loop with LLM, Tools, and TaskTree.

This is a baseline agent loop implementation that uses TaskTree for state management
and terse intents (DWIM-style) for communication, avoiding extra features
like memory triggers or ephemeral caching.

Design principles:
- Minimal: LLM + Tools + TaskTree
- Terse: One command per response, no JSON overhead
- Baseline: Useful for testing and evaluation
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from datetime import UTC, datetime
from enum import Enum, auto
from typing import TYPE_CHECKING, Any

from moss.dwim_loop import build_tool_call, parse_intent
from moss.task_tree import TaskTree

if TYPE_CHECKING:
    from moss.agent_loop import LLMConfig, ToolExecutor
    from moss.moss_api import MossAPI

logger = logging.getLogger(__name__)


class VanillaLoopState(Enum):
    """State of the vanilla agent loop."""

    RUNNING = auto()
    DONE = auto()
    FAILED = auto()
    MAX_TURNS = auto()


@dataclass
class VanillaTurnResult:
    """Result of a single turn in the vanilla loop."""

    turn_number: int
    prompt: str
    response: str
    tool_name: str | None = None
    tool_params: dict[str, Any] | None = None
    tool_output: Any | None = None
    error: str | None = None
    duration_ms: int = 0


@dataclass
class VanillaLoopResult:
    """Result of running the vanilla agent loop."""

    state: VanillaLoopState
    turns: list[VanillaTurnResult]
    final_output: Any = None
    error: str | None = None
    total_duration_ms: int = 0


class VanillaAgentLoop:
    """Minimal agent loop implementation.

    This loop uses:
    - TaskTree for hierarchical task state
    - Terse Intents for communication (token efficient)
    - ToolExecutor (MossAPI) for execution
    """

    def __init__(
        self,
        api: MossAPI,
        llm_config: LLMConfig,
        executor: ToolExecutor,
        max_turns: int = 20,
    ):
        self.api = api
        self.llm_config = llm_config
        self.executor = executor
        self.max_turns = max_turns
        self._task_tree: TaskTree | None = None
        self._turns: list[VanillaTurnResult] = []

    def _build_system_prompt(self) -> str:
        """Build the system prompt for the vanilla loop."""
        return """You are a code assistant. Output ONE terse command per response.

Commands:
- skeleton <file> - show file structure
- expand <symbol> - show full source
- grep <pattern> <path> - search
- validation.validate <file> - run linters
- patch.apply <description> - describe fix
- done [summary] - task complete

IMPORTANT: Never output prose. No JSON. Just the command."""

    def _build_prompt(self, last_result: Any | None = None) -> str:
        """Build the prompt for the current turn."""
        parts = []

        # Task tree context
        if self._task_tree:
            parts.append(self._task_tree.format_context())

        # Last tool result
        if last_result is not None:
            parts.append(f"\nLast result:\n{last_result}")

        return "\n".join(parts) if parts else "(no context)"

    async def run(self, task: str) -> VanillaLoopResult:
        """Run the vanilla agent loop on a task."""
        import litellm

        from moss.agent_loop import LoopContext, LoopStep, StepType

        self._task_tree = TaskTree(task)
        self._turns = []
        start_time = datetime.now(UTC)
        last_result = None

        try:
            for turn_num in range(1, self.max_turns + 1):
                turn_start = datetime.now(UTC)

                # 1. Build prompt
                prompt = self._build_prompt(last_result)

                # 2. Get LLM response
                messages = [
                    {"role": "system", "content": self._build_system_prompt()},
                    {"role": "user", "content": prompt},
                ]

                response = await litellm.acompletion(
                    model=self.llm_config.model,
                    messages=messages,
                    temperature=self.llm_config.temperature,
                )

                response_text = (response.choices[0].message.content or "").strip()

                # 3. Parse intent
                intent = parse_intent(response_text)

                # 4. Handle completion
                if intent.verb == "done":
                    summary = intent.target or intent.content or "Task completed"
                    self._task_tree.complete(summary)

                    duration = int((datetime.now(UTC) - turn_start).total_seconds() * 1000)
                    self._turns.append(
                        VanillaTurnResult(
                            turn_number=turn_num,
                            prompt=prompt,
                            response=response_text,
                            tool_name="done",
                            duration_ms=duration,
                        )
                    )

                    total_duration = int((datetime.now(UTC) - start_time).total_seconds() * 1000)
                    return VanillaLoopResult(
                        state=VanillaLoopState.DONE,
                        turns=self._turns,
                        final_output=summary,
                        total_duration_ms=total_duration,
                    )

                # 5. Build and execute tool call
                tool_name, params = build_tool_call(intent, self.api)
                tool_output = None
                error = None

                try:
                    # Create a dummy step for the executor
                    step = LoopStep(
                        name=f"vanilla_turn_{turn_num}",
                        tool=tool_name,
                        parameters=params,
                        step_type=StepType.TOOL,
                    )
                    # Create context
                    context = LoopContext(input={"task": task})

                    # Execute
                    tool_output, _, _ = await self.executor.execute(tool_name, context, step)
                    last_result = tool_output
                except Exception as e:
                    error = str(e)
                    last_result = f"Error: {error}"

                # 6. Record turn
                duration = int((datetime.now(UTC) - turn_start).total_seconds() * 1000)
                self._turns.append(
                    VanillaTurnResult(
                        turn_number=turn_num,
                        prompt=prompt,
                        response=response_text,
                        tool_name=tool_name,
                        tool_params=params,
                        tool_output=tool_output,
                        error=error,
                        duration_ms=duration,
                    )
                )

            # Max turns reached
            total_duration = int((datetime.now(UTC) - start_time).total_seconds() * 1000)
            return VanillaLoopResult(
                state=VanillaLoopState.MAX_TURNS,
                turns=self._turns,
                error=f"Max turns ({self.max_turns}) reached",
                total_duration_ms=total_duration,
            )

        except Exception as e:
            total_duration = int((datetime.now(UTC) - start_time).total_seconds() * 1000)
            return VanillaLoopResult(
                state=VanillaLoopState.FAILED,
                turns=self._turns,
                error=str(e),
                total_duration_ms=total_duration,
            )


__all__ = [
    "VanillaAgentLoop",
    "VanillaLoopResult",
    "VanillaLoopState",
    "VanillaTurnResult",
]
