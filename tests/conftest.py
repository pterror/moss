"""Pytest configuration and fixtures."""

import pytest

from moss.output import reset_output


@pytest.fixture(autouse=True)
def clean_output():
    """Reset global output instance between tests.

    This ensures tests don't pollute each other with stale
    stdout/stderr references.
    """
    reset_output()
    yield
    reset_output()
