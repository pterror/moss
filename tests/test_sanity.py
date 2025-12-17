"""Sanity tests to verify the test harness works."""


def test_import():
    """Verify moss package can be imported."""
    import moss

    assert moss.__version__ == "0.1.0"
