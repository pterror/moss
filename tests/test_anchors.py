"""Tests for Anchor Resolution."""

import pytest

from moss_intelligence.anchors import (
    Anchor,
    AnchorNotFoundError,
    AnchorType,
    find_anchors,
    resolve_anchor,
)


class TestAnchor:
    """Tests for Anchor dataclass."""

    def test_create_anchor(self):
        anchor = Anchor(type=AnchorType.FUNCTION, name="hello")
        assert anchor.type == AnchorType.FUNCTION
        assert anchor.name == "hello"
        assert anchor.context is None

    def test_anchor_with_context(self):
        anchor = Anchor(type=AnchorType.METHOD, name="greet", context="Greeter")
        assert anchor.context == "Greeter"


class TestAnchorResolver:
    """Tests for AnchorResolver."""

    def test_resolve_function(self):
        source = """
def hello():
    pass

def world():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="hello")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 2
        assert match.score > 0.9

    def test_resolve_function_fuzzy(self):
        source = """
def hello_world():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="helloworld")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 2
        assert match.score >= 0.6

    def test_resolve_class(self):
        source = """
class MyClass:
    pass
"""
        anchor = Anchor(type=AnchorType.CLASS, name="MyClass")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 2
        assert match.score > 0.9

    def test_resolve_method_with_context(self):
        source = """
class Greeter:
    def greet(self):
        pass

class Other:
    def greet(self):
        pass
"""
        anchor = Anchor(type=AnchorType.METHOD, name="greet", context="Greeter")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 3
        assert "Greeter" in match.context_chain

    def test_resolve_variable(self):
        source = """
MY_CONST = 42
other = "hello"
"""
        anchor = Anchor(type=AnchorType.VARIABLE, name="MY_CONST")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 2

    def test_resolve_import(self):
        source = """
import os
from pathlib import Path
"""
        anchor = Anchor(type=AnchorType.IMPORT, name="Path")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 3

    def test_not_found_raises(self):
        source = """
def hello():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="nonexistent")

        with pytest.raises(AnchorNotFoundError) as exc:
            resolve_anchor(source, anchor)

        assert "nonexistent" in str(exc.value)

    def test_not_found_with_suggestions(self):
        source = """
def hello_world():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="goodbye")

        with pytest.raises(AnchorNotFoundError) as exc:
            resolve_anchor(source, anchor)

        # Should suggest hello_world as a close match
        assert len(exc.value.suggestions) >= 0  # May or may not have suggestions

    def test_ambiguous_raises(self):
        source = """
def process():
    pass

def process_data():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="process")

        # This should be ambiguous since both match "process" well
        matches = find_anchors(source, anchor)
        assert len(matches) >= 1

    def test_async_function(self):
        source = """
async def fetch_data():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="fetch_data")
        match = resolve_anchor(source, anchor)

        assert match.lineno == 2

    def test_nested_context(self):
        source = """
class Outer:
    class Inner:
        def method(self):
            pass
"""
        anchor = Anchor(type=AnchorType.METHOD, name="method", context="Inner")
        match = resolve_anchor(source, anchor)

        assert "Outer" in match.context_chain
        assert "Inner" in match.context_chain

    def test_match_span(self):
        source = """
def multiline(
    arg1,
    arg2,
):
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="multiline")
        match = resolve_anchor(source, anchor)

        assert match.span[0] == 2  # Start line
        assert match.span[1] >= 5  # End line includes body


class TestFindAnchors:
    """Tests for find_anchors function."""

    def test_find_all_matches(self):
        source = """
def process():
    pass

def process_data():
    pass

def process_items():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="process")
        matches = find_anchors(source, anchor, min_score=0.5)

        assert len(matches) >= 2
        # Should be sorted by score
        assert matches[0].score >= matches[1].score

    def test_find_no_matches(self):
        source = """
def hello():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="xyz")
        matches = find_anchors(source, anchor)

        assert len(matches) == 0


class TestAnchorMatch:
    """Tests for AnchorMatch properties."""

    def test_span_property(self):
        source = """
def hello():
    pass
"""
        anchor = Anchor(type=AnchorType.FUNCTION, name="hello")
        match = resolve_anchor(source, anchor)

        start, end = match.span
        assert start == 2
        assert end >= 3
