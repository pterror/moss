# normalize sessions

Analyze Claude Code, Codex, Gemini CLI, and Normalize agent session logs.

## Usage

```bash
normalize sessions <SUBCOMMAND> [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | List available sessions |
| `show` | Show a specific session (summary or full conversation) |
| `stats` | Show aggregate statistics across sessions |
| `messages` | Extract all messages across sessions into a flat, queryable form |
| `patterns` | Analyze tool call sequence patterns using Markov chain transition matrices |
| `plans` | List and view agent plans |
| `blame` | Trace a file's line provenance through AI/coding sessions, not just commits |

### list

List available sessions:

```bash
normalize sessions list                          # List sessions for current project
normalize sessions list --all-projects           # All projects
normalize sessions list --format codex           # Codex sessions
normalize sessions list --grep "benchmark"       # Filter by content
normalize sessions list --days 7                 # Last 7 days
normalize sessions list --since 2025-01-01       # Since date
normalize sessions list -n 50                    # Limit results
normalize sessions list --project /path/to/repo  # Specific project
normalize sessions list --mode subagent          # Subagent sessions only
normalize sessions list --agent-type Explore     # Only Explore agents
normalize sessions list --mode subagent --agent-type general-purpose  # General-purpose subagents
```

Options:
- `--format <FORMAT>` — Force specific format: `claude`, `codex`, `gemini`, `normalize`
- `--grep <PATTERN>` — Filter sessions by content pattern
- `--days <N>` — Filter sessions from the last N days
- `--since <DATE>` — Filter sessions since date (YYYY-MM-DD)
- `--until <DATE>` — Filter sessions until date (YYYY-MM-DD)
- `--project <PATH>` — Filter by specific project path
- `--all-projects` — Show sessions from all projects
- `-n, --limit <N>` — Maximum number of sessions
- `--mode <MODE>` — Session mode: `interactive` (default), `subagent`, or `all`
- `--agent-type <TYPE>` — Filter by agent type (e.g. `Explore`, `general-purpose`, `Plan`)

### show

Show a specific session:

```bash
normalize sessions show abc123                   # Session summary
normalize sessions show abc123 --analyze         # Full analysis
normalize sessions show abc123 --full            # Full conversation log
normalize sessions show abc123 --exact           # Exact/prefix match only
normalize sessions show abc123 --format codex    # Force format
```

Arguments:
- `[SESSION]` — Session ID or path

Options:
- `--analyze` — Run full analysis instead of summary
- `--full` — Show full conversation log
- `--exact` — Require exact/prefix match (disable fuzzy)
- `--format <FORMAT>` — Force specific format: `claude`, `codex`, `gemini`, `normalize`

### stats

Show aggregate statistics across sessions:

```bash
normalize sessions stats                         # Stats for current project
normalize sessions stats --all-projects          # All projects
normalize sessions stats --days 30               # Last 30 days
normalize sessions stats --format codex          # Codex sessions
```

Options: same filtering as `list` (`--format`, `--grep`, `--days`, `--since`, `--until`, `--project`, `--all-projects`, `-n`, `--mode`, `--agent-type`).

### messages

Extract all messages across sessions into a flat, queryable form:

```bash
normalize sessions messages                              # User messages (default)
normalize sessions messages --role all                   # All messages
normalize sessions messages --role assistant              # Assistant only
normalize sessions messages --grep "TODO"                # Filter by content
normalize sessions messages --no-truncate                # Full message text
normalize sessions messages --max-chars 500              # Custom truncation
normalize sessions messages --jq '.[] | select(.role == "user")'
```

Options:
- `--role <ROLE>` — Filter by role: `user` (default), `assistant`, `all`
- `--grep <PATTERN>` — Filter messages by content pattern
- `--max-chars <N>` — Truncate message text to N chars (default: 200)
- `--no-truncate` — Don't truncate message text
- Plus same filtering options as `list` (`--mode`, `--agent-type`, etc.)

### patterns

Analyze tool call sequence patterns across sessions using Markov chain transition matrices:

```bash
normalize sessions patterns                         # Patterns for current project
normalize sessions patterns --days 30               # Last 30 days
normalize sessions patterns --mode subagent         # Subagent sessions only
normalize sessions patterns --all-projects          # All projects
normalize sessions patterns --json                  # Full data as JSON
```

Shows:
- **Population transition matrix** — probability of transitioning from tool A to tool B, aggregated across all sessions
- **Most common starting/ending tools** — which tools sessions begin and end with
- **Outlier sessions** — sessions ranked by divergence from the population matrix (Frobenius norm)

Options: same filtering as `list` (`--format`, `--grep`, `--days`, `--since`, `--until`, `--project`, `--all-projects`, `-n`, `--mode`, `--agent-type`).

### plans

List and view agent plans:

```bash
normalize sessions plans                         # List all plans
normalize sessions plans my-plan                 # View specific plan
normalize sessions plans -n 10                   # Limit results
```

Arguments:
- `[NAME]` — Plan name to view (omit to list all)

Options:
- `-n, --limit <N>` — Maximum number of plans

### blame

Trace a file's line provenance through AI/coding sessions — an analog of `git
blame` that goes one hop further, attributing each blamed commit's content to
the specific session `Edit`/`Write` tool call that produced it (by matching
recorded `old_string`/`new_string` content against the commit's actual change,
not by timestamp proximity):

```bash
normalize sessions blame src/lib.rs                        # whole-file session provenance
normalize sessions blame src/lib.rs --start-line 10 --end-line 40
normalize sessions blame src/lib.rs --days 30               # only consider sessions from the last 30 days
normalize sessions blame src/lib.rs --all-projects          # search sessions across all projects
```

Arguments:
- `[PATH]` — File path to trace (repo-relative)

Options:
- `--start-line <N>` / `--end-line <N>` — Restrict to a 1-based inclusive line range
- Plus the same session filtering options as `list` (`--format`, `--grep`, `--days`, `--since`, `--until`, `--project`, `--all-projects`, `-n`, `--mode`, `--agent-type`)

Each line is attributed one of three ways:
- **Matched** — exactly one session edit's recorded content matches the commit's change; reports session ID, agent, and tool.
- **Ambiguous** — more than one session edit matches (e.g. two sessions made byte-identical changes); all candidates are listed rather than guessed between.
- **Unattributed** — no session edit matches (manual commit, pre-instrumentation history, a non-Edit/Write tool, a human touch-up after the last session edit, or content too short/generic to trust a match).

This is intentionally conservative: a human edit made after the last matching
session edit but before commit is a known false negative (correctly reported
unattributed rather than misattributed). See `normalize_sessions::blame`'s
module docs for the full algorithm and its limitations.

## Formats

| Format | Directory | File Pattern |
|--------|-----------|--------------|
| `claude` | `~/.claude/projects/<encoded-path>/` | `*.jsonl` |
| `codex` | `~/.codex/sessions/YYYY/MM/DD/` | `*.jsonl` |
| `gemini` | `~/.gemini/tmp/<hash>/` | `logs.json` |
| `normalize` | `.normalize/agent/logs/` | `*.jsonl` |
