"""Tests for Patch Application."""

from moss_intelligence.anchors import Anchor, AnchorType
from moss_intelligence.patches import (
    Patch,
    PatchType,
    apply_patch,
    apply_patch_with_fallback,
    apply_text_patch,
)


class TestApplyPatch:
    """Tests for apply_patch function."""

    def test_replace_function(self):
        source = """
def hello():
    return "hello"

def world():
    return "world"
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="hello"),
            patch_type=PatchType.REPLACE,
            content='def hello():\n    return "HELLO"',
        )

        result = apply_patch(source, patch)

        assert result.success
        assert 'return "HELLO"' in result.patched
        assert 'return "world"' in result.patched

    def test_replace_preserves_indent(self):
        source = """
class MyClass:
    def method(self):
        pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.METHOD, name="method", context="MyClass"),
            patch_type=PatchType.REPLACE,
            content="def method(self):\n    return 42",
        )

        result = apply_patch(source, patch)

        assert result.success
        # Method should still be indented inside class
        assert "    def method(self):" in result.patched

    def test_insert_before(self):
        source = """
def second():
    pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="second"),
            patch_type=PatchType.INSERT_BEFORE,
            content="def first():\n    pass",
        )

        result = apply_patch(source, patch)

        assert result.success
        # first should come before second
        assert result.patched.index("first") < result.patched.index("second")

    def test_insert_after(self):
        source = """
def first():
    pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="first"),
            patch_type=PatchType.INSERT_AFTER,
            content="def second():\n    pass",
        )

        result = apply_patch(source, patch)

        assert result.success
        assert result.patched.index("first") < result.patched.index("second")

    def test_delete(self):
        source = """
def keep():
    pass

def delete_me():
    pass

def also_keep():
    pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="delete_me"),
            patch_type=PatchType.DELETE,
        )

        result = apply_patch(source, patch)

        assert result.success
        assert "delete_me" not in result.patched
        assert "keep" in result.patched
        assert "also_keep" in result.patched

    def test_anchor_not_found(self):
        source = """
def hello():
    pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="nonexistent"),
            patch_type=PatchType.REPLACE,
            content="def nonexistent(): pass",
        )

        result = apply_patch(source, patch)

        assert not result.success
        assert "not found" in result.error.lower()

    def test_invalid_syntax_detected(self):
        source = """
def hello():
    pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="hello"),
            patch_type=PatchType.REPLACE,
            content="def hello(:",  # Invalid syntax
        )

        result = apply_patch(source, patch)

        assert not result.success
        assert "syntax" in result.error.lower()


class TestApplyTextPatch:
    """Tests for apply_text_patch function."""

    def test_simple_replace(self):
        source = 'x = "hello"'
        result = apply_text_patch(source, '"hello"', '"world"')

        assert result.success
        assert result.patched == 'x = "world"'
        assert result.used_fallback

    def test_replace_all_occurrences(self):
        source = "a = 1\nb = 1\nc = 1"
        result = apply_text_patch(source, "1", "2", occurrence=0)

        assert result.success
        assert result.patched == "a = 2\nb = 2\nc = 2"

    def test_replace_specific_occurrence(self):
        source = "a = 1\nb = 1\nc = 1"
        result = apply_text_patch(source, "1", "2", occurrence=2)

        assert result.success
        assert result.patched == "a = 1\nb = 2\nc = 1"

    def test_not_found(self):
        source = 'x = "hello"'
        result = apply_text_patch(source, "nonexistent", "replacement")

        assert not result.success
        assert "not found" in result.error.lower()

    def test_occurrence_not_found(self):
        source = "a = 1"
        result = apply_text_patch(source, "1", "2", occurrence=5)

        assert not result.success
        assert "occurrence" in result.error.lower()


class TestApplyPatchWithFallback:
    """Tests for apply_patch_with_fallback function."""

    def test_ast_patch_succeeds(self):
        source = """
def hello():
    pass
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="hello"),
            patch_type=PatchType.REPLACE,
            content="def hello():\n    return 42",
        )

        result = apply_patch_with_fallback(source, patch)

        assert result.success
        assert not result.used_fallback

    def test_fallback_on_broken_ast(self):
        source = 'def broken(\n    x = "hello"'  # Broken syntax

        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="broken"),
            patch_type=PatchType.REPLACE,
            content="def broken(): pass",
        )

        result = apply_patch_with_fallback(
            source,
            patch,
            fallback_search='"hello"',
            fallback_replace='"world"',
        )

        assert result.success
        assert result.used_fallback
        assert '"world"' in result.patched

    def test_fallback_on_anchor_not_found(self):
        source = """
def hello():
    x = "value"
"""
        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="nonexistent"),
            patch_type=PatchType.REPLACE,
            content="def nonexistent(): pass",
        )

        result = apply_patch_with_fallback(
            source,
            patch,
            fallback_search='"value"',
            fallback_replace='"new_value"',
        )

        assert result.success
        assert result.used_fallback
        assert '"new_value"' in result.patched

    def test_no_fallback_provided(self):
        source = "def broken(\n    pass"

        patch = Patch(
            anchor=Anchor(type=AnchorType.FUNCTION, name="broken"),
            patch_type=PatchType.REPLACE,
            content="def broken(): pass",
        )

        result = apply_patch_with_fallback(source, patch)

        assert not result.success
        assert "syntax errors" in result.error.lower()
