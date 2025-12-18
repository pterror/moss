"""Tests for the roadmap visualization module."""

from pathlib import Path
from textwrap import dedent

import pytest

from moss.roadmap import (
    Phase,
    PhaseStatus,
    Roadmap,
    TaskItem,
    find_todo_md,
    format_plain,
    format_tui,
    parse_todo_md,
)

# =============================================================================
# TaskItem Tests
# =============================================================================


class TestTaskItem:
    def test_create_task(self):
        task = TaskItem(description="Test task", completed=False)
        assert task.description == "Test task"
        assert task.completed is False
        assert task.indent == 0

    def test_completed_task(self):
        task = TaskItem(description="Done task", completed=True)
        assert task.completed is True


# =============================================================================
# Phase Tests
# =============================================================================


class TestPhase:
    def test_create_phase(self):
        phase = Phase(id="1", title="Test Phase", status=PhaseStatus.IN_PROGRESS)
        assert phase.id == "1"
        assert phase.title == "Test Phase"
        assert phase.status == PhaseStatus.IN_PROGRESS

    def test_progress_empty(self):
        phase = Phase(id="1", title="Test", status=PhaseStatus.FUTURE)
        completed, total = phase.progress
        assert completed == 0
        assert total == 0

    def test_progress_with_tasks(self):
        phase = Phase(
            id="1",
            title="Test",
            status=PhaseStatus.IN_PROGRESS,
            tasks=[
                TaskItem(description="Task 1", completed=True),
                TaskItem(description="Task 2", completed=False),
                TaskItem(description="Task 3", completed=True),
            ],
        )
        completed, total = phase.progress
        assert completed == 2
        assert total == 3

    def test_progress_with_subphases(self):
        subphase = Phase(
            id="1a",
            title="Sub",
            status=PhaseStatus.COMPLETE,
            tasks=[
                TaskItem(description="Sub task", completed=True),
            ],
        )
        phase = Phase(
            id="1",
            title="Test",
            status=PhaseStatus.IN_PROGRESS,
            tasks=[
                TaskItem(description="Main task", completed=False),
            ],
            subphases=[subphase],
        )
        completed, total = phase.progress
        assert completed == 1  # 0 from main + 1 from subphase
        assert total == 2  # 1 from main + 1 from subphase

    def test_progress_percent(self):
        phase = Phase(
            id="1",
            title="Test",
            status=PhaseStatus.IN_PROGRESS,
            tasks=[
                TaskItem(description="Task 1", completed=True),
                TaskItem(description="Task 2", completed=True),
                TaskItem(description="Task 3", completed=False),
                TaskItem(description="Task 4", completed=False),
            ],
        )
        assert phase.progress_percent == 50.0

    def test_progress_percent_empty(self):
        phase = Phase(id="1", title="Test", status=PhaseStatus.FUTURE)
        assert phase.progress_percent == 100.0  # No tasks = complete


# =============================================================================
# parse_todo_md Tests
# =============================================================================


class TestParseTodoMd:
    def test_parse_simple_todo(self, tmp_path: Path):
        todo_content = dedent("""
            # Roadmap

            ## In Progress

            ### Phase 1: First Phase

            Description of first phase.

            - [x] Task one
            - [ ] Task two
            - [x] Task three

            ## Future Work

            ### Phase 2: Second Phase

            - [ ] Future task
        """).strip()

        todo_file = tmp_path / "TODO.md"
        todo_file.write_text(todo_content)

        roadmap = parse_todo_md(todo_file)

        assert len(roadmap.in_progress) == 1
        assert len(roadmap.future) == 1

        phase1 = roadmap.in_progress[0]
        assert phase1.id == "1"
        assert phase1.title == "First Phase"
        assert phase1.status == PhaseStatus.IN_PROGRESS
        assert len(phase1.tasks) == 3

        phase2 = roadmap.future[0]
        assert phase2.id == "2"
        assert phase2.title == "Second Phase"
        assert phase2.status == PhaseStatus.FUTURE

    def test_parse_completed_phase(self, tmp_path: Path):
        todo_content = dedent("""
            ## In Progress

            ### Phase 1: Done Phase ✅

            - [x] All done
        """).strip()

        todo_file = tmp_path / "TODO.md"
        todo_file.write_text(todo_content)

        roadmap = parse_todo_md(todo_file)

        assert len(roadmap.in_progress) == 1
        phase = roadmap.in_progress[0]
        assert phase.status == PhaseStatus.COMPLETE

    def test_parse_subphases(self, tmp_path: Path):
        todo_content = dedent("""
            ## In Progress

            ### Phase 1: Main Phase

            #### 1a: Sub Phase A ✅

            - [x] Sub task A

            #### 1b: Sub Phase B

            - [ ] Sub task B
        """).strip()

        todo_file = tmp_path / "TODO.md"
        todo_file.write_text(todo_content)

        roadmap = parse_todo_md(todo_file)

        phase = roadmap.in_progress[0]
        assert len(phase.subphases) == 2

        sub_a = phase.subphases[0]
        assert sub_a.id == "1a"
        assert sub_a.status == PhaseStatus.COMPLETE

        sub_b = phase.subphases[1]
        assert sub_b.id == "1b"


