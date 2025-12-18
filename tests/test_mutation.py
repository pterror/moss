"""Tests for the mutation testing module."""

from pathlib import Path

from moss.mutation import (
    MutationAnalyzer,
    MutationResult,
    SurvivingMutant,
    create_mutation_analyzer,
)

# =============================================================================
# SurvivingMutant Tests
# =============================================================================


class TestSurvivingMutant:
    def test_create_mutant(self):
        mutant = SurvivingMutant(
            file=Path("test.py"),
            line=10,
            original="x > 0",
            mutated="x >= 0",
            mutation_type="comparison",
        )
        assert mutant.file == Path("test.py")
        assert mutant.line == 10
        assert mutant.original == "x > 0"
        assert mutant.mutated == "x >= 0"
        assert mutant.mutation_type == "comparison"

    def test_to_dict(self):
        mutant = SurvivingMutant(
            file=Path("test.py"),
            line=10,
            original="x + 1",
            mutated="x - 1",
            mutation_type="arithmetic",
            mutant_id=42,
        )
        d = mutant.to_dict()
        assert d["file"] == "test.py"
        assert d["line"] == 10
        assert d["original"] == "x + 1"
        assert d["mutated"] == "x - 1"
        assert d["mutation_type"] == "arithmetic"
        assert d["mutant_id"] == 42


# =============================================================================
# MutationResult Tests
# =============================================================================


class TestMutationResult:
    def test_empty_result(self):
        result = MutationResult()
        assert result.total_mutants == 0
        assert result.killed == 0
        assert result.survived == 0
        assert result.mutation_score == 1.0  # No mutants = perfect score
        assert not result.has_survivors

    def test_mutation_score_calculation(self):
        result = MutationResult(
            total_mutants=100,
            killed=80,
            survived=15,
            timeout=5,
            skipped=0,
        )
        assert result.mutation_score == 0.8  # 80/100

    def test_mutation_score_with_skipped(self):
        result = MutationResult(
            total_mutants=100,
            killed=72,
            survived=18,
            timeout=0,
            skipped=10,  # 10 skipped, so only 90 testable
        )
        assert result.mutation_score == 0.8  # 72/90

    def test_has_survivors(self):
        result = MutationResult(survived=3)
        assert result.has_survivors

    def test_to_dict(self):
        result = MutationResult(
            total_mutants=50,
            killed=40,
            survived=10,
            execution_time_seconds=120.5,
            survivors=[
                SurvivingMutant(
                    file=Path("a.py"),
                    line=1,
                    original="x",
                    mutated="y",
                    mutation_type="other",
                )
            ],
            tested_files=[Path("a.py"), Path("b.py")],
        )
        d = result.to_dict()
        assert d["stats"]["total_mutants"] == 50
        assert d["stats"]["killed"] == 40
        assert d["stats"]["survived"] == 10
        assert d["stats"]["mutation_score"] == 0.8
        assert len(d["survivors"]) == 1
        assert len(d["tested_files"]) == 2

    def test_to_markdown_no_survivors(self):
        result = MutationResult(
            total_mutants=10,
            killed=10,
            survived=0,
        )
        md = result.to_markdown()
        assert "100%" in md
        assert "All mutants were killed" in md

    def test_to_markdown_with_survivors(self):
        result = MutationResult(
            total_mutants=10,
            killed=8,
            survived=2,
            survivors=[
                SurvivingMutant(
                    file=Path("test.py"),
                    line=5,
                    original="a > b",
                    mutated="a >= b",
                    mutation_type="comparison",
                ),
                SurvivingMutant(
                    file=Path("test.py"),
                    line=10,
                    original="return True",
                    mutated="return False",
                    mutation_type="boolean",
                ),
            ],
        )
        md = result.to_markdown()
        assert "80%" in md
        assert "Surviving Mutants" in md
        assert "test.py" in md
        assert "Line 5" in md
        assert "comparison" in md

    def test_format_time_seconds(self):
        assert MutationResult._format_time(45.3) == "45.3s"

    def test_format_time_minutes(self):
        assert MutationResult._format_time(125.5) == "2m 6s"


# =============================================================================
# MutationAnalyzer Tests
# =============================================================================


class TestMutationAnalyzer:
    def test_create_analyzer(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer.root == tmp_path

    def test_is_available_when_not_installed(self, tmp_path: Path, monkeypatch):
        # Mock shutil.which to return None
        monkeypatch.setattr("shutil.which", lambda x: None)
        analyzer = MutationAnalyzer(tmp_path)
        assert not analyzer.is_available()

    def test_auto_detect_paths_src(self, tmp_path: Path):
        # Create src directory
        (tmp_path / "src").mkdir()
        analyzer = MutationAnalyzer(tmp_path)
        paths = analyzer._auto_detect_paths()
        assert tmp_path / "src" in paths

    def test_auto_detect_paths_lib(self, tmp_path: Path):
        # Create lib directory
        (tmp_path / "lib").mkdir()
        analyzer = MutationAnalyzer(tmp_path)
        paths = analyzer._auto_detect_paths()
        assert tmp_path / "lib" in paths

    def test_auto_detect_paths_fallback(self, tmp_path: Path):
        # No src or lib - should fall back to root
        analyzer = MutationAnalyzer(tmp_path)
        paths = analyzer._auto_detect_paths()
        assert tmp_path in paths

    def test_classify_mutation_boolean(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("return True", "return False") == "boolean"
        assert analyzer._classify_mutation("x = False", "x = True") == "boolean"

    def test_classify_mutation_comparison(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("x > 0", "x >= 0") == "comparison"
        assert analyzer._classify_mutation("a == b", "a != b") == "comparison"
        assert analyzer._classify_mutation("x < y", "x <= y") == "comparison"

    def test_classify_mutation_arithmetic(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("x + 1", "x - 1") == "arithmetic"
        assert analyzer._classify_mutation("a * b", "a / b") == "arithmetic"

    def test_classify_mutation_return(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("return x", "return None") == "return"

    def test_classify_mutation_negation(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("not x", "x") == "negation"

    def test_classify_mutation_logical(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("a and b", "a or b") == "logical"

    def test_classify_mutation_other(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        assert analyzer._classify_mutation("foo", "bar") == "other"

    def test_parse_results_output(self, tmp_path: Path):
        analyzer = MutationAnalyzer(tmp_path)
        output = """
Killed: 45
Survived: 3
Timeout: 2
Suspicious: 0
Skipped: 5
"""
        result = analyzer._parse_results_output(output)
        assert result.killed == 45
        assert result.survived == 3
        assert result.timeout == 2
        assert result.suspicious == 0
        assert result.skipped == 5
        assert result.total_mutants == 55


# =============================================================================
# Factory Function Tests
# =============================================================================


class TestCreateMutationAnalyzer:
    def test_create_with_default_root(self, monkeypatch, tmp_path: Path):
        monkeypatch.chdir(tmp_path)
        analyzer = create_mutation_analyzer()
        assert analyzer.root == tmp_path

    def test_create_with_explicit_root(self, tmp_path: Path):
        analyzer = create_mutation_analyzer(root=tmp_path)
        assert analyzer.root == tmp_path

    def test_create_with_custom_paths(self, tmp_path: Path):
        paths = [tmp_path / "custom"]
        analyzer = create_mutation_analyzer(root=tmp_path, paths_to_mutate=paths)
        assert analyzer.paths_to_mutate == paths
