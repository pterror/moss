"""Tests for View Provider protocol."""

from pathlib import Path

import pytest

from moss.views import (
    Intent,
    RawViewProvider,
    View,
    ViewOptions,
    ViewRegistry,
    ViewTarget,
    ViewType,
    create_default_registry,
)


class TestViewTarget:
    """Tests for ViewTarget."""

    def test_create_target(self, tmp_path: Path):
        target = ViewTarget(path=tmp_path / "test.py")
        assert target.path == tmp_path / "test.py"
        assert target.language is None
        assert target.symbol is None

    def test_target_with_language(self, tmp_path: Path):
        target = ViewTarget(path=tmp_path / "test.py", language="python")
        assert target.language == "python"

    def test_target_with_symbol(self, tmp_path: Path):
        target = ViewTarget(path=tmp_path / "test.py", symbol="MyClass")
        assert target.symbol == "MyClass"


class TestViewOptions:
    """Tests for ViewOptions."""

    def test_default_options(self):
        opts = ViewOptions()
        assert opts.max_depth is None
        assert opts.include_private is False
        assert opts.include_docstrings is True
        assert opts.line_range is None

    def test_custom_options(self):
        opts = ViewOptions(max_depth=3, include_private=True, line_range=(1, 10))
        assert opts.max_depth == 3
        assert opts.include_private is True
        assert opts.line_range == (1, 10)


class TestView:
    """Tests for View."""

    def test_create_view(self, tmp_path: Path):
        target = ViewTarget(path=tmp_path / "test.py")
        view = View(target=target, view_type=ViewType.RAW, content="hello world")
        assert view.content == "hello world"
        assert view.view_type == ViewType.RAW

    def test_token_estimate(self, tmp_path: Path):
        target = ViewTarget(path=tmp_path / "test.py")
        # 10 words ~= 13 tokens
        content = "one two three four five six seven eight nine ten"
        view = View(target=target, view_type=ViewType.RAW, content=content)
        assert view.token_estimate >= 10


class TestRawViewProvider:
    """Tests for RawViewProvider."""

    @pytest.fixture
    def provider(self):
        return RawViewProvider()

    @pytest.fixture
    def python_file(self, tmp_path: Path) -> Path:
        f = tmp_path / "test.py"
        f.write_text("def hello():\n    return 'world'\n")
        return f

    def test_view_type(self, provider: RawViewProvider):
        assert provider.view_type == ViewType.RAW

    def test_supported_languages_empty(self, provider: RawViewProvider):
        assert provider.supported_languages == set()

    def test_supports_existing_file(self, provider: RawViewProvider, python_file: Path):
        target = ViewTarget(path=python_file)
        assert provider.supports(target) is True

    def test_supports_missing_file(self, provider: RawViewProvider, tmp_path: Path):
        target = ViewTarget(path=tmp_path / "missing.py")
        assert provider.supports(target) is False

    async def test_render_full_file(self, provider: RawViewProvider, python_file: Path):
        target = ViewTarget(path=python_file)
        view = await provider.render(target)

        assert view.view_type == ViewType.RAW
        assert "def hello():" in view.content
        assert view.metadata["lines"] == 2

    async def test_render_line_range(self, provider: RawViewProvider, tmp_path: Path):
        f = tmp_path / "multi.py"
        f.write_text("line1\nline2\nline3\nline4\nline5\n")
        target = ViewTarget(path=f)

        view = await provider.render(target, ViewOptions(line_range=(2, 4)))

        assert view.content == "line2\nline3\nline4"


class TestViewProvider:
    """Tests for ViewProvider language detection."""

    @pytest.fixture
    def provider(self):
        return RawViewProvider()

    def test_detect_python(self, provider: RawViewProvider):
        assert provider._detect_language(Path("test.py")) == "python"

    def test_detect_javascript(self, provider: RawViewProvider):
        assert provider._detect_language(Path("test.js")) == "javascript"

    def test_detect_typescript(self, provider: RawViewProvider):
        assert provider._detect_language(Path("test.ts")) == "typescript"
        assert provider._detect_language(Path("test.tsx")) == "typescript"

    def test_detect_rust(self, provider: RawViewProvider):
        assert provider._detect_language(Path("test.rs")) == "rust"

    def test_detect_unknown(self, provider: RawViewProvider):
        assert provider._detect_language(Path("test.xyz")) == "unknown"


class TestViewRegistry:
    """Tests for ViewRegistry."""

    @pytest.fixture
    def registry(self):
        return ViewRegistry()

    def test_register_provider(self, registry: ViewRegistry):
        provider = RawViewProvider()
        registry.register(provider)

        providers = registry.get_providers(ViewType.RAW)
        assert provider in providers

    def test_get_empty_providers(self, registry: ViewRegistry):
        providers = registry.get_providers(ViewType.SKELETON)
        assert providers == []

    def test_find_provider(self, registry: ViewRegistry, tmp_path: Path):
        provider = RawViewProvider()
        registry.register(provider)

        f = tmp_path / "test.py"
        f.write_text("test")
        target = ViewTarget(path=f)

        found = registry.find_provider(target, ViewType.RAW)
        assert found is provider

    def test_find_provider_not_found(self, registry: ViewRegistry, tmp_path: Path):
        f = tmp_path / "test.py"
        f.write_text("test")
        target = ViewTarget(path=f)

        found = registry.find_provider(target, ViewType.SKELETON)
        assert found is None

    def test_suggest_views_explore(self, registry: ViewRegistry):
        views = registry.suggest_views(Intent.EXPLORE)
        assert ViewType.SKELETON in views
        assert ViewType.DEPENDENCY in views

    def test_suggest_views_debug(self, registry: ViewRegistry):
        views = registry.suggest_views(Intent.DEBUG)
        assert ViewType.CFG in views
        assert ViewType.RAW in views

    async def test_render(self, registry: ViewRegistry, tmp_path: Path):
        registry.register(RawViewProvider())

        f = tmp_path / "test.py"
        f.write_text("hello")
        target = ViewTarget(path=f)

        view = await registry.render(target, ViewType.RAW)
        assert view is not None
        assert view.content == "hello"

    async def test_render_no_provider(self, registry: ViewRegistry, tmp_path: Path):
        f = tmp_path / "test.py"
        f.write_text("hello")
        target = ViewTarget(path=f)

        view = await registry.render(target, ViewType.SKELETON)
        assert view is None

    async def test_render_multi(self, registry: ViewRegistry, tmp_path: Path):
        registry.register(RawViewProvider())

        f = tmp_path / "test.py"
        f.write_text("hello")
        target = ViewTarget(path=f)

        views = await registry.render_multi(target, [ViewType.RAW, ViewType.SKELETON])
        assert len(views) == 1  # Only RAW has a provider
        assert views[0].view_type == ViewType.RAW


class TestDefaultRegistry:
    """Tests for default registry creation."""

    def test_create_default_registry(self):
        registry = create_default_registry()
        providers = registry.get_providers(ViewType.RAW)
        assert len(providers) == 1
        assert isinstance(providers[0], RawViewProvider)
