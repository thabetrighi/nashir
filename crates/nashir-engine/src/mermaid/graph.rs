//! The node-and-edge model shared by flowcharts, state, class and ER
//! diagrams, and its rendering into a [`Scene`].

use std::collections::HashMap;

use super::layout::{layout, Dir, LayoutInput, Route};
use super::scene::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Rect,
    Round,
    Stadium,
    Diamond,
    Circle,
    DoubleCircle,
    Hexagon,
    Subroutine,
    Cylinder,
    Asymmetric,
    Parallelogram,
    ParallelogramAlt,
    Trapezoid,
    TrapezoidAlt,
    /// A state diagram's start: a filled dot.
    Start,
    /// A state diagram's end: a ringed dot.
    End,
    /// A fork/join bar.
    Bar,
    /// A box with a header and compartments (class and entity).
    Compartments,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    None,
    Arrow,
    Circle,
    Cross,
    /// Inheritance / realisation: a hollow triangle.
    Triangle,
    DiamondFilled,
    DiamondHollow,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub lines: Vec<String>,
    pub shape: Shape,
    /// For [`Shape::Compartments`]: a small annotation over the title
    /// (`«interface»`) and the compartments under it.
    pub annotation: Option<String>,
    pub compartments: Vec<Vec<String>>,
    /// Whether the label was set explicitly (later bare mentions keep it).
    pub labelled: bool,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: Vec<String>,
    pub dashed: bool,
    pub thick: bool,
    pub invisible: bool,
    pub start: Marker,
    pub end: Marker,
    /// Text near each end (class multiplicities, ER cardinalities).
    pub start_text: Option<String>,
    pub end_text: Option<String>,
}

