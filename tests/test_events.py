"""Tests for the Event Bus."""

import asyncio

import pytest

from moss.events import Event, EventBus, EventType


class TestEvent:
    """Tests for Event dataclass."""

    def test_event_has_id(self):
        event = Event(type=EventType.USER_MESSAGE)
        assert event.id is not None

    def test_event_has_timestamp(self):
        event = Event(type=EventType.USER_MESSAGE)
        assert event.timestamp is not None

    def test_event_immutable(self):
        event = Event(type=EventType.USER_MESSAGE)
        with pytest.raises(AttributeError):
            event.type = EventType.TOOL_CALL  # type: ignore[misc]

    def test_event_with_payload(self):
        payload = {"message": "hello"}
        event = Event(type=EventType.USER_MESSAGE, payload=payload)
        assert event.payload == payload


class TestEventBus:
    """Tests for EventBus."""

    @pytest.fixture
    def bus(self):
        return EventBus()

    async def test_subscribe_and_publish(self, bus: EventBus):
        received: list[Event] = []

        async def handler(event: Event):
            received.append(event)

        bus.subscribe(EventType.USER_MESSAGE, handler)
        await bus.emit(EventType.USER_MESSAGE, {"text": "hello"})

        assert len(received) == 1
        assert received[0].payload["text"] == "hello"

    async def test_subscribe_specific_type(self, bus: EventBus):
        received: list[Event] = []

        async def handler(event: Event):
            received.append(event)

        bus.subscribe(EventType.USER_MESSAGE, handler)
        await bus.emit(EventType.TOOL_CALL)  # Different type

        assert len(received) == 0

    async def test_subscribe_all(self, bus: EventBus):
        received: list[Event] = []

        async def handler(event: Event):
            received.append(event)

        bus.subscribe_all(handler)
        await bus.emit(EventType.USER_MESSAGE)
        await bus.emit(EventType.TOOL_CALL)

        assert len(received) == 2

    async def test_unsubscribe(self, bus: EventBus):
        received: list[Event] = []

        async def handler(event: Event):
            received.append(event)

        unsubscribe = bus.subscribe(EventType.USER_MESSAGE, handler)
        await bus.emit(EventType.USER_MESSAGE)
        unsubscribe()
        await bus.emit(EventType.USER_MESSAGE)

        assert len(received) == 1

    async def test_multiple_handlers(self, bus: EventBus):
        results: list[str] = []

        async def handler1(event: Event):
            results.append("h1")

        async def handler2(event: Event):
            results.append("h2")

        bus.subscribe(EventType.USER_MESSAGE, handler1)
        bus.subscribe(EventType.USER_MESSAGE, handler2)
        await bus.emit(EventType.USER_MESSAGE)

        assert set(results) == {"h1", "h2"}

    async def test_history(self, bus: EventBus):
        await bus.emit(EventType.USER_MESSAGE)
        await bus.emit(EventType.TOOL_CALL)

        assert len(bus.history()) == 2
        assert len(bus.history(EventType.USER_MESSAGE)) == 1
        assert len(bus.history(EventType.TOOL_CALL)) == 1

    async def test_clear_history(self, bus: EventBus):
        await bus.emit(EventType.USER_MESSAGE)
        bus.clear_history()

        assert len(bus.history()) == 0

    async def test_concurrent_handlers(self, bus: EventBus):
        """Handlers run concurrently."""
        order: list[int] = []

        async def slow_handler(event: Event):
            await asyncio.sleep(0.1)
            order.append(1)

        async def fast_handler(event: Event):
            order.append(2)

        bus.subscribe(EventType.USER_MESSAGE, slow_handler)
        bus.subscribe(EventType.USER_MESSAGE, fast_handler)
        await bus.emit(EventType.USER_MESSAGE)

        # Fast handler should finish first due to concurrency
        assert order == [2, 1]

    async def test_handler_exception_does_not_block(self, bus: EventBus):
        """Handler exceptions don't block other handlers."""
        received: list[Event] = []

        async def bad_handler(event: Event):
            raise ValueError("oops")

        async def good_handler(event: Event):
            received.append(event)

        bus.subscribe(EventType.USER_MESSAGE, bad_handler)
        bus.subscribe(EventType.USER_MESSAGE, good_handler)
        await bus.emit(EventType.USER_MESSAGE)

        assert len(received) == 1
