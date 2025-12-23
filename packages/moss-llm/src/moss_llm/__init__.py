"""moss-llm: LLM adapters for moss packages.

Provides LLM-backed implementations of protocols defined in moss-context
and moss-orchestration, using litellm for provider abstraction.

Example:
    from moss_llm import LLMSummarizer, LLMDecider
    from moss_context import WorkingMemory

    memory = WorkingMemory(
        summarizer=LLMSummarizer(model="claude-3-haiku-20240307")
    )

Supported models (via litellm):
    - claude-3-haiku-20240307, claude-3-sonnet-20240229, claude-3-opus-20240229
    - gpt-4, gpt-4-turbo, gpt-3.5-turbo
    - And many more: https://docs.litellm.ai/docs/providers
"""

from dataclasses import dataclass
from typing import Any

import litellm


@dataclass
class LLMConfig:
    """Configuration for LLM calls."""

    model: str = "claude-3-haiku-20240307"
    temperature: float = 0.0
    max_tokens: int = 1024
    timeout: float = 30.0


class LLMSummarizer:
    """Implements Summarizer protocol using litellm.

    For use with moss_context.WorkingMemory.
    """

    def __init__(
        self,
        model: str = "claude-3-haiku-20240307",
        temperature: float = 0.0,
        max_tokens: int = 512,
    ):
        self.model = model
        self.temperature = temperature
        self.max_tokens = max_tokens

    def summarize(self, items: list[str]) -> str:
        """Summarize multiple items into one concise summary."""
        combined = "\n\n---\n\n".join(items)
        prompt = f"""Summarize the following items concisely, preserving key information:

{combined}

Summary:"""

        response = litellm.completion(
            model=self.model,
            messages=[{"role": "user", "content": prompt}],
            temperature=self.temperature,
            max_tokens=self.max_tokens,
        )
        return response.choices[0].message.content.strip()


class LLMTokenCounter:
    """Token counter using litellm's token counting.

    For use with moss_context.WorkingMemory.
    """

    def __init__(self, model: str = "claude-3-haiku-20240307"):
        self.model = model

    def count(self, text: str) -> int:
        """Count tokens in text."""
        try:
            return litellm.token_counter(model=self.model, text=text)
        except Exception:
            # Fallback to simple estimation
            return int(len(text.split()) * 1.3)


class LLMDecider:
    """Implements decision-making protocol using litellm.

    For use with moss_orchestration agents.
    """

    def __init__(
        self,
        model: str = "claude-sonnet-4-20250514",
        temperature: float = 0.0,
        max_tokens: int = 2048,
        system_prompt: str | None = None,
    ):
        self.model = model
        self.temperature = temperature
        self.max_tokens = max_tokens
        self.system_prompt = system_prompt or "You are a helpful coding assistant."

    async def decide(self, context: str, tools: list[dict] | None = None) -> dict[str, Any]:
        """Make a decision based on context.

        Args:
            context: Current context/state as string
            tools: Optional tool definitions

        Returns:
            Decision dict with 'action' and 'parameters'
        """
        messages = [
            {"role": "system", "content": self.system_prompt},
            {"role": "user", "content": context},
        ]

        kwargs: dict[str, Any] = {
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
        }

        if tools:
            kwargs["tools"] = tools

        response = await litellm.acompletion(**kwargs)
        message = response.choices[0].message

        # Handle tool calls if present
        if hasattr(message, "tool_calls") and message.tool_calls:
            tool_call = message.tool_calls[0]
            return {
                "action": tool_call.function.name,
                "parameters": tool_call.function.arguments,
                "raw": message.content,
            }

        # Otherwise return text response
        return {
            "action": "respond",
            "parameters": {"text": message.content},
            "raw": message.content,
        }

    def decide_sync(self, context: str, tools: list[dict] | None = None) -> dict[str, Any]:
        """Synchronous version of decide."""
        messages = [
            {"role": "system", "content": self.system_prompt},
            {"role": "user", "content": context},
        ]

        kwargs: dict[str, Any] = {
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
        }

        if tools:
            kwargs["tools"] = tools

        response = litellm.completion(**kwargs)
        message = response.choices[0].message

        if hasattr(message, "tool_calls") and message.tool_calls:
            tool_call = message.tool_calls[0]
            return {
                "action": tool_call.function.name,
                "parameters": tool_call.function.arguments,
                "raw": message.content,
            }

        return {
            "action": "respond",
            "parameters": {"text": message.content},
            "raw": message.content,
        }


class LLMCompletion:
    """Simple completion wrapper for general LLM use."""

    def __init__(
        self,
        model: str = "claude-3-haiku-20240307",
        temperature: float = 0.0,
        max_tokens: int = 1024,
    ):
        self.model = model
        self.temperature = temperature
        self.max_tokens = max_tokens

    def complete(self, prompt: str, system: str | None = None) -> str:
        """Get a completion for a prompt."""
        messages = []
        if system:
            messages.append({"role": "system", "content": system})
        messages.append({"role": "user", "content": prompt})

        response = litellm.completion(
            model=self.model,
            messages=messages,
            temperature=self.temperature,
            max_tokens=self.max_tokens,
        )
        return response.choices[0].message.content

    async def acomplete(self, prompt: str, system: str | None = None) -> str:
        """Async version of complete."""
        messages = []
        if system:
            messages.append({"role": "system", "content": system})
        messages.append({"role": "user", "content": prompt})

        response = await litellm.acompletion(
            model=self.model,
            messages=messages,
            temperature=self.temperature,
            max_tokens=self.max_tokens,
        )
        return response.choices[0].message.content


__all__ = [
    "LLMConfig",
    "LLMSummarizer",
    "LLMTokenCounter",
    "LLMDecider",
    "LLMCompletion",
]
