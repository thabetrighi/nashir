//! `classDiagram`.

use super::graph::{Edge, Graph, Marker, Shape};
use super::layout::Dir;
use super::scene::{label_lines, Scene};
use super::{statements_keep_braces, MermaidError};

pub fn render(src: &str) -> Result<Scene, MermaidError> {
    Ok(parse(src)?.render("class"))
}

/// Relation operators, longest first: (token, start marker, end marker, dashed).
const RELATIONS: &[(&str, Marker, Marker, bool)] = &[
    ("<|--", Marker::Triangle, Marker::None, false),
    ("--|>", Marker::None, Marker::Triangle, false),
    ("<|..", Marker::Triangle, Marker::None, true),
    ("..|>", Marker::None, Marker::Triangle, true),
    ("*--", Marker::DiamondFilled, Marker::None, false),
    ("--*", Marker::None, Marker::DiamondFilled, false),
    ("o--", Marker::DiamondHollow, Marker::None, false),
    ("--o", Marker::None, Marker::DiamondHollow, false),
    ("<--", Marker::Arrow, Marker::None, false),
    ("-->", Marker::None, Marker::Arrow, false),
    ("<..", Marker::Arrow, Marker::None, true),
    ("..>", Marker::None, Marker::Arrow, true),
    ("--", Marker::None, Marker::None, false),
    ("..", Marker::None, Marker::None, true),
];

struct Members {
    attributes: Vec<String>,
    methods: Vec<String>,
}

pub fn parse(src: &str) -> Result<Graph, MermaidError> {
    let mut g = Graph::new(Dir::TopBottom);
    let mut members: Vec<Members> = Vec::new();
    let mut open: Option<usize> = None;
    let mut lines = statements_keep_braces(src);
    let _header = lines.next();

    let ensure = |g: &mut Graph, members: &mut Vec<Members>, raw: &str| -> usize {
        let (id, generic) = match raw.split_once('~') {
            Some((a, b)) => (a.trim(), Some(b.trim_end_matches('~'))),
            None => (raw.trim(), None),
        };
        let fresh = g.find(id).is_none();
        let n = g.node(id);
        if fresh {
            let title = match generic {
                Some(t) => format!("{id}<{t}>"),
                None => id.to_owned(),
            };
            g.nodes[n].lines = vec![title];
            g.nodes[n].shape = Shape::Compartments;
        }
        while members.len() < g.nodes.len() {
            members.push(Members {
                attributes: Vec::new(),
                methods: Vec::new(),
            });
        }
        n
    };

    for line in lines {
        let s = line.trim();
        if s.is_empty()
            || s.starts_with("%%")
            || s.starts_with("note")
            || s.starts_with("style ")
            || s.starts_with("classDef ")
            || s.starts_with("cssClass")
            || s.starts_with("click ")
            || s.starts_with("link ")
            || s.starts_with("callback ")
        {
            continue;
        }
        if let Some(d) = s.strip_prefix("direction ") {
            g.dir = Dir::parse(d);
            continue;
        }
        if let Some(c) = open {
            if s == "}" {
                open = None;
            } else if let Some(a) = s.strip_prefix("<<") {
                g.nodes[c].annotation = Some(format!("«{}»", a.trim_end_matches(">>").trim()));
            } else {
                push_member(&mut members[c], s);
            }
            continue;
        }
        if let Some(rest) = s.strip_prefix("<<") {
            // <<interface>> Name
            if let Some((ann, name)) = rest.split_once(">>") {
                let n = ensure(&mut g, &mut members, name.trim());
                g.nodes[n].annotation = Some(format!("«{}»", ann.trim()));
            }
            continue;
        }
        if let Some(rest) = s.strip_prefix("class ") {
            let rest = rest.trim();
            let (head, opens) = match rest.strip_suffix('{') {
                Some(h) => (h.trim(), true),
                None => (rest, false),
            };
            // class Name["Label"]
            let (name, label) = match (head.find('['), head.rfind(']')) {
                (Some(a), Some(b)) if b > a => {
                    (head[..a].trim(), Some(head[a + 1..b].trim().trim_matches('"')))
                }
                _ => (head, None),
            };
            let name = name.split_whitespace().next().unwrap_or(name);
            let n = ensure(&mut g, &mut members, name);
            if let Some(l) = label {
                g.nodes[n].lines = label_lines(l);
            }
            if opens {
                open = Some(n);
            }
            continue;
        }
        if let Some((a, op, b, label)) = relation(s) {
            let (a_name, a_card) = split_card(a, true);
            let (b_name, b_card) = split_card(b, false);
            let x = ensure(&mut g, &mut members, a_name);
            let y = ensure(&mut g, &mut members, b_name);
            let mut e = Edge::new(x, y);
            e.start = op.1.clone();
            e.end = op.2.clone();
            e.dashed = op.3;
            e.start_text = a_card;
            e.end_text = b_card;
            if let Some(l) = label {
                e.label = label_lines(l);
            }
            g.edges.push(e);
            continue;
        }
        if let Some((name, member)) = s.split_once(':') {
            let name = name.trim();
            if !name.is_empty() && !name.contains(' ') {
                let n = ensure(&mut g, &mut members, name);
                push_member(&mut members[n], member.trim());
            }
            continue;
        }
        if !s.contains(' ') {
            ensure(&mut g, &mut members, s);
        }
    }
    if g.nodes.is_empty() {
        return Err(MermaidError::Empty);
    }
    for (node, m) in g.nodes.iter_mut().zip(members) {
        node.compartments = vec![m.attributes, m.methods];
    }
    Ok(g)
}

