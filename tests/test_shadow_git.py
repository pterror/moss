"""Tests for Shadow Git wrapper."""

import asyncio
from pathlib import Path

import pytest

from moss.shadow_git import CommitHandle, GitError, ShadowGit


@pytest.fixture
async def git_repo(tmp_path: Path):
    """Create a temporary git repository."""
    repo = tmp_path / "repo"
    repo.mkdir()

    # Initialize repo
    proc = await asyncio.create_subprocess_exec(
        "git",
        "init",
        cwd=repo,
        stdout=asyncio.subprocess.DEVNULL,
        stderr=asyncio.subprocess.DEVNULL,
    )
    await proc.wait()

    # Configure git user
    proc = await asyncio.create_subprocess_exec(
        "git",
        "config",
        "user.email",
        "test@test.com",
        cwd=repo,
    )
    await proc.wait()
    proc = await asyncio.create_subprocess_exec(
        "git",
        "config",
        "user.name",
        "Test User",
        cwd=repo,
    )
    await proc.wait()

    # Create initial commit
    (repo / "README.md").write_text("# Test Repo")
    proc = await asyncio.create_subprocess_exec("git", "add", "-A", cwd=repo)
    await proc.wait()
    proc = await asyncio.create_subprocess_exec(
        "git",
        "commit",
        "-m",
        "Initial commit",
        cwd=repo,
    )
    await proc.wait()

    return repo


@pytest.fixture
def shadow_git(git_repo: Path):
    """Create ShadowGit instance."""
    return ShadowGit(git_repo)


class TestShadowGit:
    """Tests for ShadowGit."""

    async def test_create_shadow_branch(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        assert branch.name == "shadow/test"
        assert branch.base_branch == "master"
        assert branch.repo_path == git_repo
        assert branch.commits == []

    async def test_create_shadow_branch_auto_name(self, shadow_git: ShadowGit):
        branch = await shadow_git.create_shadow_branch()

        assert branch.name.startswith("shadow/")

    async def test_commit_creates_handle(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        # Make a change
        (git_repo / "file.txt").write_text("hello")

        handle = await shadow_git.commit(branch, "Add file")

        assert handle.sha is not None
        assert handle.message == "Add file"
        assert handle.branch == "shadow/test"
        assert handle in branch.commits

    async def test_commit_no_changes_raises(self, shadow_git: ShadowGit):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        with pytest.raises(GitError, match="Nothing to commit"):
            await shadow_git.commit(branch, "Empty commit")

    async def test_commit_allow_empty(self, shadow_git: ShadowGit):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        handle = await shadow_git.commit(branch, "Empty commit", allow_empty=True)

        assert handle.sha is not None

    async def test_multiple_commits(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file1.txt").write_text("one")
        await shadow_git.commit(branch, "First")

        (git_repo / "file2.txt").write_text("two")
        await shadow_git.commit(branch, "Second")

        assert len(branch.commits) == 2
        assert branch.commits[0].message == "First"
        assert branch.commits[1].message == "Second"

    async def test_rollback(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file1.txt").write_text("one")
        await shadow_git.commit(branch, "First")

        (git_repo / "file2.txt").write_text("two")
        await shadow_git.commit(branch, "Second")

        await shadow_git.rollback(branch, steps=1)

        assert len(branch.commits) == 1
        assert not (git_repo / "file2.txt").exists()

    async def test_rollback_multiple(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        for i in range(3):
            (git_repo / f"file{i}.txt").write_text(str(i))
            await shadow_git.commit(branch, f"Commit {i}")

        await shadow_git.rollback(branch, steps=2)

        assert len(branch.commits) == 1
        assert (git_repo / "file0.txt").exists()
        assert not (git_repo / "file1.txt").exists()
        assert not (git_repo / "file2.txt").exists()

    async def test_rollback_too_many_raises(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file.txt").write_text("one")
        await shadow_git.commit(branch, "First")

        with pytest.raises(ValueError, match="Cannot rollback 5 commits"):
            await shadow_git.rollback(branch, steps=5)

    async def test_rollback_to_specific_commit(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file1.txt").write_text("one")
        first = await shadow_git.commit(branch, "First")

        (git_repo / "file2.txt").write_text("two")
        await shadow_git.commit(branch, "Second")

        (git_repo / "file3.txt").write_text("three")
        await shadow_git.commit(branch, "Third")

        await shadow_git.rollback_to(branch, first)

        assert len(branch.commits) == 1
        assert (git_repo / "file1.txt").exists()
        assert not (git_repo / "file2.txt").exists()

    async def test_squash_merge(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file1.txt").write_text("one")
        await shadow_git.commit(branch, "First")

        (git_repo / "file2.txt").write_text("two")
        await shadow_git.commit(branch, "Second")

        handle = await shadow_git.squash_merge(branch, "Merged feature")

        assert handle.branch == "master"
        assert (git_repo / "file1.txt").exists()
        assert (git_repo / "file2.txt").exists()

    async def test_squash_merge_no_commits_raises(self, shadow_git: ShadowGit):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        with pytest.raises(GitError, match="No commits to merge"):
            await shadow_git.squash_merge(branch, "Empty merge")

    async def test_abort_branch(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file.txt").write_text("one")
        await shadow_git.commit(branch, "First")

        await shadow_git.abort(branch)

        # Branch should be deleted
        assert shadow_git.get_branch("shadow/test") is None
        # Should be back on master
        proc = await asyncio.create_subprocess_exec(
            "git",
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
            cwd=git_repo,
            stdout=asyncio.subprocess.PIPE,
        )
        stdout, _ = await proc.communicate()
        assert stdout.decode().strip() == "master"

    async def test_diff(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file.txt").write_text("hello world")
        await shadow_git.commit(branch, "Add file")

        diff = await shadow_git.diff(branch)

        assert "+hello world" in diff

    async def test_diff_stat(self, shadow_git: ShadowGit, git_repo: Path):
        branch = await shadow_git.create_shadow_branch("shadow/test")

        (git_repo / "file.txt").write_text("hello world")
        await shadow_git.commit(branch, "Add file")

        stat = await shadow_git.diff_stat(branch)

        assert "file.txt" in stat

    async def test_active_branches(self, shadow_git: ShadowGit):
        branch1 = await shadow_git.create_shadow_branch("shadow/one")
        # Need to go back to master to create another branch
        await shadow_git._run_git("checkout", "master")
        branch2 = await shadow_git.create_shadow_branch("shadow/two")

        branches = shadow_git.active_branches

        assert len(branches) == 2
        assert branch1 in branches
        assert branch2 in branches


class TestCommitHandle:
    """Tests for CommitHandle."""

    def test_commit_handle_frozen(self):
        handle = CommitHandle(
            sha="abc123",
            message="test",
            timestamp=None,  # type: ignore[arg-type]
            branch="main",
        )
        with pytest.raises(AttributeError):
            handle.sha = "def456"  # type: ignore[misc]
