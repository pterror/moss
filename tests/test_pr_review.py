"""Tests for PR review module."""

from pathlib import Path

import pytest

from moss.diff_analysis import DiffAnalysis, FileDiff, SymbolChange
from moss.pr_review import (
    ChangeCategory,
    Issue,
    PRReview,
    analyze_pr,
    assess_impact,
    categorize_changes,
    detect_issues,
    generate_summary,
    suggest_title,
)


class TestIssue:
    """Tests for Issue dataclass."""

    def test_basic_issue(self):
        issue = Issue(
            severity="warning",
            category="size",
            message="Large change detected",
        )

        assert issue.severity == "warning"
        assert issue.category == "size"
        assert issue.file_path is None
        assert issue.suggestion is None

    def test_issue_with_details(self):
        issue = Issue(
            severity="error",
            category="security",
            message="Sensitive file",
            file_path="config/.env",
            suggestion="Review for secrets",
        )

        assert issue.file_path == "config/.env"
        assert issue.suggestion == "Review for secrets"


class TestChangeCategory:
    """Tests for ChangeCategory dataclass."""

    def test_empty_category(self):
        cat = ChangeCategory(name="Tests", description="Test changes")

        assert cat.name == "Tests"
        assert cat.files == []
        assert cat.symbols == []

    def test_category_with_data(self):
        cat = ChangeCategory(
            name="Features",
            description="New features",
            files=["src/foo.py", "src/bar.py"],
            symbols=["function new_func", "class NewClass"],
        )

        assert len(cat.files) == 2
        assert len(cat.symbols) == 2


class TestPRReview:
    """Tests for PRReview dataclass."""

    def test_empty_review(self):
        analysis = DiffAnalysis()
        review = PRReview(diff_analysis=analysis)

        assert review.title_suggestion == ""
        assert review.summary == ""
        assert review.categories == []
        assert review.issues == []
        assert review.impact_level == "low"

    def test_review_to_dict(self):
        analysis = DiffAnalysis(
            files_changed=2,
            total_additions=50,
            total_deletions=10,
        )
        review = PRReview(
            diff_analysis=analysis,
            title_suggestion="feat: add feature",
            summary="Test summary",
            categories=[
                ChangeCategory(name="Features", description="New stuff", files=["a.py"]),
            ],
            issues=[
                Issue(severity="info", category="tests", message="No tests"),
            ],
            impact_level="medium",
            impact_areas=["src/moss"],
        )

        d = review.to_dict()

        assert d["title_suggestion"] == "feat: add feature"
        assert d["stats"]["files_changed"] == 2
        assert len(d["categories"]) == 1
        assert d["categories"][0]["name"] == "Features"
        assert len(d["issues"]) == 1
        assert d["impact"]["level"] == "medium"


class TestDetectIssues:
    """Tests for detect_issues function."""

    def test_no_issues_small_change(self):
        analysis = DiffAnalysis(
            files_changed=2,
            total_additions=20,
            total_deletions=10,
            file_diffs=[
                FileDiff(path=Path("src/foo.py"), additions=10, deletions=5),
                FileDiff(path=Path("tests/test_foo.py"), additions=10, deletions=5),
            ],
        )

        issues = detect_issues(analysis)

        # Should have no issues - small change with tests
        assert len(issues) == 0

    def test_large_change_warning(self):
        analysis = DiffAnalysis(
            total_additions=400,
            total_deletions=100,
        )

        issues = detect_issues(analysis)

        size_issues = [i for i in issues if i.category == "size"]
        assert len(size_issues) >= 1
        assert any("Large change" in i.message for i in size_issues)

    def test_many_files_warning(self):
        analysis = DiffAnalysis(
            files_changed=25,
            file_diffs=[FileDiff(path=Path(f"file{i}.py")) for i in range(25)],
        )

        issues = detect_issues(analysis)

        size_issues = [i for i in issues if "Many files" in i.message]
        assert len(size_issues) == 1

    def test_missing_tests_warning(self):
        analysis = DiffAnalysis(
            files_changed=1,
            file_diffs=[
                FileDiff(path=Path("src/feature.py"), additions=50),
            ],
        )

        issues = detect_issues(analysis)

        test_issues = [i for i in issues if i.category == "tests"]
        assert len(test_issues) == 1
        assert "without test" in test_issues[0].message.lower()

    def test_sensitive_file_warning(self):
        analysis = DiffAnalysis(
            files_changed=1,
            file_diffs=[
                FileDiff(path=Path(".env.local"), status="modified"),
            ],
        )

        issues = detect_issues(analysis)

        security_issues = [i for i in issues if i.category == "security"]
        assert len(security_issues) == 1


