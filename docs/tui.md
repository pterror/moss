# TUI Development Notes

Textual framework conventions and lessons learned.

## Markup Escaping

Textual uses Rich markup but with its own escaping rules:
- Escape literal brackets with `\[` not `[[` (Rich style) or `rich.markup.escape()`
- Example: `\[Q]uit` displays as `[Q]uit`

## Actions and Click Handlers

- `@click=app.foo` calls method `action_foo` on the app
- Always use `action_` prefix for methods called via `@click`
- Action methods can take string arguments: `@click=app.cd_to('/path')`

## Styling Clickable Text

- Bold/underline markup inside `@click` regions splits the hover background
- Don't style text inside click handlers: `[@click=app.quit]Quit[/]` not `[@click=app.quit][b]Q[/b]uit[/]`
- Each markup tag creates a separate hover region

## Theme System

- Use `self.theme` for Textual's built-in theme system
- Don't hardcode theme names like "github-dark"
- Watch theme changes with `watch_theme(self, theme: str)` method

## Tree Widget

- `on_tree_node_selected` fires on single click/enter, not double-click
- Implement double-click manually with timestamp tracking
- `on_tree_node_highlighted` fires on cursor movement (hover/arrow keys)
- Indentation controlled by `guide_depth` property (default 4), not CSS
- Set `self.guide_depth = 2` in `__init__` for minimal indent

## Command Palette

- Built-in action is `command_palette` not `action_command_palette`
- Add custom commands via `get_system_commands(self, screen)` method
- Yields `SystemCommand(title, help, callback)` from `textual.app`
- NOT `DiscoveryHit` - that's for search providers
- CSS selectors: `CommandInput` (not `Input`), `#--container`, `#--input`, `#--results`

## Markdown Files

- ViewAPI doesn't handle markdown symbols - need TUI-side handling
- Extract headings as pseudo-symbols with `kind="heading"`
- Build nested tree based on heading levels (h1 > h2 > h3)
- Store symbol object when selected for later use (can't recover from path string)
- Show section content from heading line to next same-or-higher level heading

## Modal Keybinds (NEEDS DESIGN)

Future feature: context-aware keybinds that change based on mode or selection.

### Current Architecture

Keybinds are global class-level `BINDINGS` on `MossTUI` (tui.py:860-874):
```python
BINDINGS: ClassVar[list[Binding]] = [
    Binding("q", "quit", "Quit"),
    Binding("v", "primitive_view", "View"),
    # ... all bindings defined here, globally
]
```

Footer (`KeybindBar` widget, lines 347-393):
- Iterates `self.app.BINDINGS`, filters by `binding.show=True`
- Transforms key names ("minus" → "-", "slash" → "/")
- Renders as clickable text: `\[Q]uit`, `\[-] Up`
- NOT mode-aware - shows same bindings regardless of mode

Modes (lines 60-194) implement `TUIMode` protocol:
- Define `name`, `color`, `placeholder`
- Call `on_enter()` to reconfigure UI layout
- NO provision for mode-specific keybinds

### Contexts Where Keybinds Could Differ

1. **Mode**: Different bindings for Explore vs Diff vs Plan mode
2. **Node type**: Different for file vs directory vs symbol selection
3. **Input focus**: Different when command input is active vs tree navigation
4. **View state**: Different for expanded vs collapsed views

### Design Questions

1. How to define mode-specific bindings? Property on TUIMode? Separate registry?
2. How to merge/override? Mode bindings extend or replace global?
3. How to update footer reactively when context changes?
4. How to handle conflicts between contexts?

### Potential Implementation

```python
class TUIMode(Protocol):
    @property
    def bindings(self) -> list[Binding]:
        """Mode-specific bindings (extend global)."""
        ...

class MossTUI(App):
    @property
    def active_bindings(self) -> list[Binding]:
        """Computed: global + mode + context bindings."""
        ...
```

Footer would watch `active_bindings` and re-render on change.

### References

- Blender-style modal keybinds (mode-specific keys)
- Vim-style modal editing (normal/insert/visual modes)
- VS Code keybind contexts ("when" clauses)
