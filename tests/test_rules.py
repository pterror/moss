"""Tests for custom analysis rules module."""

from pathlib import Path

from moss.rules import (
    BUILTIN_RULES,
    Rule,
    RuleEngine,
    RuleResult,
    Violation,
    create_engine_with_builtins,
    load_rules_from_config,
    load_rules_from_toml,
)


class TestRule:
    """Tests for Rule dataclass."""

    def test_default_values(self):
        rule = Rule(
            name="test-rule",
            pattern=r"\btest\b",
            message="Test message",
        )

        assert rule.name == "test-rule"
        assert rule.severity == "warning"
        assert rule.category == "custom"
        assert rule.file_pattern == "*.py"
        assert rule.enabled is True

    def test_custom_values(self):
        rule = Rule(
            name="custom-rule",
            pattern=r"\bfoo\b",
            message="Custom message",
            severity="error",
            category="security",
            file_pattern="*.txt",
            enabled=False,
        )

        assert rule.severity == "error"
        assert rule.category == "security"
        assert rule.file_pattern == "*.txt"
        assert rule.enabled is False


class TestViolation:
    """Tests for Violation dataclass."""

    def test_basic_violation(self):
        rule = Rule(name="test", pattern="x", message="msg")
        violation = Violation(
            rule=rule,
            file_path=Path("test.py"),
            line=10,
            column=5,
            match_text="xxx",
        )

        assert violation.line == 10
        assert violation.column == 5
        assert violation.match_text == "xxx"


