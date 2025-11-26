//! Riff DAG TUI (3-pane) using ratatui + crossterm + petgraph.
//!
//! - Left pane: Node list (filterable)
//! - Top-right: Node details for selection
//! - Bottom-right: Layered DAG textual view (depth-limited)
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, BufRead, BufReader},
    time::{Duration, Instant},
};

use clap::Parser;
use color_eyre::eyre::{eyre, Result, WrapErr};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::Direction::{Incoming, Outgoing};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, canvas::Canvas, List, ListItem, ListState, Paragraph, Wrap, Clear},
    Terminal,
};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum EventLine {
    #[serde(rename = "node")]
    Node {
        id: String,
        label: Option<String>,
        span: Option<String>,
        tags: Option<Vec<String>>,
        ts: Option<String>,
    },
    #[serde(rename = "edge")]
    Edge { from: String, to: String },
}

#[derive(Debug, Clone)]
struct NodeData {
    id: String,
    label: String,
    span: String,
    tags: Vec<String>,
    ts: String,
}

impl NodeData {
    fn display_label(&self) -> String {
        if self.label.is_empty() {
            self.id.clone()
        } else {
            format!("{} · {}", self.id, self.label)
        }
    }
}

#[derive(Debug, Default)]
struct GraphModel {
    graph: StableDiGraph<NodeData, ()>,
    // map id -> node index
    indices: HashMap<String, NodeIndex>,
}

impl GraphModel {
    fn new() -> Self {
        Self::default()
    }
    fn upsert_node(&mut self, id: &str, nd: NodeData) -> NodeIndex {
        if let Some(idx) = self.indices.get(id) {
            let idx = *idx;
            if let Some(node) = self.graph.node_weight_mut(idx) {
                *node = nd;
            }
            idx
        } else {
            let idx = self.graph.add_node(nd);
            self.indices.insert(id.to_string(), idx);
            idx
        }
    }
    fn ensure_node_id(&mut self, id: &str) -> NodeIndex {
        if let Some(idx) = self.indices.get(id) {
            *idx
        } else {
            let nd = NodeData {
                id: id.to_string(),
                label: "".into(),
                span: "".into(),
                tags: vec![],
                ts: "".into(),
            };
            self.upsert_node(id, nd)
        }
    }
    fn add_edge(&mut self, from: &str, to: &str) {
        if let (Some(&a), Some(&b)) = (self.indices.get(from), self.indices.get(to)) {
            self.graph.add_edge(a, b, ());
        }
    }
    fn parents_of(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(idx, Incoming).collect()
    }
    fn children_of(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(idx, Outgoing).collect()
    }
    fn degree(&self, idx: NodeIndex) -> (usize, usize) {
        (self.parents_of(idx).len(), self.children_of(idx).len())
    }
}

