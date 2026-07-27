import os
import sys
from collections import defaultdict
from typing import Optional, List, Dict
from dataclasses import dataclass, field


class DataProcessor:
    """Process data items."""

    def __init__(self, name: str):
        self.name = name
        self.items: List[str] = []

    def add(self, item: str) -> None:
        self.items.append(item)

    # Process all items
    @property
    def process(self) -> List[str]:
        result = []
        for item in self.items:
            if item.startswith("_"):
                continue
            result.append(item.upper())
        return result

    def make_adder(self, base: int):
        # Closure: nested function capturing an enclosing-scope variable.
        # `base` and `x` must never appear in tags as top-level definitions.
        def adder(x: int) -> int:
            return base + x

        return adder


@dataclass
class Config:
    name: str
    values: Dict[str, int] = field(default_factory=dict)
    tags: List[str] = field(default_factory=list)


class Cache(dict):
    """Multiple inheritance and a generic-ish base."""


class LoggingCache(Cache, DataProcessor):
    pass


def load_file(path: str) -> Optional[str]:
    if not os.path.exists(path):
        return None
    with open(path) as f:
        return f.read()


def count_words(text: str) -> dict:
    counts = defaultdict(int)
    for word in text.split():
        counts[word] += 1
    return dict(counts)


def app_route(path):
    def decorator(fn):
        return fn

    return decorator


@app_route("/status")
@staticmethod
def status_handler():
    return {"ok": True}


async def fetch_all(urls: List[str]) -> List[str]:
    results = []
    for url in urls:
        # Iterator-chain idiom: filter then transform then materialize.
        result = await fetch_one(url)
        results.append(result)
    filtered = [r.upper() for r in results if r]
    return filtered


async def fetch_one(url: str) -> str:
    return url


def parse_all(values: List[str]) -> List[int]:
    return [int(v) for v in values if v.isdigit()]


def dispatch(event: str, handlers: dict):
    # Subscript-dispatched call idiom (command/event-routing pattern).
    return handlers[event]()


def summarize(items: List[int]) -> str:
    if (n := len(items)) > 10:
        return f"large batch of {n!r} items"
    return f"small batch of {n} items"


def classify(command: list):
    match command:
        case ["add", a, b]:
            return a + b
        case ["sub", a, b]:
            return a - b
        case {"op": op, **rest}:
            return op, rest
        case _:
            return None


def make_id_generator():
    counter = 0

    def next_id() -> int:
        nonlocal counter
        counter += 1
        return counter

    return next_id
