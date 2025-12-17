"""Tests for configurable output verbosity module."""

import io


class TestVerbosity:
    """Tests for Verbosity enum."""

    def test_verbosity_levels(self):
        from moss.output import Verbosity

        assert Verbosity.QUIET == 0
        assert Verbosity.NORMAL == 1
        assert Verbosity.VERBOSE == 2
        assert Verbosity.DEBUG == 3

    def test_verbosity_ordering(self):
        from moss.output import Verbosity

        assert Verbosity.QUIET < Verbosity.NORMAL
        assert Verbosity.NORMAL < Verbosity.VERBOSE
        assert Verbosity.VERBOSE < Verbosity.DEBUG


class TestOutputStyle:
    """Tests for OutputStyle."""

    def test_default_style(self):
        from moss.output import OutputStyle

        style = OutputStyle()

        assert style.use_colors is True
        assert style.use_emoji is True
        assert style.indent_size == 2
        assert "reset" in style.colors
        assert "error" in style.emoji

    def test_custom_style(self):
        from moss.output import OutputStyle

        style = OutputStyle(use_colors=False, use_emoji=False, indent_size=4)

        assert style.use_colors is False
        assert style.use_emoji is False
        assert style.indent_size == 4


class TestTextFormatter:
    """Tests for TextFormatter."""

    def test_format_message_with_emoji(self):
        from moss.output import OutputStyle, TextFormatter

        formatter = TextFormatter()
        style = OutputStyle(use_colors=False, use_emoji=True)

        result = formatter.format_message("error", "Test error", style, False)
        assert "[X] Test error" in result

    def test_format_message_no_emoji(self):
        from moss.output import OutputStyle, TextFormatter

        formatter = TextFormatter()
        style = OutputStyle(use_colors=False, use_emoji=False)

        result = formatter.format_message("info", "Test info", style, False)
        assert result == "Test info"

    def test_format_dict(self):
        from moss.output import OutputStyle, TextFormatter

        formatter = TextFormatter()
        style = OutputStyle()

        data = {"key1": "value1", "key2": "value2"}
        result = formatter.format_data(data, style)

        assert "key1: value1" in result
        assert "key2: value2" in result

    def test_format_list(self):
        from moss.output import OutputStyle, TextFormatter

        formatter = TextFormatter()
        style = OutputStyle()

        data = ["item1", "item2"]
        result = formatter.format_data(data, style)

        assert "- item1" in result
        assert "- item2" in result


class TestJSONFormatter:
    """Tests for JSONFormatter."""

    def test_format_message(self):
        from moss.output import JSONFormatter, OutputStyle

        formatter = JSONFormatter()
        style = OutputStyle()

        result = formatter.format_message("info", "Test message", style, False)

        assert '"level": "info"' in result
        assert '"message": "Test message"' in result

    def test_format_data(self):
        import json

        from moss.output import JSONFormatter, OutputStyle

        formatter = JSONFormatter()
        style = OutputStyle()

        data = {"key": "value"}
        result = formatter.format_data(data, style)

        parsed = json.loads(result)
        assert parsed == data


class TestCompactFormatter:
    """Tests for CompactFormatter."""

    def test_format_message(self):
        from moss.output import CompactFormatter, OutputStyle

        formatter = CompactFormatter()
        style = OutputStyle()

        result = formatter.format_message("error", "Test", style, False)
        assert result == "[E] Test"

    def test_format_data(self):
        from moss.output import CompactFormatter, OutputStyle

        formatter = CompactFormatter()
        style = OutputStyle()

        data = {"a": 1}
        result = formatter.format_data(data, style)

        assert '"a":1' in result