#[derive(Parser, Debug)]
#[command(name = "riff-dag-tui")]
#[command(about = "Three-pane DAG inspector for riff/memory spans")]
struct Args {
    /// Optional path to a JSONL file with node/edge events
    #[arg(short, long)]
    input: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeType {
    Prompt,
    Response,
    Tool,
    Error,
    Event,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Filter,
    HelpOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DagViewMode {
    Text,
    Canvas,
}

struct App {
    gm: GraphModel,
    order: Vec<NodeIndex>,         // filtered display order
    list_state: ListState,
    filter_text: String,
    mode: Mode,
    dag_view_mode: DagViewMode,    // Text or Canvas view for DAG panel
    last_tick: Instant,
    tick_rate: Duration,
}

impl App {
    fn new(gm: GraphModel) -> Self {
        let mut list_state = ListState::default();
        let order: Vec<NodeIndex> = gm.graph.node_indices().collect();
        if !order.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            gm,
            order,
            list_state,
            filter_text: String::new(),
            mode: Mode::Normal,
            dag_view_mode: DagViewMode::Text,
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(200),
        }
    }

    fn selected(&self) -> Option<NodeIndex> {
        self.list_state.selected().map(|i| self.order.get(i).copied()).flatten()
    }

    fn with_filter(mut self, query: &str) -> Self {
        self.apply_filter(query);
        self
    }

    fn apply_filter(&mut self, query: &str) {
        let q = query.trim().to_lowercase();
        self.filter_text = q.clone();
        if q.is_empty() {
            self.order = self.gm.graph.node_indices().collect();
        } else {
            self.order = self
                .gm
                .graph
                .node_indices()
                .filter(|&idx| {
                    let nd = &self.gm.graph[idx];
                    let hay = format!(
                        "{} {} {} {}",
                        nd.id,
                        nd.label,
                        nd.span,
                        nd.tags.iter().cloned().collect::<Vec<_>>().join(" ")
                    )
                    .to_lowercase();
                    hay.contains(&q)
                })
                .collect();
        }
        // reset selection into range
        let len = self.order.len();
        if len == 0 {
            self.list_state.select(None);
        } else {
            let i = self.list_state.selected().unwrap_or(0).min(len - 1);
            self.list_state.select(Some(i));
        }
    }

    fn on_up(&mut self) {
        if self.order.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let i = if i == 0 { self.order.len() - 1 } else { i - 1 };
        self.list_state.select(Some(i));
    }
    fn on_down(&mut self) {
        if self.order.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let i = if i >= self.order.len() - 1 { 0 } else { i + 1 };
        self.list_state.select(Some(i));
    }

    fn toggle_dag_view(&mut self) {
        self.dag_view_mode = match self.dag_view_mode {
            DagViewMode::Text => DagViewMode::Canvas,
            DagViewMode::Canvas => DagViewMode::Text,
        };
    }
}

fn load_graph_from_jsonl(path: Option<String>) -> Result<GraphModel> {
    let mut gm = GraphModel::new();

    let reader: Box<dyn BufRead> = if let Some(p) = path {
        let f = File::open(&p).wrap_err("failed to open input file")?;
        Box::new(BufReader::new(f))
    } else {
        // Fallback to embedded sample (the same as assets/sample.jsonl).
        let data = include_str!("../assets/sample.jsonl");
        Box::new(BufReader::new(data.as_bytes()))
    };

    for (lineno, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<EventLine>(&line) {
            Ok(EventLine::Node { id, label, span, tags, ts }) => {
                let nd = NodeData {
                    id: id.clone(),
                    label: label.unwrap_or_default(),
                    span: span.unwrap_or_default(),
                    tags: tags.unwrap_or_default(),
                    ts: ts.unwrap_or_default(),
                };
                gm.upsert_node(&id, nd);
            }
            Ok(EventLine::Edge { from, to }) => {
                // Only add the edge if both endpoints exist; otherwise skip silently.
                if gm.indices.contains_key(&from) && gm.indices.contains_key(&to) {
                    gm.add_edge(&from, &to);
                } else {
                    eprintln!("[warn] edge references missing node(s) at line {}: {} -> {}", lineno + 1, from, to);
                }
            }
            Err(err) => eprintln!("[warn] bad JSON at line {}: {} (content: {})", lineno + 1, err, line),
        }
    }

    Ok(gm)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)].as_ref())
        .split(f.size());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)].as_ref())
        .split(main_chunks[1]);

    // LEFT: Node list + filter status
    let items: Vec<ListItem> = app
        .order
        .iter()
        .map(|&idx| {
            let nd = &app.gm.graph[idx];
            let (pin, pout) = app.gm.degree(idx);
            let text = format!("{}  (↑{} ↓{})", nd.display_label(), pin, pout);
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Nodes ").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    // RIGHT TOP: Details of selected node
    let details = if let Some(idx) = app.selected() {
        let nd = &app.gm.graph[idx];
        let parents = app.gm.parents_of(idx);
        let children = app.gm.children_of(idx);
        let parent_lines = parents
            .iter()
            .map(|&p| format!("  ← {}", app.gm.graph[p].display_label()))
            .join("\n");
        let child_lines = children
            .iter()
            .map(|&c| format!("  → {}", app.gm.graph[c].display_label()))
            .join("\n");

        let info = format!(
            "id: {}\nlabel: {}\nspan: {}\nts: {}\ntags: {}\n\nparents:\n{}\n\nchildren:\n{}\n",
            nd.id,
            if nd.label.is_empty() { "(none)" } else { &nd.label },
            if nd.span.is_empty() { "(none)" } else { &nd.span },
            if nd.ts.is_empty() { "(n/a)" } else { &nd.ts },
            if nd.tags.is_empty() { "(none)".to_string() } else { nd.tags.join(", ") },
            if parent_lines.is_empty() { "(none)".to_string() } else { parent_lines },
            if child_lines.is_empty() { "(none)".to_string() } else { child_lines },
        );

        Paragraph::new(info)
            .block(Block::default().title(" Node Details ").borders(Borders::ALL))
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new("No selection")
            .block(Block::default().title(" Node Details ").borders(Borders::ALL))
    };
    f.render_widget(details, right_chunks[0]);

    // RIGHT BOTTOM: Layered DAG text or canvas view (toggle with Tab)
    if let Some(idx) = app.selected() {
        match app.dag_view_mode {
            DagViewMode::Text => {
                let dag_text = build_layered_dag_text(&app.gm, idx, 2);
                let dag_paragraph = Paragraph::new(dag_text)
                    .block(Block::default().title(" DAG View (text, depth 2) ").borders(Borders::ALL))
                    .wrap(Wrap { trim: false });
                f.render_widget(dag_paragraph, right_chunks[1]);
            }
            DagViewMode::Canvas => {
                // Canvas widget with node shapes and edges
                let positions = layout_nodes(&app.gm, idx, 2);
                let dag_canvas = Canvas::default()
                    .block(Block::default().title(" DAG View (shapes, depth 2) ").borders(Borders::ALL))
                    .x_bounds([0.0, 100.0])
                    .y_bounds([0.0, 50.0])
                    .paint(|ctx| {
                        // Draw edges first (so they appear behind nodes)
                        for (from_idx, from_pos) in &positions {
                            for to_idx in app.gm.children_of(*from_idx) {
                                if let Some(to_pos) = positions.get(&to_idx) {
                                    draw_edge_line(ctx, *from_pos, *to_pos, Color::Gray);
                                }
                            }
                        }

                        // Draw nodes
                        for (node_idx, pos) in &positions {
                            let nd = &app.gm.graph[*node_idx];
                            let node_type = classify_node_type(&nd.tags);
                            let is_selected = *node_idx == idx;
                            draw_node_shape(ctx, pos.0, pos.1, node_type, is_selected);
                        }
                    });
                f.render_widget(dag_canvas, right_chunks[1]);
            }
        }
    } else {
        let empty = Paragraph::new("No selection")
            .block(Block::default().title(" DAG View ").borders(Borders::ALL));
        f.render_widget(empty, right_chunks[1]);
    }

    // Status / help line overlay at bottom
    let dag_mode_str = match app.dag_view_mode {
        DagViewMode::Text => "text",
        DagViewMode::Canvas => "canvas",
    };
    let status = match app.mode {
        Mode::Normal => format!("Normal | / filter | c clear | Tab toggle DAG ({}) | q quit | ? help | filter: '{}'", dag_mode_str, app.filter_text),
        Mode::Filter => format!("Filter mode — type to filter, Enter accept, Esc exit, Backspace delete | query: '{}'", app.filter_text),
        Mode::HelpOverlay => "Help — Up/Down/j/k move · / filter · c clear filter · Tab toggle DAG view · q quit".to_string(),
    };
    let area = Rect {
        x: f.size().x,
        y: f.size().y.saturating_sub(1),
        width: f.size().width,
        height: 1,
    };
    let status_paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default());
    f.render_widget(Clear, area);
    f.render_widget(status_paragraph, area);
}

