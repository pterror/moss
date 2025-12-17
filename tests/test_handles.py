"""Tests for Handle system."""

from pathlib import Path

import pytest

from moss.handles import (
    BinaryFileHandle,
    DeferredHandle,
    FileHandle,
    HandleMetadata,
    HandleRegistry,
    MemoryHandle,
)


class TestFileHandle:
    """Tests for FileHandle."""

    @pytest.fixture
    def text_file(self, tmp_path: Path) -> Path:
        f = tmp_path / "test.txt"
        f.write_text("hello world")
        return f

    async def test_resolve_file(self, text_file: Path):
        handle = FileHandle(text_file)

        content = await handle.get()

        assert content == "hello world"

    async def test_lazy_loading(self, text_file: Path):
        handle = FileHandle(text_file)

        assert not handle.is_resolved

        await handle.get()

        assert handle.is_resolved

    async def test_caching(self, text_file: Path):
        handle = FileHandle(text_file)

        first = await handle.get()
        # Modify file after first read
        text_file.write_text("modified")
        second = await handle.get()

        # Should return cached value
        assert first == second == "hello world"

    async def test_invalidate_cache(self, text_file: Path):
        handle = FileHandle(text_file)

        await handle.get()
        text_file.write_text("modified")
        handle.invalidate()
        content = await handle.get()

        assert content == "modified"

    async def test_exists(self, text_file: Path, tmp_path: Path):
        handle = FileHandle(text_file)
        missing_handle = FileHandle(tmp_path / "missing.txt")

        assert await handle.exists()
        assert not await missing_handle.exists()

    async def test_path_property(self, text_file: Path):
        handle = FileHandle(text_file)

        assert handle.path == text_file

    async def test_encoding(self, tmp_path: Path):
        f = tmp_path / "unicode.txt"
        f.write_text("привет мир", encoding="utf-8")

        handle = FileHandle(f, encoding="utf-8")
        content = await handle.get()

        assert content == "привет мир"


class TestBinaryFileHandle:
    """Tests for BinaryFileHandle."""

    async def test_resolve_binary(self, tmp_path: Path):
        f = tmp_path / "test.bin"
        data = b"\x00\x01\x02\xff"
        f.write_bytes(data)

        handle = BinaryFileHandle(f)
        content = await handle.get()

        assert content == data


class TestMemoryHandle:
    """Tests for MemoryHandle."""

    async def test_resolve_memory(self):
        handle = MemoryHandle({"key": "value"})

        content = await handle.get()

        assert content == {"key": "value"}

    async def test_resolve_list(self):
        handle = MemoryHandle([1, 2, 3])

        content = await handle.get()

        assert content == [1, 2, 3]

    async def test_always_resolved_immediately(self):
        handle = MemoryHandle("test")

        # MemoryHandle should resolve instantly
        content = await handle.get()

        assert content == "test"
        assert handle.is_resolved


class TestDeferredHandle:
    """Tests for DeferredHandle."""

    async def test_resolve_deferred(self):
        call_count = 0

        async def resolver():
            nonlocal call_count
            call_count += 1
            return "resolved"

        handle = DeferredHandle(resolver)

        content = await handle.get()

        assert content == "resolved"
        assert call_count == 1

    async def test_caches_result(self):
        call_count = 0

        async def resolver():
            nonlocal call_count
            call_count += 1
            return "resolved"

        handle = DeferredHandle(resolver)

        await handle.get()
        await handle.get()

        assert call_count == 1


class TestHandleMetadata:
    """Tests for HandleMetadata."""

    async def test_default_metadata(self, tmp_path: Path):
        f = tmp_path / "test.txt"
        f.write_text("test")
        handle = FileHandle(f)

        assert handle.metadata.created_by is None
        assert handle.metadata.tags == ()

    async def test_custom_metadata(self, tmp_path: Path):
        f = tmp_path / "test.txt"
        f.write_text("test")
        meta = HandleMetadata(
            created_by="agent-1",
            tags=("important", "reviewed"),
            description="Test file",
        )
        handle = FileHandle(f, metadata=meta)

        assert handle.metadata.created_by == "agent-1"
        assert handle.metadata.tags == ("important", "reviewed")
        assert handle.metadata.description == "Test file"


class TestHandleRegistry:
    """Tests for HandleRegistry."""

    @pytest.fixture
    def registry(self):
        return HandleRegistry()

    async def test_register_and_get(self, registry: HandleRegistry):
        handle = MemoryHandle("test")
        handle_id = registry.register(handle)

        retrieved = registry.get(handle_id)

        assert retrieved is handle

    async def test_get_missing(self, registry: HandleRegistry):
        from uuid import uuid4

        result = registry.get(uuid4())

        assert result is None

    async def test_remove(self, registry: HandleRegistry):
        handle = MemoryHandle("test")
        handle_id = registry.register(handle)

        removed = registry.remove(handle_id)

        assert removed
        assert registry.get(handle_id) is None

    async def test_remove_missing(self, registry: HandleRegistry):
        from uuid import uuid4

        removed = registry.remove(uuid4())

        assert not removed

    async def test_count(self, registry: HandleRegistry):
        assert registry.count == 0

        registry.register(MemoryHandle("one"))
        registry.register(MemoryHandle("two"))

        assert registry.count == 2

    async def test_clear(self, registry: HandleRegistry):
        registry.register(MemoryHandle("one"))
        registry.register(MemoryHandle("two"))

        registry.clear()

        assert registry.count == 0

    async def test_to_ref_file_handle(self, registry: HandleRegistry, tmp_path: Path):
        f = tmp_path / "test.txt"
        f.write_text("test")
        handle = FileHandle(f, metadata=HandleMetadata(created_by="test"))
        registry.register(handle)

        ref = registry.to_ref(handle)

        assert ref.handle_id == handle.id
        assert ref.handle_type == "file"
        assert ref.location == str(f)
        assert ref.metadata.created_by == "test"

    async def test_to_ref_binary_handle(self, registry: HandleRegistry, tmp_path: Path):
        f = tmp_path / "test.bin"
        f.write_bytes(b"test")
        handle = BinaryFileHandle(f)
        registry.register(handle)

        ref = registry.to_ref(handle)

        assert ref.handle_type == "binary_file"

    async def test_to_ref_memory_handle(self, registry: HandleRegistry):
        handle = MemoryHandle("test")
        registry.register(handle)

        ref = registry.to_ref(handle)

        assert ref.handle_type == "memory"

    async def test_from_ref(self, registry: HandleRegistry):
        handle = MemoryHandle("test")
        registry.register(handle)
        ref = registry.to_ref(handle)

        resolved = registry.from_ref(ref)

        assert resolved is handle
