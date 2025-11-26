# Riff Consolidation: Quick Start Guide

**TL;DR**: Integrate riff-cli and riff-dag-tui through a simple subprocess bridge in Python. No major refactoring needed.

---

## The Big Picture

```
Two complementary tools:

riff-cli (Python)           riff-dag-tui (Rust)
┌──────────────────┐       ┌──────────────────┐
│ Semantic Search  │       │ DAG Visualization │
│ (Qdrant)         │──────→│ (Ratatui)         │
│ JSONL Repair     │       │ Interactive TUI   │
│ Time Filtering   │       │ 3-pane layout    │
└──────────────────┘       └──────────────────┘

Unified Product: Conversation Intelligence Toolkit
```

---

## Recommended Approach: Option A (Python-First)

### What This Means

- Keep riff-cli as the primary entry point
- Add a `riff visualize` command that spawns riff-dag-tui
- Handle the Rust binary as a subprocess (subprocess.run)
- Export search results to JSONL format
- Pass JSONL file to riff-dag-tui for visualization

### Why This Works

✅ **Low Risk**: No code migration, no dependency conflicts
✅ **Fast**: Can be done in 1 week
✅ **Modular**: Each tool remains independent
✅ **Flexible**: Easy to replace either component later
✅ **Tested**: Both projects already production-ready

---

## Implementation Overview (4 Phases)

### Phase 1: Design Subprocess Handler (Day 1)

**Goal**: Create a reusable module for launching riff-dag-tui

**File**: `src/riff/visualization/handler.py`

```python
class RiffDagTUIHandler:
    """Manage riff-dag-tui subprocess lifecycle."""

    def __init__(self):
        self.binary_path = self._discover_binary()

    def _discover_binary(self) -> Path:
        """Find riff-dag-tui binary in PATH or standard locations."""
        # Check ~/.cargo/bin/riff-dag-tui
        # Check ./target/release/riff-dag-tui
        # Check standard PATH
        # Raise error if not found

    def launch(self, jsonl_path: Path) -> int:
        """Spawn riff-dag-tui with JSONL file."""
        result = subprocess.run(
            [str(self.binary_path), "--input", str(jsonl_path)],
            capture_output=False
        )
        return result.returncode

    def verify_installed(self) -> bool:
        """Check if riff-dag-tui is available."""
        try:
            self._discover_binary()
            return True
        except FileNotFoundError:
            return False
```

### Phase 2: Add Visualize Command (Day 2)

**Goal**: Expose visualization through riff CLI

**File**: `src/riff/cli.py`

```python
@app.command()
def visualize(
    input_file: str = typer.Argument(..., help="JSONL file to visualize"),
):
    """Explore conversation DAG interactively (opens riff-dag-tui)."""
    handler = RiffDagTUIHandler()

    if not handler.verify_installed():
        typer.echo(
            "❌ riff-dag-tui not found. Install with:\n"
            "  cargo install --path ../riff-dag-tui",
            err=True
        )
        raise typer.Exit(1)

    typer.echo(f"🚀 Opening {input_file} in interactive viewer...")
    exit_code = handler.launch(Path(input_file))

    if exit_code != 0:
        typer.echo(f"❌ Visualization exited with code {exit_code}", err=True)
        raise typer.Exit(exit_code)
```

### Phase 3: Wire Search → Visualize (Day 3)

**Goal**: Create workflow: search → export → visualize

**File**: `src/riff/search/integration.py`

```python
def search_and_visualize(
    query: str,
    output_file: str = "search_results.jsonl",
) -> Path:
    """Execute search and export results for visualization."""
    searcher = QdrantSearcher()
    results = searcher.search(query)

    # Convert results to riff-dag-tui format
    jsonl_data = _convert_to_dag_format(results)

    # Write to temporary file
    output_path = Path(output_file)
    with open(output_path, 'w') as f:
        for item in jsonl_data:
            f.write(json.dumps(item) + '\n')

    typer.echo(f"✅ Exported {len(results)} results to {output_file}")
    return output_path

def _convert_to_dag_format(results):
    """Convert search results to riff-dag-tui JSONL format."""
    for result in results:
        yield {
            "type": "node",
            "id": result['id'],
            "label": result.get('title', 'Untitled'),
            "span": result.get('session_id', 'unknown'),
            "tags": result.get('tags', []),
            "ts": result.get('timestamp', '')
        }
```

### Phase 4: Testing & Documentation (Day 4-5)

