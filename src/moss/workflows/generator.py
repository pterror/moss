"""Workflow generator.

Generates workflows based on detected architectural patterns and project configuration.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from moss.patterns import PatternAnalyzer, PatternInstance
from moss.workflows import Workflow, WorkflowLimits, WorkflowStep

logger = logging.getLogger(__name__)


@dataclass
class DetectedCapability:
    """A detected project capability."""

    name: str  # e.g., "python", "rust", "pytest", "ruff"
    confidence: float
    details: dict[str, Any]


class WorkflowGenerator:
    """Generates workflows from project analysis."""

    def __init__(self, project_root: Path):
        self.root = project_root
        self.analyzer = PatternAnalyzer(project_root)

    def generate(self) -> list[Workflow]:
        """Generate workflows for the project."""
        workflows: list[Workflow] = []

        # 1. Analyze architecture
        analysis = self.analyzer.analyze()

        # 2. Detect capabilities (tools, frameworks)
        capabilities = self._detect_capabilities()

        # 3. Generate standard workflows (validate, fix)
        workflows.extend(self._generate_standard_workflows(capabilities))

        # 4. Generate pattern-specific workflows
        workflows.extend(self._generate_pattern_workflows(analysis.patterns))

        return workflows

    def _detect_capabilities(self) -> list[DetectedCapability]:
        """Detect project capabilities."""
        caps = []

        # Python detection
        if list(self.root.glob("**/*.py")):
            caps.append(DetectedCapability("python", 1.0, {}))

        # Config file detection
        if (self.root / "pyproject.toml").exists():
            content = (self.root / "pyproject.toml").read_text()
            if "tool.ruff" in content:
                caps.append(DetectedCapability("ruff", 1.0, {}))
            if "tool.pytest" in content or "pytest" in content:
                caps.append(DetectedCapability("pytest", 1.0, {}))
            if "tool.mypy" in content:
                caps.append(DetectedCapability("mypy", 1.0, {}))

        return caps

    def _generate_standard_workflows(self, caps: list[DetectedCapability]) -> list[Workflow]:
        """Generate standard workflows based on capabilities."""
        workflows = []
        cap_names = {c.name for c in caps}

        # validate-fix (Python)
        if "python" in cap_names:
            steps = []

            # Linting
            if "ruff" in cap_names:
                steps.append(
                    WorkflowStep(
                        name="lint",
                        tool="ruff.check",
                        on_error="continue",  # Try to fix later
                    )
                )

            # Type checking
            if "mypy" in cap_names:
                steps.append(WorkflowStep(name="typecheck", tool="mypy.run", on_error="fail"))

            # Testing
            if "pytest" in cap_names:
                steps.append(WorkflowStep(name="test", tool="pytest.run", on_error="fail"))

            if steps:
                workflows.append(
                    Workflow(
                        name="validate",
                        description="Run project validation",
                        steps=steps,
                        limits=WorkflowLimits(max_steps=len(steps) + 2),
                    )
                )

        return workflows

    def _generate_pattern_workflows(self, patterns: list[PatternInstance]) -> list[Workflow]:
        """Generate workflows based on architectural patterns."""
        workflows = []

        # Group patterns by type
        plugin_systems = [p for p in patterns if p.pattern_type == "plugin"]

        for system in plugin_systems:
            # Generate a "create-plugin" workflow for this system
            wf = Workflow(
                name=f"create-{system.name.lower()}-plugin",
                description=f"Scaffold a new plugin for {system.name}",
                steps=[
                    WorkflowStep(
                        name="analyze",
                        tool="skeleton.format",
                        parameters={"file_path": system.file_path},
                    ),
                    WorkflowStep(
                        name="scaffold",
                        type="llm",
                        tool="llm.generate",
                        input_from="analyze",
                        prompt=f"""
I need to create a new plugin for {system.name} (defined in {system.file_path}).
The protocol definition is provided above.

Create a new class that implements this protocol.
""",
                    ),
                ],
            )
            workflows.append(wf)

        return workflows
