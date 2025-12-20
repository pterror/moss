"""Tests for DWIM-driven agent loop."""

from moss.dwim_loop import (
    LoopConfig,
    parse_intent,
)


class TestParseIntent:
    """Tests for parse_intent function."""

    def test_simple_command(self):
        """Parse simple verb + target."""
        result = parse_intent("skeleton foo.py")
        assert result.verb == "skeleton"
        assert result.target == "foo.py"
        assert result.content is None
        assert result.confidence == 1.0

    def test_command_with_path(self):
        """Parse command with file path."""
        result = parse_intent("skeleton src/moss/dwim.py")
        assert result.verb == "skeleton"
        assert result.target == "src/moss/dwim.py"

    def test_expand_symbol(self):
        """Parse expand with symbol."""
        result = parse_intent("expand Patch.apply")
        assert result.verb == "expand"
        assert result.target == "Patch.apply"

    def test_fix_with_content(self):
        """Parse fix: format."""
        result = parse_intent("fix: add null check for anchor")
        assert result.verb == "fix"
        assert result.content == "add null check for anchor"
        assert result.target is None

    def test_done_signal(self):
        """Parse done signal."""
        result = parse_intent("done")
        assert result.verb == "done"
        assert result.target is None

    def test_done_alternatives(self):
        """Parse alternative done signals."""
        for word in ["done", "finished", "complete"]:
            result = parse_intent(word)
            assert result.verb == "done", f"'{word}' should map to 'done'"

    def test_verb_aliases(self):
        """Parse verb aliases."""
        result = parse_intent("skel foo.py")
        assert result.verb == "skeleton"

        result = parse_intent("show main")
        assert result.verb == "expand"

    def test_grep_with_pattern(self):
        """Parse grep with pattern and path."""
        result = parse_intent("grep 'def main' src/")
        assert result.verb == "grep"
        assert result.target == "'def main' src/"

    def test_validate_no_target(self):
        """Parse validate without target."""
        result = parse_intent("validate")
        assert result.verb == "validate"
        assert result.target is None

    def test_empty_input(self):
        """Parse empty input."""
        result = parse_intent("")
        assert result.verb == ""
        assert result.confidence == 0.0

    def test_whitespace_handling(self):
        """Handle leading/trailing whitespace."""
        result = parse_intent("  skeleton foo.py  ")
        assert result.verb == "skeleton"
        assert result.target == "foo.py"

    def test_natural_language_fallback(self):
        """Natural language falls back to query."""
        result = parse_intent("what functions are in this file")
        assert result.verb == "query"
        assert result.content == "what functions are in this file"
        assert result.confidence == 0.5


class TestLoopConfig:
    """Tests for LoopConfig."""

    def test_defaults(self):
        """Test default values."""
        config = LoopConfig()
        assert config.max_turns == 50
        assert config.stall_threshold == 5
        assert config.temperature == 0.0

    def test_custom_values(self):
        """Test custom values."""
        config = LoopConfig(max_turns=10, temperature=0.5)
        assert config.max_turns == 10
        assert config.temperature == 0.5
