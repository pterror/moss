"""Tests for moss.execution module."""

from moss.execution import (
    Transition,
    WorkflowState,
    evaluate_condition,
    state_machine_loop,
)


class TestConditionPlugins:
    """Test condition plugin system."""

    def test_has_errors_true(self):
        assert evaluate_condition("has_errors", "", "Found 3 errors in code")

    def test_has_errors_false(self):
        assert not evaluate_condition("has_errors", "", "All checks passed")

    def test_success_true(self):
        assert evaluate_condition("success", "", "Operation completed")

    def test_success_false(self):
        assert not evaluate_condition("success", "", "[Error] Failed to parse")

    def test_empty_true(self):
        assert evaluate_condition("empty", "", "")
        assert evaluate_condition("empty", "", "   ")

    def test_empty_false(self):
        assert not evaluate_condition("empty", "", "content")

    def test_contains_with_param(self):
        assert evaluate_condition("contains:TypeError", "", "TypeError: foo")
        assert not evaluate_condition("contains:TypeError", "", "ValueError: bar")

    def test_contains_no_param(self):
        assert not evaluate_condition("contains", "", "anything")

    def test_unknown_condition(self):
        assert not evaluate_condition("nonexistent", "", "result")


class TestStateMachineLoop:
    """Test state machine execution."""

    def test_terminal_state_immediate(self):
        states = [
            WorkflowState(name="done", terminal=True),
        ]
        result = state_machine_loop(states, initial="done")
        assert "Terminal state reached" in result

    def test_simple_transition(self):
        states = [
            WorkflowState(
                name="start",
                transitions=[Transition(next="end")],
            ),
            WorkflowState(name="end", terminal=True),
        ]
        result = state_machine_loop(states, initial="start")
        assert "state: start" in result or "state: end" in result

    def test_conditional_transition(self):
        # Uses NoLLM by default, so action won't actually run
        # but we can test the transition logic
        states = [
            WorkflowState(
                name="check",
                action="view .",  # won't actually execute in test
                transitions=[
                    Transition(next="error_state", condition="has_errors"),
                    Transition(next="ok_state"),  # default
                ],
            ),
            WorkflowState(name="error_state", terminal=True),
            WorkflowState(name="ok_state", terminal=True),
        ]
        # Without actual execution, result is empty, so "has_errors" is False
        # Should go to ok_state
        result = state_machine_loop(states, initial="check")
        assert "ok_state" in result or "Terminal" in result

    def test_invalid_initial_state(self):
        states = [WorkflowState(name="start", terminal=True)]
        result = state_machine_loop(states, initial="nonexistent")
        assert "not found" in result

    def test_no_valid_transition(self):
        states = [
            WorkflowState(
                name="stuck",
                transitions=[
                    Transition(next="other", condition="has_errors"),
                    # No default transition!
                ],
            ),
            WorkflowState(name="other", terminal=True),
        ]
        result = state_machine_loop(states, initial="stuck")
        assert "No valid transition" in result

    def test_max_transitions_limit(self):
        # Infinite loop between two states
        states = [
            WorkflowState(
                name="a",
                transitions=[Transition(next="b")],
            ),
            WorkflowState(
                name="b",
                transitions=[Transition(next="a")],
            ),
        ]
        # Should stop after max_transitions
        result = state_machine_loop(states, initial="a", max_transitions=5)
        # Just verify it returns without hanging
        assert result is not None

    def test_initial_context(self):
        states = [WorkflowState(name="done", terminal=True)]
        result = state_machine_loop(
            states,
            initial="done",
            initial_context={"file": "main.py"},
        )
        assert "file: main.py" in result
