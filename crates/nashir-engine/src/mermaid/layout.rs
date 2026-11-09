//! A compact layered graph layout (Sugiyama style) for flowcharts, state,
//! class and ER diagrams.
//!
//! 1. Cycles are broken by reversing DFS back edges.
//! 2. Nodes are ranked by longest path, then sources are pulled down next
//!    to their children.
//! 3. Each rank is ordered by repeated barycenter sweeps to reduce crossings.
//! 4. Ranks are placed along the main axis; within a rank, nodes are pulled
//!    toward the mean of their neighbours while keeping their order and
//!    spacing.
//! 5. Edges become cubic curves from the source's side facing the target to
//!    the target's opposite side; back edges curve around on the outside.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl Dir {
    pub fn parse(s: &str) -> Dir {
        match s.trim().to_ascii_uppercase().as_str() {
            "LR" => Dir::LeftRight,
            "RL" => Dir::RightLeft,
            "BT" => Dir::BottomTop,
            _ => Dir::TopBottom,
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, Dir::LeftRight | Dir::RightLeft)
    }

    fn flipped(self) -> bool {
        matches!(self, Dir::BottomTop | Dir::RightLeft)
    }
}

/// Where a node ends up: its centre.
#[derive(Debug, Clone, Copy, Default)]
pub struct Placed {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// An edge's route: a cubic Bézier from `from` to `to`.
#[derive(Debug, Clone, Copy)]
pub struct Route {
    pub from: (f32, f32),
    pub c1: (f32, f32),
    pub c2: (f32, f32),
    pub to: (f32, f32),
}

impl Route {
    pub fn point_at(&self, t: f32) -> (f32, f32) {
        let mt = 1.0 - t;
        let a = mt * mt * mt;
        let b = 3.0 * mt * mt * t;
        let c = 3.0 * mt * t * t;
        let d = t * t * t;
        (
            a * self.from.0 + b * self.c1.0 + c * self.c2.0 + d * self.to.0,
            a * self.from.1 + b * self.c1.1 + c * self.c2.1 + d * self.to.1,
        )
    }

    /// The direction of travel at the end (for arrowheads).
    pub fn end_tangent(&self) -> (f32, f32) {
        let (dx, dy) = (self.to.0 - self.c2.0, self.to.1 - self.c2.1);
        if dx.abs() + dy.abs() < 0.01 {
            (self.to.0 - self.from.0, self.to.1 - self.from.1)
        } else {
            (dx, dy)
        }
    }

    /// The direction of travel at the start, reversed (for start markers).
    pub fn start_tangent(&self) -> (f32, f32) {
        let (dx, dy) = (self.from.0 - self.c1.0, self.from.1 - self.c1.1);
        if dx.abs() + dy.abs() < 0.01 {
            (self.from.0 - self.to.0, self.from.1 - self.to.1)
        } else {
            (dx, dy)
        }
    }

    pub fn path(&self) -> String {
        use super::scene::n;
        format!(
            "M{} {}C{} {} {} {} {} {}",
            n(self.from.0),
            n(self.from.1),
            n(self.c1.0),
            n(self.c1.1),
            n(self.c2.0),
            n(self.c2.1),
            n(self.to.0),
            n(self.to.1)
        )
    }
}

pub struct LayoutInput<'a> {
    pub sizes: &'a [(f32, f32)],
    pub edges: &'a [(usize, usize)],
    pub dir: Dir,
    /// Extra room between ranks (for edge labels).
    pub rank_gap: f32,
    pub node_gap: f32,
}

pub struct LayoutOutput {
    pub nodes: Vec<Placed>,
    pub routes: Vec<Route>,
}