fn push_member(m: &mut Members, s: &str) {
    let s = s.trim();
    if s.is_empty() {
        return;
    }
    let clean = s.replace(['~', '$', '*'], "");
    if s.contains('(') {
        m.methods.push(clean);
    } else {
        m.attributes.push(clean);
    }
}

fn relation(
    s: &str,
) -> Option<(
    &str,
    &'static (&'static str, Marker, Marker, bool),
    &str,
    Option<&str>,
)> {
    // Find the operator outside quotes.
    let mut in_quote = false;
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'"' {
            in_quote = !in_quote;
            continue;
        }
        if in_quote {
            continue;
        }
        for op in RELATIONS {
            if s[i..].starts_with(op.0) {
                let left = s[..i].trim();
                let right_all = s[i + op.0.len()..].trim();
                let (right, label) = match right_all.split_once(':') {
                    Some((r, l)) => (r.trim(), Some(l.trim())),
                    None => (right_all, None),
                };
                if left.is_empty() || right.is_empty() {
                    return None;
                }
                return Some((left, op, right, label));
            }
        }
    }
    None
}

/// `Name "1"` / `"*" Name` → (name, cardinality).
fn split_card(s: &str, left: bool) -> (&str, Option<String>) {
    let s = s.trim();
    if let (Some(a), Some(b)) = (s.find('"'), s.rfind('"')) {
        if b > a {
            let card = s[a + 1..b].to_owned();
            let name = if left { s[..a].trim() } else { s[b + 1..].trim() };
            return (name, Some(card));
        }
    }
    (s, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_members_relations() {
        let g = parse("classDiagram\n  Animal <|-- Duck : يرث\n  Animal : +int age\n  Animal: +isMammal() bool\n  class Duck{\n    +String beakColor\n    +swim()\n  }\n  <<interface>> Swimmer\n  Customer \"1\" --> \"*\" Order\n").unwrap();
        let animal = g.find("Animal").unwrap();
        assert_eq!(g.nodes[animal].compartments[0], vec!["+int age"]);
        assert_eq!(g.nodes[animal].compartments[1], vec!["+isMammal() bool"]);
        let duck = g.find("Duck").unwrap();
        assert_eq!(g.nodes[duck].compartments[1], vec!["+swim()"]);
        assert_eq!(g.edges[0].start, Marker::Triangle);
        assert_eq!(g.edges[0].label, vec!["يرث"]);
        assert_eq!(
            g.nodes[g.find("Swimmer").unwrap()].annotation.as_deref(),
            Some("«interface»")
        );
        assert_eq!(g.edges[1].start_text.as_deref(), Some("1"));
        assert_eq!(g.edges[1].end_text.as_deref(), Some("*"));
        assert!(render("classDiagram\nA --> B").is_ok());
    }
}
