//! `mindmap` and `gitGraph`.

use std::collections::HashMap;

use super::scene::*;
use super::MermaidError;

const SERIES: [&str; 8] = ["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7"];

// ---------------------------------------------------------------- mindmap

struct MNode {
    lines: Vec<String>,
    shape: u8, // 0 plain/rounded, 1 square, 2 circle, 3 cloud/bang → rounded
    children: Vec<usize>,
    branch: usize,
    w: f32,
    h: f32,
    x: f32,
    y: f32,
}

fn mind_label(raw: &str) -> (Vec<String>, u8) {
    let t = raw.trim();
    // Strip an optional id before the shape: `id[Text]`, `id((Text))`.
    let shapes: &[(&str, &str, u8)] = &[
        ("((", "))", 2),
        ("))", "((", 3),
        (")", "(", 3),
        ("{{", "}}", 1),
        ("[", "]", 1),
        ("(", ")", 0),
    ];
    for (open, close, shape) in shapes {
        if let Some(p) = t.find(open) {
            if t.ends_with(close) && p + open.len() <= t.len() - close.len() {
                return (label_lines(&t[p + open.len()..t.len() - close.len()]), *shape);
            }
        }
    }
    (label_lines(t), 0)
}

pub fn mindmap(src: &str) -> Result<Scene, MermaidError> {
    let mut nodes: Vec<MNode> = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (indent, node)
    for line in src
        .lines()
        .skip_while(|l| !l.trim_start().starts_with("mindmap"))
        .skip(1)
    {
        if line.trim().is_empty()
            || line.trim_start().starts_with("%%")
            || line.trim_start().starts_with("::icon")
            || line.trim_start().starts_with(":::")
        {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let (lines, shape) = mind_label(line);
        let (w, h) = block_size(&lines, 14.0);
        let id = nodes.len();
        while stack.last().is_some_and(|&(i, _)| i >= indent) {
            stack.pop();
        }
        let parent = stack.last().map(|&(_, p)| p);
        let branch = match parent {
            None => 0,
            Some(p) if nodes[p].branch == usize::MAX => nodes[p].children.len(),
            Some(p) => nodes[p].branch,
        };
        nodes.push(MNode {
            lines,
            shape,
            children: Vec::new(),
            branch: if parent.is_none() { usize::MAX } else { branch },
            w: w + 28.0,
            h: h + 16.0,
            x: 0.0,
            y: 0.0,
        });
        match parent {
            Some(p) => nodes[p].children.push(id),
            None if id != 0 => {
                // A second root: attach under the first.
                nodes[0].children.push(id);
                nodes[id].branch = nodes[0].children.len() - 1;
            }
            None => {}
        }
        stack.push((indent, id));
    }
    if nodes.is_empty() {
        return Err(MermaidError::Empty);
    }
    // The root's children split between the right and left sides.
    let root_children = nodes[0].children.clone();
    let right: Vec<usize> = root_children.iter().step_by(2).copied().collect();
    let left: Vec<usize> = root_children.iter().skip(1).step_by(2).copied().collect();

    fn subtree_height(nodes: &[MNode], v: usize) -> f32 {
        let own = nodes[v].h + 12.0;
        let kids: f32 = nodes[v].children.iter().map(|&c| subtree_height(nodes, c)).sum();
        own.max(kids)
    }
    fn place(nodes: &mut Vec<MNode>, v: usize, x: f32, top: f32, side: f32) {
        let height = subtree_height(nodes, v);
        nodes[v].x = x;
        nodes[v].y = top + height / 2.0;
        let kids_h: f32 = nodes[v].children.iter().map(|&c| subtree_height(nodes, c)).sum();
        let mut y = top + (height - kids_h) / 2.0;
        let next_x = x + side * (nodes[v].w / 2.0 + 48.0);
        for c in nodes[v].children.clone() {
            let h = subtree_height(nodes, c);
            let cx = next_x + side * nodes[c].w / 2.0;
            place(nodes, c, cx, y, side);
            y += h;
        }
    }
    let right_h: f32 = right.iter().map(|&c| subtree_height(&nodes, c)).sum();
    let left_h: f32 = left.iter().map(|&c| subtree_height(&nodes, c)).sum();
    let total = right_h.max(left_h).max(nodes[0].h);
    nodes[0].x = 0.0;
    nodes[0].y = total / 2.0;
    let root_w = nodes[0].w;
    for (side, group, h) in [(1.0f32, &right, right_h), (-1.0f32, &left, left_h)] {
        let mut y = (total - h) / 2.0;
        for &c in group {
            let ch = subtree_height(&nodes, c);
            let cx = side * (root_w / 2.0 + 60.0 + nodes[c].w / 2.0);
            place(&mut nodes, c, cx, y, side);
            y += ch;
        }
    }

    let mut scene = Scene::new("mindmap");
    // Edges.
    for v in 0..nodes.len() {
        for &c in &nodes[v].children {
            let (p, q) = (&nodes[v], &nodes[c]);
            let side = if q.x >= p.x { 1.0 } else { -1.0 };
            let (x1, y1) = (p.x + side * p.w / 2.0, p.y);
            let (x2, y2) = (q.x - side * q.w / 2.0, q.y);
            let mid = (x1 + x2) / 2.0;
            let color = SERIES[q.branch % SERIES.len()];
            scene.path(
                format!(
                    "M{} {}C{} {} {} {} {} {}",
                    n(x1),
                    n(y1),
                    n(mid),
                    n(y1),
                    n(mid),
                    n(y2),
                    n(x2),
                    n(y2)
                ),
                None,
                Some(color),
                if v == 0 { 3.0 } else { 2.0 },
                false,
            );
        }
    }
    for (i, m) in nodes.iter().enumerate() {
        let (x, y) = (m.x - m.w / 2.0, m.y - m.h / 2.0);
        let (fill, text) = if i == 0 {
            ("accent", "onSeries")
        } else {
            (SERIES[m.branch % SERIES.len()], "onSeries")
        };
        let d = match m.shape {
            1 => rect(x, y, m.w, m.h, 3.0),
            2 => {
                let r = m.w.max(m.h) / 2.0;
                circle(m.x, m.y, r)
            }
            _ => rect(x, y, m.w, m.h, m.h / 2.0),
        };
        scene.path(d, Some(fill), None, 0.0, false);
        let (_, th) = block_size(&m.lines, 14.0);
        scene.label(
            x + 6.0,
            m.y - th / 2.0,
            m.w - 12.0,
            th,
            &m.lines.join("\n"),
            14.0,
            if i == 0 { 700 } else { 500 },
            text,
        );
    }
    scene.fit(12.0);
    Ok(scene)
}

// ---------------------------------------------------------------- gitGraph

struct Commit {
    id: String,
    branch: usize,
    parents: Vec<usize>,
    tag: Option<String>,
    highlight: bool,
}

fn attr(s: &str, key: &str) -> Option<String> {
    let p = s.find(&format!("{key}:"))?;
    let rest = s[p + key.len() + 1..].trim_start();
    if let Some(r) = rest.strip_prefix('"') {
        Some(r[..r.find('"')?].to_owned())
    } else {
        Some(rest.split_whitespace().next()?.to_owned())
    }
}

pub fn gitgraph(src: &str) -> Result<Scene, MermaidError> {
    let mut branches: Vec<String> = vec!["main".to_owned()];
    let mut heads: HashMap<usize, usize> = HashMap::new();
    let mut commits: Vec<Commit> = Vec::new();
    let mut current = 0usize;
    let mut horizontal = true;
    let mut first = true;
    for line in src.lines() {
        let s = line.trim();
        if s.is_empty() || s.starts_with("%%") {
            continue;
        }
        if first {
            first = false;
            if s.contains("TB") || s.contains("BT") {
                horizontal = false;
            }
            continue;
        }
        let word = s.split_whitespace().next().unwrap_or("");
        match word {
            "commit" => {
                let parents = heads.get(&current).copied().into_iter().collect();
                let id = attr(s, "id").unwrap_or_else(|| format!("{}", commits.len()));
                commits.push(Commit {
                    id,
                    branch: current,
                    parents,
                    tag: attr(s, "tag"),
                    highlight: s.contains("HIGHLIGHT"),
                });
                heads.insert(current, commits.len() - 1);
            }
            "branch" => {
                let name = s[6..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("branch")
                    .trim_matches('"')
                    .to_owned();
                branches.push(name);
                let b = branches.len() - 1;
                if let Some(&h) = heads.get(&current) {
                    heads.insert(b, h);
                }
                current = b;
            }
            "checkout" | "switch" => {
                let name = s[word.len()..].trim().trim_matches('"');
                if let Some(b) = branches.iter().position(|x| x == name) {
                    current = b;
                }
            }
            "merge" => {
                let name = s[5..].split_whitespace().next().unwrap_or("").trim_matches('"');
                if let Some(b) = branches.iter().position(|x| x == name) {
                    let mut parents: Vec<usize> = heads.get(&current).copied().into_iter().collect();
                    if let Some(&h) = heads.get(&b) {
                        parents.push(h);
                    }
                    let id = attr(s, "id").unwrap_or_else(|| format!("{}", commits.len()));
                    commits.push(Commit {
                        id,
                        branch: current,
                        parents,
                        tag: attr(s, "tag"),
                        highlight: false,
                    });
                    heads.insert(current, commits.len() - 1);
                }
            }
            "cherry-pick" => {
                let parents = heads.get(&current).copied().into_iter().collect();
                let id = attr(s, "id")
                    .map(|i| format!("{i}'"))
                    .unwrap_or_else(|| format!("{}", commits.len()));
                commits.push(Commit {
                    id,
                    branch: current,
                    parents,
                    tag: attr(s, "tag"),
                    highlight: true,
                });
                heads.insert(current, commits.len() - 1);
            }
            _ => {}
        }
    }
    if commits.is_empty() {
        return Err(MermaidError::Empty);
    }
    let lane = 50.0f32;
    let step = 56.0f32;
    let label_w = branches.iter().map(|b| text_width(b, 13.0)).fold(40.0, f32::max) + 24.0;
    let pos = |i: usize, b: usize| -> (f32, f32) {
        if horizontal {
            (label_w + 30.0 + i as f32 * step, 30.0 + b as f32 * lane)
        } else {
            (label_w + b as f32 * lane * 1.6, 40.0 + i as f32 * step)
        }
    };
    let mut scene = Scene::new("gitGraph");
    // Branch lanes.
    for (b, name) in branches.iter().enumerate() {
        let on: Vec<usize> = (0..commits.len()).filter(|&i| commits[i].branch == b).collect();
        let (Some(&f), Some(&l)) = (on.first(), on.last()) else {
            continue;
        };
        let start = commits[f].parents.first().copied().unwrap_or(f);
        let (x1, y1) = pos(start, b);
        let (x2, y2) = pos(l, b);
        let color = SERIES[b % SERIES.len()];
        scene.path(
            format!("M{} {}L{} {}", n(x1), n(y1), n(x2), n(y2)),
            None,
            Some(color),
            3.0,
            false,
        );
        let (lx, ly) = if horizontal {
            (6.0, y1 - 10.0)
        } else {
            (x1 - 40.0, 6.0)
        };
        scene.path(
            rect(lx, ly, label_w - 12.0, 20.0, 10.0),
            Some(color),
            None,
            0.0,
            false,
        );
        scene.label(lx, ly + 1.0, label_w - 12.0, 18.0, name, 12.5, 700, "onSeries");
    }
    // Links to parents on other branches.
    for (i, c) in commits.iter().enumerate() {
        for &p in &c.parents {
            if commits[p].branch != c.branch {
                let (x1, y1) = pos(p, commits[p].branch);
                let (x2, y2) = pos(i, c.branch);
                let color = SERIES[if commits[p].branch == 0 {
                    c.branch
                } else {
                    commits[p].branch
                } % SERIES.len()];
                let d = if horizontal {
                    format!(
                        "M{} {}C{} {} {} {} {} {}",
                        n(x1),
                        n(y1),
                        n(x2 - step * 0.5),
                        n(y1),
                        n(x2 - step * 0.5),
                        n(y2),
                        n(x2),
                        n(y2)
                    )
                } else {
                    format!(
                        "M{} {}C{} {} {} {} {} {}",
                        n(x1),
                        n(y1),
                        n(x1),
                        n(y2 - step * 0.5),
                        n(x2),
                        n(y2 - step * 0.5),
                        n(x2),
                        n(y2)
                    )
                };
                scene.path(d, None, Some(color), 2.5, false);
            }
        }
    }
    for (i, c) in commits.iter().enumerate() {
        let (x, y) = pos(i, c.branch);
        let color = SERIES[c.branch % SERIES.len()];
        let merge = c.parents.len() > 1;
        scene.path(
            circle(x, y, if merge { 8.0 } else { 9.0 }),
            Some(if c.highlight || merge { "bg" } else { color }),
            Some(color),
            3.0,
            false,
        );
        let id_w = text_width(&c.id, 11.0) + 8.0;
        if horizontal {
            scene.label(x - id_w / 2.0, y + 13.0, id_w, 16.0, &c.id, 11.0, 400, "muted");
        } else {
            scene.label_aligned(x + 14.0, y - 8.0, id_w, 16.0, &c.id, 11.0, 400, "muted", "start");
        }
        if let Some(t) = &c.tag {
            let w = text_width(t, 11.5) + 14.0;
            let (tx, ty) = if horizontal {
                (x - w / 2.0, y - 34.0)
            } else {
                (x - w - 16.0, y - 9.0)
            };
            scene.path(
                rect(tx, ty, w, 18.0, 4.0),
                Some("note"),
                Some("noteBorder"),
                1.0,
                false,
            );
            scene.label(tx, ty + 1.0, w, 16.0, t, 11.5, 600, "text");
        }
    }
    scene.fit(12.0);
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mindmap_tree() {
        let s = mindmap("mindmap\n  root((ناشر))\n    القراءة\n      RTL\n      البحث\n    Mermaid\n      [المخططات]\n    الأداء\n").unwrap();
        assert!(s.labels.iter().any(|l| l.text == "ناشر" && l.weight == 700));
        assert_eq!(s.labels.len(), 7);
    }

    #[test]
    fn git_graph() {
        let s = gitgraph("gitGraph\n  commit\n  commit id: \"init\"\n  branch develop\n  checkout develop\n  commit tag: \"v0.1\"\n  checkout main\n  merge develop\n  commit\n").unwrap();
        assert!(s.labels.iter().any(|l| l.text == "develop"));
        assert!(s.labels.iter().any(|l| l.text == "init"));
        assert!(s.labels.iter().any(|l| l.text == "v0.1"));
    }
}