class TestRuleResult:
    """Tests for RuleResult dataclass."""

    def test_empty_result(self):
        result = RuleResult()

        assert result.violations == []
        assert result.files_checked == 0
        assert result.rules_applied == 0

    def test_by_severity(self):
        error_rule = Rule(name="err", pattern="x", message="m", severity="error")
        warning_rule = Rule(name="warn", pattern="y", message="m", severity="warning")

        result = RuleResult(
            violations=[
                Violation(
                    rule=error_rule, file_path=Path("a.py"), line=1, column=1, match_text="x"
                ),
                Violation(
                    rule=error_rule, file_path=Path("b.py"), line=2, column=1, match_text="x"
                ),
                Violation(
                    rule=warning_rule, file_path=Path("c.py"), line=3, column=1, match_text="y"
                ),
            ]
        )

        assert len(result.by_severity("error")) == 2
        assert len(result.by_severity("warning")) == 1
        assert len(result.by_severity("info")) == 0

    def test_by_rule(self):
        rule1 = Rule(name="rule1", pattern="x", message="m")
        rule2 = Rule(name="rule2", pattern="y", message="m")

        result = RuleResult(
            violations=[
                Violation(rule=rule1, file_path=Path("a.py"), line=1, column=1, match_text="x"),
                Violation(rule=rule2, file_path=Path("a.py"), line=2, column=1, match_text="y"),
            ]
        )

        assert len(result.by_rule("rule1")) == 1
        assert len(result.by_rule("rule2")) == 1

    def test_to_dict(self):
        rule = Rule(name="test", pattern="x", message="Test", severity="warning")
        result = RuleResult(
            violations=[
                Violation(rule=rule, file_path=Path("test.py"), line=5, column=3, match_text="x"),
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
        assert d["violations"][0]["rule"] == "test"


class TestRuleEngine:
    """Tests for RuleEngine class."""

    def test_empty_engine(self):
        engine = RuleEngine()
        assert engine.rules == []

    def test_add_rule(self):
        engine = RuleEngine()
        rule = Rule(name="test", pattern="x", message="m")

        engine.add_rule(rule)

        assert len(engine.rules) == 1

    def test_remove_rule(self):
        rule = Rule(name="test", pattern="x", message="m")
        engine = RuleEngine([rule])

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

        rule = Rule(
            name="no-print",
            pattern=r"\bprint\s*\(",
            message="No print",
        )
        engine = RuleEngine([rule])

        violations = engine.check_file(test_file)

        assert len(violations) == 1
        assert violations[0].line == 2
        assert "print" in violations[0].match_text

    def test_check_file_multiple_matches(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print(1)\nprint(2)\nprint(3)\n")

        rule = Rule(name="no-print", pattern=r"\bprint\s*\(", message="No print")
        engine = RuleEngine([rule])

        violations = engine.check_file(test_file)

        assert len(violations) == 3

    def test_check_file_multiple_rules(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print(1)\nbreakpoint()\n")

        rules = [
            Rule(name="no-print", pattern=r"\bprint\s*\(", message="No print"),
            Rule(name="no-breakpoint", pattern=r"\bbreakpoint\s*\(", message="No breakpoint"),
        ]
        engine = RuleEngine(rules)

        violations = engine.check_file(test_file)

        assert len(violations) == 2

    def test_check_file_disabled_rule(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print('hello')\n")

        rule = Rule(name="no-print", pattern=r"\bprint\s*\(", message="No print", enabled=False)
        engine = RuleEngine([rule])

        violations = engine.check_file(test_file)

        assert len(violations) == 0

    def test_check_file_nonexistent(self, tmp_path: Path):
        rule = Rule(name="test", pattern="x", message="m")
        engine = RuleEngine([rule])

        violations = engine.check_file(tmp_path / "nonexistent.py")

        assert len(violations) == 0

    def test_check_file_includes_context(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("line1\nline2\nprint('x')\nline4\nline5\n")

        rule = Rule(name="no-print", pattern=r"\bprint\s*\(", message="No print")
        engine = RuleEngine([rule])

        violations = engine.check_file(test_file)

        assert len(violations) == 1
        assert "line2" in violations[0].context
        assert "print" in violations[0].context

    def test_check_directory(self, tmp_path: Path):
        (tmp_path / "a.py").write_text("print(1)\n")
        (tmp_path / "b.py").write_text("print(2)\nprint(3)\n")
        (tmp_path / "c.txt").write_text("print(4)\n")  # Not .py

        rule = Rule(name="no-print", pattern=r"\bprint\s*\(", message="No print")
        engine = RuleEngine([rule])

        result = engine.check_directory(tmp_path)

        assert result.files_checked == 2
        assert len(result.violations) == 3

    def test_check_directory_with_subdirs(self, tmp_path: Path):
        sub = tmp_path / "sub"
        sub.mkdir()
        (tmp_path / "a.py").write_text("print(1)\n")
        (sub / "b.py").write_text("print(2)\n")

        rule = Rule(name="no-print", pattern=r"\bprint\s*\(", message="No print")
        engine = RuleEngine([rule])

        result = engine.check_directory(tmp_path)

        assert result.files_checked == 2
        assert len(result.violations) == 2


class TestBuiltinRules:
    """Tests for built-in rules."""

    def test_builtin_rules_exist(self):
        assert len(BUILTIN_RULES) > 0

    def test_no_print_rule(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("print('hello')\n")

        engine = create_engine_with_builtins()
        violations = engine.check_file(test_file)

        print_violations = [v for v in violations if v.rule.name == "no-print"]
        assert len(print_violations) == 1

    def test_no_breakpoint_rule(self, tmp_path: Path):
        test_file = tmp_path / "test.py"
        test_file.write_text("breakpoint()\n")

        engine = create_engine_with_builtins()
        violations = engine.check_file(test_file)

        bp_violations = [v for v in violations if v.rule.name == "no-breakpoint"]
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
        assert rules[0].severity == "warning"
        assert rules[1].name == "no-star"
        assert rules[1].category == "style"

    def test_load_rules_with_all_fields(self, tmp_path: Path):
        rules_file = tmp_path / "rules.toml"
        rules_file.write_text("""
[[rules]]
name = "custom"
pattern = "foobar"
message = "Found foobar"
severity = "error"
category = "security"
file_pattern = "*.txt"
enabled = false
fix = "Remove foobar"
documentation = "See docs"
""")

        rules = load_rules_from_toml(rules_file)

        assert len(rules) == 1
        rule = rules[0]
        assert rule.severity == "error"
        assert rule.category == "security"
        assert rule.file_pattern == "*.txt"
        assert rule.enabled is False
        assert rule.fix == "Remove foobar"
        assert rule.documentation == "See docs"


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

    def test_load_from_multiple_sources(self, tmp_path: Path):
        (tmp_path / "moss.toml").write_text("""
[[rules]]
name = "rule1"
pattern = "a"
message = "m"
""")

        moss_dir = tmp_path / ".moss"
        moss_dir.mkdir()
        (moss_dir / "rules.toml").write_text("""
[[rules]]
name = "rule2"
pattern = "b"
message = "m"
""")

        rules = load_rules_from_config(tmp_path)

        assert len(rules) == 2


class TestCreateEngineWithBuiltins:
    """Tests for create_engine_with_builtins."""

    def test_with_builtins(self):
        engine = create_engine_with_builtins(include_builtins=True)
        assert len(engine.rules) >= len(BUILTIN_RULES)

    def test_without_builtins(self):
        engine = create_engine_with_builtins(include_builtins=False)
        assert len(engine.rules) == 0

    def test_with_custom_rules(self):
        custom = [Rule(name="custom", pattern="x", message="m")]
        engine = create_engine_with_builtins(include_builtins=False, custom_rules=custom)

        assert len(engine.rules) == 1
        assert engine.rules[0].name == "custom"

    def test_combined(self):
        custom = [Rule(name="custom", pattern="x", message="m")]
        engine = create_engine_with_builtins(include_builtins=True, custom_rules=custom)

        assert len(engine.rules) == len(BUILTIN_RULES) + 1
