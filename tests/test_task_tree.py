"""Tests for TaskTree hierarchical task state."""

import pytest

from moss.task_tree import NoteExpiry, TaskNode, TaskStatus, TaskTree


class TestTaskNode:
    """Tests for TaskNode."""

    def test_create_node(self):
        node = TaskNode(goal="Fix bug")
        assert node.goal == "Fix bug"
        assert node.status == TaskStatus.PENDING
        assert node.is_leaf

    def test_add_child(self):
        parent = TaskNode(goal="Fix bug")
        child = parent.add_child("Find root cause")
        assert child.parent is parent
        assert child in parent.children
        assert not parent.is_leaf

    def test_path_to_root(self):
        root = TaskNode(goal="Task")
        child1 = root.add_child("Subtask")
        child2 = child1.add_child("Sub-subtask")

        path = child2.path_to_root
        assert len(path) == 3
        assert path[0] is root
        assert path[1] is child1
        assert path[2] is child2

    def test_mark_done(self):
        node = TaskNode(goal="Fix bug")
        node.mark_done("Fixed null check")
        assert node.status == TaskStatus.DONE
        assert node.summary == "Fixed null check"

    def test_note_expiry_on_done(self):
        node = TaskNode(goal="Fix bug")
        node.add_note("Important context", NoteExpiry.ON_DONE)
        assert len(node.notes) == 1

        node.mark_done("Fixed")
        assert len(node.notes) == 0  # Note expired

    def test_note_manual_persists(self):
        node = TaskNode(goal="Fix bug")
        node.add_note("Keep this", NoteExpiry.MANUAL)

        node.mark_done("Fixed")
        assert len(node.notes) == 1  # Manual note persists


class TestTaskTree:
    """Tests for TaskTree."""

    def test_create_tree(self):
        tree = TaskTree("Fix authentication bug")
        assert tree.root.goal == "Fix authentication bug"
        assert tree.current is tree.root
        assert tree.root.status == TaskStatus.ACTIVE

    def test_breakdown(self):
        tree = TaskTree("Fix bug")
        tree.breakdown(["Find cause", "Implement fix", "Test"])

        assert len(tree.root.children) == 3
        assert tree.current.goal == "Find cause"
        assert tree.current.status == TaskStatus.ACTIVE
        assert tree.root.children[1].status == TaskStatus.PENDING

    def test_complete_moves_to_sibling(self):
        tree = TaskTree("Fix bug")
        tree.breakdown(["Step 1", "Step 2"])

        tree.complete("Done step 1")
        assert tree.current.goal == "Step 2"
        assert tree.current.status == TaskStatus.ACTIVE

    def test_complete_all_completes_parent(self):
        tree = TaskTree("Fix bug")
        tree.breakdown(["Step 1", "Step 2"])

        tree.complete("Done 1")
        result = tree.complete("Done 2")

        assert result is None  # Root complete
        assert tree.root.status == TaskStatus.DONE

    def test_nested_breakdown(self):
        tree = TaskTree("Fix bug")
        tree.breakdown(["Find cause", "Fix"])

        # Now at "Find cause", break it down further
        tree.breakdown(["Check logs", "Reproduce"])

        assert tree.current.goal == "Check logs"
        path = tree.current.path_to_root
        assert len(path) == 3
        assert path[0].goal == "Fix bug"
        assert path[1].goal == "Find cause"
        assert path[2].goal == "Check logs"

    def test_format_context(self):
        tree = TaskTree("Fix auth bug")
        tree.breakdown(["Find failure", "Implement fix"])

        # Before completion - at "Find failure"
        ctx = tree.format_context()
        assert "Fix auth bug" in ctx
        assert "Find failure" in ctx
        assert "[now]" in ctx

        # After completion - at "Implement fix"
        # Note: completed siblings NOT in path (path-based model)
        tree.complete("Token expires during refresh")
        ctx = tree.format_context()
        assert "Fix auth bug" in ctx
        assert "Implement fix" in ctx
        assert "[now]" in ctx
        # "Find failure" is a sibling, not in path - use notes for such context

    def test_format_context_with_notes(self):
        tree = TaskTree("Fix bug")
        tree.add_note("Check callers first", NoteExpiry.ON_DONE)

        ctx = tree.format_context()
        assert "[note: Check callers first]" in ctx

    def test_serialization_roundtrip(self):
        tree = TaskTree("Fix bug")
        tree.breakdown(["Step 1", "Step 2"])
        tree.add_note("Remember this", NoteExpiry.MANUAL)
        tree.complete("Done 1")

        data = tree.to_dict()
        restored = TaskTree.from_dict(data)

        assert restored.root.goal == "Fix bug"
        assert restored.current.goal == "Step 2"
        assert len(restored.root.children) == 2
        assert restored.root.children[0].status == TaskStatus.DONE

    def test_breakdown_empty_raises(self):
        tree = TaskTree("Fix bug")
        with pytest.raises(ValueError, match="empty"):
            tree.breakdown([])


class TestNotes:
    """Tests for note functionality."""

    def test_turns_remaining(self):
        tree = TaskTree("Task")
        tree.add_note("Temporary", turns=2)

        assert tree.current.notes[0].turns_remaining == 2

        tree.tick_notes()
        assert tree.current.notes[0].turns_remaining == 1

        tree.tick_notes()
        assert len(tree.current.notes) == 0  # Expired
