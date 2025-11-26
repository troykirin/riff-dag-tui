# Riff Consolidation: REVISED Strategy with nabi-tui Integration

**Status**: Architecture Update (nabi-tui component integration)
**Date**: 2025-11-08 (Updated from initial analysis)
**Critical Change**: riff-dag-tui becomes a component of nabi-tui, not standalone

---

## 🔄 Architectural Clarification

### Initial Analysis (Incomplete)

```
riff-cli (Python)
    └── spawns riff-dag-tui (Rust subprocess)
```

### REVISED: Multi-Layer Federation Architecture

```
┌─────────────────────────────────────────────────────┐
│          nabi-tui (dag-tui)                         │
│   Federation Visualization Platform (Main)          │
│   - Tokio async runtime                             │
│   - WebSocket/real-time streaming                   │
│   - Kernel integration                              │
│   - WILL INCLUDE: riff-dag-tui as specialized view  │
└─────────────────────────┬───────────────────────────┘
                          ▲
                          │ integration
                          │
┌─────────────────────────┴───────────────────────────┐
│         riff-dag-tui                                │
│   Conversation DAG Visualization Component          │
│   - 3-pane layout for riff exploration              │
│   - JSONL input format                              │
│   - WILL BE: Integrated into nabi-tui              │
└─────────────────────────┬───────────────────────────┘
                          ▲
                          │ spawns/uses
                          │
┌─────────────────────────┴───────────────────────────┐
│         riff-cli (Python)                           │
│   Conversation Search & Discovery Layer             │
│   - Semantic search (Qdrant)                        │
│   - JSONL repair                                    │
│   - Entry point: $ riff search --visualize          │
└─────────────────────────────────────────────────────┘
```

---

## 📊 Three Consolidation Pathways

### PATHWAY 1: riff-dag-tui as Standalone + Python Bridge (PHASE 1)

**Timeline**: Immediate (this sprint)
**Scope**: Option A from original analysis
**Architecture**:
```
riff-cli Python
    └── subprocess: riff-dag-tui
        └── spawns binary with --input flag
```

**Advantages**:
- ✅ Fast implementation (4-5 days)
- ✅ Works immediately
- ✅ Independent components
- ✅ Low risk

**Limitations**:
- ❌ Doesn't integrate with nabi-tui
- ❌ Duplicate visualization code
- ❌ Not federation-aware (no tokio/streaming)

**Use Case**: Rapid MVP, conversation-focused exploration

---

### PATHWAY 2: riff-dag-tui Module → nabi-tui (PHASE 2)

**Timeline**: 2-3 weeks after Phase 1
**Scope**: Integrate riff-dag-tui code into nabi-tui
**Architecture**:
```
nabi-tui (dag-tui)
    ├── Core visualization engine (tokio async)
    ├── Streaming handler (WebSocket)
    ├── riff module (from riff-dag-tui)
    │   ├── 3-pane layout
    │   ├── Memory span visualization
    │   └── Riff-specific rendering
    └── Other modules (federation, agents, etc.)

riff-cli (Python)
    └── Spawns nabi-tui with --riff-mode flag
```

**Integration Points**:
- Convert riff-dag-tui src/ → nabi-tui/src/modules/riff/
- Add feature flag: `nabi-tui --features riff`
- Unify JSONL contract
- Share rendering infrastructure

**Advantages**:
- ✅ Single visualization platform (nabi-tui)
- ✅ Real-time streaming capability
- ✅ Federation-integrated
- ✅ Code sharing (ratatui, crossterm, petgraph)
- ✅ Unified kernel integration

**Timeline**: 2-3 weeks, ~500 lines refactoring

---

### PATHWAY 3: Full Python-Rust Unification (FUTURE)

**Timeline**: Post-consolidation (future enhancement)
**Scope**: Rust CLI wrapper for riff toolkit
**Architecture**:
```
riff (Rust unified binary)
    ├── Python bindings for search (via Pyo3)
    ├── Native Qdrant interface
    └── Direct nabi-tui integration
```

**Not Recommended Now**: Too complex, premature

---

## ✅ RECOMMENDED: Two-Phase Consolidation

### PHASE 1: Python-First Bootstrap (This Week)

**Goal**: Get riff-cli + riff-dag-tui working together immediately

**Implementation** (from original analysis):
- Add subprocess handler to riff-cli
- Add `--visualize` flag to search
- Export results to JSONL
- Spawn riff-dag-tui

