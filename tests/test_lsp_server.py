"""Tests for LSP server module."""

import pytest

# Check if LSP dependencies are available
try:
    from lsprotocol import types as lsp

    HAS_LSP = True
except ImportError:
    HAS_LSP = False

pytestmark = pytest.mark.skipif(not HAS_LSP, reason="LSP dependencies not installed")


@pytest.fixture
def simple_python_source() -> str:
    return '''
def simple_function():
    """A simple function."""
    x = 1
    return x


def complex_function(a, b, c):
    """A more complex function with branches."""
    if a > 0:
        if b > 0:
            if c > 0:
                return a + b + c
            else:
                return a + b
        else:
            if c > 0:
                return a + c
            else:
                return a
    else:
        if b > 0:
            if c > 0:
                return b + c
            else:
                return b
        else:
            if c > 0:
                return c
            else:
                return 0


class MyClass:
    """A test class."""

    def method_one(self):
        return 1

    def method_two(self, x):
        if x > 0:
            return x
        return -x
'''


class TestMossLSPConfig:
    """Tests for MossLSPConfig."""

    def test_default_config(self):
        from moss_lsp.server import MossLSPConfig

        config = MossLSPConfig()

        assert config.enable_diagnostics is True
        assert config.complexity_warning_threshold == 10
        assert config.complexity_error_threshold == 20
        assert config.enable_hover is True

    def test_custom_config(self):
        from moss_lsp.server import MossLSPConfig

        config = MossLSPConfig(
            complexity_warning_threshold=5,
            complexity_error_threshold=15,
            enable_hover=False,
        )

        assert config.complexity_warning_threshold == 5
        assert config.complexity_error_threshold == 15
        assert config.enable_hover is False


class TestAnalysisCache:
    """Tests for AnalysisCache."""

    def test_get_set(self):
        from moss_lsp.server import AnalysisCache, DocumentAnalysis

        cache = AnalysisCache()

        analysis = DocumentAnalysis(uri="file:///test.py", version=1)
        cache.set(analysis)

        result = cache.get("file:///test.py", 1)
        assert result is not None
        assert result.uri == "file:///test.py"

    def test_get_wrong_version(self):
        from moss_lsp.server import AnalysisCache, DocumentAnalysis

        cache = AnalysisCache()

        analysis = DocumentAnalysis(uri="file:///test.py", version=1)
        cache.set(analysis)

        result = cache.get("file:///test.py", 2)
        assert result is None

    def test_invalidate(self):
        from moss_lsp.server import AnalysisCache, DocumentAnalysis

        cache = AnalysisCache()

        analysis = DocumentAnalysis(uri="file:///test.py", version=1)
        cache.set(analysis)

        cache.invalidate("file:///test.py")

        result = cache.get("file:///test.py", 1)
        assert result is None

    def test_clear(self):
        from moss_lsp.server import AnalysisCache, DocumentAnalysis

        cache = AnalysisCache()

        cache.set(DocumentAnalysis(uri="file:///a.py", version=1))
        cache.set(DocumentAnalysis(uri="file:///b.py", version=1))

        cache.clear()

        assert cache.get("file:///a.py", 1) is None
        assert cache.get("file:///b.py", 1) is None


class TestMossLanguageServer:
    """Tests for MossLanguageServer."""

    def test_create_server(self):
        from moss_lsp.server import MossLanguageServer

        server = MossLanguageServer()

        assert server.name == "moss-lsp"
        assert server.version == "0.1.0"
        assert server.config is not None
        assert server.cache is not None

    def test_analyze_python(self, simple_python_source: str):
        from moss_lsp.server import MossLanguageServer

        server = MossLanguageServer()

        analysis = server._analyze_python("file:///test.py", 1, simple_python_source)

        assert analysis.uri == "file:///test.py"
        assert len(analysis.cfgs) > 0

        # Check we found the functions
        names = [cfg["name"] for cfg in analysis.cfgs]
        assert "simple_function" in names
        assert "complex_function" in names

    def test_generate_diagnostics(self, simple_python_source: str):
        from moss_lsp.server import MossLanguageServer

        server = MossLanguageServer()
        # Lower thresholds for testing
        server.config.complexity_warning_threshold = 2
        server.config.complexity_error_threshold = 5

        analysis = server._analyze_python("file:///test.py", 1, simple_python_source)
        diagnostics = server._generate_diagnostics(analysis)

        # complex_function should trigger diagnostics
        assert len(diagnostics) > 0

        # Check diagnostic properties
        has_complexity_diagnostic = False
        for diag in diagnostics:
            if "complex_function" in diag.message:
                has_complexity_diagnostic = True
                assert diag.source == "moss"
                assert "complexity" in diag.code
        assert has_complexity_diagnostic

    def test_extract_symbols(self, simple_python_source: str):
        from moss_lsp.server import MossLanguageServer

        server = MossLanguageServer()

        analysis = server._analyze_python("file:///test.py", 1, simple_python_source)

        # Should have symbols for functions and class
        assert len(analysis.symbols) > 0

        names = [s["name"] for s in analysis.symbols]
        # Check we found top-level items
        assert any("simple_function" in n or "function" in n.lower() for n in names)


class TestSymbolKind:
    """Tests for symbol kind conversion."""

    def test_class_kind(self):
        from moss_lsp.server import _symbol_kind

        kind = _symbol_kind("class")
        assert kind == lsp.SymbolKind.Class

    def test_function_kind(self):
        from moss_lsp.server import _symbol_kind

        kind = _symbol_kind("function")
        assert kind == lsp.SymbolKind.Function

    def test_method_kind(self):
        from moss_lsp.server import _symbol_kind

        kind = _symbol_kind("method")
        assert kind == lsp.SymbolKind.Method

    def test_unknown_kind(self):
        from moss_lsp.server import _symbol_kind

        kind = _symbol_kind("unknown")
        assert kind == lsp.SymbolKind.Variable


class TestGetWordAtPosition:
    """Tests for word extraction."""

    def test_simple_word(self):
        from moss_lsp.server import _get_word_at_position

        source = "def hello_world():\n    pass"
        pos = lsp.Position(line=0, character=6)

        word = _get_word_at_position(source, pos)
        assert word == "hello_world"

    def test_at_start_of_word(self):
        from moss_lsp.server import _get_word_at_position

        source = "variable = 123"
        pos = lsp.Position(line=0, character=0)

        word = _get_word_at_position(source, pos)
        assert word == "variable"

    def test_no_word(self):
        from moss_lsp.server import _get_word_at_position

        source = "x = 1"
        pos = lsp.Position(line=0, character=2)  # at the space

        word = _get_word_at_position(source, pos)
        assert word is None or word == ""

    def test_out_of_bounds(self):
        from moss_lsp.server import _get_word_at_position

        source = "x = 1"
        pos = lsp.Position(line=5, character=0)  # line doesn't exist

        word = _get_word_at_position(source, pos)
        assert word is None


class TestCreateServer:
    """Tests for server creation."""

    def test_creates_server_with_handlers(self):
        from moss_lsp.server import create_server

        server = create_server()

        assert server is not None
        # Check server has essential attributes (workspace not available until initialized)
        assert hasattr(server, "protocol")
        assert server.name == "moss-lsp"


class TestStartServer:
    """Tests for server startup."""

    def test_unknown_transport_raises(self):
        from moss_lsp.server import start_server

        with pytest.raises(ValueError, match="Unknown transport"):
            start_server("unknown")
