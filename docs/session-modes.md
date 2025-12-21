# Session Modes

Two working modes for Claude Code sessions. User can request either explicitly.

## Fresh Mode (default)

Standard collaborative mode. Consider wrapping up when:
- Major feature complete
- 50+ tool calls
- Re-reading files (sign of context degradation)
- Conversation drifted across unrelated topics

Best for: exploratory work, design discussions, uncertain scope.

## Marathon Mode

Continuous autonomous work through TODO.md until empty or blocked.

Rules:
- Commit after each logical unit (creates resume points)
- Bail if stuck in a loop (3+ retries on same error)
- Re-reading files repeatedly = context degrading, wrap up soon
- If genuinely blocked, document state in TODO.md and stop

Best for: overnight runs, batch processing TODO items, well-defined tasks.

Trigger with: "marathon mode", "work overnight", "keep going until done"
