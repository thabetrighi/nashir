//! `flowchart` / `graph` diagrams.

use super::graph::{Cluster, Edge, Graph, Marker, Shape};
use super::layout::Dir;
use super::scene::Scene;
use super::{statements, MermaidError};

pub fn render(src: &str) -> Result<Scene, MermaidError> {
    Ok(parse(src)?.render("flowchart"))
}

pub fn parse(src: &str) -> Result<Graph, MermaidError> {
    let mut lines = statements(src);
    let header = lines.next().unwrap_or_default();
    let dir = header
        .split_whitespace()
        .nth(1)
        .map(Dir::parse)
        .unwrap_or(Dir::TopBottom);
    let mut g = Graph::new(dir);
    let mut cluster_stack: Vec<usize> = Vec::new();

    for stmt in lines {
        let s = stmt.trim();
        if s.is_empty() || is_ignored(s) {
            continue;
        }
        if let Some(rest) = s.strip_prefix("subgraph") {
            let rest = rest.trim();
            let title = match (rest.find('['), rest.rfind(']')) {
                (Some(a), Some(b)) if b > a => rest[a + 1..b].trim().trim_matches('"').to_owned(),
                _ => rest.trim_matches('"').to_owned(),
            };
            g.clusters.push(Cluster {
                title,
                members: Vec::new(),
            });
            cluster_stack.push(g.clusters.len() - 1);
            continue;
        }
        if s == "end" {
            cluster_stack.pop();
            continue;
        }
        if s.starts_with("direction ") {
            continue;
        }
        let touched = parse_chain(&mut g, s)?;
        for &c in &cluster_stack {
            for &n in &touched {
                if !g.clusters[c].members.contains(&n) {
                    g.clusters[c].members.push(n);
                }
            }
        }
    }
    if g.nodes.is_empty() {
        return Err(MermaidError::Empty);
    }
    Ok(g)
}

fn is_ignored(s: &str) -> bool {
    [
        "classDef ",
        "class ",
        "style ",
        "linkStyle ",
        "click ",
        "accTitle",
        "accDescr",
        "%%",
    ]
    .iter()
    .any(|p| s.starts_with(p))
}

struct Cursor<'a> {
    chars: Vec<char>,
    i: usize,
    _src: &'a str,
}

impl<'a> Cursor<'a> {
    fn ws(&mut self) {
        while self.i < self.chars.len() && self.chars[self.i].is_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }
    fn starts(&self, pat: &str) -> bool {
        pat.chars()
            .enumerate()
            .all(|(k, p)| self.chars.get(self.i + k) == Some(&p))
    }
    fn done(&self) -> bool {
        self.i >= self.chars.len()
    }
}

fn is_id_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Parses `A[x] --> B & C -- label --- D` and returns the nodes it touched.
fn parse_chain(g: &mut Graph, s: &str) -> Result<Vec<usize>, MermaidError> {
    let mut c = Cursor {
        chars: s.chars().collect(),
        i: 0,
        _src: s,
    };
    let mut touched = Vec::new();
    let mut prev = parse_group(g, &mut c)?;
    touched.extend(prev.iter().copied());
    loop {
        c.ws();
        if c.done() {
            break;
        }
        let Some(link) = parse_link(&mut c) else {
            // Trailing text we do not understand: ignore the rest.
            break;
        };
        c.ws();
        let next = parse_group(g, &mut c)?;
        if next.is_empty() {
            break;
        }
        for &a in &prev {
            for &b in &next {
                let mut e = Edge::new(a, b);
                e.label = if link.label.is_empty() {
                    Vec::new()
                } else {
                    super::scene::label_lines(&link.label)
                };
                e.dashed = link.dashed;
                e.thick = link.thick;
                e.invisible = link.invisible;
                e.start = link.start.clone();
                e.end = link.end.clone();
                g.edges.push(e);
            }
        }
        touched.extend(next.iter().copied());
        prev = next;
    }
    Ok(touched)
}