impl Edge {
    pub fn new(from: usize, to: usize) -> Edge {
        Edge {
            from,
            to,
            label: Vec::new(),
            dashed: false,
            thick: false,
            invisible: false,
            start: Marker::None,
            end: Marker::Arrow,
            start_text: None,
            end_text: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub title: String,
    pub members: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub dir: Dir,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub clusters: Vec<Cluster>,
    index: HashMap<String, usize>,
}

impl Graph {
    pub fn new(dir: Dir) -> Graph {
        Graph {
            dir,
            nodes: Vec::new(),
            edges: Vec::new(),
            clusters: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// The node with `id`, created with its id as label if new.
    pub fn node(&mut self, id: &str) -> usize {
        if let Some(&i) = self.index.get(id) {
            return i;
        }
        self.nodes.push(Node {
            id: id.to_owned(),
            lines: vec![id.to_owned()],
            shape: Shape::Rect,
            annotation: None,
            compartments: Vec::new(),
            labelled: false,
        });
        self.index.insert(id.to_owned(), self.nodes.len() - 1);
        self.nodes.len() - 1
    }

    pub fn find(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    pub fn set_label(&mut self, i: usize, text: &str, shape: Shape) {
        let n = &mut self.nodes[i];
        n.lines = label_lines(text);
        n.shape = shape;
        n.labelled = true;
    }

    fn node_size(&self, n: &Node) -> (f32, f32) {
        let (tw, th) = block_size(&n.lines, FONT);
        match n.shape {
            Shape::Start => (18.0, 18.0),
            Shape::End => (22.0, 22.0),
            Shape::Bar => (70.0, 8.0),
            Shape::Compartments => {
                let mut w = tw.max(n.annotation.as_ref().map(|a| text_width(a, 12.0)).unwrap_or(0.0));
                let mut h =
                    th + if n.annotation.is_some() {
                        12.0 * LINE_HEIGHT
                    } else {
                        0.0
                    } + 16.0;
                for c in &n.compartments {
                    let (cw, ch) = block_size(c, 13.0);
                    w = w.max(cw);
                    h += if c.is_empty() { 10.0 } else { ch + 12.0 };
                }
                ((w + 28.0).max(110.0), h)
            }
            Shape::Diamond => {
                let s = (tw + 24.0).max(th + 24.0) * 1.15 + 16.0;
                (s.max(64.0), (s * 0.8).max(64.0))
            }
            Shape::Circle | Shape::DoubleCircle => {
                let d = (tw + 24.0).max(th + 24.0).max(48.0)
                    + if n.shape == Shape::DoubleCircle { 8.0 } else { 0.0 };
                (d, d)
            }
            Shape::Hexagon => ((tw + 52.0).max(80.0), (th + 20.0).max(40.0)),
            Shape::Parallelogram
            | Shape::ParallelogramAlt
            | Shape::Trapezoid
            | Shape::TrapezoidAlt
            | Shape::Asymmetric => ((tw + 56.0).max(80.0), (th + 20.0).max(40.0)),
            Shape::Cylinder => ((tw + 28.0).max(70.0), (th + 34.0).max(54.0)),
            Shape::Subroutine => ((tw + 40.0).max(80.0), (th + 20.0).max(40.0)),
            _ => ((tw + 30.0).max(64.0), (th + 20.0).max(40.0)),
        }
    }

    /// Lays the graph out and draws it.
    pub fn render(&self, kind: &'static str) -> Scene {
        let mut scene = Scene::new(kind);
        let sizes: Vec<(f32, f32)> = self.nodes.iter().map(|n| self.node_size(n)).collect();
        let pairs: Vec<(usize, usize)> = self.edges.iter().map(|e| (e.from, e.to)).collect();
        let labelled = self.edges.iter().any(|e| !e.label.is_empty());
        let out = layout(&LayoutInput {
            sizes: &sizes,
            edges: &pairs,
            dir: self.dir,
            rank_gap: if labelled { 64.0 } else { 46.0 },
            node_gap: 34.0,
        });

        // Clusters behind everything.
        for c in &self.clusters {
            if c.members.is_empty() {
                continue;
            }
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for &m in &c.members {
                let p = out.nodes[m];
                x0 = x0.min(p.x - p.w / 2.0);
                y0 = y0.min(p.y - p.h / 2.0);
                x1 = x1.max(p.x + p.w / 2.0);
                y1 = y1.max(p.y + p.h / 2.0);
            }
            let title_h = if c.title.is_empty() { 0.0 } else { 24.0 };
            let (x, y) = (x0 - 16.0, y0 - 16.0 - title_h);
            let (w, h) = (x1 - x0 + 32.0, y1 - y0 + 32.0 + title_h);
            let w = w.max(text_width(&c.title, 13.0) + 24.0);
            scene.path(
                rect(x, y, w, h, 10.0),
                Some("cluster"),
                Some("clusterBorder"),
                1.0,
                false,
            );
            if !c.title.is_empty() {
                scene.label(x + 8.0, y + 4.0, w - 16.0, 20.0, &c.title, 13.0, 600, "muted");
            }
        }

        // Edges.
        for (e, r) in self.edges.iter().zip(out.routes.iter()) {
            if e.invisible {
                continue;
            }
            let width = if e.thick { 2.6 } else { 1.5 };
            scene.path(r.path(), None, Some("edge"), width, e.dashed);
            draw_marker(&mut scene, &e.end, r.to, r.end_tangent(), width);
            draw_marker(&mut scene, &e.start, r.from, r.start_tangent(), width);
            if !e.label.is_empty() {
                let (mx, my) = r.point_at(0.5);
                let (tw, th) = block_size(&e.label, 12.5);
                let (w, h) = (tw + 12.0, th + 4.0);
                scene.labels.push(Label {
                    x: mx - w / 2.0,
                    y: my - h / 2.0,
                    w,
                    h,
                    text: e.label.join("\n"),
                    size: 12.5,
                    weight: 400,
                    color: "text",
                    align: "center",
                    dir: dir_of(&e.label.join(" ")),
                    bg: Some("edgeLabel"),
                });
            }
            end_text(&mut scene, r, e.start_text.as_deref(), true);
            end_text(&mut scene, r, e.end_text.as_deref(), false);
        }

        // Nodes.
        for (node, p) in self.nodes.iter().zip(out.nodes.iter()) {
            draw_node(&mut scene, node, p.x - p.w / 2.0, p.y - p.h / 2.0, p.w, p.h);
        }
        scene.fit(12.0);
        scene
    }
}

fn dir_of(text: &str) -> &'static str {
    match crate::direction::first_strong(text) {
        Some(crate::direction::Direction::Rtl) => "rtl",
        _ => "ltr",
    }
}

fn end_text(scene: &mut Scene, r: &Route, text: Option<&str>, start: bool) {
    let Some(text) = text.filter(|t| !t.is_empty()) else {
        return;
    };
    let (x, y) = r.point_at(if start { 0.12 } else { 0.88 });
    let (tx, ty) = if start { r.start_tangent() } else { r.end_tangent() };
    let len = (tx * tx + ty * ty).sqrt().max(0.001);
    let (px, py) = (-ty / len * 12.0, tx / len * 12.0);
    let w = text_width(text, 11.5) + 6.0;
    scene.label(x + px - w / 2.0, y + py - 8.0, w, 16.0, text, 11.5, 600, "muted");
}

fn draw_marker(scene: &mut Scene, m: &Marker, at: (f32, f32), dir: (f32, f32), width: f32) {
    let size = 8.0 + width;
    match m {
        Marker::None => {}
        Marker::Arrow => scene.path(
            arrow_head(at.0, at.1, dir.0, dir.1, size),
            Some("edge"),
            None,
            0.0,
            false,
        ),
        Marker::Triangle => scene.path(
            arrow_head(at.0, at.1, dir.0, dir.1, size + 4.0),
            Some("bg"),
            Some("edge"),
            1.3,
            false,
        ),
        Marker::DiamondFilled => scene.path(
            diamond_marker(at.0, at.1, dir.0, dir.1, size),
            Some("edge"),
            Some("edge"),
            1.0,
            false,
        ),
        Marker::DiamondHollow => scene.path(
            diamond_marker(at.0, at.1, dir.0, dir.1, size),
            Some("bg"),
            Some("edge"),
            1.3,
            false,
        ),
        Marker::Circle => {
            let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.001);
            let (cx, cy) = (at.0 - dir.0 / len * 5.0, at.1 - dir.1 / len * 5.0);
            scene.path(circle(cx, cy, 4.5), Some("bg"), Some("edge"), 1.5, false)
        }
        Marker::Cross => {
            let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.001);
            let (cx, cy) = (at.0 - dir.0 / len * 6.0, at.1 - dir.1 / len * 6.0);
            let s = 4.5;
            scene.path(
                format!(
                    "M{} {}L{} {}M{} {}L{} {}",
                    n(cx - s),
                    n(cy - s),
                    n(cx + s),
                    n(cy + s),
                    n(cx - s),
                    n(cy + s),
                    n(cx + s),
                    n(cy - s)
                ),
                None,
                Some("edge"),
                2.0,
                false,
            )
        }
    }
}

fn draw_node(scene: &mut Scene, node: &Node, x: f32, y: f32, w: f32, h: f32) {
    let (fill, stroke) = (Some("node"), Some("nodeBorder"));
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    match node.shape {
        Shape::Rect => scene.path(rect(x, y, w, h, 3.0), fill, stroke, 1.4, false),
        Shape::Round => scene.path(rect(x, y, w, h, 10.0), fill, stroke, 1.4, false),
        Shape::Stadium => scene.path(rect(x, y, w, h, h / 2.0), fill, stroke, 1.4, false),
        Shape::Diamond => scene.path(
            polygon(&[(cx, y), (x + w, cy), (cx, y + h), (x, cy)]),
            fill,
            stroke,
            1.4,
            false,
        ),
        Shape::Circle => scene.path(circle(cx, cy, w / 2.0), fill, stroke, 1.4, false),
        Shape::DoubleCircle => {
            scene.path(circle(cx, cy, w / 2.0), fill, stroke, 1.4, false);
            scene.path(circle(cx, cy, w / 2.0 - 5.0), None, stroke, 1.4, false);
        }
        Shape::Hexagon => {
            let k = 18.0;
            scene.path(
                polygon(&[
                    (x + k, y),
                    (x + w - k, y),
                    (x + w, cy),
                    (x + w - k, y + h),
                    (x + k, y + h),
                    (x, cy),
                ]),
                fill,
                stroke,
                1.4,
                false,
            )
        }
        Shape::Subroutine => {
            scene.path(rect(x, y, w, h, 2.0), fill, stroke, 1.4, false);
            scene.path(
                format!(
                    "M{} {}V{}M{} {}V{}",
                    n(x + 8.0),
                    n(y),
                    n(y + h),
                    n(x + w - 8.0),
                    n(y),
                    n(y + h)
                ),
                None,
                stroke,
                1.4,
                false,
            );
        }
        Shape::Cylinder => {
            let ry = 7.0;
            let rx = w / 2.0;
            scene.path(
                format!(
                    "M{} {}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}Z",
                    n(x),
                    n(y + ry),
                    n(rx),
                    n(ry),
                    n(x + w),
                    n(y + ry),
                    n(y + h - ry),
                    n(rx),
                    n(ry),
                    n(x),
                    n(y + h - ry)
                ),
                fill,
                stroke,
                1.4,
                false,
            );
            scene.path(
                format!(
                    "M{} {}A{} {} 0 0 0 {} {}",
                    n(x),
                    n(y + ry),
                    n(rx),
                    n(ry),
                    n(x + w),
                    n(y + ry)
                ),
                None,
                stroke,
                1.4,
                false,
            );
        }
        Shape::Asymmetric => scene.path(
            polygon(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x + 16.0, cy)]),
            fill,
            stroke,
            1.4,
            false,
        ),
        Shape::Parallelogram => scene.path(
            polygon(&[(x + 14.0, y), (x + w, y), (x + w - 14.0, y + h), (x, y + h)]),
            fill,
            stroke,
            1.4,
            false,
        ),
        Shape::ParallelogramAlt => scene.path(
            polygon(&[(x, y), (x + w - 14.0, y), (x + w, y + h), (x + 14.0, y + h)]),
            fill,
            stroke,
            1.4,
            false,
        ),
        Shape::Trapezoid => scene.path(
            polygon(&[(x + 14.0, y), (x + w - 14.0, y), (x + w, y + h), (x, y + h)]),
            fill,
            stroke,
            1.4,
            false,
        ),
        Shape::TrapezoidAlt => scene.path(
            polygon(&[(x, y), (x + w, y), (x + w - 14.0, y + h), (x + 14.0, y + h)]),
            fill,
            stroke,
            1.4,
            false,
        ),
        Shape::Start => scene.path(circle(cx, cy, w / 2.0), Some("text"), None, 0.0, false),
        Shape::End => {
            scene.path(circle(cx, cy, w / 2.0), Some("bg"), Some("text"), 1.6, false);
            scene.path(circle(cx, cy, w / 2.0 - 4.5), Some("text"), None, 0.0, false);
        }
        Shape::Bar => scene.path(rect(x, y, w, h, 2.0), Some("text"), None, 0.0, false),
        Shape::Compartments => {
            scene.path(rect(x, y, w, h, 4.0), fill, stroke, 1.4, false);
            let mut cy = y + 8.0;
            if let Some(a) = &node.annotation {
                scene.label(x, cy, w, 12.0 * LINE_HEIGHT, a, 12.0, 400, "muted");
                cy += 12.0 * LINE_HEIGHT;
            }
            let (_, th) = block_size(&node.lines, FONT);
            scene.label(
                x + 6.0,
                cy,
                w - 12.0,
                th,
                &node.lines.join("\n"),
                FONT,
                700,
                "text",
            );
            cy += th + 8.0;
            for c in &node.compartments {
                scene.path(
                    format!("M{} {}H{}", n(x), n(cy), n(x + w)),
                    None,
                    stroke,
                    1.0,
                    false,
                );
                if c.is_empty() {
                    cy += 10.0;
                    continue;
                }
                let (_, ch) = block_size(c, 13.0);
                scene.label_aligned(
                    x + 10.0,
                    cy + 6.0,
                    w - 20.0,
                    ch,
                    &c.join("\n"),
                    13.0,
                    400,
                    "text",
                    "start",
                );
                cy += ch + 12.0;
            }
            return;
        }
    }
    if matches!(node.shape, Shape::Start | Shape::End | Shape::Bar) {
        return;
    }
    let (_, th) = block_size(&node.lines, FONT);
    let inset = match node.shape {
        Shape::Diamond => w * 0.18,
        Shape::Hexagon
        | Shape::Parallelogram
        | Shape::ParallelogramAlt
        | Shape::Trapezoid
        | Shape::TrapezoidAlt => 18.0,
        Shape::Subroutine => 12.0,
        _ => 6.0,
    };
    scene.label(
        x + inset,
        cy - th / 2.0 + if node.shape == Shape::Cylinder { 4.0 } else { 0.0 },
        w - inset * 2.0,
        th,
        &node.lines.join("\n"),
        FONT,
        500,
        "text",
    );
}