**Deliverable**: Unified `riff search "query" --visualize` command

**Timeline**: 4-5 days

**Outcome**: Production-ready conversation exploration toolkit

---

### PHASE 2: Merge into nabi-tui (2-3 weeks later)

**Goal**: Make riff-dag-tui a native nabi-tui module

**Implementation**:
1. **Analyze nabi-tui architecture**
   - Study tokio integration
   - Understand module pattern
   - Review streaming setup

2. **Extract riff-dag-tui logic**
   - Copy src/ → nabi-tui/src/modules/riff/
   - Adapt for tokio async patterns
   - Integrate rendering pipeline

3. **Unify interfaces**
   - Keep JSONL as contract
   - Add streaming input (WebSocket)
   - Support both static and live data

4. **Federation integration**
   - Hook into Loki/Grafana
   - Register with service inventory
   - Add federation event streaming

5. **User-facing changes**
   ```bash
   # Phase 1 (current)
   riff search "query" --visualize

   # Phase 2 (later)
   nabi-tui --riff-mode       # Launch nabi-tui in riff view
   nabi-tui --federation      # Switch to federation DAG view
   riff search "query" | nabi-tui --stream  # Real-time streaming
   ```

**Timeline**: 2-3 weeks, ~500 lines refactoring

**Outcome**: Unified federation visualization platform

---

## 🎯 Phase 1: Immediate Actions (This Sprint)

### Tasks for riff-cli (Python)

1. **Subprocess handler module** (Day 1)
   ```python
   src/riff/visualization/handler.py
   - RiffDagTUIHandler class
   - Binary discovery
   - Subprocess lifecycle
   ```

2. **CLI integration** (Day 2)
   ```python
   # Add to search command
   @app.command()
   def search(..., visualize: bool = False):
       results = semantic_search(...)
       if visualize:
           export_and_visualize(results)

   # Add visualize command
   @app.command()
   def visualize(input_file: str):
       handler.launch(input_file)
   ```

3. **JSONL export** (Day 2)
   ```python
   src/riff/visualization/formatter.py
   - Convert search results to riff-dag-tui format
   - Ensure compatibility
   - Handle edge cases
   ```

4. **Testing & docs** (Days 3-5)
   - Unit + integration tests
   - Cross-platform validation
   - Documentation updates
   - Federation registration (Nabi CLI)

### Tasks for riff-dag-tui (Rust)

**NO CHANGES NEEDED** (for Phase 1)
- Binary continues to work as-is
- Just needs to be callable via subprocess
- Can verify binary discovery works
- Consider adding `--input` validation

---

## 🔮 Phase 2: Deep Integration (Future)

### Prerequisites

Before starting Phase 2:
1. Phase 1 complete and stable in production
2. nabi-tui architecture fully documented
3. Decision: keep riff-dag-tui separate or merge?
4. Resource allocation for refactoring

### Integration Checklist

- [ ] nabi-tui architecture understood
- [ ] Module pattern documented
- [ ] Tokio integration strategy defined
- [ ] Streaming interface designed
- [ ] JSONL to streaming protocol mapping defined
- [ ] Testing strategy for integrated platform
- [ ] Federation event hooks identified
- [ ] Performance benchmarks established

### File Structure (Post-Integration)

```
nabi-tui/
├── src/
│   ├── main.rs                  # Entry point
│   ├── modules/
│   │   ├── riff/                # ← From riff-dag-tui
│   │   │   ├── lib.rs
│   │   │   ├── ui/
│   │   │   ├── model/
│   │   │   └── renderer/
│   │   ├── federation/          # Existing
│   │   └── agents/              # Existing
│   ├── streaming/               # Shared
│   └── kernel/                  # Shared
├── Cargo.toml                   # Updated deps
└── features/
    ├── riff                     # Optional feature
    ├── federation               # Optional feature
    └── agents                   # Optional feature
```

---

## 📋 JSONL Format Specification

**Ensure consistency across all phases**:

```json
{"type":"node","id":"mem_001","label":"search:memory","span":"session_123","tags":["search","memory"],"ts":"2025-11-08T10:30:00Z"}
{"type":"edge","from":"mem_001","to":"mem_002","label":"caused_by"}
{"type":"node","id":"mem_002","label":"found:patterns","span":"session_123","tags":["result"],"ts":"2025-11-08T10:30:05Z"}
```

