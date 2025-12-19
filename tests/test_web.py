"""Tests for token-efficient web fetching and search (web.py)."""

from moss.web import (
    ContentExtractor,
    SearchResult,
    SearchResults,
    WebContent,
)


class TestWebContent:
    """Tests for WebContent dataclass."""

    def test_token_estimate(self):
        content = WebContent(url="https://example.com", text="Hello world test")
        assert content.token_estimate == 4  # 16 chars / 4

    def test_to_dict(self):
        content = WebContent(
            url="https://example.com",
            title="Test",
            text="Content",
            summary="Sum",
            metadata={"key": "value"},
        )
        d = content.to_dict()
        assert d["url"] == "https://example.com"
        assert d["title"] == "Test"
        assert d["text"] == "Content"
        assert d["summary"] == "Sum"
        assert d["metadata"] == {"key": "value"}

    def test_from_dict(self):
        d = {
            "url": "https://test.com",
            "title": "Title",
            "text": "Text content",
            "summary": "",
            "metadata": {},
            "fetched_at": 12345.0,
        }
        content = WebContent.from_dict(d)
        assert content.url == "https://test.com"
        assert content.title == "Title"
        assert content.text == "Text content"
        assert content.fetched_at == 12345.0


class TestSearchResults:
    """Tests for SearchResults."""

    def test_token_estimate(self):
        results = SearchResults(
            query="test query",
            results=[
                SearchResult(title="Result 1", url="https://a.com", snippet="Snip1"),
                SearchResult(title="Result 2", url="https://b.com", snippet="Snip2"),
            ],
        )
        # Rough estimate: query + titles + urls + snippets / 4
        assert results.token_estimate > 0

    def test_to_compact(self):
        results = SearchResults(
            query="test query",
            results=[
                SearchResult(title="Result 1", url="https://a.com", snippet="Snippet 1", rank=1),
            ],
        )
        compact = results.to_compact()
        assert "Query: test query" in compact
        assert "Result 1" in compact
        assert "https://a.com" in compact


class TestContentExtractor:
    """Tests for ContentExtractor."""

    def test_extract_basic(self):
        html = "<html><head><title>Test</title></head><body><p>Hello</p></body></html>"
        extractor = ContentExtractor()
        content = extractor.extract(html, "https://test.com")
        assert content.title == "Test"
        assert "Hello" in content.text

    def test_extract_removes_script(self):
        html = """
        <html>
        <body>
        <p>Keep this</p>
        <script>ignore this</script>
        </body>
        </html>
        """
        extractor = ContentExtractor()
        content = extractor.extract(html, "")
        assert "Keep this" in content.text
        assert "ignore this" not in content.text

    def test_extract_removes_nav_footer(self):
        html = """
        <html>
        <body>
        <nav>Navigation</nav>
        <main><p>Main content</p></main>
        <footer>Footer</footer>
        </body>
        </html>
        """
        extractor = ContentExtractor()
        content = extractor.extract(html, "")
        assert "Main content" in content.text
        assert "Navigation" not in content.text
        assert "Footer" not in content.text

    def test_extract_cleans_whitespace(self):
        html = """
        <html>
        <body>
        <p>Line 1</p>



        <p>Line 2</p>
        </body>
        </html>
        """
        extractor = ContentExtractor()
        content = extractor.extract(html, "")
        # Should not have more than 2 consecutive newlines
        assert "\n\n\n" not in content.text

    def test_regex_fallback(self):
        html = (
            "<html><head><title>Test</title></head>"
            "<body><p>Content</p><script>js</script></body></html>"
        )
        extractor = ContentExtractor()
        # Force regex extraction
        content = extractor._extract_with_regex(html, "https://test.com")
        assert content.title == "Test"
        assert "Content" in content.text
        assert "js" not in content.text


class TestSearchResult:
    """Tests for SearchResult."""

    def test_to_dict(self):
        result = SearchResult(
            title="Test Result",
            url="https://example.com",
            snippet="A snippet",
            rank=1,
        )
        d = result.to_dict()
        assert d["title"] == "Test Result"
        assert d["url"] == "https://example.com"
        assert d["snippet"] == "A snippet"
        assert d["rank"] == 1
