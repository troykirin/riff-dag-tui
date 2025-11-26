# Riff Project Consolidation Analysis

**Status**: Strategic Architecture Planning
**Date**: 2025-11-08
**Purpose**: Define consolidation strategy for riff-cli and riff-dag-tui

---

## Executive Summary

Two complementary riff projects exist in the federation:

| Aspect | riff-cli | riff-dag-tui |
|--------|----------|--------------|
| **Language** | Python 3.13+ | Rust 2021 |
| **Location** | ~/nabia/tools/riff-cli | ~/nabia/tui/production/riff-dag-tui |
| **Purpose** | Search/repair Claude conversations | Visualize conversation DAGs |
| **Architecture** | 3-layer (CLI→Search/Repair→Qdrant) | 3-pane TUI (Ratatui + Petgraph) |
| **Status** | Production (search), Dev (TUI) | Complete, production-ready |
| **Entry Point** | CLI with subcommands | Standalone TUI application |

**Strategic Insight**: These are **complementary components** of a unified conversation intelligence toolkit:
- **riff-cli**: Data discovery and indexing layer (Python semantic search)
- **riff-dag-tui**: Visualization and exploration layer (Rust interactive UI)
- **Together**: Complete conversation analysis platform

---

## Project-by-Project Analysis

### 1. riff-cli (Python) - Search & Repair Toolkit

**Purpose Statement**:
Unified Claude conversation search and JSONL repair toolkit providing semantic search with content preview, conversation indexing, and data integrity management.

**Confidence Level**: HIGH (0.95) — Documentation explicit, architecture clear

**Key Technologies**:
- Python 3.13+ (primary runtime)
- Qdrant (semantic search with 384-dim vectors)
- Prompt Toolkit (interactive TUI)
- Task automation (Taskfile.yml)

**Core Capabilities**:
1. **Semantic Search**: Find conversations by meaning (Qdrant-powered)
2. **Content Preview**: See actual text snippets in results
3. **Time-based Filtering**: Filter by --days, --since, --until
4. **AI Enhancement**: Intent-driven query expansion with Grok
5. **JSONL Repair**: Scan and fix malformed conversation exports
6. **TUI Module**: Interactive explorer (modular architecture)
7. **Graph Generation**: Mermaid/DOT conversation graphs

**Architecture**:
```
CLI Entry Point (cli.py)
    ├── Search Mode (semantic + time filters + AI enhancement)
    ├── Repair Mode (scan/fix JSONL integrity)
    ├── TUI Mode (interactive file browser)
    └── Graph Mode (conversation visualization)
         └── Qdrant/Classic/Enhance backends
```

**Module Structure**:
```
src/riff/
├── cli.py               # Entry point
├── search/              # Qdrant semantic search
├── enhance/             # AI query expansion
├── classic/             # Original scan/fix/tui/graph
└── tui/                 # Interactive TUI (modular)
```

**Maturity Assessment**:
- ✅ Search: Production-ready
- ✅ Repair: Production-ready
- 🚧 TUI: Week 1 complete, Week 2 in progress
- 📋 Graph: Available

**Federation Integration**:
- XDG-compliant paths (~/.nabi/venvs/riff-cli/)
- Registered with Nabi CLI
- Docker-based Qdrant orchestration
- Task-driven workflows

---

### 2. riff-dag-tui (Rust) - DAG Visualization TUI

**Purpose Statement**:
Interactive terminal UI for exploring DAGs of "riffs" or memory spans with three-pane layout: node list (searchable), node details (rich), and layered DAG visualization.

**Confidence Level**: HIGH (0.95) — Complete, purpose clear

**Key Technologies**:
- Rust 2021 edition (primary runtime)
- Ratatui 0.26 (TUI framework)
- Crossterm 0.27 (terminal control)
- Petgraph 0.6 (graph algorithms)
- Serde/Serde JSON (data serialization)

**Core Capabilities**:
1. **Node List Pane**: Filterable list of conversation nodes
2. **Details Pane**: Rich node information (spans, tags, neighbors)
3. **DAG Visualization**: Reactive, depth-limited layered graph view
4. **Interactive Filtering**: Real-time search and filter
5. **Keyboard Navigation**: vim-style controls (hjkl, /, q, etc.)
6. **JSONL Input**: Standard format with embedded fallback data

**Architecture**:
```
┌─────────────────────────────┐
│    Interactive TUI Loop      │ (Ratatui event handling)
├─────────────────────────────┤
│   3-Pane Layout Manager      │
├──────────────┬──────────────┐
│ Node List    │ Details Pane │ (Top-right split)
│ (Filterable) │ + DAG View   │ (Bottom-right)
└──────────────┴──────────────┘
         ▼
    Graph Model (Petgraph)
         ▼
    JSONL Data Source
```

**UI Controls**:
- `Up/Down` or `k/j` — Navigate
- `/` — Filter mode
- `Enter` — Accept filter
- `Esc` — Exit filter (preserve)
- `c` — Clear filter
- `q` — Quit
- `?` — Help