fn parse_group(g: &mut Graph, c: &mut Cursor) -> Result<Vec<usize>, MermaidError> {
    let mut out = Vec::new();
    loop {
        c.ws();
        match parse_node(g, c)? {
            Some(n) => out.push(n),
            None => break,
        }
        c.ws();
        if c.peek() == Some('&') {
            c.i += 1;
            continue;
        }
        break;
    }
    Ok(out)
}

fn parse_node(g: &mut Graph, c: &mut Cursor) -> Result<Option<usize>, MermaidError> {
    let start = c.i;
    while let Some(ch) = c.peek() {
        if is_id_char(ch)
            || (ch == '-' || ch == '.')
                && c.chars.get(c.i + 1).is_some_and(|n| n.is_alphanumeric())
                && c.i > start
        {
            c.i += 1;
        } else {
            break;
        }
    }
    if c.i == start {
        return Ok(None);
    }
    let id: String = c.chars[start..c.i].iter().collect();
    let node = g.node(&id);
    // Shape and label.
    const SHAPES: &[(&str, &str, Shape)] = &[
        ("(((", ")))", Shape::DoubleCircle),
        ("((", "))", Shape::Circle),
        ("([", "])", Shape::Stadium),
        ("[[", "]]", Shape::Subroutine),
        ("[(", ")]", Shape::Cylinder),
        ("{{", "}}", Shape::Hexagon),
        ("[/", "/]", Shape::Parallelogram),
        ("[/", "\\]", Shape::Trapezoid),
        ("[\\", "\\]", Shape::ParallelogramAlt),
        ("[\\", "/]", Shape::TrapezoidAlt),
        (">", "]", Shape::Asymmetric),
        ("[", "]", Shape::Rect),
        ("(", ")", Shape::Round),
        ("{", "}", Shape::Diamond),
    ];
    for (open, close, shape) in SHAPES {
        if !c.starts(open) {
            continue;
        }
        let body_start = c.i + open.chars().count();
        let rest: String = c.chars[body_start..].iter().collect();
        let (text, consumed) = if rest.trim_start().starts_with('"') {
            // A quoted label may contain the closing characters.
            let lead = rest.len() - rest.trim_start().len();
            let after_quote = &rest[lead + 1..];
            match after_quote.find('"') {
                Some(q) => {
                    let text = after_quote[..q].to_owned();
                    let tail = &after_quote[q + 1..];
                    let tail_trim = tail.trim_start();
                    if tail_trim.starts_with(close) {
                        let used = lead + 1 + q + 1 + (tail.len() - tail_trim.len()) + close.len();
                        (text, rest[..used].chars().count())
                    } else {
                        continue;
                    }
                }
                None => continue,
            }
        } else {
            match rest.find(close) {
                Some(p) => (rest[..p].to_owned(), rest[..p + close.len()].chars().count()),
                None => continue,
            }
        };
        c.i = body_start + consumed;
        g.set_label(node, &text, *shape);
        break;
    }
    // `:::className` and `@{ … }` (new shape syntax) are skipped.
    if c.starts(":::") {
        c.i += 3;
        while c.peek().is_some_and(|ch| is_id_char(ch) || ch == '-') {
            c.i += 1;
        }
    }
    if c.starts("@{") {
        while let Some(ch) = c.peek() {
            c.i += 1;
            if ch == '}' {
                break;
            }
        }
    }
    Ok(Some(node))
}

struct Link {
    label: String,
    dashed: bool,
    thick: bool,
    invisible: bool,
    start: Marker,
    end: Marker,
}

fn link_run(c: &mut Cursor) -> String {
    let start = c.i;
    while let Some(ch) = c.peek() {
        if matches!(ch, '-' | '=' | '.' | '<' | '>' | '~') {
            c.i += 1;
        } else {
            break;
        }
    }
    // A trailing `x` or `o` end marker, when followed by a space or a node.
    if c.i > start {
        if let Some(ch) = c.peek() {
            if (ch == 'x' || ch == 'o')
                && c.chars
                    .get(c.i + 1)
                    .map_or(true, |n| n.is_whitespace() || *n == '|')
            {
                c.i += 1;
            }
        }
    }
    c.chars[start..c.i].iter().collect()
}

