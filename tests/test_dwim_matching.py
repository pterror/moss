"""Tests for DWIM tool matching behavior.

These tests define the expected matching behavior for common query patterns.
The goal is to ensure users can find tools naturally regardless of:
- Separator style (todo-list vs todo_list vs todo list)
- Typos and minor variations
- Case sensitivity
"""

import pytest

from moss_orchestration.dwim import analyze_intent

# Expected matches: (query, expected_tool, min_confidence)
EXACT_MATCH_CASES = [
    # Exact tool names
    ("skeleton", "skeleton", 0.95),
    ("deps", "deps", 0.95),
    ("cfg", "cfg", 0.95),
    ("anchors", "anchors", 0.95),
    ("query", "query", 0.95),
    ("callers", "callers", 0.95),
    ("callees", "callees", 0.95),
    ("view", "view", 0.95),
]

TYPO_CASES = [
    # Common typos should still match
    ("skelton", "skeleton", 0.70),  # missing 'e'
    ("skeletn", "skeleton", 0.70),  # missing 'o'
    ("depss", "deps", 0.70),  # extra 's'
    ("anchorss", "anchors", 0.70),  # extra 's'
]

NEGATIVE_CASES = [
    # These should NOT match with high confidence
    ("random gibberish xyz", None, 0.30),
    ("make coffee", None, 0.30),
    ("send email", None, 0.30),
]


class TestExactMatches:
    """Test exact tool name matching."""

    @pytest.mark.parametrize("query,expected_tool,min_confidence", EXACT_MATCH_CASES)
    def test_exact_match(self, query: str, expected_tool: str, min_confidence: float):
        results = analyze_intent(query)
        assert len(results) > 0, f"No results for query: {query}"
        top = results[0]
        assert top.tool == expected_tool, (
            f"Query '{query}' matched '{top.tool}' instead of '{expected_tool}'"
        )
        assert top.confidence >= min_confidence, (
            f"Query '{query}' confidence {top.confidence:.2f} < {min_confidence}"
        )


class TestTypoTolerance:
    """Test that typos are handled gracefully."""

    @pytest.mark.parametrize("query,expected_tool,min_confidence", TYPO_CASES)
    def test_typo(self, query: str, expected_tool: str, min_confidence: float):
        results = analyze_intent(query)
        assert len(results) > 0, f"No results for query: {query}"

        # Check if expected tool is in top 3 results
        top_tools = [r.tool for r in results[:3]]
        assert expected_tool in top_tools, (
            f"Typo query '{query}' didn't have '{expected_tool}' in top 3. Got: {top_tools}"
        )


class TestNegativeCases:
    """Test that irrelevant queries don't get high confidence matches."""

    @pytest.mark.parametrize("query,expected_tool,max_confidence", NEGATIVE_CASES)
    def test_negative(self, query: str, expected_tool, max_confidence: float):
        results = analyze_intent(query)
        if not results:
            return  # No results is fine for gibberish

        top = results[0]
        assert top.confidence <= max_confidence, (
            f"Irrelevant query '{query}' got confidence {top.confidence:.2f} "
            f"for '{top.tool}' (expected <= {max_confidence})"
        )


class TestConsistency:
    """Test matching consistency across equivalent queries."""

    def test_separator_equivalence(self):
        """Different separators should give same results."""
        queries = ["apply_patch", "apply-patch", "apply patch"]
        results = [analyze_intent(q) for q in queries]

        top_tools = [r[0].tool for r in results if r]
        assert len(set(top_tools)) == 1, f"Inconsistent top tools: {top_tools}"

    def test_case_insensitivity(self):
        """Matching should be case-insensitive."""
        queries = ["skeleton", "Skeleton", "SKELETON", "SkElEtOn"]
        results = [analyze_intent(q) for q in queries]

        top_tools = [r[0].tool for r in results if r]
        assert len(set(top_tools)) == 1, f"Case sensitivity issue: {top_tools}"

    def test_plural_equivalence(self):
        """Singular and plural should match similarly."""
        pairs = [
            ("todo", "todos"),
            ("dependency", "dependencies"),
            ("anchor", "anchors"),
        ]
        for singular, plural in pairs:
            r1 = analyze_intent(singular)
            r2 = analyze_intent(plural)
            if r1 and r2:
                t1, t2 = r1[0].tool, r2[0].tool
                assert t1.split("_")[0] == t2.split("_")[0] or t1 == t2, (
                    f"'{singular}' -> '{t1}' but '{plural}' -> '{t2}'"
                )
