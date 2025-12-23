"""Tests for Scope Analysis."""

from pathlib import Path

import pytest

from moss_intelligence.scopes import (
    FileScopeReport,
    ProjectScopeReport,
    ScopeStats,
    analyze_file_scopes,
    analyze_project_scopes,
    format_scope_report,
)


class TestScopeStats:
    """Tests for ScopeStats dataclass."""

    def test_total(self):
        stats = ScopeStats(public=5, private=3, dunder=2)
        assert stats.total == 10

    def test_public_ratio(self):
        stats = ScopeStats(public=8, private=2, dunder=0)
        assert stats.public_ratio == 0.8

    def test_public_ratio_empty(self):
        stats = ScopeStats()
        assert stats.public_ratio == 1.0


class TestFileScopeReport:
    """Tests for FileScopeReport."""

    def test_totals(self, tmp_path: Path):
        report = FileScopeReport(path=tmp_path / "test.py")
        report.classes = ScopeStats(public=2, private=1, dunder=0)
        report.functions = ScopeStats(public=5, private=2, dunder=0)
        report.methods = ScopeStats(public=10, private=3, dunder=5)

        assert report.total_public == 17
        assert report.total_private == 6
        assert report.total_dunder == 5
        assert report.total == 28

    def test_to_dict(self, tmp_path: Path):
        report = FileScopeReport(path=tmp_path / "test.py")
        report.classes = ScopeStats(public=1, private=0, dunder=0)

        result = report.to_dict()
        assert result["classes"]["public"] == 1
        assert "totals" in result

    def test_to_compact(self, tmp_path: Path):
        report = FileScopeReport(path=tmp_path / "test.py")
        report.classes = ScopeStats(public=2, private=1, dunder=0)
        report.functions = ScopeStats(public=5, private=2, dunder=0)

        output = report.to_compact()
        assert "test.py" in output
        assert "public" in output


class TestAnalyzeFileScopes:
    """Tests for analyze_file_scopes."""

    def test_counts_public_functions(self, tmp_path: Path):
        f = tmp_path / "test.py"
        f.write_text("""
def public_one(): pass
def public_two(): pass
def _private(): pass
""")
        report = analyze_file_scopes(f)

        assert report.functions.public == 2
        assert report.functions.private == 1

    def test_counts_dunder_methods(self, tmp_path: Path):
        f = tmp_path / "test.py"
        f.write_text("""
class Foo:
    def __init__(self): pass
    def __str__(self): pass
    def public(self): pass
    def _private(self): pass
""")
        report = analyze_file_scopes(f)

        assert report.methods.dunder == 2
        assert report.methods.public == 1
        assert report.methods.private == 1

    def test_counts_classes(self, tmp_path: Path):
        f = tmp_path / "test.py"
        f.write_text("""
class Public: pass
class _Private: pass
""")
        report = analyze_file_scopes(f)

        assert report.classes.public == 1
        assert report.classes.private == 1

    def test_counts_variables(self, tmp_path: Path):
        f = tmp_path / "test.py"
        f.write_text("""
PUBLIC = 1
_private = 2
__dunder__ = 3
""")
        report = analyze_file_scopes(f)

        assert report.variables.public == 1
        assert report.variables.private == 1
        assert report.variables.dunder == 1

    def test_handles_syntax_error(self, tmp_path: Path):
        f = tmp_path / "bad.py"
        f.write_text("def broken(")

        report = analyze_file_scopes(f)
        assert report.total == 0


class TestAnalyzeProjectScopes:
    """Tests for analyze_project_scopes."""

    @pytest.fixture
    def project(self, tmp_path: Path) -> Path:
        src = tmp_path / "src"
        src.mkdir()
        (src / "a.py").write_text("""
def public_a(): pass
def _private_a(): pass
""")
        (src / "b.py").write_text("""
class PublicB: pass
class _PrivateB: pass
""")
        return tmp_path

    def test_aggregates_files(self, project: Path):
        report = analyze_project_scopes(project)

        assert report.file_count == 2
        assert report.total_public == 2  # public_a + PublicB
        assert report.total_private == 2  # _private_a + _PrivateB

    def test_skips_venv(self, tmp_path: Path):
        venv = tmp_path / "venv" / "lib"
        venv.mkdir(parents=True)
        (venv / "pkg.py").write_text("def should_skip(): pass")

        src = tmp_path / "src"
        src.mkdir()
        (src / "main.py").write_text("def real(): pass")

        report = analyze_project_scopes(tmp_path)

        # Only main.py should be counted
        assert report.file_count == 1
        assert report.total_public == 1

    def test_to_compact(self, project: Path):
        report = analyze_project_scopes(project)
        output = report.to_compact()

        assert "Project Scope Analysis" in output
        assert "Files analyzed: 2" in output


class TestFormatScopeReport:
    """Tests for format_scope_report."""

    def test_formats_file_report(self, tmp_path: Path):
        report = FileScopeReport(path=tmp_path / "test.py")
        report.functions = ScopeStats(public=3, private=1, dunder=0)

        output = format_scope_report(report)
        assert "test.py" in output
        assert "3" in output

    def test_formats_project_report(self, tmp_path: Path):
        report = ProjectScopeReport(root=tmp_path)
        output = format_scope_report(report)
        assert "Project Scope Analysis" in output
