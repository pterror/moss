"""Tests for multi-backend custom analysis rules module."""

from pathlib import Path

from moss.rules import (
    CodeContext,
    Location,
    RuleContext,
    RuleEngine,
    RuleResult,
    Severity,
    Violation,
    create_engine_with_builtins,
    detect_context,
    get_builtin_rules,
    load_rules_from_config,
    load_rules_from_toml,
    pattern_rule,
    rule,
)


class TestViolation:
    """Tests for Violation dataclass."""

    def test_basic_violation(self):
        location = Location(file_path=Path("test.py"), line=10, column=5)
        violation = Violation(
            rule_name="test-rule",
            message="Test message",
            location=location,
            severity=Severity.WARNING,
        )

        assert violation.location.line == 10
        assert violation.location.column == 5
        assert violation.severity == Severity.WARNING

    def test_to_dict(self):
        location = Location(file_path=Path("test.py"), line=5, column=3)
        violation = Violation(
            rule_name="test-rule",
            message="Test message",
            location=location,
            severity=Severity.ERROR,
            category="security",
        )

        d = violation.to_dict()

        assert d["rule"] == "test-rule"
        assert d["line"] == 5
        assert d["severity"] == "error"
        assert d["category"] == "security"


class TestRuleResult:
    """Tests for RuleResult dataclass."""

    def test_empty_result(self):
        result = RuleResult()

        assert result.violations == []
        assert result.files_checked == 0
        assert result.rules_applied == 0

    def test_by_severity(self):
        result = RuleResult(
            violations=[
                Violation(
                    rule_name="err",
                    message="m",
                    location=Location(Path("a.py"), 1, 1),
                    severity=Severity.ERROR,
                ),
                Violation(
                    rule_name="err",
                    message="m",
                    location=Location(Path("b.py"), 2, 1),
                    severity=Severity.ERROR,
                ),
                Violation(
                    rule_name="warn",
                    message="m",
                    location=Location(Path("c.py"), 3, 1),
                    severity=Severity.WARNING,
                ),
            ]
        )

        assert len(result.by_severity(Severity.ERROR)) == 2
        assert len(result.by_severity(Severity.WARNING)) == 1
        assert len(result.by_severity(Severity.INFO)) == 0

    def test_by_rule(self):
        result = RuleResult(
            violations=[
                Violation(
                    rule_name="rule1",
                    message="m",
                    location=Location(Path("a.py"), 1, 1),
                ),
                Violation(
                    rule_name="rule2",
                    message="m",
                    location=Location(Path("a.py"), 2, 1),
                ),
            ]
        )

        assert len(result.by_rule("rule1")) == 1
        assert len(result.by_rule("rule2")) == 1

    def test_count_properties(self):
        result = RuleResult(
            violations=[
                Violation(
                    rule_name="e",
                    message="m",
                    location=Location(Path("a.py"), 1, 1),
                    severity=Severity.ERROR,
                ),
                Violation(
                    rule_name="w",
                    message="m",
                    location=Location(Path("b.py"), 2, 1),
                    severity=Severity.WARNING,
                ),
                Violation(
                    rule_name="i",
                    message="m",
                    location=Location(Path("c.py"), 3, 1),
                    severity=Severity.INFO,
                ),
            ]
        )

        assert result.error_count == 1
        assert result.warning_count == 1
        assert result.info_count == 1

    def test_to_dict(self):
        result = RuleResult(
            violations=[
                Violation(
                    rule_name="test",
                    message="Test",
                    location=Location(Path("test.py"), 5, 3),
                    severity=Severity.WARNING,
                ),
            ],
            files_checked=10,
            rules_applied=3,
        )

        d = result.to_dict()

        assert d["files_checked"] == 10
        assert d["rules_applied"] == 3
        assert d["total_violations"] == 1
        assert d["by_severity"]["warning"] == 1
        assert len(d["violations"]) == 1


