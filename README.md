# Riff DAG TUI

A **three-pane** terminal UI for exploring a DAG of "riffs" or memory spans. Built with
[`ratatui`](https://crates.io/crates/ratatui), `crossterm`, and `petgraph`.

```
+---------------------------------------------------------------+
|  Node List (Left)        |  Node Details (Top Right)         |
|                          |  DAG View (Bottom Right)          |
+---------------------------------------------------------------+
```

- Left: list of nodes (filterable).  
- Top-right: rich details for the selected node (spans, tags, neighbors).  
- Bottom-right: a reactive **layered DAG** textual view that updates when selection changes.

## Quick start

You need Rust (stable). Then:

```bash
cd riff-dag-tui
cargo run -- --input assets/sample.jsonl
```

If you **omit** `--input`, the app will fall back to the embedded sample dataset.

## Controls

- `Up` / `Down` or `k` / `j` – move selection
- `/` – enter filtering mode; type to filter
- `Backspace` – delete in filter mode
- `Enter` – accept filter and return to normal mode
- `Esc` – exit filter mode (keeps current filter)
- `c` – clear filter
- `q` – quit
- `?` – show help in the status bar

## Data format

Newline-delimited JSON (JSONL). Two kinds of lines are recognized:
- `{"type":"node","id":"mem_001","label":"ingest","span":"epoch:init","tags":["io"],"ts":"2025-05-22T10:00:00Z"}`
- `{"type":"edge","from":"mem_001","to":"mem_002"}`

Nodes may appear before or after edges; edges referencing missing nodes will be ignored with a warning.

## Notes

- The bottom-right DAG panel renders a **layered**, depth-limited textual view (parents/children up to depth 2).
- This is a solid base; you can later swap the textual renderer for a canvas/graph layout if you prefer.
