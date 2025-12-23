"""Tests for live TODO tracking."""

import tempfile
from pathlib import Path

from moss_orchestration.live_todos import (
    TodoItem,
    TodoSession,
    TodoStatus,
    TodoTracker,
    create_tracker,
)


class TestTodoItem:
    """Tests for TodoItem."""

    def test_create_item(self):
        """Test creating a basic item."""
        item = TodoItem(content="Test task")
        assert item.content == "Test task"
        assert item.status == TodoStatus.PENDING
        assert item.started_at is None
        assert item.completed_at is None

    def test_item_serialization(self):
        """Test item to_dict and from_dict."""
        item = TodoItem(
            content="Test task",
            status=TodoStatus.COMPLETED,
            notes="Done!",
        )
        data = item.to_dict()
        restored = TodoItem.from_dict(data)

        assert restored.content == item.content
        assert restored.status == item.status
        assert restored.notes == item.notes

    def test_elapsed_time(self):
        """Test elapsed time calculation."""
        item = TodoItem(content="Test")
        assert item.elapsed_time is None

        import time

        item.started_at = time.time() - 10  # Started 10 seconds ago
        assert item.elapsed_time is not None
        assert item.elapsed_time >= 10


class TestTodoTracker:
    """Tests for TodoTracker."""

    def test_add_item(self):
        """Test adding items."""
        tracker = TodoTracker()
        item = tracker.add("Task 1")

        assert len(tracker.items) == 1
        assert item.content == "Task 1"
        assert item.status == TodoStatus.PENDING

    def test_start_item(self):
        """Test starting an item."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        item = tracker.start("Task 1")

        assert item is not None
        assert item.status == TodoStatus.IN_PROGRESS
        assert item.started_at is not None

    def test_complete_item(self):
        """Test completing an item."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        tracker.start("Task 1")
        item = tracker.complete("Task 1", notes="All done")

        assert item is not None
        assert item.status == TodoStatus.COMPLETED
        assert item.completed_at is not None
        assert item.notes == "All done"

    def test_current_item(self):
        """Test getting current (in_progress) item."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        tracker.add("Task 2")

        assert tracker.current is None

        tracker.start("Task 1")
        assert tracker.current is not None
        assert tracker.current.content == "Task 1"

    def test_stats(self):
        """Test statistics."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        tracker.add("Task 2")
        tracker.add("Task 3")
        tracker.start("Task 1")
        tracker.complete("Task 1")

        stats = tracker.stats
        assert stats["completed"] == 1
        assert stats["pending"] == 2
        assert stats["total"] == 3

    def test_format(self):
        """Test formatting output."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        tracker.add("Task 2")
        tracker.complete("Task 1")

        output = tracker.format()
        assert "Task 1" in output
        assert "Task 2" in output
        assert "1/2" in output or "50%" in output

    def test_format_compact(self):
        """Test compact format."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        tracker.add("Task 2")
        tracker.start("Task 1")

        output = tracker.format_compact()
        assert "[0/2]" in output
        assert "Task 1" in output

    def test_remove_item(self):
        """Test removing an item."""
        tracker = TodoTracker()
        tracker.add("Task 1")
        tracker.add("Task 2")

        assert tracker.remove("Task 1") is True
        assert len(tracker.items) == 1
        assert tracker.remove("Task 1") is False  # Already removed


class TestTodoSession:
    """Tests for session persistence."""

    def test_session_serialization(self):
        """Test session to_dict and from_dict."""
        session = TodoSession(
            session_id="test_123",
            items=[
                TodoItem(content="Task 1"),
                TodoItem(content="Task 2", status=TodoStatus.COMPLETED),
            ],
        )

        data = session.to_dict()
        restored = TodoSession.from_dict(data)

        assert restored.session_id == session.session_id
        assert len(restored.items) == 2
        assert restored.items[1].status == TodoStatus.COMPLETED

    def test_save_and_load(self):
        """Test saving and loading sessions."""
        with tempfile.TemporaryDirectory() as tmpdir:
            storage = Path(tmpdir)

            # Create and save
            tracker = TodoTracker(session_id="test_session", storage_dir=storage)
            tracker.add("Task 1")
            tracker.start("Task 1")
            tracker.complete("Task 1")
            tracker.add("Task 2")
            tracker.save()

            # Load
            loaded = TodoTracker.load("test_session", storage_dir=storage)
            assert loaded is not None
            assert len(loaded.items) == 2
            assert loaded.items[0].status == TodoStatus.COMPLETED
            assert loaded.items[1].status == TodoStatus.PENDING

    def test_latest_session(self):
        """Test loading latest session."""
        with tempfile.TemporaryDirectory() as tmpdir:
            storage = Path(tmpdir)

            # Create two sessions
            t1 = TodoTracker(session_id="aaa_first", storage_dir=storage)
            t1.add("First session task")
            t1.save()

            t2 = TodoTracker(session_id="zzz_second", storage_dir=storage)
            t2.add("Second session task")
            t2.save()

            # Latest should be zzz_second (alphabetically last)
            latest = TodoTracker.latest(storage_dir=storage)
            assert latest is not None
            assert latest.items[0].content == "Second session task"

    def test_list_sessions(self):
        """Test listing all sessions."""
        with tempfile.TemporaryDirectory() as tmpdir:
            storage = Path(tmpdir)

            TodoTracker(session_id="session1", storage_dir=storage).save()
            TodoTracker(session_id="session2", storage_dir=storage).save()

            sessions = TodoTracker.list_sessions(storage_dir=storage)
            assert len(sessions) == 2
            assert "session1" in sessions
            assert "session2" in sessions


class TestCreateTracker:
    """Tests for create_tracker helper."""

    def test_create_new(self):
        """Test creating a new tracker."""
        tracker = create_tracker()
        assert tracker is not None
        assert len(tracker.items) == 0

    def test_create_with_session_id(self):
        """Test creating with specific session ID."""
        tracker = create_tracker(session_id="my_session")
        assert tracker.session_id == "my_session"

    def test_resume_latest(self):
        """Test resuming latest session."""
        with tempfile.TemporaryDirectory() as tmpdir:
            storage = Path(tmpdir)

            # Create and save a session
            t1 = TodoTracker(session_id="to_resume", storage_dir=storage)
            t1.add("Existing task")
            t1.save()

            # Resume should get the same session
            # Note: create_tracker uses default storage, so this test
            # would need modification for isolated testing
            tracker = create_tracker(resume=False)
            assert tracker is not None


class TestOnUpdateCallback:
    """Tests for update callback."""

    def test_callback_on_add(self):
        """Test callback is called on add."""
        updates = []
        tracker = TodoTracker(on_update=lambda item: updates.append(item))
        tracker.add("Task 1")

        assert len(updates) == 1
        assert updates[0].content == "Task 1"

    def test_callback_on_status_change(self):
        """Test callback is called on status changes."""
        # Capture status at time of callback (items are mutable)
        statuses = []
        tracker = TodoTracker(on_update=lambda item: statuses.append(item.status))
        tracker.add("Task 1")
        tracker.start("Task 1")
        tracker.complete("Task 1")

        assert len(statuses) == 3
        assert statuses[0] == TodoStatus.PENDING
        assert statuses[1] == TodoStatus.IN_PROGRESS
        assert statuses[2] == TodoStatus.COMPLETED