**Fields**:
- `type`: "node" or "edge"
- `id`: Unique identifier
- `label`: Display text
- `span`: Session/span grouping
- `tags`: Array of strings
- `ts`: ISO 8601 timestamp
- `from`/`to`: Edge endpoints (optional)

**Compatibility Requirement**: Must work with both riff-dag-tui (standalone) and nabi-tui (integrated)

---

## 🚀 User Experience Timeline

### Now (Phase 1)
```bash
# Immediate capability
$ riff search "memory patterns" --visualize
✓ Automatic DAG exploration
✓ Works today
✓ Python + Rust subprocess

# Backward compatible
$ riff search "query"           # Text results
$ riff visualize file.jsonl     # Manual visualization
```

### Later (Phase 2)
```bash
# Enhanced capability
$ nabi-tui --riff              # Launch in riff view
$ nabi-tui --federation        # Federation view
$ riff search "query" | nabi-tui --stream  # Real-time

# Same API, better platform
$ riff search --visualize      # Still works, uses nabi-tui internally
```

---

## ⚠️ Decision Points for Stakeholders

### For Phase 1 (Immediate)

1. **Subprocess TTY behavior**: New terminal or take over?
2. **Result limit**: Max nodes for safety? (recommend 500)
3. **Temp directory**: ~/.cache/riff or /tmp?
4. **Auto-spawn**: Default to --visualize or opt-in?
5. **Fallback**: JSON if riff-dag-tui unavailable?

### For Phase 2 (Planning)

1. **Timeline**: Start Phase 2 after Phase 1 stable in production?
2. **Scope**: Merge riff-dag-tui or keep separate?
3. **Streaming**: Support real-time DAG streaming?
4. **Features**: Which nabi-tui features to include in riff module?
5. **Performance**: Acceptable latency for large graphs?

---

## 📊 Risk & Mitigation

### Phase 1 Risks (Low)

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Binary not found | High | Graceful error + installation instructions |
| Large JSONL files | Medium | Size limit + pagination |
| Subprocess crash | Medium | Exit code handling + fallback |
| Cross-platform issues | Low | Test on macOS + Linux |

### Phase 2 Risks (Medium)

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Refactoring complexity | High | Clear module boundaries, incremental |
| Performance regression | High | Benchmarking before/after |
| Breaking changes | High | Feature flags for compatibility |
| nabi-tui stability | Medium | Wait for nabi-tui to stabilize first |

---

## 🗓️ Timeline Summary

| Phase | Timeline | Scope | Deliverable |
|-------|----------|-------|-------------|
| **1** | This week (4-5 days) | riff-cli + riff-dag-tui subprocess | `riff search --visualize` |
| **2** | 2-3 weeks later | Integrate into nabi-tui | Native nabi-tui riff module |
| **3** | Future | Full unification (optional) | Unified Rust CLI |

---

## ✅ Success Criteria

### Phase 1
- [ ] `riff search "query" --visualize` works end-to-end
- [ ] Subprocess lifecycle properly managed
- [ ] Cross-platform testing passes
- [ ] Documentation complete
- [ ] Federation registration done (Nabi CLI)
- [ ] Production-ready, no blockers

### Phase 2
- [ ] riff-dag-tui integrated into nabi-tui
- [ ] Streaming support working
- [ ] Federation event hooks integrated
- [ ] Performance benchmarks acceptable
- [ ] Backward compatibility maintained
- [ ] All tests passing

---

## 📚 Related Documentation

**Original Analysis**: PURPOSE_DISCOVERY_REPORT.md
**Phase 1 Guide**: CONSOLIDATION_QUICK_START.md
**Detailed Roadmap**: CONSOLIDATION_ANALYSIS.md

---

## 🎯 Recommendation

### PROCEED WITH: Two-Phase Consolidation

**Phase 1 (Immediate)**: Python-first subprocess integration
- Timeline: This week
- Effort: 4-5 days
- Risk: Low
- Value: High

**Phase 2 (Future)**: Integration with nabi-tui
- Timeline: 2-3 weeks post-Phase 1
- Effort: 2-3 weeks
- Risk: Medium (refactoring)
- Value: Very High (unified platform)

**Reasoning**:
- Get value immediately (Phase 1)
- Don't block on nabi-tui integration
- Allow time to learn nabi-tui architecture
- Preserve flexibility for design changes

---

*Strategic update accounting for nabi-tui integration context*
*Maintains Phase 1 momentum while enabling future consolidation*
