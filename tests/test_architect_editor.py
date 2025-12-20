"""Tests for architect_editor module."""

from __future__ import annotations

import pytest

from moss.architect_editor import (
    ArchitectEditorLoop,
    EditPlan,
    EditResult,
    EditStatus,
    EditStep,
    EditType,
    LLMArchitect,
    LoopResult,
    StructuredEditor,
)


class TestEditStep:
    """Tests for EditStep dataclass."""

    def test_create_replace_step(self):
        step = EditStep(
            edit_type=EditType.REPLACE,
            target="my_function",
            new_content="def my_function(): pass",
            reason="Fix implementation",
        )
        assert step.edit_type == EditType.REPLACE
        assert step.target == "my_function"
        assert step.new_content == "def my_function(): pass"
        assert step.reason == "Fix implementation"

    def test_to_dict(self):
        step = EditStep(
            edit_type=EditType.INSERT_AFTER,
            target="class_name",
            new_content="    def new_method(self): pass",
            reason="Add method",
            depends_on=[0, 1],
        )
        d = step.to_dict()
        assert d["edit_type"] == "insert_after"
        assert d["target"] == "class_name"
        assert d["depends_on"] == [0, 1]

    def test_from_dict(self):
        d = {
            "edit_type": "delete",
            "target": "old_function",
            "reason": "Remove deprecated",
        }
        step = EditStep.from_dict(d)
        assert step.edit_type == EditType.DELETE
        assert step.target == "old_function"
        assert step.new_content is None


class TestEditPlan:
    """Tests for EditPlan dataclass."""

    def test_create_plan(self):
        plan = EditPlan(
            task="Fix type error",
            file_path="src/main.py",
            approach="Add type check at function entry",
            steps=[
                EditStep(
                    edit_type=EditType.REPLACE,
                    target="process_data",
                    new_content="def process_data(x: int): ...",
                )
            ],
            context_needed=["process_data", "helper"],
            risks=["May break callers"],
            success_criteria="Type check passes",
        )
        assert plan.task == "Fix type error"
        assert len(plan.steps) == 1
        assert len(plan.context_needed) == 2
        assert len(plan.risks) == 1

    def test_serialization_roundtrip(self):
        plan = EditPlan(
            task="Add docstring",
            file_path="test.py",
            approach="Insert docstring after def",
            steps=[
                EditStep(
                    edit_type=EditType.INSERT_AFTER,
                    target="my_func",
                    new_content='    """Doc."""',
                ),
            ],
        )
        d = plan.to_dict()
        restored = EditPlan.from_dict(d)
        assert restored.task == plan.task
        assert restored.file_path == plan.file_path
        assert len(restored.steps) == 1
        assert restored.steps[0].edit_type == EditType.INSERT_AFTER


class TestLoopResult:
    """Tests for LoopResult dataclass."""

    def test_success_result(self):
        result = LoopResult(
            success=True,
            results=[
                EditResult(step_index=0, status=EditStatus.SUCCESS),
                EditResult(step_index=1, status=EditStatus.SUCCESS),
            ],
            total_tokens=100,
            iterations=1,
        )
        assert result.success
        assert "✓" in result.to_compact()
        assert "2/2" in result.to_compact()

    def test_failure_result(self):
        result = LoopResult(
            success=False,
            results=[
                EditResult(step_index=0, status=EditStatus.SUCCESS),
                EditResult(step_index=1, status=EditStatus.FAILED, error="Syntax error"),
            ],
            error="Validation failed",
            iterations=3,
        )
        assert not result.success
        assert "✗" in result.to_compact()
        assert "1/2" in result.to_compact()


class TestLLMArchitect:
    """Tests for LLMArchitect."""

    @pytest.mark.asyncio
    async def test_mock_plan(self):
        architect = LLMArchitect(mock=True)
        plan = await architect.plan(
            task="Fix bug",
            file_path="test.py",
            skeleton="def foo(): pass",
        )
        assert plan.task == "Fix bug"
        assert plan.file_path == "test.py"
        assert len(plan.steps) > 0
        assert "Mock" in plan.approach

    def test_parse_step_header(self):
        architect = LLMArchitect(mock=True)

        # Test basic parsing
        result = architect._parse_step_header("1. TYPE:replace TARGET:my_func REASON:fix it")
        assert result["type"] == "replace"
        assert result["target"] == "my_func"

        # Test with different order
        result = architect._parse_step_header("2. TARGET:other TYPE:insert_after")
        assert result["type"] == "insert_after"
        assert result["target"] == "other"

    def test_parse_plan_response(self):
        architect = LLMArchitect(mock=True)

        response = """APPROACH: Add null check at entry
CONTEXT_NEEDED: process, helper
RISKS: May slow down hot path
SUCCESS_CRITERIA: No more null pointer errors

STEPS:
1. TYPE:replace TARGET:process REASON:add null check
   CONTENT:
   def process(x):
       if x is None:
           return None
       return x * 2
"""
        plan = architect._parse_plan_response(response, "fix null", "main.py")
        assert plan.approach == "Add null check at entry"
        assert plan.context_needed == ["process", "helper"]
        assert plan.risks == ["May slow down hot path"]
        assert len(plan.steps) == 1
        assert plan.steps[0].edit_type == EditType.REPLACE
        assert plan.steps[0].target == "process"
        assert "if x is None" in (plan.steps[0].new_content or "")


class TestStructuredEditor:
    """Tests for StructuredEditor."""

    @pytest.mark.asyncio
    async def test_execute_empty_plan(self):
        editor = StructuredEditor()
        plan = EditPlan(
            task="test",
            file_path="test.py",
            approach="test",
            steps=[],
        )
        results = await editor.execute(plan)
        assert results == []

    @pytest.mark.asyncio
    async def test_skip_on_dependency_failure(self):
        editor = StructuredEditor()

        # Create a plan where step 1 depends on step 0
        plan = EditPlan(
            task="test",
            file_path="nonexistent.py",
            approach="test",
            steps=[
                EditStep(edit_type=EditType.REPLACE, target="foo", new_content="x"),
                EditStep(
                    edit_type=EditType.REPLACE,
                    target="bar",
                    new_content="y",
                    depends_on=[0],
                ),
            ],
        )

        results = await editor.execute(plan)
        # First step fails (file doesn't exist)
        assert results[0].status == EditStatus.FAILED
        # Second step should be skipped due to dependency
        assert results[1].status == EditStatus.SKIPPED


class TestArchitectEditorLoop:
    """Tests for ArchitectEditorLoop."""

    @pytest.mark.asyncio
    async def test_loop_with_mock(self):
        architect = LLMArchitect(mock=True)
        editor = StructuredEditor()
        loop = ArchitectEditorLoop(architect, editor, max_iterations=2)

        # This will fail because mock plan targets nonexistent file
        result = await loop.run("fix bug", "nonexistent.py")
        # Should fail gracefully
        assert isinstance(result, LoopResult)
        assert result.plan is not None or result.error is not None
