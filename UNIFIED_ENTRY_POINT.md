# Unified Riff Entry Point Architecture

**Purpose**: Design the consolidated riff toolkit as a single unified CLI with multiple modes
**Status**: Architecture Design
**Date**: 2025-11-08

---

## Current State: Fragmented

```
User wants to search conversations
    ↓
Opens riff-cli
    ↓
Runs: riff search "query"
    ↓
Gets text results
    ↓
Manual export to JSONL
    ↓
Manually opens riff-dag-tui
    ↓
Runs: riff-dag-tui --input file.jsonl
    ↓
Explores DAG
```

**Problem**: Two separate entry points, manual handoff, context loss

---

## Desired State: Unified

```
User wants to search and explore conversations
    ↓
Opens riff (unified entry point)
    ↓
Runs: riff search "query" --visualize
    ↓
Automatically opens interactive DAG view
    ↓
Explores results with full search context
    ↓
Seamless workflow
```

---

## Unified CLI Architecture

### Single Binary Entry Point

```bash
# Current state (fragmented)
riff-cli search "memory"
riff-dag-tui --input results.jsonl

# Unified state
riff search "memory" --visualize     # Same command, enhanced
riff explore --from-file results.jsonl
```

### Command Hierarchy

```
riff (main command)
├── search              # Semantic search (riff-cli primary)
│   ├── --visualize     # NEW: Show results in DAG view
│   ├── --export FILE   # Export to JSONL
│   ├── --days N        # Time filtering
│   └── --ai            # AI query enhancement
├── repair              # JSONL repair (riff-cli original)
│   ├── scan
│   └── fix
├── explore             # NEW: Direct visualization
│   ├── --from-file     # Load JSONL file
│   ├── --from-search   # Use search results
│   └── --layout        # Layout options (3-pane, graph, etc.)
├── graph               # Generate graphs (riff-cli original)
│   ├── --format        # Mermaid, DOT, etc.
│   └── --output
└── tui                 # Interactive TUI (riff-cli original)
```

### Implementation Pattern

All subcommands delegate to appropriate backends:

```python
@app.command()
def search(
    query: str,
    visualize: bool = typer.Option(False, "--visualize", help="Open in DAG viewer"),
    export: str = typer.Option(None, "--export", help="Export results to JSONL"),
    days: int = typer.Option(None, "--days", help="Filter by days"),
    ai: bool = typer.Option(False, "--ai", help="AI-enhanced search"),
):
    """Search conversations semantically."""
    # Execute riff-cli search logic
    results = semantic_search(query, days=days, ai_enhance=ai)

    if export:
        # Export to JSONL
        export_to_jsonl(results, export)
        typer.echo(f"✅ Exported to {export}")

    if visualize:
        # Automatically spawn visualization
        jsonl_data = _convert_to_dag_format(results)
        temp_file = _write_temp_jsonl(jsonl_data)
        launch_visualization(temp_file)
        # Cleanup after visualization exits
        temp_file.unlink()

    return results
```

---

## Two Implementation Pathways

### Pathway 1: Gradual Integration (Recommended)

**Current**: Two separate tools
**Phase 1** (Week 1): Add `--visualize` flag to riff search
**Phase 2** (Week 2): Create `riff explore` command
**Phase 3** (Week 3): Unified documentation and federation registration
**Final**: Transparent to end user (they use `riff` for everything)

**Advantages**:
- ✅ Backward compatible
- ✅ Can ship incrementally
- ✅ Low risk integration
- ✅ Allows parallel development

### Pathway 2: Complete Unification (Future)

**Current**: Separate projects
**Target**: Single `riff` crate/package with all capabilities
**Method**: Create meta-package that wraps both

**Advantages**:
- ✅ Single install command
- ✅ No subprocess management
- ✅ Unified versioning

**Disadvantages**:
- ❌ Requires Rust bindings for Python (Pyo3 or similar)
- ❌ Complex build system
- ❌ Maintenance overhead
- ❌ Not recommended for now

---

## Option A: Python Wrapper (Recommended for Now)

Keep riff-cli as primary, make it aware of riff-dag-tui

```
pip install riff-cli
  └── Includes subprocess launcher for riff-dag-tui
      └── Downloads or finds precompiled riff-dag-tui binary
```

**Architecture**:
```python
# riff-cli package structure
riff-cli/
├── src/riff/
│   ├── __main__.py            # Entry point: `riff` command
│   ├── cli.py                 # Command routing
│   ├── search/
│   ├── repair/
│   ├── explore/               # NEW: Visualization module
│   │   ├── handler.py         # Subprocess management
│   │   └── formatter.py       # JSONL conversion
│   └── classic/
```

**User Experience**:
```bash
# Install
pip install riff-cli

# Use unified CLI
riff search "memory architecture" --visualize

# Behind the scenes:
# 1. riff-cli executes search via Qdrant
# 2. Converts results to JSONL
# 3. Spawns riff-dag-tui subprocess
# 4. User interacts with 3-pane TUI
# 5. Closes riff-dag-tui, returns to shell
```

---

## Option B: Cargo Wrapper (Future Alternative)

Create a Rust binary that orchestrates both

```
cargo install riff
  ├── Installs: riff-cli via Python subprocess launcher
  └── Includes: riff-dag-tui native TUI
```

**Advantages** (future):
- Single binary installation
- Native Rust distribution
- Direct subprocess communication

