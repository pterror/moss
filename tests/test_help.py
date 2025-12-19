"""Tests for the help system."""

import pytest

from moss.help import (
    CATEGORIES,
    COMMANDS,
    CommandExample,
    CommandHelp,
    format_category_list,
    format_command_help,
    get_all_commands,
    get_categories,
    get_command_help,
    get_mcp_tool_description,
)


class TestCommandHelp:
    """Tests for CommandHelp dataclass."""

    def test_basic_command(self):
        """Test creating a basic command."""
        cmd = CommandHelp(
            name="test",
            summary="Test command",
        )
        assert cmd.name == "test"
        assert cmd.summary == "Test command"
        assert cmd.examples == []
        assert cmd.see_also == []

    def test_command_with_examples(self):
        """Test command with examples."""
        cmd = CommandHelp(
            name="test",
            summary="Test command",
            examples=[
                CommandExample("moss test", "Run test"),
                CommandExample("moss test --verbose", "Verbose output"),
            ],
        )
        assert len(cmd.examples) == 2
        assert cmd.examples[0].command == "moss test"


class TestGetHelp:
    """Tests for help retrieval functions."""

    def test_get_command_help_exists(self):
        """Test getting help for existing command."""
        cmd = get_command_help("skeleton")
        assert cmd is not None
        assert cmd.name == "skeleton"
        assert "skeleton" in cmd.summary.lower() or "code" in cmd.summary.lower()

    def test_get_command_help_not_exists(self):
        """Test getting help for non-existent command."""
        cmd = get_command_help("nonexistent")
        assert cmd is None

    def test_get_all_commands(self):
        """Test getting all commands."""
        commands = get_all_commands()
        assert len(commands) > 20
        assert "skeleton" in commands
        assert "deps" in commands
        assert "health" in commands

    def test_get_categories(self):
        """Test getting categories."""
        categories = get_categories()
        assert "Structure Analysis" in categories
        assert "Code Quality" in categories
        assert "skeleton" in categories["Structure Analysis"]


class TestFormatting:
    """Tests for help formatting."""

    def test_format_command_help(self):
        """Test formatting command help."""
        cmd = CommandHelp(
            name="test",
            summary="Test command",
            description="Extended description.",
            examples=[CommandExample("moss test", "Run test")],
            see_also=["other"],
        )
        formatted = format_command_help(cmd)
        assert "moss test" in formatted
        assert "Test command" in formatted
        assert "Extended description" in formatted
        assert "Examples:" in formatted
        assert "moss test" in formatted
        assert "See also: other" in formatted

    def test_format_category_list(self):
        """Test formatting category list."""
        formatted = format_category_list()
        assert "moss" in formatted
        assert "Structure Analysis:" in formatted
        assert "skeleton" in formatted
        assert "help <command>" in formatted


class TestMCPIntegration:
    """Tests for MCP tool description enhancement."""

    def test_get_mcp_tool_description_known(self):
        """Test getting enhanced description for known API."""
        desc = get_mcp_tool_description("skeleton", "format")
        assert desc is not None
        # Should include example
        assert "skeleton" in desc.lower() or "Example" in desc

    def test_get_mcp_tool_description_unknown(self):
        """Test getting description for unknown API."""
        desc = get_mcp_tool_description("unknown", "method")
        # Should return empty string for unknown
        assert desc == ""


class TestCategoriesComplete:
    """Test that all commands are categorized."""

    def test_all_commands_in_categories(self):
        """Verify all defined commands appear in categories."""
        categorized = set()
        for cmds in CATEGORIES.values():
            categorized.update(cmds)

        # Commands should be in categories
        for cmd_name in COMMANDS:
            assert cmd_name in categorized, f"Command '{cmd_name}' not in any category"

    def test_all_category_commands_exist(self):
        """Verify all commands in categories are defined."""
        for category, cmds in CATEGORIES.items():
            for cmd_name in cmds:
                assert cmd_name in COMMANDS, (
                    f"Command '{cmd_name}' in category '{category}' not defined"
                )


class TestCommandExamples:
    """Test that commands have good examples."""

    @pytest.mark.parametrize("cmd_name", list(COMMANDS.keys()))
    def test_command_has_summary(self, cmd_name):
        """Every command should have a summary."""
        cmd = COMMANDS[cmd_name]
        assert cmd.summary, f"Command '{cmd_name}' missing summary"
        assert len(cmd.summary) > 5, f"Command '{cmd_name}' summary too short"

    def test_common_commands_have_examples(self):
        """Common commands should have at least one example."""
        common_commands = [
            "skeleton",
            "deps",
            "health",
            "complexity",
            "synthesize",
            "security",
        ]
        for cmd_name in common_commands:
            cmd = COMMANDS[cmd_name]
            assert len(cmd.examples) > 0, f"Command '{cmd_name}' has no examples"
