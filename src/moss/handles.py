"""Handle System: Lazy references to files, memory, and artifacts."""

from __future__ import annotations

import asyncio
from abc import ABC, abstractmethod
from collections.abc import Callable, Coroutine
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any
from uuid import UUID, uuid4


@dataclass(frozen=True)
class HandleMetadata:
    """Metadata for a handle."""

    created_by: str | None = None
    tags: tuple[str, ...] = ()
    description: str | None = None


class Handle[T](ABC):
    """Abstract base for lazy references."""

    def __init__(self, *, metadata: HandleMetadata | None = None):
        self._id = uuid4()
        self._metadata = metadata or HandleMetadata()
        self._cached: T | None = None
        self._resolved = False

    @property
    def id(self) -> UUID:
        return self._id

    @property
    def metadata(self) -> HandleMetadata:
        return self._metadata

    @property
    def is_resolved(self) -> bool:
        return self._resolved

    @abstractmethod
    async def resolve(self) -> T:
        """Resolve the handle to its actual content."""
        ...

    async def get(self) -> T:
        """Get content, using cache if available."""
        if not self._resolved:
            self._cached = await self.resolve()
            self._resolved = True
        return self._cached  # type: ignore[return-value]

    def invalidate(self) -> None:
        """Invalidate the cache, forcing re-resolution."""
        self._cached = None
        self._resolved = False


class FileHandle(Handle[str]):
    """Lazy reference to a file's content."""

    def __init__(
        self,
        path: Path | str,
        *,
        encoding: str = "utf-8",
        metadata: HandleMetadata | None = None,
    ):
        super().__init__(metadata=metadata)
        self._path = Path(path)
        self._encoding = encoding

    @property
    def path(self) -> Path:
        return self._path

    async def resolve(self) -> str:
        """Read file content."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            lambda: self._path.read_text(encoding=self._encoding),
        )

    async def exists(self) -> bool:
        """Check if the file exists."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._path.exists)

    def __repr__(self) -> str:
        return f"FileHandle({self._path})"


class BinaryFileHandle(Handle[bytes]):
    """Lazy reference to binary file content."""

    def __init__(self, path: Path | str, *, metadata: HandleMetadata | None = None):
        super().__init__(metadata=metadata)
        self._path = Path(path)

    @property
    def path(self) -> Path:
        return self._path

    async def resolve(self) -> bytes:
        """Read binary file content."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._path.read_bytes)

    def __repr__(self) -> str:
        return f"BinaryFileHandle({self._path})"


class MemoryHandle[T](Handle[T]):
    """Lazy reference to an in-memory value."""

    def __init__(self, value: T, *, metadata: HandleMetadata | None = None):
        super().__init__(metadata=metadata)
        self._value = value

    async def resolve(self) -> T:
        """Return the stored value."""
        return self._value

    def __repr__(self) -> str:
        return f"MemoryHandle({type(self._value).__name__})"


class DeferredHandle[T](Handle[T]):
    """Handle that resolves via a callback."""

    def __init__(
        self,
        resolver: Callable[[], Coroutine[Any, Any, T]],
        *,
        metadata: HandleMetadata | None = None,
    ):
        super().__init__(metadata=metadata)
        self._resolver = resolver

    async def resolve(self) -> T:
        """Execute the resolver callback."""
        return await self._resolver()

    def __repr__(self) -> str:
        return f"DeferredHandle({self._resolver})"


@dataclass
class HandleRef:
    """Serializable reference to a handle for passing between agents."""

    handle_id: UUID
    handle_type: str
    location: str  # Path, key, or other identifier
    metadata: HandleMetadata = field(default_factory=HandleMetadata)


class HandleRegistry:
    """Registry for tracking and resolving handles."""

    def __init__(self) -> None:
        self._handles: dict[UUID, Handle[Any]] = {}

    def register(self, handle: Handle[Any]) -> UUID:
        """Register a handle and return its ID."""
        self._handles[handle.id] = handle
        return handle.id

    def get(self, handle_id: UUID) -> Handle[Any] | None:
        """Get a handle by ID."""
        return self._handles.get(handle_id)

    def remove(self, handle_id: UUID) -> bool:
        """Remove a handle from the registry."""
        if handle_id in self._handles:
            del self._handles[handle_id]
            return True
        return False

    def to_ref(self, handle: Handle[Any]) -> HandleRef:
        """Create a serializable reference from a handle."""
        if isinstance(handle, FileHandle):
            handle_type = "file"
            location = str(handle.path)
        elif isinstance(handle, BinaryFileHandle):
            handle_type = "binary_file"
            location = str(handle.path)
        elif isinstance(handle, MemoryHandle):
            handle_type = "memory"
            location = str(handle.id)
        else:
            handle_type = "deferred"
            location = str(handle.id)

        return HandleRef(
            handle_id=handle.id,
            handle_type=handle_type,
            location=location,
            metadata=handle.metadata,
        )

    def from_ref(self, ref: HandleRef) -> Handle[Any] | None:
        """Resolve a reference to its handle."""
        return self.get(ref.handle_id)

    @property
    def count(self) -> int:
        """Number of registered handles."""
        return len(self._handles)

    def clear(self) -> None:
        """Clear all registered handles."""
        self._handles.clear()