fn build_layered_dag_text(gm: &GraphModel, center: NodeIndex, depth: usize) -> Vec<Line<'static>> {
    // BFS layers outward (incoming = negative depth, outgoing = positive depth)
    // We'll collect up to depth for both directions and render columns.
    let mut parents_layers: Vec<Vec<NodeIndex>> = Vec::new();
    let mut frontier: Vec<NodeIndex> = vec![center];
    for _ in 0..depth {
        let mut next = Vec::new();
        let mut layer = Vec::new();
        for &n in &frontier {
            for p in gm.graph.neighbors_directed(n, Incoming) {
                if !layer.contains(&p) {
                    layer.push(p);
                    next.push(p);
                }
            }
        }
        if layer.is_empty() { break; }
        parents_layers.push(layer);
        frontier = next;
    }

    let mut children_layers: Vec<Vec<NodeIndex>> = Vec::new();
    let mut frontier: Vec<NodeIndex> = vec![center];
    for _ in 0..depth {
        let mut next = Vec::new();
        let mut layer = Vec::new();
        for &n in &frontier {
            for c in gm.graph.neighbors_directed(n, Outgoing) {
                if !layer.contains(&c) {
                    layer.push(c);
                    next.push(c);
                }
            }
        }
        if layer.is_empty() { break; }
        children_layers.push(layer);
        frontier = next;
    }

    // Prepare columns: grand-parents ... parents | [center] | children ... grand-children
    let mut columns: Vec<Vec<String>> = Vec::new();

    // parents (furthest first)
    for layer in parents_layers.iter().rev() {
        columns.push(layer.iter().map(|&idx| label_for(gm, idx)).collect());
    }

    // center
    columns.push(vec![format!("[{}]", label_for(gm, center))]);

    // children
    for layer in children_layers.iter() {
        columns.push(layer.iter().map(|&idx| label_for(gm, idx)).collect());
    }

    // Normalize column heights
    let max_rows = columns.iter().map(|col| col.len()).max().unwrap_or(0).max(1);
    for col in columns.iter_mut() {
        while col.len() < max_rows {
            col.push(String::new());
        }
    }

    // Render as grid with connectors simplified using arrows on the center rows
    let mut lines: Vec<Line> = Vec::new();
    // Header
    lines.push(Line::from(Span::styled(
        "Layered DAG (parents ← [selected] → children)",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // Grid
    for row in 0..max_rows {
        let cells = columns.iter().map(|col| format!("{: ^24}", col[row])).collect::<Vec<_>>();
        lines.push(Line::from(cells.join("|")));
    }

    // Legend
    lines.push(Line::from(" "));
    lines.push(Line::from(Span::styled(
        "Note: This is a textual, depth-limited view. Use selection to explore neighbors.",
        Style::default().fg(Color::Gray),
    )));
    lines
}

fn label_for(gm: &GraphModel, idx: NodeIndex) -> String {
    let nd = &gm.graph[idx];
    let base = if nd.label.is_empty() { nd.id.clone() } else { format!("{} · {}", nd.id, nd.label) };
    base
}

fn classify_node_type(tags: &[String]) -> NodeType {
    // Classify node type based on tags
    for tag in tags {
        let lower = tag.to_lowercase();
        if lower.contains("prompt") {
            return NodeType::Prompt;
        }
        if lower.contains("response") {
            return NodeType::Response;
        }
        if lower.contains("tool") {
            return NodeType::Tool;
        }
        if lower.contains("error") {
            return NodeType::Error;
        }
        if lower.contains("event") {
            return NodeType::Event;
        }
    }
    NodeType::Unknown
}

fn draw_node_shape(ctx: &mut ratatui::widgets::canvas::Context, x: f64, y: f64, node_type: NodeType, selected: bool) {
    use ratatui::widgets::canvas::{Points, Line};

    let color = match node_type {
        NodeType::Prompt => Color::Cyan,
        NodeType::Response => Color::Green,
        NodeType::Tool => Color::Yellow,
        NodeType::Error => Color::Red,
        NodeType::Event => Color::Magenta,
        NodeType::Unknown => Color::White,
    };

    let size = if selected { 2.0 } else { 1.5 };

    match node_type {
        NodeType::Prompt => {
            // Rectangle for prompt
            let sz = size;
            ctx.draw(&Line::new(x - sz, y - sz, x + sz, y - sz, color));
            ctx.draw(&Line::new(x + sz, y - sz, x + sz, y + sz, color));
            ctx.draw(&Line::new(x + sz, y + sz, x - sz, y + sz, color));
            ctx.draw(&Line::new(x - sz, y + sz, x - sz, y - sz, color));
        }
        NodeType::Response => {
            // Circle for response - approximate with points
            let mut points = Vec::new();
            for angle in (0..360).step_by(15) {
                let rad = (angle as f64).to_radians();
                let px = x + size * rad.cos();
                let py = y + size * rad.sin();
                points.push((px, py));
            }
            ctx.draw(&Points { coords: &points, color });
        }
        NodeType::Tool => {
            // Diamond for tool
            ctx.draw(&Line::new(x, y + size, x + size, y, color));
            ctx.draw(&Line::new(x + size, y, x, y - size, color));
            ctx.draw(&Line::new(x, y - size, x - size, y, color));
            ctx.draw(&Line::new(x - size, y, x, y + size, color));
        }
        NodeType::Error => {
            // X for error
            ctx.draw(&Line::new(x - size, y - size, x + size, y + size, color));
            ctx.draw(&Line::new(x - size, y + size, x + size, y - size, color));
        }
        NodeType::Event => {
            // Dot for event
            let point = vec![(x, y)];
            ctx.draw(&Points { coords: &point, color });
        }
        NodeType::Unknown => {
            // Square for unknown
            let sz = size;
            ctx.draw(&Line::new(x - sz, y - sz, x + sz, y - sz, color));
            ctx.draw(&Line::new(x + sz, y - sz, x + sz, y + sz, color));
            ctx.draw(&Line::new(x + sz, y + sz, x - sz, y + sz, color));
            ctx.draw(&Line::new(x - sz, y + sz, x - sz, y - sz, color));
        }
    }
}

fn layout_nodes(gm: &GraphModel, center: NodeIndex, depth: usize) -> HashMap<NodeIndex, (f64, f64)> {
    // Position nodes in a layered graph: parents | [center] | children
    let mut positions: HashMap<NodeIndex, (f64, f64)> = HashMap::new();

    // Collect parent and child layers
    let mut parents_layers: Vec<Vec<NodeIndex>> = Vec::new();
    let mut frontier: Vec<NodeIndex> = vec![center];
    for _ in 0..depth {
        let mut next = Vec::new();
        let mut layer = Vec::new();
        for &n in &frontier {
            for p in gm.graph.neighbors_directed(n, Incoming) {
                if !layer.contains(&p) {
                    layer.push(p);
                    next.push(p);
                }
            }
        }
        if layer.is_empty() { break; }
        parents_layers.push(layer);
        frontier = next;
    }

    let mut children_layers: Vec<Vec<NodeIndex>> = Vec::new();
    let mut frontier: Vec<NodeIndex> = vec![center];
    for _ in 0..depth {
        let mut next = Vec::new();
        let mut layer = Vec::new();
        for &n in &frontier {
            for c in gm.graph.neighbors_directed(n, Outgoing) {
                if !layer.contains(&c) {
                    layer.push(c);
                    next.push(c);
                }
            }
        }
        if layer.is_empty() { break; }
        children_layers.push(layer);
        frontier = next;
    }

    // Position parents (reversed to show grandparents on left)
    for (layer_idx, layer) in parents_layers.iter().rev().enumerate() {
        let x = 10.0 + layer_idx as f64 * 15.0;
        for (node_idx, &node) in layer.iter().enumerate() {
            let y = 25.0 - (layer.len() as f64 / 2.0) + node_idx as f64;
            positions.insert(node, (x, y));
        }
    }

    // Position center
    positions.insert(center, (50.0, 25.0));

    // Position children
    for (layer_idx, layer) in children_layers.iter().enumerate() {
        let x = 70.0 + layer_idx as f64 * 15.0;
        for (node_idx, &node) in layer.iter().enumerate() {
            let y = 25.0 - (layer.len() as f64 / 2.0) + node_idx as f64;
            positions.insert(node, (x, y));
        }
    }

    positions
}

fn draw_edge_line(ctx: &mut ratatui::widgets::canvas::Context, from: (f64, f64), to: (f64, f64), color: Color) {
    use ratatui::widgets::canvas::Line;

    // Draw a line from source to destination
    ctx.draw(&Line::new(from.0, from.1, to.0, to.1, color));

    // Draw arrowhead
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.1 {
        let ux = dx / len;
        let uy = dy / len;

        // Arrow tip at destination
        let arrow_size = 1.5;
        let left = (to.0 - arrow_size * (ux + uy), to.1 - arrow_size * (uy - ux));
        let right = (to.0 - arrow_size * (ux - uy), to.1 - arrow_size * (uy + ux));

        ctx.draw(&Line::new(to.0, to.1, left.0, left.1, color));
        ctx.draw(&Line::new(to.0, to.1, right.0, right.1, color));
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let gm = load_graph_from_jsonl(args.input)?;
    let mut app = App::new(gm);

    let mut terminal = setup_terminal()?;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(terminal)?;
    res
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        // Input handling with periodic tick to keep UI responsive
        let timeout = app.tick_rate.checked_sub(app.last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_key(app, key)? {
                    break; // quit
                }
            }
        }
        if app.last_tick.elapsed() >= app.tick_rate {
            app.last_tick = Instant::now();
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    match app.mode {
        Mode::Normal => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => app.on_up(),
            KeyCode::Down | KeyCode::Char('j') => app.on_down(),
            KeyCode::Char('/') => app.mode = Mode::Filter,
            KeyCode::Char('c') => {
                app.apply_filter("");
            }
            KeyCode::Char('?') => app.mode = Mode::HelpOverlay,
            KeyCode::Tab => app.toggle_dag_view(),
            _ => {}
        },
        Mode::Filter => match key.code {
            KeyCode::Esc => app.mode = Mode::Normal,
            KeyCode::Enter => app.mode = Mode::Normal,
            KeyCode::Backspace => {
                app.filter_text.pop();
                let q = app.filter_text.clone();
                app.apply_filter(&q);
            }
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.filter_text.push(ch);
                    let q = app.filter_text.clone();
                    app.apply_filter(&q);
                }
            }
            _ => {}
        },
        Mode::HelpOverlay => match key.code {
            KeyCode::Esc | KeyCode::Char('?') => app.mode = Mode::Normal,
            _ => {}
        },
    }
    Ok(false)
}
