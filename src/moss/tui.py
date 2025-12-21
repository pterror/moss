"""TUI Interface: Interactive terminal UI for Moss.

Uses Textual for a modern, reactive terminal experience.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, ClassVar, Protocol, runtime_checkable

try:
    from textual.app import App, ComposeResult
    from textual.containers import Container, Horizontal, Vertical
    from textual.reactive import reactive
    from textual.widgets import Footer, Header, Input, Static, Tree
    from textual.widgets.tree import TreeNode
except ImportError:
    # TUI dependencies not installed
    class App:
        pass

    class ComposeResult:
        pass


if TYPE_CHECKING:
    from moss.moss_api import MossAPI
    from moss.task_tree import TaskNode, TaskTree


@runtime_checkable
class TUIMode(Protocol):
    """Protocol for TUI operating modes."""

    @property
    def name(self) -> str:
        """Mode name."""
        ...

    @property
    def color(self) -> str:
        """Mode color for indicator."""
        ...

    @property
    def placeholder(self) -> str:
        """Command input placeholder."""
        ...

    async def on_enter(self, app: MossTUI) -> None:
        """Called when entering this mode."""
        ...


class PlanMode:
    name = "PLAN"
    color = "blue"
    placeholder = "What is the plan? (e.g. breakdown...)"

    async def on_enter(self, app: MossTUI) -> None:
        app.query_one("#log-view").display = True
        app.query_one("#git-view").display = False
        app.query_one("#content-header").update("Agent Log")


class ReadMode:
    name = "READ"
    color = "green"
    placeholder = "Explore codebase... (e.g. skeleton, grep, expand)"

    async def on_enter(self, app: MossTUI) -> None:
        app.query_one("#log-view").display = True
        app.query_one("#git-view").display = False
        app.query_one("#content-header").update("Agent Log")


class WriteMode:
    name = "WRITE"
    color = "red"
    placeholder = "Modify code... (e.g. write, replace, insert)"

    async def on_enter(self, app: MossTUI) -> None:
        app.query_one("#log-view").display = True
        app.query_one("#git-view").display = False
        app.query_one("#content-header").update("Agent Log")


class DiffMode:
    name = "DIFF"
    color = "magenta"
    placeholder = "Review changes... (revert <file> <line> to undo)"

    async def on_enter(self, app: MossTUI) -> None:
        app.query_one("#log-view").display = False
        app.query_one("#git-view").display = True
        app.query_one("#content-header").update("Shadow Git")
        await app._update_git_view()


class ModeRegistry:
    """Registry for extensible TUI modes."""

    def __init__(self):
        self._modes: dict[str, TUIMode] = {
            "PLAN": PlanMode(),
            "READ": ReadMode(),
            "WRITE": WriteMode(),
            "DIFF": DiffMode(),
        }
        self._order: list[str] = ["PLAN", "READ", "WRITE", "DIFF"]

    def get_mode(self, name: str) -> TUIMode | None:
        return self._modes.get(name)

    def next_mode(self, current_name: str) -> TUIMode:
        idx = self._order.index(current_name)
        next_idx = (idx + 1) % len(self._order)
        return self._modes[self._order[next_idx]]

    def register_mode(self, mode: TUIMode) -> None:
        self._modes[mode.name] = mode
        if mode.name not in self._order:
            self._order.append(mode.name)


class ModeIndicator(Static):
    """Widget to display the current agent mode."""

    mode_name = reactive("PLAN")
    mode_color = reactive("blue")

    def render(self) -> str:
        return f"Mode: [{self.mode_color} b]{self.mode_name}[/]"


class TaskTreeWidget(Tree[str]):
    """Widget for visualizing the task tree."""

    def update_from_tree(self, task_tree: TaskTree) -> None:
        """Update the widget content from a TaskTree instance."""
        self.clear()
        root = self.root
        root.label = task_tree.root.goal
        self._add_node(root, task_tree.root)
        root.expand()

    def _add_node(self, tree_node: TreeNode[str], task_node: TaskNode) -> None:
        """Recursively add nodes to the tree widget."""
        for child in task_node.children:
            status_icon = "✓" if child.status.name == "DONE" else "→"
            label = f"{status_icon} {child.goal}"
            if child.summary:
                label += f" ({child.summary})"

            new_node = tree_node.add(label, expand=True)
            self._add_node(new_node, child)


class MossTUI(App):
    """The main Moss TUI application."""

    CSS = """
    Screen {
        background: $surface;
    }

    #main-container {
        height: 1fr;
    }

    #sidebar {
        width: 30%;
        height: 1fr;
        border-right: tall $primary;
        background: $surface-darken-1;
    }

    #content-area {
        width: 70%;
        height: 1fr;
        padding: 1;
    }

    #command-input {
        dock: bottom;
        margin: 1;
    }

    .log-entry {
        margin-bottom: 1;
        padding: 0 1;
        border-left: solid $accent;
    }

    #git-view {
        display: none;
    }

    #diff-view {
        height: 1fr;
        border: solid $secondary;
    }

    #history-tree {
        height: 30%;
        border: solid $secondary;
    }

    ModeIndicator {
        background: $surface-lighten-1;
        padding: 0 1;
        text-align: center;
        border: round $primary;
        margin: 0 1;
    }
    """

    BINDINGS: ClassVar[list[tuple[str, str, str]]] = [
        ("q", "quit", "Quit"),
        ("d", "toggle_dark", "Toggle Dark Mode"),
        ("shift+tab", "next_mode", "Next Mode"),
    ]

    current_mode_name = reactive("PLAN")

    def __init__(self, api: MossAPI):
        super().__init__()
        self.api = api
        self._task_tree: TaskTree | None = None
        self._registry = ModeRegistry()

    def compose(self) -> ComposeResult:
        """Create child widgets for the app."""
        from textual.widgets import RichLog

        yield Header(show_clock=True)
        yield Horizontal(ModeIndicator(id="mode-indicator"), id="header-bar", height="auto")
        yield Container(
            Horizontal(
                Vertical(
                    Static("Task Tree", classes="sidebar-header"),
                    TaskTreeWidget("Tasks", id="task-tree"),
                    id="sidebar",
                ),
                Vertical(
                    Static("Agent Log", id="content-header"),
                    Container(id="log-view"),
                    Container(
                        Static("Shadow Git History", classes="sidebar-header"),
                        Tree("Commits", id="history-tree"),
                        Static("Diff", classes="sidebar-header"),
                        RichLog(id="diff-view", highlight=True, markup=True),
                        id="git-view",
                    ),
                    id="content-area",
                ),
                id="main-container",
            ),
            Input(placeholder="Enter command...", id="command-input"),
        )
        yield Footer()

    def on_mount(self) -> None:
        """Called when the app is mounted."""
        self.title = "Moss TUI"
        self.sub_title = f"Project: {self.api.root.name}"
        self.query_one("#command-input").focus()
        # Initialize first mode
        self.current_mode_name = "PLAN"

    async def watch_current_mode_name(self, name: str) -> None:
        """React to mode changes."""
        mode = self._registry.get_mode(name)
        if not mode:
            return

        indicator = self.query_one("#mode-indicator")
        indicator.mode_name = mode.name
        indicator.mode_color = mode.color

        self.query_one("#command-input").placeholder = mode.placeholder

        await mode.on_enter(self)

    def action_next_mode(self) -> None:
        """Switch to the next mode."""
        next_mode = self._registry.next_mode(self.current_mode_name)
        self.current_mode_name = next_mode.name
        self._log(f"Switched to {self.current_mode_name} mode")

    async def _update_git_view(self) -> None:
        """Fetch and display shadow git data."""
        try:
            # Get current shadow branch diff
            # In a real TUI we'd track the current branch
            diff = await self.api.shadow_git.get_diff("shadow/current")
            diff_view = self.query_one("#diff-view")
            diff_view.clear()
            diff_view.write(diff)

            # Update history (hunks)
            hunks = await self.api.shadow_git.get_hunks("shadow/current")
            history = self.query_one("#history-tree")
            history.clear()
            root = history.root
            root.label = "Current Hunks"
            for hunk in hunks:
                label = f"{hunk['file_path']}:{hunk['new_start']} ({hunk['symbol'] or 'no symbol'})"
                root.add_leaf(label)
            root.expand()
        except Exception as e:
            self._log(f"Failed to fetch git data: {e}")

    async def on_input_submitted(self, event: Input.Submitted) -> None:
        """Handle command input."""
        command = event.value.strip()
        if not command:
            return

        self.query_one("#command-input").value = ""
        self._log(f"[{self.current_mode_name}] {command}")

        # TODO: Integrate with AgentLoop or DWIM
        if command == "exit":
            self.exit()

    def _log(self, message: str) -> None:
        """Add a message to the log view."""
        log_view = self.query_one("#log-view")
        log_view.mount(Static(message, classes="log-entry"))
        log_view.scroll_end()


def run_tui(api: MossAPI) -> None:
    """Run the Moss TUI."""
    try:
        from textual.app import App as _App  # noqa: F401
    except ImportError:
        print("Error: textual not installed. Install with: pip install 'moss[tui]'")
        return

    app = MossTUI(api)
    app.run()