**Data Format**:
```json
{"type":"node","id":"mem_001","label":"ingest","span":"epoch:init","tags":["io"],"ts":"2025-05-22T10:00:00Z"}
{"type":"edge","from":"mem_001","to":"mem_002"}
```

**Maturity Assessment**:
- ✅ Core functionality: Complete
- ✅ UI/UX: Production-ready
- ✅ Data loading: Robust
- ✅ Graph rendering: Stable
- 📋 Edge cases: Well-handled

**Federation Integration**:
- Rust-based (compilable binary)
- No external service dependencies
- Self-contained data loading
- Embedded sample data for fallback

---

## Consolidation Strategic Options

### Option A: Python-First Consolidation (RECOMMENDED)

**Approach**: Keep riff-cli as primary project, integrate riff-dag-tui as rendering backend

**Architecture**:
```
riff-cli (Python - primary)
├── Search → Qdrant
├── Repair → File Operations
├── Export → JSONL
└── Visualize → Spawn riff-dag-tui subprocess
    └── Feeds JSONL to Rust renderer
```

**Advantages**:
- ✅ Minimal disruption (riff-cli stays Python)
- ✅ Leverages existing Qdrant search
- ✅ Modular: TUI remains replaceable
- ✅ Fast integration (subprocess IPC)
- ✅ Easier to maintain (Python focus)

**Implementation Path**:
1. Add `visualize` command to riff-cli
2. Generate temporary JSONL from search results
3. Spawn riff-dag-tui with `--input` flag
4. Handle process lifecycle and output
5. Clean up temporary files

**Code Changes Required**:
- ~30 lines in riff-cli/cli.py (new subcommand)
- ~50 lines in subprocess handler
- Zero changes to riff-dag-tui

---

### Option B: Rust-First Consolidation (COMPLEX)

**Approach**: Migrate riff-cli to Rust, make it the primary project

**Architecture**:
```
riff (Rust - unified)
├── CLI subcommands
├── JSONL loading/repair (Rust)
├── Semantic search (Rust bindings to Qdrant)
└── TUI visualization (Ratatui native)
```

**Advantages**:
- ✅ Single language ecosystem
- ✅ Compiled binary (no runtime deps)
- ✅ Better performance

**Disadvantages**:
- ❌ Major Python→Rust migration (weeks of work)
- ❌ Need Qdrant Rust client (adds complexity)
- ❌ AI enhancement (Grok) requires new integration
- ❌ Disrupts existing riff-cli workflows
- ❌ Higher maintenance burden

**Not Recommended**: Unless there's a specific performance requirement

---

### Option C: Workspace Unification (PRAGMATIC)

**Approach**: Create new `riff-workspace` umbrella project with both as subprojects

**Architecture**:
```
riff-workspace (root)
├── riff-cli/          (symlink to ~/nabia/tools/riff-cli)
├── riff-dag-tui/      (symlink to ~/nabia/tui/production/riff-dag-tui)
├── Makefile           (unified build)
├── docs/              (shared documentation)
└── integration-tests/ (end-to-end testing)
```

**Advantages**:
- ✅ Maintains independence of each project
- ✅ Shared documentation and governance
- ✅ Clear integration contracts
- ✅ Easier for new developers

**Disadvantages**:
- ❌ Doesn't truly consolidate (creates wrapper)
- ❌ Complexity management overhead
- ❌ Doesn't solve deployment/installation

---

## Recommended Consolidation Strategy

### PRIMARY RECOMMENDATION: Option A (Python-First)

**Rationale**:
1. **Minimal Risk**: Leverages existing, production-ready riff-cli
2. **Clear Contract**: Subprocess IPC is well-defined
3. **Fast Iteration**: Can start immediately (days, not weeks)
4. **Modular**: Either TUI can be replaced independently
5. **Federated**: Respects federation principles (separation of concerns)

**Implementation Timeline**:
- **Phase 1** (Day 1-2): Design subprocess handler and JSONL export
- **Phase 2** (Day 2-3): Implement `riff visualize` command
- **Phase 3** (Day 3-4): Integration testing and error handling
- **Phase 4** (Day 4-5): Documentation and federation registration

**Success Criteria**:
- [ ] `riff search "query" | riff visualize` works end-to-end
- [ ] Subprocess lifecycle properly managed
- [ ] Error states handled gracefully
- [ ] Temporary files cleaned up
- [ ] Performance acceptable (<2s spawn time)
- [ ] Documentation updated
- [ ] Federation integration complete

---

## Implementation Roadmap

### Phase 1: Add Visualization Command to riff-cli

**File**: `src/riff/cli.py`

```python
@app.command()
def visualize(
    input_file: str = typer.Argument(..., help="JSONL file to visualize"),
    width: int = typer.Option(200, help="Terminal width"),
    height: int = typer.Option(50, help="Terminal height"),
):
    """Visualize conversation DAG with interactive TUI."""
    # Delegate to riff-dag-tui subprocess
    # Handle binary discovery + launch
    # Stream output back to user
```