pub fn layout(input: &LayoutInput) -> LayoutOutput {
    let n = input.sizes.len();
    if n == 0 {
        return LayoutOutput {
            nodes: Vec::new(),
            routes: Vec::new(),
        };
    }
    // Main-axis and cross-axis sizes.
    let (main, cross): (Vec<f32>, Vec<f32>) = input
        .sizes
        .iter()
        .map(|&(w, h)| if input.dir.horizontal() { (w, h) } else { (h, w) })
        .unzip();

    // 1. Break cycles.
    let mut out_adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in input.edges {
        if a != b && a < n && b < n {
            out_adj[a].push(b);
        }
    }
    let mut state = vec![0u8; n]; // 0 new, 1 on stack, 2 done
    let mut reversed_pairs = std::collections::HashSet::new();
    for root in 0..n {
        if state[root] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
        state[root] = 1;
        while let Some(top) = stack.last_mut() {
            let v = top.0;
            if top.1 < out_adj[v].len() {
                let w = out_adj[v][top.1];
                top.1 += 1;
                match state[w] {
                    0 => {
                        state[w] = 1;
                        stack.push((w, 0));
                    }
                    1 => {
                        reversed_pairs.insert((v, w));
                    }
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    let dag: Vec<(usize, usize, bool)> = input
        .edges
        .iter()
        .filter(|(a, b)| a != b && *a < n && *b < n)
        .map(|&(a, b)| {
            if reversed_pairs.contains(&(a, b)) {
                (b, a, true)
            } else {
                (a, b, false)
            }
        })
        .collect();

    // 2. Longest-path ranking.
    let mut indeg = vec![0usize; n];
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b, _) in &dag {
        succ[a].push(b);
        pred[b].push(a);
        indeg[b] += 1;
    }
    let mut rank = vec![0usize; n];
    let mut queue: std::collections::VecDeque<usize> = (0..n).filter(|&v| indeg[v] == 0).collect();
    let mut order = Vec::with_capacity(n);
    let mut deg = indeg.clone();
    while let Some(v) = queue.pop_front() {
        order.push(v);
        for &w in &succ[v] {
            rank[w] = rank[w].max(rank[v] + 1);
            deg[w] -= 1;
            if deg[w] == 0 {
                queue.push_back(w);
            }
        }
    }
    // Pull sources down next to their highest child.
    for &v in order.iter().rev() {
        if pred[v].is_empty() && !succ[v].is_empty() {
            let min_child = succ[v].iter().map(|&w| rank[w]).min().unwrap();
            rank[v] = min_child.saturating_sub(1).max(rank[v]);
        }
    }
    let ranks = rank.iter().max().copied().unwrap_or(0) + 1;
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); ranks];
    for v in 0..n {
        layers[rank[v]].push(v);
    }

    // 3. Order with barycenter sweeps.
    let mut pos_in_layer = vec![0f32; n];
    let refresh = |layers: &Vec<Vec<usize>>, pos: &mut Vec<f32>| {
        for layer in layers {
            for (i, &v) in layer.iter().enumerate() {
                pos[v] = i as f32;
            }
        }
    };
    refresh(&layers, &mut pos_in_layer);
    for sweep in 0..8 {
        let down = sweep % 2 == 0;
        let range: Vec<usize> = if down {
            (1..ranks).collect()
        } else {
            (0..ranks.saturating_sub(1)).rev().collect()
        };
        for r in range {
            let mut keyed: Vec<(f32, usize)> = layers[r]
                .iter()
                .map(|&v| {
                    let neighbours = if down { &pred[v] } else { &succ[v] };
                    let adj: Vec<f32> = neighbours
                        .iter()
                        .filter(|&&u| rank[u] + 1 == rank[v] || rank[v] + 1 == rank[u])
                        .map(|&u| pos_in_layer[u])
                        .collect();
                    let key = if adj.is_empty() {
                        pos_in_layer[v]
                    } else {
                        adj.iter().sum::<f32>() / adj.len() as f32
                    };
                    (key, v)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            layers[r] = keyed.into_iter().map(|(_, v)| v).collect();
            refresh(&layers, &mut pos_in_layer);
        }
    }

    // 4. Coordinates.
    let mut main_pos = vec![0f32; n];
    let mut offset = 0.0;
    for layer in &layers {
        let thick = layer.iter().map(|&v| main[v]).fold(0.0, f32::max);
        for &v in layer {
            main_pos[v] = offset + thick / 2.0;
        }
        offset += thick + input.rank_gap;
    }
    let mut cross_pos = vec![0f32; n];
    for layer in &layers {
        let total: f32 = layer.iter().map(|&v| cross[v]).sum::<f32>()
            + input.node_gap * (layer.len().saturating_sub(1)) as f32;
        let mut c = -total / 2.0;
        for &v in layer {
            cross_pos[v] = c + cross[v] / 2.0;
            c += cross[v] + input.node_gap;
        }
    }
    let all_adj: Vec<Vec<usize>> = (0..n)
        .map(|v| pred[v].iter().chain(succ[v].iter()).copied().collect())
        .collect();
    for _ in 0..12 {
        for layer in &layers {
            if layer.is_empty() {
                continue;
            }
            let desired: Vec<f32> = layer
                .iter()
                .map(|&v| {
                    let a = &all_adj[v];
                    if a.is_empty() {
                        cross_pos[v]
                    } else {
                        a.iter().map(|&u| cross_pos[u]).sum::<f32>() / a.len() as f32
                    }
                })
                .collect();
            let mut placed = desired.clone();
            for i in 1..layer.len() {
                let min = placed[i - 1] + (cross[layer[i - 1]] + cross[layer[i]]) / 2.0 + input.node_gap;
                if placed[i] < min {
                    placed[i] = min;
                }
            }
            let shift = desired.iter().sum::<f32>() / desired.len() as f32
                - placed.iter().sum::<f32>() / placed.len() as f32;
            for (i, &v) in layer.iter().enumerate() {
                cross_pos[v] = placed[i] + shift;
            }
        }
    }

    let total_main = offset - input.rank_gap;
    let mut nodes: Vec<Placed> = (0..n)
        .map(|v| {
            let m = if input.dir.flipped() {
                total_main - main_pos[v]
            } else {
                main_pos[v]
            };
            let (x, y) = if input.dir.horizontal() {
                (m, cross_pos[v])
            } else {
                (cross_pos[v], m)
            };
            Placed {
                x,
                y,
                w: input.sizes[v].0,
                h: input.sizes[v].1,
            }
        })
        .collect();
    // Shift so nothing is negative (the scene is fitted later anyway).
    let min_x = nodes.iter().map(|p| p.x - p.w / 2.0).fold(f32::MAX, f32::min);
    let min_y = nodes.iter().map(|p| p.y - p.h / 2.0).fold(f32::MAX, f32::min);
    for p in &mut nodes {
        p.x -= min_x;
        p.y -= min_y;
    }

    // 5. Routes, in the order of the input edges.
    let routes = input
        .edges
        .iter()
        .map(|&(a, b)| {
            let a = a.min(n - 1);
            let b = b.min(n - 1);
            if a == b {
                return self_loop(nodes[a], input.dir);
            }
            let back = reversed_pairs.contains(&(a, b));
            route(nodes[a], nodes[b], input.dir, back)
        })
        .collect();
    LayoutOutput { nodes, routes }
}

fn self_loop(p: Placed, dir: Dir) -> Route {
    if dir.horizontal() {
        let y = p.y - p.h / 2.0;
        Route {
            from: (p.x - 8.0, y),
            c1: (p.x - 30.0, y - 40.0),
            c2: (p.x + 30.0, y - 40.0),
            to: (p.x + 8.0, y),
        }
    } else {
        let x = p.x + p.w / 2.0;
        Route {
            from: (x, p.y - 8.0),
            c1: (x + 40.0, p.y - 30.0),
            c2: (x + 40.0, p.y + 30.0),
            to: (x, p.y + 8.0),
        }
    }
}

fn route(a: Placed, b: Placed, dir: Dir, back: bool) -> Route {
    if dir.horizontal() {
        let forward = (b.x > a.x) != back || (b.x - a.x).abs() < 1.0;
        let (sx, tx) = if b.x >= a.x {
            (a.x + a.w / 2.0, b.x - b.w / 2.0)
        } else {
            (a.x - a.w / 2.0, b.x + b.w / 2.0)
        };
        if back && !forward {
            // Around the outside, below both nodes.
            let y1 = a.y + a.h / 2.0;
            let y2 = b.y + b.h / 2.0;
            let low = y1.max(y2) + 40.0;
            return Route {
                from: (a.x, y1),
                c1: (a.x, low),
                c2: (b.x, low),
                to: (b.x, y2),
            };
        }
        let dx = (tx - sx) * 0.5;
        Route {
            from: (sx, a.y),
            c1: (sx + dx, a.y),
            c2: (tx - dx, b.y),
            to: (tx, b.y),
        }
    } else {
        let (sy, ty) = if b.y >= a.y {
            (a.y + a.h / 2.0, b.y - b.h / 2.0)
        } else {
            (a.y - a.h / 2.0, b.y + b.h / 2.0)
        };
        if back {
            let x1 = a.x + a.w / 2.0;
            let x2 = b.x + b.w / 2.0;
            let right = x1.max(x2) + 40.0;
            return Route {
                from: (x1, a.y),
                c1: (right, a.y),
                c2: (right, b.y),
                to: (x2, b.y),
            };
        }
        let dy = (ty - sy) * 0.5;
        Route {
            from: (a.x, sy),
            c1: (a.x, sy + dy),
            c2: (b.x, ty - dy),
            to: (b.x, ty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_goes_down() {
        let sizes = [(80.0, 40.0), (80.0, 40.0), (80.0, 40.0)];
        let edges = [(0, 1), (1, 2)];
        let out = layout(&LayoutInput {
            sizes: &sizes,
            edges: &edges,
            dir: Dir::TopBottom,
            rank_gap: 40.0,
            node_gap: 30.0,
        });
        assert!(out.nodes[0].y < out.nodes[1].y && out.nodes[1].y < out.nodes[2].y);
        assert!((out.nodes[0].x - out.nodes[2].x).abs() < 1.0);
    }

    #[test]
    fn left_right_and_cycles() {
        let sizes = [(80.0, 40.0), (80.0, 40.0), (80.0, 40.0)];
        let edges = [(0, 1), (1, 2), (2, 0), (1, 1)];
        let out = layout(&LayoutInput {
            sizes: &sizes,
            edges: &edges,
            dir: Dir::LeftRight,
            rank_gap: 40.0,
            node_gap: 30.0,
        });
        assert!(out.nodes[0].x < out.nodes[1].x && out.nodes[1].x < out.nodes[2].x);
        assert_eq!(out.routes.len(), 4);
    }

    #[test]
    fn siblings_do_not_overlap() {
        let sizes = [(80.0, 40.0), (100.0, 40.0), (100.0, 40.0), (100.0, 40.0)];
        let edges = [(0, 1), (0, 2), (0, 3)];
        let out = layout(&LayoutInput {
            sizes: &sizes,
            edges: &edges,
            dir: Dir::TopBottom,
            rank_gap: 40.0,
            node_gap: 30.0,
        });
        let mut xs: Vec<f32> = (1..4).map(|i| out.nodes[i].x).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(xs[1] - xs[0] >= 129.0 && xs[2] - xs[1] >= 129.0);
    }
}