**Current Status**: Not recommended (too complex now)

---

## Migration Path for Users

### Today (Fragmented)

```bash
# User workflow
riff search "topic"          # Use riff-cli Python
# Manual steps to visualize
riff-dag-tui --input data.jsonl  # Use riff-dag-tui Rust
```

### Tomorrow (Unified)

```bash
# Same workflow, better UX
riff search "topic" --visualize  # Single command, automatic visualization
```

### Seamless Upgrade

```bash
# Old way still works for backward compatibility
riff search "topic"
riff visualize custom_file.jsonl

# New shorthand
riff search "topic" --visualize

# New explore command
riff explore --from-file data.jsonl
```

---

## Federation Integration Points

### Nabi CLI Registration

```bash
# Single entry point registered
nabi exec riff search "query" --visualize

# Automatically finds:
# - Python environment (riff-cli)
# - Rust binary (riff-dag-tui)
# - Qdrant service
# - Data directories
```

### Service Registry

```yaml
# In Memchain/federation service inventory
riff-toolkit:
  primary: riff-cli (Python semantic search)
  visualization: riff-dag-tui (Rust interactive TUI)
  entry_point: riff (unified CLI)
  status: Production (search), Stable (visualization)
  federation_role: MemRiff operator tooling
```

### Loki Event Streaming

```python
# Log all searches and visualizations to federation
log_search_event(
    query=query,
    session_id=session.id,
    result_count=len(results),
    visualization_enabled=visualize
)
```

---

## Data Flow Architecture

### Search → Visualize Flow

```
User Input: riff search "memory patterns" --visualize
    ↓
[riff-cli Python]
    ├── Parse arguments
    ├── Connect to Qdrant
    ├── Execute semantic search
    ├── Get 50-500 results (configurable)
    ├── Apply time filters
    └── Convert to DAG format
         ↓
    [Generate JSONL]
         ├── nodes: (id, label, span, tags, timestamp)
         ├── edges: (from, to) relationships
         └── metadata: (search_query, result_count, timestamp)
         ↓
    [Spawn riff-dag-tui subprocess]
         ├── Write temp JSONL file
         ├── Launch: riff-dag-tui --input /tmp/search-xyz.jsonl
         ├── Maintain subprocess lifecycle
         └── Cleanup temp files on exit
         ↓
[User interacts with 3-pane TUI]
    ├── Left: Node list (searchable)
    ├── Top-right: Node details (metadata)
    └── Bottom-right: DAG visualization (reactive)
         ↓
User closes (q key or Ctrl+C)
    ↓
[riff-cli resumes]
    ├── Subprocess cleanup
    ├── Temp file removal
    ├── Optional: Log to federation
    └── Return to shell
```

---

## Configuration & Customization

### User Config File

```yaml
# ~/.config/nabi/riff/config.yaml

search:
  max_results: 100
  default_time_window: 7  # days
  highlight_color: "cyan"

visualization:
  auto_spawn: true        # Auto-open riff-dag-tui with --visualize
  max_nodes: 500          # Warn if > this
  depth_limit: 2          # DAG render depth
  temp_dir: ~/.cache/riff # Where to store temporary JSONL

formatting:
  show_metadata: true
  preview_length: 200
```

### Command-Line Overrides

```bash
# Override config values
riff search "topic" --visualize --max-results 500
riff search "topic" --config ~/custom-riff-config.yaml
```

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| **Time to search + visualize** | 3+ minutes (manual) | < 30 seconds (automated) |
| **User friction** | High (2 separate tools) | Low (1 unified CLI) |
| **Entry point clarity** | Confusing (which tool?) | Clear (always `riff`) |
| **Federation integration** | Partial | Complete |

---

## Implementation Checklist

### Phase 1: Subprocess Integration (Week 1)

- [ ] Create `explore` module in riff-cli
- [ ] Implement subprocess handler for riff-dag-tui
- [ ] Add `--visualize` flag to search command
- [ ] Test subprocess lifecycle management
- [ ] Error handling and graceful failures
- [ ] Documentation updates

### Phase 2: User Experience Refinement (Week 2)

- [ ] Create `riff explore` command as primary interface
- [ ] Config file support (XDG-compliant)
- [ ] Time-to-visualization optimization
- [ ] Performance profiling
- [ ] Federation event logging

### Phase 3: Federation Integration (Week 3)

- [ ] Nabi CLI registration
- [ ] Service inventory update
- [ ] Documentation in MemRiff ecosystem
- [ ] Cross-platform testing (macOS + Linux)
- [ ] Release preparation

---

## Questions for Design Review

1. **Default Behavior**: Should `riff search` auto-visualize or require `--visualize` flag?
2. **Result Limits**: How many results maximum? When to warn user about size?
3. **Temp Directory**: Use `~/.cache/riff` or system temp `/tmp`?
4. **Subprocess Timeout**: Maximum time to wait for riff-dag-tui startup?
5. **Fallback Mode**: If riff-dag-tui unavailable, should riff-cli still work?

---

## Related Documents

- [CONSOLIDATION_ANALYSIS.md](CONSOLIDATION_ANALYSIS.md) — Strategic overview
- [CONSOLIDATION_QUICK_START.md](CONSOLIDATION_QUICK_START.md) — Implementation guide
- [README.md](README.md) — Original riff-dag-tui documentation
- Related: riff-cli/docs/ARCHITECTURE.md

---

*Architecture design ready for stakeholder review*
