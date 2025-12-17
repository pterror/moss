"""Tests for watch mode."""

from pathlib import Path

from moss.watch_tests import RunResult, WatchRunner, WatchTestConfig


class TestRunResult:
    """Tests for RunResult."""

    def test_success_result(self):
        result = RunResult(
            success=True,
            exit_code=0,
            stdout="1 passed",
            stderr="",
            duration_ms=100.0,
        )

        assert result.success is True
        assert result.exit_code == 0

    def test_failure_result(self):
        result = RunResult(
            success=False,
            exit_code=1,
            stdout="1 failed",
            stderr="Error",
            duration_ms=200.0,
        )

        assert result.success is False
        assert result.exit_code == 1


class TestWatchTestConfig:
    """Tests for WatchTestConfig."""

    def test_default_config(self):
        config = WatchTestConfig()

        assert "pytest" in config.test_command[2]  # [python, -m, pytest, -v]
        assert config.debounce_ms == 500
        assert config.clear_screen is True
        assert config.run_on_start is True

    def test_custom_test_command(self):
        config = WatchTestConfig(
            test_command=["python", "-m", "unittest"],
        )

        assert "unittest" in config.test_command[2]

    def test_custom_debounce(self):
        config = WatchTestConfig(debounce_ms=1000)
        assert config.debounce_ms == 1000


class TestWatchRunner:
    """Tests for WatchRunner."""

    def test_create_watcher(self, tmp_path: Path):
        watcher = WatchRunner(tmp_path)

        assert watcher.path == tmp_path
        assert watcher._running is False

    def test_create_with_config(self, tmp_path: Path):
        config = WatchTestConfig(debounce_ms=1000)
        watcher = WatchRunner(tmp_path, config=config)

        assert watcher.config.debounce_ms == 1000

    def test_stop_sets_running_false(self, tmp_path: Path):
        watcher = WatchRunner(tmp_path)
        watcher._running = True

        watcher.stop()

        assert watcher._running is False

    def test_initial_counts_are_zero(self, tmp_path: Path):
        watcher = WatchRunner(tmp_path)

        assert watcher._run_count == 0
        assert watcher._pass_count == 0
        assert watcher._fail_count == 0