fn valid_run(run: &str) -> bool {
    run.contains("--")
        || run.contains("==")
        || run.contains("-.")
        || run.contains(".-")
        || run.contains("~~~")
}

fn parse_link(c: &mut Cursor) -> Option<Link> {
    let save = c.i;
    // `o--o` / `x--x` style starts are rare; plain starts only.
    let mut run = link_run(c);
    if !valid_run(&run) {
        c.i = save;
        return None;
    }
    let mut label = String::new();
    // Text form: `-- text -->`, `-. text .->`, `== text ==>`.
    if matches!(run.as_str(), "--" | "==" | "-." | "<--" | "<==" | "<-.") {
        let rest: String = c.chars[c.i..].iter().collect();
        let closers = [
            "-->", "---", "==>", "===", ".->", "-.-", "--x", "--o", "==x", "==o", ".-x", ".-o",
        ];
        let found = closers
            .iter()
            .filter_map(|k| rest.find(k).map(|p| (p, *k)))
            .min_by_key(|(p, _)| *p);
        if let Some((p, _)) = found {
            label = rest[..p].trim().to_owned();
            c.i += rest[..p].chars().count();
            let close_run = link_run(c);
            run = format!("{}{}", if run.starts_with('<') { "<" } else { "" }, close_run);
        }
    }
    c.ws();
    if c.peek() == Some('|') {
        c.i += 1;
        let start = c.i;
        while c.peek().is_some_and(|ch| ch != '|') {
            c.i += 1;
        }
        label = c.chars[start..c.i].iter().collect::<String>().trim().to_owned();
        c.i += 1;
    }
    let end = if run.ends_with('>') {
        Marker::Arrow
    } else if run.ends_with('x') {
        Marker::Cross
    } else if run.ends_with('o') {
        Marker::Circle
    } else {
        Marker::None
    };
    let start = if run.starts_with('<') {
        Marker::Arrow
    } else {
        Marker::None
    };
    Some(Link {
        label,
        dashed: run.contains('.'),
        thick: run.contains('='),
        invisible: run.contains("~~~"),
        start,
        end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_and_links() {
        let g = parse("flowchart LR\n  A[ابدأ] --> B{قرار؟}\n  B -->|نعم| C((دائرة))\n  B -- لا --> D([نهاية])\n  C -.-> D\n  A ==> E[(DB)] & F>flag]\n").unwrap();
        assert_eq!(g.dir, Dir::LeftRight);
        let id = |s: &str| g.find(s).unwrap();
        assert_eq!(g.nodes[id("A")].lines, vec!["ابدأ"]);
        assert_eq!(g.nodes[id("B")].shape, Shape::Diamond);
        assert_eq!(g.nodes[id("C")].shape, Shape::Circle);
        assert_eq!(g.nodes[id("D")].shape, Shape::Stadium);
        assert_eq!(g.nodes[id("E")].shape, Shape::Cylinder);
        assert_eq!(g.nodes[id("F")].shape, Shape::Asymmetric);
        assert_eq!(g.edges.len(), 6);
        assert_eq!(g.edges[1].label, vec!["نعم"]);
        assert_eq!(g.edges[2].label, vec!["لا"]);
        assert!(g.edges[3].dashed);
        assert!(g.edges[4].thick);
    }

    #[test]
    fn quoted_labels_and_subgraphs() {
        let g = parse("graph TD\nsubgraph one [المرحلة الأولى]\n  a1[\"a [b] c\"] --- a2\nend\na2 --> b1\n")
            .unwrap();
        assert_eq!(g.nodes[g.find("a1").unwrap()].lines, vec!["a [b] c"]);
        assert_eq!(g.clusters[0].title, "المرحلة الأولى");
        assert_eq!(g.clusters[0].members.len(), 2);
        assert_eq!(g.edges[0].end, Marker::None);
    }

    #[test]
    fn semicolons_and_chains() {
        let g = parse("graph TB; A-->B-->C; C-->A").unwrap();
        assert_eq!(g.edges.len(), 3);
        let s = render("graph TB; A-->B-->C; C-->A").unwrap();
        assert!(s.width > 0.0 && s.height > 0.0);
        assert_eq!(s.labels.len(), 3);
    }
}