**Test Coverage**:
- ✅ Binary discovery works on different platforms
- ✅ JSONL export format matches riff-dag-tui expectations
- ✅ Subprocess lifecycle handled correctly
- ✅ Error messages are helpful
- ✅ Temporary files cleaned up

**Documentation**:
- [ ] Add `visualize` command to README
- [ ] Create example workflows
- [ ] Document the JSONL export format
- [ ] Add federation integration notes

---

## Usage Examples

### Example 1: Search and Visualize

```bash
# Search for conversations about memory architecture
riff search "memory architecture" --export search_results.jsonl

# Then visualize the results
riff visualize search_results.jsonl
```

### Example 2: Interactive Exploration

```bash
# One-liner workflow
riff search "federation patterns" --export /tmp/riff-results.jsonl && \
riff visualize /tmp/riff-results.jsonl
```

### Example 3: Time-Filtered Visualization

```bash
# Search from last 7 days and visualize
riff search "session tracking" --days 7 --export recent.jsonl
riff visualize recent.jsonl
```

---

## JSONL Format Specification

**Input Format** (what riff-dag-tui expects):

```json
{"type":"node","id":"mem_001","label":"ingest","span":"epoch:init","tags":["io"],"ts":"2025-05-22T10:00:00Z"}
{"type":"node","id":"mem_002","label":"process","span":"epoch:init","tags":["compute"],"ts":"2025-05-22T10:00:05Z"}
{"type":"edge","from":"mem_001","to":"mem_002"}
```

**Fields**:
- `type`: "node" or "edge"
- `id`: Unique identifier
- `label`: Display name
- `span`: Span/session identifier
- `tags`: Array of tag strings
- `ts`: ISO 8601 timestamp
- `from`/`to`: Edge endpoints

---

## Dependency & Installation

### Current State

```
riff-cli/
└── depends on: Python 3.13+, Qdrant, Qdrant client

riff-dag-tui/
└── depends on: Rust, ratatui, crossterm, petgraph
```

### After Consolidation

```
riff (consolidated entry point)
├── riff-cli Python
│   └── depends on: Qdrant client, subprocess
│
└── riff-dag-tui Rust
    └── depends on: ratatui, crossterm, petgraph
```

### Installation

```bash
# Install both tools
cargo install --path ./riff-dag-tui

cd riff-cli && task dev:setup
```

---

## Risk Mitigation

### Risk: Binary Not Found

```python
# Solution: Bundled discovery with helpful error
handler.verify_installed()  # Returns helpful installation instructions
```

### Risk: Large JSONL Files

```python
# Solution: Streaming parser + pagination
if os.path.getsize(jsonl_path) > 100_000_000:  # 100MB
    typer.echo("⚠️  Large file detected. Pagination enabled.", err=True)
    # Implement streaming or chunking
```

### Risk: Subprocess Crash

```python
# Solution: Capture exit codes and provide feedback
try:
    exit_code = handler.launch(jsonl_path)
    if exit_code != 0:
        log_error(f"Visualization failed with code {exit_code}")
except OSError as e:
    typer.echo(f"Failed to launch riff-dag-tui: {e}", err=True)
```

---

## Federation Integration

After implementation, register with federation:

```bash
# Register riff-cli as primary entry point
nabi exec riff search "query"
nabi exec riff visualize file.jsonl

# Add to MemRiff service registry
# Update LEGEN_MEMRIFF_MEMCHAIN_SYSTEM_MAP.md
```

---

## Success Checklist

- [ ] Phase 1: Subprocess handler module complete
- [ ] Phase 2: `riff visualize` command working
- [ ] Phase 3: Search → export → visualize workflow tested
- [ ] Phase 4: Documentation and federation integration
- [ ] Tests: 100% coverage for new modules
- [ ] Cross-platform: Tested on macOS and Linux
- [ ] Performance: Subprocess spawn < 2 seconds
- [ ] Error handling: Graceful failures with helpful messages

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1 | 1 day | Subprocess handler module |
| 2 | 1 day | `riff visualize` command |
| 3 | 1 day | Integration testing |
| 4 | 1 day | Documentation & federation |
| **Total** | **~4 days** | **Production-ready consolidation** |

---

## Questions & Decisions Needed

1. **Binary Distribution**: Ship riff-dag-tui binary with riff-cli or install separately?
2. **Performance**: Acceptable subprocess spawn time?
3. **Temporary Files**: Where should JSONL exports live? (~/.cache/riff-cli or /tmp?)
4. **Logging**: Should visualization be added to Loki federation logs?
5. **Versioning**: Semantic version constraints between projects?

---

*Ready to implement* | *Ask for clarification on any decision points above*
