//! `stateDiagram` / `stateDiagram-v2`.

use super::graph::{Cluster, Edge, Graph, Shape};
use super::layout::Dir;
use super::scene::{label_lines, Scene};
use super::{statements_keep_braces, MermaidError};

pub fn render(src: &str) -> Result<Scene, MermaidError> {
    Ok(parse(src)?.render("state"))
}

pub fn parse(src: &str) -> Result<Graph, MermaidError> {
    let mut g = Graph::new(Dir::TopBottom);
    let mut scopes: Vec<(String, Option<usize>)> = vec![(String::new(), None)];
    let mut in_note = false;
    let mut lines = statements_keep_braces(src);
    let _header = lines.next();
    for line in lines {
        let s = line.trim();
        if s.is_empty()
            || s.starts_with("%%")
            || s.starts_with("classDef ")
            || s.starts_with("class ")
            || s == "--"
        {
            continue;
        }
        if in_note {
            if s == "end note" {
                in_note = false;
            }
            continue;
        }
        if s.starts_with("note ") {
            if !s.contains(':') {
                in_note = true;
            }
            continue;
        }
        if let Some(d) = s.strip_prefix("direction ") {
            if scopes.len() == 1 {
                g.dir = Dir::parse(d);
            }
            continue;
        }
        if s == "}" {
            if scopes.len() > 1 {
                scopes.pop();
            }
            continue;
        }
        let scope = scopes.last().unwrap().0.clone();
        let cluster = scopes.last().unwrap().1;
        if let Some(rest) = s.strip_prefix("state ") {
            let rest = rest.trim();
            // state "Description" as Id   |   state Id <<fork>>   |   state Id {
            if let Some(body) = rest.strip_suffix('{') {
                let body = body.trim();
                let (id, title) = alias(body);
                let n = g.node(&id);
                g.set_label(n, &title, Shape::Round);
                add_member(&mut g, cluster, n);
                g.clusters.push(Cluster {
                    title,
                    members: Vec::new(),
                });
                let c = g.clusters.len() - 1;
                scopes.push((id, Some(c)));
                continue;
            }
            if let Some((id, kind)) = rest.split_once("<<") {
                let id = id.trim();
                let n = g.node(id);
                let shape = match kind.trim_end_matches(">>").trim() {
                    "fork" | "join" => Shape::Bar,
                    "choice" => Shape::Diamond,
                    _ => Shape::Round,
                };
                g.set_label(n, if shape == Shape::Diamond { "" } else { id }, shape);
                add_member(&mut g, cluster, n);
                continue;
            }
            let (id, title) = alias(rest);
            let n = g.node(&id);
            g.set_label(n, &title, Shape::Round);
            add_member(&mut g, cluster, n);
            continue;
        }
        if let Some(pos) = s.find("-->") {
            let left = s[..pos].trim();
            let right_all = s[pos + 3..].trim();
            let (right, label) = match right_all.split_once(':') {
                Some((r, l)) => (r.trim(), l.trim()),
                None => (right_all, ""),
            };
            let a = endpoint(&mut g, left, &scope, true, cluster);
            let b = endpoint(&mut g, right, &scope, false, cluster);
            let mut e = Edge::new(a, b);
            if !label.is_empty() {
                e.label = label_lines(label);
            }
            g.edges.push(e);
            continue;
        }
        if let Some((id, desc)) = s.split_once(':') {
            let id = id.trim();
            if !id.is_empty() && !id.contains(' ') {
                let n = endpoint(&mut g, id, &scope, false, cluster);
                let node = &mut g.nodes[n];
                if !node.labelled {
                    node.lines = vec![id.to_owned()];
                    node.labelled = true;
                    node.shape = Shape::Round;
                }
                node.lines.extend(label_lines(desc));
            }
            continue;
        }
        if !s.contains(' ') {
            let n = endpoint(&mut g, s, &scope, false, cluster);
            let _ = n;
        }
    }
    if g.nodes.is_empty() {
        return Err(MermaidError::Empty);
    }
    Ok(g)
}

fn alias(body: &str) -> (String, String) {
    // `"Long title" as Id` or `Id`.
    if let Some((title, id)) = body.split_once(" as ") {
        (id.trim().to_owned(), title.trim().trim_matches('"').to_owned())
    } else {
        (body.trim().to_owned(), body.trim().trim_matches('"').to_owned())
    }
}

fn add_member(g: &mut Graph, cluster: Option<usize>, n: usize) {
    if let Some(c) = cluster {
        if !g.clusters[c].members.contains(&n) {
            g.clusters[c].members.push(n);
        }
    }
}

fn endpoint(g: &mut Graph, token: &str, scope: &str, left: bool, cluster: Option<usize>) -> usize {
    if token == "[*]" {
        let id = format!("[*]{}{}", if left { "start" } else { "end" }, scope);
        let fresh = g.find(&id).is_none();
        let n = g.node(&id);
        if fresh {
            g.set_label(n, "", if left { Shape::Start } else { Shape::End });
        }
        add_member(g, cluster, n);
        return n;
    }
    let fresh = g.find(token).is_none();
    let n = g.node(token);
    if fresh {
        let node = &mut g.nodes[n];
        node.shape = Shape::Round;
    }
    add_member(g, cluster, n);
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_and_transitions() {
        let g = parse("stateDiagram-v2\n  [*] --> Still\n  Still --> Moving : يتحرك\n  Moving --> Still\n  Moving --> Crash\n  Crash --> [*]\n  state \"وصف طويل\" as Long\n  Long : سطر إضافي\n  state Busy {\n    [*] --> Working\n  }\n  state f <<fork>>\n").unwrap();
        assert_eq!(g.nodes[g.find("[*]start").unwrap()].shape, Shape::Start);
        assert_eq!(g.nodes[g.find("[*]end").unwrap()].shape, Shape::End);
        assert_eq!(g.edges[1].label, vec!["يتحرك"]);
        assert_eq!(
            g.nodes[g.find("Long").unwrap()].lines,
            vec!["وصف طويل", "سطر إضافي"]
        );
        assert_eq!(g.clusters.len(), 1);
        assert!(g.find("[*]startBusy").is_some());
        assert_eq!(g.nodes[g.find("f").unwrap()].shape, Shape::Bar);
        assert!(render("stateDiagram\n A --> B").is_ok());
    }
}