class TestCategorizeChanges:
    """Tests for categorize_changes function."""

    def test_empty_changes(self):
        analysis = DiffAnalysis()
        categories = categorize_changes(analysis)
        assert categories == []

    def test_source_only(self):
        analysis = DiffAnalysis(
            file_diffs=[
                FileDiff(path=Path("src/foo.py")),
                FileDiff(path=Path("src/bar.py")),
            ],
        )

        categories = categorize_changes(analysis)

        assert len(categories) == 1
        assert "src/foo.py" in categories[0].files

    def test_mixed_categories(self):
        analysis = DiffAnalysis(
            file_diffs=[
                FileDiff(path=Path("src/feature.py")),
                FileDiff(path=Path("tests/test_feature.py")),
                FileDiff(path=Path("README.md")),
                FileDiff(path=Path("pyproject.toml")),
            ],
        )

        categories = categorize_changes(analysis)

        names = {c.name for c in categories}
        assert "Tests" in names
        assert "Documentation" in names
        assert "Configuration" in names

    def test_new_features_category(self):
        analysis = DiffAnalysis(
            file_diffs=[FileDiff(path=Path("src/new.py"))],
            symbol_changes=[
                SymbolChange(
                    name="new_func",
                    kind="function",
                    change_type="added",
                    file_path=Path("src/new.py"),
                ),
            ],
        )

        categories = categorize_changes(analysis)

        assert len(categories) == 1
        assert "New Features" in categories[0].name


class TestAssessImpact:
    """Tests for assess_impact function."""

    def test_low_impact(self):
        analysis = DiffAnalysis(
            files_changed=2,
            total_additions=30,
            total_deletions=10,
        )

        level, _areas = assess_impact(analysis)

        assert level == "low"

    def test_medium_impact(self):
        analysis = DiffAnalysis(
            files_changed=8,
            total_additions=100,
            total_deletions=50,
        )

        level, _areas = assess_impact(analysis)

        assert level == "medium"

    def test_high_impact(self):
        analysis = DiffAnalysis(
            files_changed=20,
            total_additions=400,
            total_deletions=200,
        )

        level, _areas = assess_impact(analysis)

        assert level == "high"

    def test_impact_areas(self):
        analysis = DiffAnalysis(
            file_diffs=[
                FileDiff(path=Path("src/moss/cli.py")),
                FileDiff(path=Path("tests/test_cli.py")),
            ],
        )

        _level, areas = assess_impact(analysis)

        assert len(areas) >= 1


class TestSuggestTitle:
    """Tests for suggest_title function."""

    def test_empty_categories(self):
        analysis = DiffAnalysis()
        title = suggest_title(analysis, [])
        assert title == "Update files"

    def test_new_feature_title(self):
        analysis = DiffAnalysis()
        categories = [
            ChangeCategory(
                name="New Features",
                description="New functionality",
                files=["src/feature.py"],
                symbols=["function new_func"],
            )
        ]

        title = suggest_title(analysis, categories)

        assert title.startswith("feat")
        assert "new_func" in title

    def test_test_title(self):
        analysis = DiffAnalysis()
        categories = [
            ChangeCategory(
                name="Tests",
                description="Test files",
                files=["tests/test_foo.py"],
            )
        ]

        title = suggest_title(analysis, categories)

        assert title.startswith("test")