### Phase 2: Subprocess Handler Module

**File**: `src/riff/visualization/subprocess_handler.py`

```python
class RiffDagTUIHandler:
    def launch(self, jsonl_path: str) -> int:
        """Launch riff-dag-tui with JSONL file."""
        # Find riff-dag-tui binary
        # Spawn with arguments
        # Handle lifecycle
        # Return exit code

    def verify_binary_exists(self) -> bool:
        """Check if riff-dag-tui is available."""
        # Search PATH, ~/.nabi/, ~/.cargo/bin/
        # Return location or raise error
```

### Phase 3: Federation Integration

**File**: `Taskfile.yml` (riff-cli)

```yaml
viz:install:
  cmds:
    - task: ../riff-dag-tui:install
    - cargo install --path ../riff-dag-tui

visualization:
  cmds:
    - cargo run --manifest-path ../riff-dag-tui/Cargo.toml -- --input {{.INPUT}}
```

### Phase 4: Documentation

**File**: `docs/CONSOLIDATED_ARCHITECTURE.md`

```markdown
# Riff Consolidated Architecture

## Components

### riff-cli (Python)
- Semantic search with Qdrant
- JSONL repair and validation
- Conversation indexing
- **New**: Visualization integration

### riff-dag-tui (Rust)
- Interactive DAG visualization
- Three-pane TUI layout
- Keyboard-driven navigation
- **Integration**: Subprocess of riff-cli

## Usage Flow

```
riff search "memory patterns"
  ↓
Qdrant returns matches
  ↓
riff visualize session.jsonl
  ↓
Spawns riff-dag-tui
  ↓
User explores DAG interactively
```
```

---

## Federation Context & Alignment

### MemRiff Ecosystem Integration

From LEGEN_MEMRIFF_MEMCHAIN_SYSTEM_MAP.md:

```
LeGen (research) → Memchain (runtime) → MemRiff (operator tooling)
                                            ↓
                                    Riff Toolkit:
                                    - Search (riff-cli)
                                    - Visualize (riff-dag-tui)
```

**Strategic Role**: MemRiff operator-facing conversation intelligence

**Federation Integration Checklist**:
- [ ] Register unified entry point with Nabi CLI
- [ ] Add to Memchain service registry
- [ ] Document in MemRiff ecosystem map
- [ ] Add federation event hooks (Loki logging)
- [ ] Update service inventory

---

## Data Flow Diagram

```
User Query
    ↓
[riff-cli Python]
    ├─→ Parse arguments
    ├─→ Connect to Qdrant
    ├─→ Run semantic search
    ├─→ Filter results (time, AI-enhanced)
    └─→ Export matching sessions to JSONL
         ↓
    [Export JSONL]
         ↓
    [riff-dag-tui Rust subprocess]
         ├─→ Load JSONL into graph
         ├─→ Render 3-pane TUI
         ├─→ Handle user input (vim-style)
         ├─→ Update display (reactive)
         └─→ Exit to shell
    ↓
User sees interactive DAG visualization
```

---

## Risk Assessment & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Subprocess spawn failure | High | Graceful error message, fallback to JSON output |
| Large JSONL files | Medium | Limit to 10K nodes, paginate, warn user |
| Binary discovery fails | High | Bundled binary, PATH search, cargo install fallback |
| Performance degradation | Medium | Profile, limit depth, cache graphs |
| Cross-platform issues | Low | Test on macOS + Linux, use crossterm |

---

## Success Metrics

- ✅ **Integration Time**: < 1 week
- ✅ **Breaking Changes**: 0 (backward compatible)
- ✅ **User Impact**: Opt-in feature (new command)
- ✅ **Performance**: Subprocess spawn < 2 seconds
- ✅ **Code Quality**: 100% test coverage for handler module
- ✅ **Documentation**: Comprehensive with examples

---

## Next Steps

1. **Review & Approve**: Share this analysis with stakeholders
2. **Design Review**: Finalize subprocess handler interface
3. **Implementation Sprint**: Execute Phase 1-4 over 4-5 days
4. **Testing**: Unit + integration tests
5. **Deployment**: Release as v2.1 of riff-cli
6. **Federation Integration**: Register with Nabi CLI and MemRiff

---

## Questions for Clarification

1. **Deployment Goal**: Should riff-cli install riff-dag-tui automatically?
2. **Distribution**: Should the Rust binary be vendored or installed separately?
3. **Data Privacy**: Should temporary JSONL files be encrypted or strictly sandboxed?
4. **Performance**: Any latency constraints on subprocess spawn?
5. **Federation Coordination**: Should visualization trigger Loki events for monitoring?

---

*Analysis Complete* | *Ready for Implementation* | *Phase 1 Timeline: Days*