# =============================================================================
# format_plain Tests
# =============================================================================


class TestFormatPlain:
    def test_format_basic(self):
        # Phase with incomplete tasks shows in "In Progress"
        roadmap = Roadmap(
            in_progress=[
                Phase(
                    id="1",
                    title="Test Phase",
                    status=PhaseStatus.IN_PROGRESS,
                    tasks=[
                        TaskItem("Task 1", completed=True),
                        TaskItem("Task 2", completed=False),
                    ],
                )
            ]
        )

        output = format_plain(roadmap)

        assert "In Progress" in output
        assert "Phase 1" in output
        assert "Test Phase" in output
        assert "[1/2]" in output
        assert "50%" in output

    def test_format_completed_phase(self):
        # Phase with all tasks done shows in "Recently Completed"
        roadmap = Roadmap(
            in_progress=[
                Phase(
                    id="1",
                    title="Test Phase",
                    status=PhaseStatus.IN_PROGRESS,
                    tasks=[TaskItem("Task", completed=True)],
                )
            ]
        )

        output = format_plain(roadmap)

        assert "Recently Completed" in output
        assert "Phase 1" in output
        assert "Test Phase" in output

    def test_format_with_future(self):
        roadmap = Roadmap(
            in_progress=[],
            future=[
                Phase(id="2", title="Future", status=PhaseStatus.FUTURE),
                Phase(id="3", title="Later", status=PhaseStatus.FUTURE),
            ],
        )

        output = format_plain(roadmap)

        assert "Next Up" in output
        assert "Phase 2" in output
        assert "Future" in output


# =============================================================================
# format_tui Tests
# =============================================================================


class TestFormatTui:
    def test_format_has_box_chars(self):
        roadmap = Roadmap(
            in_progress=[
                Phase(
                    id="1",
                    title="Test",
                    status=PhaseStatus.IN_PROGRESS,
                    tasks=[TaskItem("Task", completed=True)],
                )
            ]
        )

        output = format_tui(roadmap, use_color=False)

        # Check for box drawing characters
        assert "┌" in output
        assert "┐" in output
        assert "└" in output
        assert "┘" in output
        assert "│" in output

    def test_format_has_progress_bar(self):
        roadmap = Roadmap(
            in_progress=[
                Phase(
                    id="1",
                    title="Test",
                    status=PhaseStatus.IN_PROGRESS,
                    tasks=[
                        TaskItem("Task 1", completed=True),
                        TaskItem("Task 2", completed=False),
                    ],
                )
            ]
        )

        output = format_tui(roadmap, use_color=False)

        # Check for progress bar characters
        assert "█" in output or "░" in output

    def test_format_respects_width(self):
        roadmap = Roadmap(in_progress=[Phase(id="1", title="Test", status=PhaseStatus.IN_PROGRESS)])

        output = format_tui(roadmap, width=60, use_color=False)

        lines = output.split("\n")
        for line in lines:
            # All lines should fit within the width
            assert len(line) <= 60


# =============================================================================
# find_todo_md Tests
# =============================================================================


class TestFindTodoMd:
    def test_find_in_current_dir(self, tmp_path: Path):
        todo_file = tmp_path / "TODO.md"
        todo_file.write_text("# TODO")

        found = find_todo_md(tmp_path)
        assert found == todo_file

    def test_find_in_parent_dir(self, tmp_path: Path):
        todo_file = tmp_path / "TODO.md"
        todo_file.write_text("# TODO")

        subdir = tmp_path / "subdir"
        subdir.mkdir()

        found = find_todo_md(subdir)
        assert found == todo_file

    def test_not_found(self, tmp_path: Path):
        found = find_todo_md(tmp_path)
        assert found is None


# =============================================================================
# Integration Tests
# =============================================================================


class TestRoadmapIntegration:
    def test_real_todo_md(self):
        """Test with the actual project TODO.md if it exists."""
        todo_path = find_todo_md(Path(__file__).parent.parent)

        if todo_path is None:
            pytest.skip("TODO.md not found in project")

        roadmap = parse_todo_md(todo_path)

        # Should have some phases
        assert len(roadmap.in_progress) > 0 or len(roadmap.future) > 0

        # Should be able to format
        plain_output = format_plain(roadmap)
        assert len(plain_output) > 0

        tui_output = format_tui(roadmap, use_color=False)
        assert len(tui_output) > 0
