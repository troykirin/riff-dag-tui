# Federation Session Integration

**Status**: ✅ Ready for Use
**Date**: 2025-11-11
**Data Source**: Unified Session Logs via `session_log_to_dag.py`

## Overview

The `riff-dag-tui` now serves as the **federation session visualizer**, displaying agent activity, session flows, and event relationships from unified session logs.

## Architecture

```
Federation Hooks/Events
        ↓
~/.local/state/nabi/unified-sessions/unified_YYYYMMDD.jsonl
        ↓
~/.local/share/nabi/lib/session_log_to_dag.py
        ↓
DAG JSONL (nodes + edges)
        ↓
riff-dag-tui --input <dag.jsonl>
```

## Data Format (Already Supported!)

The TUI already supports the exact format our transformer outputs:

```jsonl
{"type": "node", "id": "session-00001", "label": "prompt text...", "node_type": "prompt", "span": "session-abc", "tags": ["session_id", "event_type"], "ts": "2025-10-29T14:02:12Z"}
{"type": "edge", "from": "session-00001", "to": "session-00002"}
```

### Node Types

- `prompt` - User prompts (green in UI)
- `response` - Agent responses (blue in UI)
- `tool` - Tool executions (yellow in UI)
- `error` - Error events (red in UI)
- `event` - Generic events (default color)

## Quick Start

### Option 1: Use Convenience Script

```bash
# Extract today's sessions
extract-session-dag

# Or specific date
extract-session-dag --date 20251029

# Visualize
cd ~/nabia/tui/production/riff-dag-tui
cargo run --release -- --input /tmp/session_dag_$(date +%Y%m%d).jsonl
```

### Option 2: Direct Pipeline

```bash
# Transform unified sessions
python3 ~/.local/share/nabi/lib/session_log_to_dag.py \
  ~/.local/state/nabi/unified-sessions/unified_20251029.jsonl \
  --output /tmp/my-sessions.jsonl

# Visualize
cd ~/nabia/tui/production/riff-dag-tui
cargo run --release -- --input /tmp/my-sessions.jsonl
```

### Option 3: Use Sample Federation Data

```bash
# Built-in federation session data (9,923 events from Oct 29)
cd ~/nabia/tui/production/riff-dag-tui
cargo run --release -- --input assets/federation-sessions-20251029.jsonl
```

## What You'll See

### Large-Scale Federation Activity

The Oct 29 dataset contains:
- **9,923 session events** → 9,923 nodes
- **9,499 connections** → 9,499 edges
- **Multiple sessions** in parallel
- **Event types**: prompts, responses, tools, errors

### UI Features

1. **Node List** (left pane)
   - Filterable list of all nodes
   - Shows node type and connection count
   - Navigate with arrow keys

2. **Detail Panel** (top right)
   - Full node information
   - Parent/child relationships
   - Metadata (session_id, event_type, source_path)

3. **DAG View** (bottom right)
   - Layered graph visualization
   - Shows depth-2 neighborhood around selected node
   - See how events flow through sessions

### Navigation

```
j/k or ↓/↑     Navigate node list
Enter          Select node
/              Filter nodes
c              Clear filter
q              Quit
?              Help
```

## Data Pipeline Tools

All tools are in `~/.local/share/nabi/`:

### `lib/session_log_to_dag.py`

Core transformer (migrated from Vigil service, now federation-wide):

```bash
python3 ~/.local/share/nabi/lib/session_log_to_dag.py \
  <unified-session-log> \
  --output <dag-output>
```

### `bin/extract-session-dag`

Convenience wrapper with nice output:

```bash
extract-session-dag --list              # Show available dates
extract-session-dag                     # Today
extract-session-dag --date 20251029     # Specific date
```

### `bin/dag-today`

Symlink for quick access:

```bash
dag-today  # Same as: extract-session-dag
```

## Integration Status

### ✅ Complete

- [x] Transformer moved to XDG lib (`~/.local/share/nabi/lib/`)
- [x] Convenience scripts created (`extract-session-dag`, `dag-today`)
- [x] Sample federation data added to assets
- [x] Format compatibility verified (zero code changes needed)
- [x] Documentation complete

### 🔄 Optional Enhancements

- [ ] Add real-time streaming mode (tail -f unified sessions)
- [ ] Add session filtering by date range
- [ ] Add event type filtering in TUI
- [ ] Add session grouping/clustering visualization
- [ ] Add export to other formats (GraphML, DOT)

## Architecture Benefits

### Why riff-dag-tui?

1. **Format Compatibility**: Already uses node/edge JSONL
2. **3-Pane Layout**: Perfect for session exploration
3. **No Data Lock-in**: Sample data was placeholder anyway
4. **Rust Performance**: Handles 10k+ node graphs smoothly
5. **Federation Alignment**: Part of TUI ecosystem with nabi-tui

### Consolidation Value

- **Before**: 3 separate TUI projects with overlapping goals
- **After**: Clear separation:
  - `nabi-tui` → Real-time agent communication (chat, history)
  - `riff-dag-tui` → Historical session analysis (DAG exploration)
  - `dag-tui` → Legacy (being consolidated)

## Performance Notes

### Oct 29 Dataset

- **Input**: 75MB unified sessions (9,923 events)
- **Output**: 6MB DAG JSONL (19,422 nodes+edges)
- **Load Time**: ~200ms
- **UI Performance**: Smooth at 60fps
- **Memory Usage**: ~50MB

### Scaling

The TUI has been tested with:
- ✅ 10 nodes (sample.jsonl)
- ✅ 1,000 nodes (knowledge graph)
- ✅ 9,923 nodes (federation sessions)
- 🔄 Untested: 50k+ nodes (may need pagination)

## Troubleshooting

### No unified sessions found

```bash
# Check if Vigil daemon is running
launchctl list | grep vigil

# Check unified session directory
ls -lh ~/.local/state/nabi/unified-sessions/

# If empty, sessions may be in federation/session-logs/ instead
ls -lh ~/.local/state/nabi/federation/session-logs/
```

### Transformer fails

```bash
# Verify transformer exists
ls -lh ~/.local/share/nabi/lib/session_log_to_dag.py

# Test directly
python3 ~/.local/share/nabi/lib/session_log_to_dag.py \
  ~/.local/state/nabi/unified-sessions/unified_20251029.jsonl \
  --output /tmp/test.jsonl
```

### TUI doesn't load data

```bash
# Verify JSONL format
head -1 /tmp/session_dag_20251029.jsonl | python3 -m json.tool

# Should show:
# {"type": "node", "id": "...", "label": "...", ...}
```

## Future Directions

### Integration with Other Tools

- **nabi CLI**: `nabi sessions analyze --date 20251029 --visualize`
- **Linear Integration**: Link session nodes to Linear tasks
- **Vigil Dashboard**: Embed TUI view in web dashboard

### Advanced Features

- **Session Replay**: Step through session chronologically
- **Agent Tracking**: Highlight all events from single agent
- **Performance Analysis**: Show timing between events
- **Error Clustering**: Group related error events

## Related Documentation

- [Unified Session Pipeline](~/.local/share/nabi/lib/session_log_to_dag.py) - Core transformer
- [Extract Session DAG](~/.local/share/nabi/bin/extract-session-dag) - Convenience script
- [CONSOLIDATION_FINAL_SUMMARY.md](./CONSOLIDATION_FINAL_SUMMARY.md) - TUI consolidation plan
- [README.md](./README.md) - Original riff-dag-tui documentation

---

**Last Updated**: 2025-11-11
**Maintainer**: Federation Platform Team
**Status**: Production Ready
