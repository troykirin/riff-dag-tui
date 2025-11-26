# Riff DAG TUI

> **Your Free Gateway Into the NabiOS Ecosystem**

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Repository](https://img.shields.io/badge/repository-troykirin%2Friff--dag--tui-black)](https://github.com/troykirin/riff-dag-tui)

Riff DAG TUI is a **lightweight, production-ready terminal UI** for exploring dependency graphs and memory hierarchies. Designed as the minimal entry point to the NabiOS federation, it's both a powerful standalone tool and a clean reference implementation for building federation-aware applications.

## Why Riff DAG TUI?

- **Lightweight Alternative**: No bloat. Fast startup. Single binary. Perfect for systems where resources matter.
- **Learning Reference**: Clean, idiomatic Rust codebase. Ratan UI mastery demonstrated through three-pane architecture.
- **Standalone Utility**: Works completely independent of NabiOS. Bring your own JSONL DAG data.
- **Federation-Ready**: When you're ready, extend it with federation capabilities or use it as a library component with independent release cycles.
- **Free Gateway**: Discover the power of structured DAG visualization before exploring the full NabiOS platform.

## Features

### Three-Pane Layout
```
+---------------------------------------------------------------+
|  Node List (Left)        |  Node Details (Top Right)         |
|                          |  DAG View (Bottom Right)          |
+---------------------------------------------------------------+
```

- **Left Pane**: Filterable node list with fuzzy search and parent/child degree indicators
- **Top-Right Pane**: Rich node details including metadata, tags, and neighbor relationships
- **Bottom-Right Pane**: **Layered DAG text rendering** showing parent-child relationships (depth ≤ 2)

### Navigation & Filtering
- Vim-style navigation (`j`/`k`, `h`/`l`)
- Real-time fuzzy filtering with `/`
- Quick filter clearing with `c`
- Responsive DAG visualization that updates as you navigate

### Data Format Support
- Newline-delimited JSON (JSONL)
- Flexible node and edge definitions
- Robust handling of missing references
- Perfect for federation event logs, task DAGs, and memory hierarchies

## Quick Start

### Installation

**Option 1: Build from Source (Recommended)**

You'll need [Rust stable](https://www.rust-lang.org/tools/install):

```bash
git clone https://github.com/troykirin/riff-dag-tui.git
cd riff-dag-tui
cargo install --path .
```

**Option 2: From Registry**

```bash
cargo install riff-dag-tui
```

### Usage

```bash
# With your own JSONL data
riff-dag-tui --input your-dag.jsonl

# Or use the embedded sample dataset
riff-dag-tui
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Move selection |
| `h` / `l` / `←` / `→` | Navigate panes |
| `/` | Enter filter mode |
| `Backspace` | Delete in filter mode |
| `Enter` | Accept filter |
| `Esc` | Exit filter (keep current filter) |
| `c` | Clear filter |
| `?` | Toggle help |
| `q` | Quit |

## Data Format

Riff expects newline-delimited JSON (JSONL) with two record types:

**Nodes:**
```json
{
  "type": "node",
  "id": "mem_001",
  "label": "ingest",
  "span": "epoch:init",
  "tags": ["io"],
  "ts": "2025-05-22T10:00:00Z"
}
```

**Edges:**
```json
{
  "type": "edge",
  "from": "mem_001",
  "to": "mem_002"
}
```

- Nodes can appear before or after edges
- Edges referencing missing nodes are ignored with a warning
- Perfect for federation event logs and task hierarchies

## Architecture

### Built With
- [**Ratatui**](https://ratatui.rs/) - Modern Rust TUI framework
- [**Petgraph**](https://docs.rs/petgraph/) - Graph algorithms and data structures
- [**Crossterm**](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

### Design Philosophy
- **Minimal Dependencies**: Only essential crates
- **Fast Startup**: Instant launch, no initialization
- **Zero Configuration**: Works out-of-the-box
- **Reference Implementation**: Clean code for learning Rust TUI patterns

## Integration with NabiOS

While Riff DAG TUI works standalone, it's designed as the gateway into the [**NabiOS ecosystem**](https://github.com/NabiaTech/nabia):

- **Explore Riff DAG TUI** → Learn terminal UI development
- **Discover Riff CLI** → Master conversation search and JSONL repair
- **Join NabiOS** → Build federation-aware applications

See the [NabiaTech Organization](https://github.com/NabiaTech) for the full federation platform.

## Development

### Building

```bash
cargo build --release
```

### Running with Sample Data

```bash
cargo run -- --input assets/sample.jsonl
```

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT OR Apache-2.0 license. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to open issues or pull requests.

## Related Projects

- **[Riff CLI](https://github.com/troykirin/riff-cli)** - Search Claude conversations semantically
- **[NabiOS](https://github.com/NabiaTech/nabia)** - Complete federation platform
- **[Riff Ecosystem](https://github.com/troykirin)** - All gateway tools and utilities

---

**Made with ❤️ by [@troykirin](https://github.com/troykirin) as the free gateway into NabiOS**
