"""Sanity tests to verify the test harness works."""


def test_import_intelligence():
    """Verify moss_intelligence package can be imported."""
    import moss_intelligence

    assert moss_intelligence is not None


def test_import_orchestration():
    """Verify moss_orchestration package can be imported."""
    import moss_orchestration

    assert moss_orchestration is not None


def test_import_context():
    """Verify moss_context package can be imported."""
    import moss_context

    assert moss_context is not None


def test_import_cli():
    """Verify moss_cli package can be imported."""
    import moss_cli

    assert moss_cli is not None