class TestRuleEngine:
    """Tests for RuleEngine class."""

    def test_empty_engine(self):
        engine = RuleEngine()
        assert engine.rules == {}

    def test_add_rule(self):
        engine = RuleEngine()
        spec = pattern_rule("test", r"\btest\b", "Test message")

        engine.add_rule(spec)

        assert len(engine.rules) == 1

    def test_remove_rule(self):
        spec = pattern_rule("test", r"\btest\b", "Test message")
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.remove_rule("test")

        assert result is True
        assert len(engine.rules) == 0

    def test_remove_nonexistent_rule(self):
        engine = RuleEngine()

        result = engine.remove_rule("nonexistent")

        assert result is False

    def test_check_file_basic(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("x = 1\nprint('hello')\ny = 2\n")

        spec = pattern_rule("no-print", r"\bprint\s*\(", "No print")
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.check_file(test_file)

        assert len(result.violations) == 1
        assert result.violations[0].location.line == 2

    def test_check_file_multiple_matches(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print(1)\nprint(2)\nprint(3)\n")

        spec = pattern_rule("no-print", r"\bprint\s*\(", "No print")
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.check_file(test_file)

        assert len(result.violations) == 3

    def test_check_file_disabled_rule(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print('hello')\n")

        spec = pattern_rule("no-print", r"\bprint\s*\(", "No print")
        spec.enabled = False
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.check_file(test_file)

        assert len(result.violations) == 0

    def test_check_file_nonexistent(self, tmp_path: Path):
        spec = pattern_rule("test", r"\btest\b", "Test")
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.check_file(tmp_path / "nonexistent.py")

        # Should handle gracefully
        assert result.files_checked == 1

    def test_check_directory(self, tmp_path: Path):
        (tmp_path / "a.py").write_text("print(1)\n")
        (tmp_path / "b.py").write_text("print(2)\nprint(3)\n")
        (tmp_path / "c.txt").write_text("print(4)\n")  # Not .py

        spec = pattern_rule("no-print", r"\bprint\s*\(", "No print")
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.check_directory(tmp_path)

        assert result.files_checked == 2
        assert len(result.violations) == 3

    def test_check_directory_with_subdirs(self, tmp_path: Path):
        sub = tmp_path / "sub"
        sub.mkdir()
        (tmp_path / "a.py").write_text("print(1)\n")
        (sub / "b.py").write_text("print(2)\n")

        spec = pattern_rule("no-print", r"\bprint\s*\(", "No print")
        engine = RuleEngine()
        engine.add_rule(spec)

        result = engine.check_directory(tmp_path)

        assert result.files_checked == 2
        assert len(result.violations) == 2


class TestBuiltinRules:
    """Tests for built-in rules."""

    def test_builtin_rules_exist(self):
        builtins = get_builtin_rules()
        assert len(builtins) > 0

    def test_no_print_rule(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print('hello')\n")

        engine = create_engine_with_builtins()
        result = engine.check_file(test_file)

        print_violations = [v for v in result.violations if v.rule_name == "no-print"]
        assert len(print_violations) == 1

    def test_no_breakpoint_rule(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("breakpoint()\n")

        engine = create_engine_with_builtins()
        result = engine.check_file(test_file)

        bp_violations = [v for v in result.violations if v.rule_name == "no-breakpoint"]
        assert len(bp_violations) == 1


class TestLoadRulesFromToml:
    """Tests for loading rules from TOML."""

    def test_load_simple_rules(self, tmp_path: Path):
        rules_file = tmp_path / "rules.toml"
        rules_file.write_text("""
[[rules]]
name = "no-debug"
pattern = "import pdb"
message = "Remove debug imports"
severity = "warning"

[[rules]]
name = "no-star"
pattern = "from .* import \\\\*"
message = "Avoid star imports"
severity = "info"
category = "style"
""")

        rules = load_rules_from_toml(rules_file)

        assert len(rules) == 2
        assert rules[0].name == "no-debug"
        assert rules[0].severity == Severity.WARNING
        assert rules[1].name == "no-star"
        assert rules[1].category == "style"

    def test_load_rules_with_backend(self, tmp_path: Path):
        rules_file = tmp_path / "rules.toml"
        rules_file.write_text("""
[[rules]]
name = "ast-rule"
pattern = "print($ARGS)"
message = "Use ast-grep pattern"
backend = "ast-grep"
""")

        rules = load_rules_from_toml(rules_file)

        assert len(rules) == 1
        assert rules[0].name == "ast-rule"
        assert "ast-grep" in rules[0].backends


class TestLoadRulesFromConfig:
    """Tests for loading rules from project config."""

    def test_load_from_moss_toml(self, tmp_path: Path):
        (tmp_path / "moss.toml").write_text("""
[[rules]]
name = "custom-rule"
pattern = "test"
message = "Test message"
""")

        rules = load_rules_from_config(tmp_path)

        assert len(rules) == 1
        assert rules[0].name == "custom-rule"

    def test_load_from_dot_moss_rules(self, tmp_path: Path):
        moss_dir = tmp_path / ".moss"
        moss_dir.mkdir()
        (moss_dir / "rules.toml").write_text("""
[[rules]]
name = "from-dot-moss"
pattern = "x"
message = "msg"
""")

        rules = load_rules_from_config(tmp_path)

        assert len(rules) == 1
        assert rules[0].name == "from-dot-moss"

    def test_load_from_pyproject(self, tmp_path: Path):
        (tmp_path / "pyproject.toml").write_text("""
[tool.moss]
[[tool.moss.rules]]
name = "from-pyproject"
pattern = "y"
message = "msg"
""")

        rules = load_rules_from_config(tmp_path)

        assert len(rules) == 1
        assert rules[0].name == "from-pyproject"


class TestCreateEngineWithBuiltins:
    """Tests for create_engine_with_builtins."""

    def test_with_builtins(self):
        engine = create_engine_with_builtins(include_builtins=True)
        assert len(engine.rules) >= len(get_builtin_rules())

    def test_without_builtins(self):
        engine = create_engine_with_builtins(include_builtins=False)
        assert len(engine.rules) == 0

    def test_with_custom_rules(self):
        custom = [pattern_rule("custom", "x", "m")]
        engine = create_engine_with_builtins(include_builtins=False, custom_rules=custom)

        assert len(engine.rules) == 1
        assert "custom" in engine.rules


class TestContextDetection:
    """Tests for code context detection."""

    def test_detect_test_file_by_path(self, tmp_path: Path):
        test_file = tmp_path / "tests" / "test_foo.py"
        test_file.parent.mkdir(parents=True)
        test_file.write_text("def test_something(): pass\n")

        context = detect_context(test_file)

        assert context == CodeContext.TEST

    def test_detect_test_file_by_imports(self, tmp_path: Path):
        test_file = tmp_path / "foo.py"
        test_file.write_text("import pytest\n\ndef test_x(): pass\n")

        context = detect_context(test_file)

        assert context == CodeContext.TEST

    def test_detect_library_code(self, tmp_path: Path):
        lib_file = tmp_path / "mymodule.py"
        lib_file.write_text("def helper(): return 42\n")

        context = detect_context(lib_file)

        assert context == CodeContext.LIBRARY


class TestRuleDecorator:
    """Tests for @rule decorator."""

    def test_rule_registration(self):
        @rule(backend="python")
        def my_test_rule(ctx: RuleContext) -> list[Violation]:
            """Test rule docstring."""
            return []

        assert my_test_rule.name == "my_test_rule"
        assert my_test_rule.description == "Test rule docstring."
        assert "python" in my_test_rule.backends

    def test_rule_with_options(self):
        @rule(
            backend="regex",
            name="custom-name",
            severity="error",
            category="security",
            context="not:test",
        )
        def another_rule(ctx: RuleContext) -> list[Violation]:
            """Another rule."""
            return []

        assert another_rule.name == "custom-name"
        assert another_rule.severity == Severity.ERROR
        assert another_rule.category == "security"
        assert another_rule.exclude_contexts == [CodeContext.TEST]
