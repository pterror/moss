# TUI Design

Conceptual model for the Moss TUI. See `tui.md` for Textual implementation notes.

## Core Concepts

### Two Trees

The TUI presents two navigable trees:

1. **Code tree** - files, symbols, dependencies (unified, already exists)
2. **Task tree** - all work: user sessions and agent tasks

These are the primary navigation structures. Everything else is a view on one of these trees.

### Tasks (Unified)

Everything is a **task**:
- User session = task (user-driven)
- Agent workflow = task (LLM-driven)
- Both can have subtasks (tree structure)
- Both accumulate changes (shadow branch)
- Both are persistent forever

No separate concepts for "session", "workflow", "swarm". Just tasks, some driven by users, some by agents, some running in parallel.

Each task has:
- Subtasks (child nodes in task tree)
- Shadow branch (changes made during this task)
- Status (active, paused, completed)
- Parent task (if spawned from another)

### Shadow Branches

Shadow branches are persistent change sets:
- Each task gets a shadow branch: `shadow/task-{id}`
- Changes are never deleted - they're history
- Branches are accessible via their task in the task tree
- View, compare, merge, cherry-pick from any task's changes

Shadow branches live in `.git`, visible across all worktrees.

### Worktrees for Parallelism

Git worktrees enable parallel file system access:
- Spawn parallel tasks in separate worktrees
- Each worktree can have active tasks
- All shadow branches visible from any worktree
- Standard git merge to combine results

## TUI Structure

### Navigation

- **Code tree** on left (files, symbols)
- **Task tree** on left (switchable with code tree)
- **Content area** on right (shows detail of selected item)
- **Footer** shows current context and available actions

No command input. Direct manipulation only.

### Panels

Top-level panels (Tab to cycle):
- **Code** - navigate codebase
- **Analysis** - codebase queries (complexity, security, imports, etc.)
- **Tasks** - all work (active, paused, completed)

Analysis sub-views (within Analysis panel):
- Complexity
- Security
- Scopes
- Imports
- Health

Each sub-view shows results of that analysis type for current selection or codebase.

### Actions

Contextual actions shown in footer, triggered by key:
- Actions depend on current panel and selection
- No modes - just context-aware actions

Examples:
- In Code panel with file selected: `e` edit, navigate with arrows
- In Tasks panel with task selected: `Enter` resume, `d` view diff
- In Analysis/Complexity: navigate to complex items

### Diff View

Diff is accessed through a task's changes:
- Select a task in task tree
- See its diff (shadow branch vs base)
- Actions: revert hunk, accept hunk, merge to main

Not a separate "Diff mode" - it's viewing a task's changes.

## Example Workflows

### Codebase Health → Parallel Refactoring

1. Open TUI, Tab to Analysis panel
2. View Complexity sub-view → see ranked list of complex files
3. Select top 3 complex files
4. For each: spawn task (creates shadow branch, optionally worktree)
5. Tab to Tasks panel → see three active tasks
6. Tasks run in parallel, each accumulating changes
7. When complete: select task → view diff → merge to main
8. All history preserved in shadow branches

### Resume Previous Work

1. Tab to Tasks panel
2. See all tasks (active, paused, completed)
3. Find previous task by description or date
4. Enter to resume → picks up where left off
5. Task's shadow branch has all previous changes

### View Task History

1. Tab to Tasks panel
2. Select any completed task
3. See: subtasks, changes (diff), timeline
4. Cherry-pick specific changes to current work
5. Or just read what was done and why

## Design Principles

- **No commands** - navigate and act, don't type
- **No modes** - context determines available actions
- **Everything is a task** - unified model for all work
- **Everything is persistent** - shadow branches never deleted
- **Two trees** - code and tasks, that's the whole model

## Open Questions

- How to surface "spawn parallel task in worktree" in the UI?
- Task naming/description: auto-generated or user-provided?
- How to handle conflicts when merging multiple shadow branches?
- Visualization for truly parallel independent tasks?