class TestGenerateSummary:
    """Tests for generate_summary function."""

    def test_minimal_summary(self):
        analysis = DiffAnalysis(files_changed=1, total_additions=10, total_deletions=5)
        review = PRReview(diff_analysis=analysis, impact_level="low")

        summary = generate_summary(review)

        assert "## Summary" in summary
        assert "1** files" in summary
        assert "+10" in summary
        assert "-5" in summary

    def test_summary_with_categories(self):
        analysis = DiffAnalysis(files_changed=2, total_additions=20, total_deletions=10)
        review = PRReview(
            diff_analysis=analysis,
            categories=[
                ChangeCategory(
                    name="Features",
                    description="New features added",
                    files=["src/a.py", "src/b.py"],
                )
            ],
        )

        summary = generate_summary(review)

        assert "## Changes" in summary
        assert "Features" in summary
        assert "src/a.py" in summary

    def test_summary_with_issues(self):
        analysis = DiffAnalysis()
        review = PRReview(
            diff_analysis=analysis,
            issues=[
                Issue(severity="warning", category="size", message="Too big"),
                Issue(severity="info", category="tests", message="Add tests"),
            ],
        )

        summary = generate_summary(review)

        assert "## Review Notes" in summary
        assert "Too big" in summary
        assert "Add tests" in summary


class TestAnalyzePR:
    """Integration tests for analyze_pr function."""

    @pytest.fixture
    def git_repo(self, tmp_path: Path):
        """Create a temporary git repository."""
        import subprocess

        subprocess.run(["git", "init"], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "config", "user.email", "test@test.com"],
            cwd=tmp_path,
            capture_output=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"],
            cwd=tmp_path,
            capture_output=True,
        )

        # Create initial commit
        (tmp_path / "initial.py").write_text("# initial\n")
        subprocess.run(["git", "add", "."], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", "initial"],
            cwd=tmp_path,
            capture_output=True,
        )

        return tmp_path

    def test_analyze_commits(self, git_repo: Path):
        """Test analyzing commit range."""
        import subprocess

        # Make a change
        (git_repo / "feature.py").write_text("def new_feature():\n    return 42\n")
        subprocess.run(["git", "add", "."], cwd=git_repo, capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", "add feature"],
            cwd=git_repo,
            capture_output=True,
        )

        review = analyze_pr(git_repo, "HEAD~1", "HEAD")

        assert review.diff_analysis.files_changed == 1
        assert review.diff_analysis.files_added == 1
        assert len(review.categories) >= 1
        assert review.summary != ""
        assert review.impact_level in ("low", "medium", "high")

    def test_analyze_staged(self, git_repo: Path):
        """Test analyzing staged changes."""
        import subprocess

        # Stage a change
        (git_repo / "staged.py").write_text("class MyClass:\n    pass\n")
        subprocess.run(["git", "add", "."], cwd=git_repo, capture_output=True)

        review = analyze_pr(git_repo, staged=True)

        assert review.diff_analysis.files_added == 1
        assert len(review.symbol_changes) >= 1 or len(review.categories) >= 1

    def test_title_suggestion(self, git_repo: Path):
        """Test that title suggestion is generated."""
        import subprocess

        # Make a change with a new function
        (git_repo / "new_module.py").write_text("def my_new_function():\n    pass\n")
        subprocess.run(["git", "add", "."], cwd=git_repo, capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", "add module"],
            cwd=git_repo,
            capture_output=True,
        )

        review = analyze_pr(git_repo, "HEAD~1", "HEAD")

        assert review.title_suggestion != ""
        assert ":" in review.title_suggestion  # Has conventional format

    @property
    def symbol_changes(self):
        """Proxy to diff_analysis.symbol_changes."""
        return self.diff_analysis.symbol_changes


# Add property to PRReview for convenience in tests
PRReview.symbol_changes = property(lambda self: self.diff_analysis.symbol_changes)
