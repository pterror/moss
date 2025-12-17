"""Tests for MCP server tool functions."""

from pathlib import Path

import pytest

from moss.mcp_server import (
    _tool_anchors,
    _tool_cfg,
    _tool_context,
    _tool_deps,
    _tool_skeleton,
)


class TestToolSkeleton:
    """Tests for skeleton tool."""

    @pytest.fixture
    def python_file(self, tmp_path: Path):
        """Create a Python file for testing."""
        py_file = tmp_path / "sample.py"
        py_file.write_text("""
class Foo:
    '''A class.'''
    def bar(self) -> str:
        '''A method.'''
        return "hello"

def baz():
    '''A function.'''
    pass
""")
        return py_file

    def test_extracts_skeleton(self, python_file: Path):
        result = _tool_skeleton({"path": str(python_file)})

        assert "file" in result
        assert "symbols" in result
        assert len(result["symbols"]) >= 2  # Foo, baz

        names = [s["name"] for s in result["symbols"]]
        assert "Foo" in names
        assert "baz" in names

    def test_handles_nonexistent_path(self):
        result = _tool_skeleton({"path": "/nonexistent/path"})
        assert "error" in result

    def test_handles_syntax_error(self, tmp_path: Path):
        bad_file = tmp_path / "bad.py"
        bad_file.write_text("def broken(")

        result = _tool_skeleton({"path": str(bad_file)})
        assert "error" in result


class TestToolAnchors:
    """Tests for anchors tool."""

    @pytest.fixture
    def python_file(self, tmp_path: Path):
        """Create a Python file for testing."""
        py_file = tmp_path / "sample.py"
        py_file.write_text("""
class MyClass:
    def method(self): pass

def my_function():
    pass
""")
        return py_file

    def test_finds_all_anchors(self, python_file: Path):
        result = _tool_anchors({"path": str(python_file)})

        assert isinstance(result, list)
        names = [r["name"] for r in result]
        assert "MyClass" in names
        assert "method" in names
        assert "my_function" in names

    def test_filters_by_type(self, python_file: Path):
        result = _tool_anchors({"path": str(python_file), "type": "class"})

        assert isinstance(result, list)
        assert all(r["type"] == "class" for r in result)
        names = [r["name"] for r in result]
        assert "MyClass" in names
        assert "my_function" not in names

    def test_filters_by_name(self, python_file: Path):
        result = _tool_anchors({"path": str(python_file), "name": "my_.*"})

        assert isinstance(result, list)
        names = [r["name"] for r in result]
        assert "my_function" in names
        assert "MyClass" not in names


class TestToolCfg:
    """Tests for cfg tool."""

    @pytest.fixture
    def python_file(self, tmp_path: Path):
        """Create a Python file with control flow."""
        py_file = tmp_path / "sample.py"
        py_file.write_text("""
def check(x):
    if x > 0:
        return "positive"
    else:
        return "non-positive"
""")
        return py_file

    def test_builds_cfg(self, python_file: Path):
        result = _tool_cfg({"path": str(python_file)})

        assert isinstance(result, list)
        assert len(result) == 1
        cfg = result[0]
        assert cfg["name"] == "check"
        assert "nodes" in cfg
        assert "edges" in cfg

    def test_filters_by_function(self, python_file: Path):
        result = _tool_cfg({"path": str(python_file), "function": "check"})

        assert isinstance(result, list)
        assert len(result) == 1
        assert result[0]["name"] == "check"

    def test_handles_nonexistent_function(self, python_file: Path):
        result = _tool_cfg({"path": str(python_file), "function": "nonexistent"})

        assert "error" in result

    def test_requires_file_not_directory(self, tmp_path: Path):
        result = _tool_cfg({"path": str(tmp_path)})
        assert "error" in result


class TestToolDeps:
    """Tests for deps tool."""

    @pytest.fixture
    def python_file(self, tmp_path: Path):
        """Create a Python file with imports."""
        py_file = tmp_path / "sample.py"
        py_file.write_text("""
import os
from pathlib import Path

def public_func():
    pass

class PublicClass:
    pass
""")
        return py_file

    def test_extracts_deps(self, python_file: Path):
        result = _tool_deps({"path": str(python_file)})

        assert "file" in result
        assert "imports" in result
        assert "exports" in result
        assert len(result["imports"]) >= 2

        modules = [i["module"] for i in result["imports"]]
        assert "os" in modules
        assert "pathlib" in modules

    def test_extracts_exports(self, python_file: Path):
        result = _tool_deps({"path": str(python_file)})

        export_names = [e["name"] for e in result["exports"]]
        assert "public_func" in export_names
        assert "PublicClass" in export_names


class TestToolContext:
    """Tests for context tool."""

    @pytest.fixture
    def python_file(self, tmp_path: Path):
        """Create a Python file for testing."""
        py_file = tmp_path / "sample.py"
        py_file.write_text("""
'''A sample module.'''

import os

class Foo:
    '''A class.'''
    def bar(self): pass

def baz():
    '''A function.'''
    pass
""")
        return py_file

    def test_generates_context(self, python_file: Path):
        result = _tool_context({"path": str(python_file)})

        assert "file" in result
        assert "summary" in result
        assert "symbols" in result
        assert "imports" in result
        assert "exports" in result

        summary = result["summary"]
        assert summary["classes"] >= 1
        assert summary["functions"] >= 1
        assert summary["methods"] >= 1
        assert summary["imports"] >= 1

    def test_requires_file_not_directory(self, tmp_path: Path):
        result = _tool_context({"path": str(tmp_path)})
        assert "error" in result
