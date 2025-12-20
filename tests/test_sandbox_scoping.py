import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

import pytest

from moss.dwim_loop import DWIMLoop
from moss.moss_api import MossAPI
from moss.task_tree import TaskNode, TaskTree


def test_task_node_scoping():
    """Test that TaskNode correctly handles sandbox scope."""
    root = TaskNode("root")
    assert root.sandbox_scope is None

    # Add child with no scope -> inherits None
    child1 = root.add_child("child1")
    assert child1.sandbox_scope is None

    # Add child with scope
    scope_path = Path("/tmp/sandbox")
    child2 = root.add_child("child2", sandbox_scope=scope_path)
    assert child2.sandbox_scope == scope_path

    # Add grandchild -> inherits scope
    grandchild = child2.add_child("grandchild")
    assert grandchild.sandbox_scope == scope_path

    # Override scope in grandchild
    scope2 = Path("/tmp/sandbox/inner")
    grandchild2 = child2.add_child("grandchild2", sandbox_scope=scope2)
    assert grandchild2.sandbox_scope == scope2


def test_task_tree_serialization_with_scope():
    """Test serialization of scope."""
    root = TaskNode("root", sandbox_scope=Path("/tmp/root"))
    root.add_child("child")

    tree = TaskTree("root")
    tree.root = root

    data = tree.to_dict()
    assert data["sandbox_scope"] == "/tmp/root"
    assert data["children"][0]["sandbox_scope"] == "/tmp/root"

    tree2 = TaskTree.from_dict(data)
    assert tree2.root.sandbox_scope == Path("/tmp/root")
    assert tree2.root.children[0].sandbox_scope == Path("/tmp/root")


@pytest.mark.asyncio
async def test_dwim_loop_enforces_scope():
    """Test that DWIMLoop blocks access outside scope."""
    api = MagicMock(spec=MossAPI)
    api.root = Path("/project")

    loop = DWIMLoop(api)
    loop._task_tree = TaskTree("test task")

    # Set scope
    scope = Path("/project/src/submodule")
    loop._task_tree.current.sandbox_scope = scope

    # Mock api.skeleton.format to succeed
    api.skeleton.format = MagicMock(return_value="content")

    # 1. Access within scope
    tool_name = "skeleton.format"

    # Mock resolve to handle paths without filesystem
    # We need to patch Path.resolve or rely on the fallback in DWIMLoop
    # which uses (path.parent.resolve() / path.name) or similar.
    # But in test environment, /project doesn't exist.
    # So resolve() might return absolute path but check logic might fail
    # if real path logic is applied.
    # We should probably use a temporary directory for real path tests or mock Path.

    # Let's use a real temporary directory for robust testing

    with tempfile.TemporaryDirectory() as tmpdir:
        root = Path(tmpdir)
        api.root = root

        # Create directory structure
        submodule = root / "src" / "submodule"
        submodule.mkdir(parents=True)
        (submodule / "foo.py").touch()
        (root / "outside.py").touch()

        # Set scope
        scope = submodule
        loop._task_tree.current.sandbox_scope = scope

        # 1. Valid access
        params = {"file_path": str(submodule / "foo.py")}
        # We need to mock _run_tool_logic to avoid actual execution logic failure
        loop._run_tool_logic = AsyncMock(return_value="success")

        result = await loop._execute_tool(tool_name, params)
        assert result == "success"

        # 2. Invalid access (outside scope)
        params = {"file_path": str(root / "outside.py")}
        result = await loop._execute_tool(tool_name, params)
        assert "Access denied" in str(result)
        assert "outside sandbox scope" in str(result)

        # 3. Invalid access (relative path that resolves outside)
        # ../../outside.py relative to submodule/foo.py?
        # API resolves relative to root.
        # So "outside.py" is root/outside.py
        params = {"file_path": "outside.py"}
        result = await loop._execute_tool(tool_name, params)
        assert "Access denied" in str(result)