class TestOutput:
    """Tests for Output class."""

    def test_create_output(self):
        from moss.output import Output, Verbosity

        output = Output(verbosity=Verbosity.VERBOSE)

        assert output.verbosity == Verbosity.VERBOSE

    def test_error_output(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        stderr = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout, stderr=stderr)
        output.style.use_colors = False

        output.error("Error message")

        assert "Error message" in stderr.getvalue()

    def test_info_output(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout)
        output.style.use_colors = False

        output.info("Info message")

        assert "Info message" in stdout.getvalue()

    def test_quiet_mode(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        stderr = io.StringIO()
        output = Output(verbosity=Verbosity.QUIET, stdout=stdout, stderr=stderr)
        output.style.use_colors = False

        output.info("Info message")  # Should not appear
        output.error("Error message")  # Should appear

        assert stdout.getvalue() == ""
        assert "Error message" in stderr.getvalue()

    def test_verbose_mode(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.VERBOSE, stdout=stdout)
        output.style.use_colors = False

        output.verbose("Verbose message")

        assert "Verbose message" in stdout.getvalue()

    def test_verbose_hidden_in_normal(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout)

        output.verbose("Verbose message")

        assert stdout.getvalue() == ""

    def test_debug_mode(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.DEBUG, stdout=stdout)
        output.style.use_colors = False

        output.debug("Debug message")

        assert "Debug message" in stdout.getvalue()

    def test_set_verbosity(self):
        from moss.output import Output, Verbosity

        output = Output()

        output.set_quiet()
        assert output.verbosity == Verbosity.QUIET

        output.set_verbose()
        assert output.verbosity == Verbosity.VERBOSE

        output.set_debug()
        assert output.verbosity == Verbosity.DEBUG

    def test_use_json(self):
        from moss.output import JSONFormatter, Output

        output = Output()
        output.use_json()

        assert isinstance(output.formatter, JSONFormatter)
        assert output.style.use_colors is False

    def test_indentation(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout)
        output.style.use_colors = False
        output.style.use_emoji = False

        output.info("Level 0")
        output.indent()
        output.info("Level 1")
        output.indent()
        output.info("Level 2")
        output.dedent()
        output.info("Level 1 again")

        lines = stdout.getvalue().strip().split("\n")
        assert "Level 0" in lines[0]
        assert lines[1].startswith("  ")  # Indented
        assert lines[2].startswith("    ")  # Double indented
        assert lines[3].startswith("  ")  # Back to single

    def test_header(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout)
        output.style.use_colors = False

        output.header("Test Header")

        result = stdout.getvalue()
        assert "Test Header" in result
        assert "-" in result

    def test_data_output(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout)

        output.data({"key": "value"})

        assert "key" in stdout.getvalue()

    def test_success_warning_step(self):
        from moss.output import Output, Verbosity

        stdout = io.StringIO()
        output = Output(verbosity=Verbosity.NORMAL, stdout=stdout)
        output.style.use_colors = False

        output.success("Success")
        output.warning("Warning")
        output.step("Step")

        result = stdout.getvalue()
        assert "Success" in result
        # warning goes to stderr by default in some configs


class TestGlobalOutput:
    """Tests for global output functions."""

    def test_get_output(self):
        from moss.output import get_output

        output = get_output()
        assert output is not None

    def test_configure_output(self):
        from moss.output import Verbosity, configure_output, get_output

        configure_output(verbosity=Verbosity.VERBOSE, no_color=True)

        output = get_output()
        assert output.verbosity == Verbosity.VERBOSE
        assert output.style.use_colors is False

    def test_convenience_functions(self):
        import moss.output as out
        from moss.output import Output, Verbosity, set_output

        stdout = io.StringIO()
        stderr = io.StringIO()
        output = Output(verbosity=Verbosity.DEBUG, stdout=stdout, stderr=stderr)
        output.style.use_colors = False
        set_output(output)

        out.info("Info")
        out.debug("Debug")
        out.error("Error")
        out.success("Success")

        assert "Info" in stdout.getvalue()
        assert "Debug" in stdout.getvalue()
        assert "Error" in stderr.getvalue()
