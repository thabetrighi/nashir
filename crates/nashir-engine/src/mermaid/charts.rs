//! `pie`, `gantt` and `journey`.

use super::scene::*;
use super::{statements, MermaidError};

const SERIES: [&str; 8] = ["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7"];

// ---------------------------------------------------------------- pie

pub fn pie(src: &str) -> Result<Scene, MermaidError> {
    let mut lines = statements(src);
    let header = lines.next().unwrap_or_default();
    let show_data = header.contains("showData");
    let mut title = header.split_once("title").map(|(_, t)| t.trim().to_owned());
    let mut slices: Vec<(String, f64)> = Vec::new();
    for line in lines {
        let s = line.trim();
        if s.is_empty() || s.starts_with("%%") {
            continue;
        }
        if let Some(t) = s.strip_prefix("title") {
            title = Some(t.trim().to_owned());
            continue;
        }
        if s == "showData" {
            continue;
        }
        if let Some((label, value)) = s.rsplit_once(':') {
            let label = label.trim().trim_matches('"').to_owned();
            if let Ok(v) = value.trim().parse::<f64>() {
                if v > 0.0 {
                    slices.push((label, v));
                }
            }
        }
    }
    if slices.is_empty() {
        return Err(MermaidError::Empty);
    }
    let total: f64 = slices.iter().map(|s| s.1).sum();
    let mut scene = Scene::new("pie");
    let r = 130.0f32;
    let (cx, cy) = (r + 10.0, r + if title.is_some() { 44.0 } else { 10.0 });
    if let Some(t) = &title {
        scene.label(0.0, 0.0, r * 2.0 + 20.0, 26.0, t, 17.0, 700, "text");
        scene.title = Some(t.clone());
    }
    let mut angle = -std::f64::consts::FRAC_PI_2;
    for (i, (label, v)) in slices.iter().enumerate() {
        let sweep = v / total * std::f64::consts::TAU;
        let color = SERIES[i % SERIES.len()];
        if slices.len() == 1 {
            scene.path(circle(cx, cy, r), Some(color), Some("bg"), 2.0, false);
        } else {
            let (x1, y1) = (cx + r * angle.cos() as f32, cy + r * angle.sin() as f32);
            let end = angle + sweep;
            let (x2, y2) = (cx + r * end.cos() as f32, cy + r * end.sin() as f32);
            let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
            scene.path(
                format!(
                    "M{} {}L{} {}A{} {} 0 {} 1 {} {}Z",
                    n(cx),
                    n(cy),
                    n(x1),
                    n(y1),
                    n(r),
                    n(r),
                    large,
                    n(x2),
                    n(y2)
                ),
                Some(color),
                Some("bg"),
                2.0,
                false,
            );
        }
        let pct = v / total * 100.0;
        if pct >= 4.0 {
            let mid = angle + sweep / 2.0;
            let (lx, ly) = (cx + r * 0.66 * mid.cos() as f32, cy + r * 0.66 * mid.sin() as f32);
            scene.label(
                lx - 30.0,
                ly - 9.0,
                60.0,
                18.0,
                &format!("{:.0}%", pct),
                12.5,
                700,
                "onSeries",
            );
        }
        // Legend.
        let ly = 14.0 + if title.is_some() { 34.0 } else { 0.0 } + i as f32 * 24.0;
        let lx = cx + r + 30.0;
        scene.path(rect(lx, ly, 14.0, 14.0, 3.0), Some(color), None, 0.0, false);
        let text = if show_data {
            format!("{label} [{}]", trim_num(*v))
        } else {
            label.clone()
        };
        scene.label_aligned(
            lx + 22.0,
            ly - 2.0,
            text_width(&text, 13.5) + 12.0,
            18.0,
            &text,
            13.5,
            400,
            "text",
            "start",
        );
        angle += sweep;
    }
    scene.fit(10.0);
    Ok(scene)
}

fn trim_num(v: f64) -> String {
    if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

// ---------------------------------------------------------------- dates

/// Days since 1970-01-01 for a civil date (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn parse_date(s: &str) -> Option<f64> {
    let s = s.trim();
    let date = s.get(..10)?;
    let mut it = date.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d) as f64)
}

fn parse_duration(s: &str) -> Option<f64> {
    let s = s.trim();
    let unit = s.chars().last()?;
    let num: f64 = s[..s.len() - unit.len_utf8()].trim().parse().ok()?;
    Some(match unit {
        'd' => num,
        'w' => num * 7.0,
        'h' => num / 24.0,
        'm' => num / 1440.0,
        _ => return None,
    })
}

// ---------------------------------------------------------------- gantt

struct Task {
    name: String,
    id: Option<String>,
    section: usize,
    start: f64,
    end: f64,
    done: bool,
    active: bool,
    crit: bool,
    milestone: bool,
}

pub fn gantt(src: &str) -> Result<Scene, MermaidError> {
    let mut title = None;
    let mut sections: Vec<String> = Vec::new();
    let mut tasks: Vec<Task> = Vec::new();
    let mut lines = statements(src);
    let _header = lines.next();
    let mut last_end: Option<f64> = None;
    for line in lines {
        let s = line.trim();
        if s.is_empty() || s.starts_with("%%") {
            continue;
        }
        let first = s.split_whitespace().next().unwrap_or("");
        match first {
            "title" => {
                title = Some(s[5..].trim().to_owned());
                continue;
            }
            "dateFormat" | "axisFormat" | "excludes" | "includes" | "todayMarker" | "tickInterval"
            | "weekday" | "displayMode" | "inclusiveEndDates" | "topAxis" => continue,
            "section" => {
                sections.push(s[7..].trim().to_owned());
                continue;
            }
            _ => {}
        }
        let Some((name, spec)) = s.split_once(':') else {
            continue;
        };
        let parts: Vec<&str> = spec
            .split(',')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        let mut flags = (false, false, false, false);
        let mut rest: Vec<&str> = Vec::new();
        for p in parts {
            match p {
                "done" => flags.0 = true,
                "active" => flags.1 = true,
                "crit" => flags.2 = true,
                "milestone" => flags.3 = true,
                _ => rest.push(p),
            }
        }
        // [id,] start, end   |   [id,] duration   |   start, duration
        let (id, start_s, end_s) = match rest.len() {
            0 => (None, None, None),
            1 => (None, None, Some(rest[0])),
            2 => {
                if parse_date(rest[0]).is_some() || rest[0].starts_with("after ") {
                    (None, Some(rest[0]), Some(rest[1]))
                } else {
                    (Some(rest[0]), None, Some(rest[1]))
                }
            }
            _ => (Some(rest[0]), Some(rest[1]), Some(rest[2])),
        };
        let start = match start_s {
            Some(s) if s.starts_with("after ") => {
                let ids: Vec<&str> = s[6..].split_whitespace().collect();
                let ends: Vec<f64> = tasks
                    .iter()
                    .filter(|t| t.id.as_deref().is_some_and(|i| ids.contains(&i)))
                    .map(|t| t.end)
                    .collect();
                ends.into_iter()
                    .fold(None, |a: Option<f64>, b| Some(a.map_or(b, |a| a.max(b))))
            }
            Some(s) => parse_date(s),
            None => last_end,
        };
        let Some(start) = start.or(last_end) else {
            continue;
        };
        let end = match end_s {
            Some(e) => parse_date(e)
                .or_else(|| parse_duration(e).map(|d| start + d))
                .unwrap_or(start + 1.0),
            None => start + 1.0,
        };
        let end = if flags.3 { start } else { end.max(start) };
        last_end = Some(end);
        tasks.push(Task {
            name: name.trim().to_owned(),
            id: id.map(str::to_owned),
            section: sections.len().saturating_sub(1),
            start,
            end,
            done: flags.0,
            active: flags.1,
            crit: flags.2,
            milestone: flags.3,
        });
    }
    if tasks.is_empty() {
        return Err(MermaidError::Empty);
    }
    let min = tasks.iter().map(|t| t.start).fold(f64::MAX, f64::min).floor();
    let max = tasks
        .iter()
        .map(|t| t.end)
        .fold(f64::MIN, f64::max)
        .ceil()
        .max(min + 1.0);
    let span = max - min;
    let ppd = (720.0 / span).clamp(3.0, 48.0) as f32;
    let label_w = tasks
        .iter()
        .map(|t| text_width(&t.name, 13.0))
        .fold(80.0, f32::max)
        + 20.0;
    let section_w = sections.iter().map(|s| text_width(s, 13.0)).fold(0.0, f32::max)
        + if sections.is_empty() { 0.0 } else { 24.0 };
    let x0 = section_w + label_w;
    let row = 28.0;
    let top = if title.is_some() { 64.0 } else { 34.0 };
    let width = x0 + span as f32 * ppd;
    let height = top + tasks.len() as f32 * row;

    let mut scene = Scene::new("gantt");
    if let Some(t) = &title {
        scene.label(0.0, 0.0, width, 26.0, t, 17.0, 700, "text");
        scene.title = Some(t.clone());
    }
    // Section bands.
    let mut row_y = top;
    let mut current = usize::MAX;
    let mut band_start = top;
    for (i, t) in tasks.iter().enumerate() {
        if t.section != current {
            if current != usize::MAX && !sections.is_empty() {
                band(
                    &mut scene, &sections, current, band_start, row_y, width, section_w,
                );
            }
            current = t.section;
            band_start = row_y;
        }
        row_y += row;
        if i == tasks.len() - 1 && !sections.is_empty() {
            band(
                &mut scene, &sections, current, band_start, row_y, width, section_w,
            );
        }
    }
    // Axis.
    let step = [1.0, 2.0, 7.0, 14.0, 30.0, 61.0, 91.0, 182.0, 365.0]
        .into_iter()
        .find(|s| span / s <= 10.0)
        .unwrap_or(365.0);
    let mut d = min;
    while d <= max {
        let x = x0 + (d - min) as f32 * ppd;
        scene.path(
            format!("M{} {}V{}", n(x), n(top - 4.0), n(height)),
            None,
            Some("clusterBorder"),
            0.8,
            true,
        );
        let (y, m, dd) = civil_from_days(d as i64);
        let label = if step >= 182.0 {
            format!("{y}")
        } else if step >= 30.0 {
            format!("{y}-{m:02}")
        } else {
            format!("{m:02}-{dd:02}")
        };
        scene.label(x - 30.0, top - 24.0, 60.0, 16.0, &label, 11.0, 400, "muted");
        d += step;
    }
    // Bars.
    for (i, t) in tasks.iter().enumerate() {
        let y = top + i as f32 * row;
        scene.label_aligned(
            section_w + 6.0,
            y + 4.0,
            label_w - 12.0,
            20.0,
            &t.name,
            13.0,
            400,
            "text",
            "end",
        );
        let x = x0 + (t.start - min) as f32 * ppd;
        let color = if t.crit {
            "danger"
        } else if t.done {
            "muted"
        } else if t.active {
            "accent"
        } else {
            "s0"
        };
        if t.milestone {
            let cy = y + row / 2.0;
            scene.path(
                polygon(&[(x, cy - 9.0), (x + 9.0, cy), (x, cy + 9.0), (x - 9.0, cy)]),
                Some(color),
                None,
                0.0,
                false,
            );
        } else {
            let w = ((t.end - t.start) as f32 * ppd).max(3.0);
            scene.path(
                rect(x, y + 5.0, w, row - 10.0, 4.0),
                Some(color),
                None,
                0.0,
                false,
            );
        }
    }
    scene.fit(10.0);
    Ok(scene)
}

fn band(scene: &mut Scene, sections: &[String], i: usize, y0: f32, y1: f32, width: f32, section_w: f32) {
    if i % 2 == 0 {
        scene.path(
            rect(0.0, y0, width, y1 - y0, 0.0),
            Some("cluster"),
            None,
            0.0,
            false,
        );
    }
    if let Some(name) = sections.get(i) {
        scene.label_aligned(
            4.0,
            y0 + 4.0,
            section_w - 8.0,
            20.0,
            name,
            13.0,
            700,
            "muted",
            "start",
        );
    }
}

// ---------------------------------------------------------------- journey

pub fn journey(src: &str) -> Result<Scene, MermaidError> {
    let mut title = None;
    let mut sections: Vec<(String, usize)> = Vec::new(); // name, first task
    let mut tasks: Vec<(String, u8, Vec<String>)> = Vec::new();
    let mut lines = statements(src);
    let _header = lines.next();
    for line in lines {
        let s = line.trim();
        if s.is_empty() || s.starts_with("%%") {
            continue;
        }
        if let Some(t) = s.strip_prefix("title") {
            title = Some(t.trim().to_owned());
            continue;
        }
        if let Some(t) = s.strip_prefix("section") {
            sections.push((t.trim().to_owned(), tasks.len()));
            continue;
        }
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        if parts.len() >= 2 {
            let score = parts[1].trim().parse::<u8>().unwrap_or(3).clamp(1, 5);
            let actors = parts
                .get(2)
                .map(|a| {
                    a.split(',')
                        .map(|x| x.trim().to_owned())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            tasks.push((parts[0].trim().to_owned(), score, actors));
        }
    }
    if tasks.is_empty() {
        return Err(MermaidError::Empty);
    }
    let col = tasks
        .iter()
        .map(|t| text_width(&t.0, 13.0) + 24.0)
        .fold(120.0, f32::max)
        .min(220.0);
    let mut scene = Scene::new("journey");
    let top = if let Some(t) = &title {
        scene.label(0.0, 0.0, col * tasks.len() as f32, 26.0, t, 17.0, 700, "text");
        scene.title = Some(t.clone());
        40.0
    } else {
        0.0
    };
    // Sections.
    for (i, (name, first)) in sections.iter().enumerate() {
        let last = sections.get(i + 1).map(|s| s.1).unwrap_or(tasks.len());
        if last <= *first {
            continue;
        }
        let x = *first as f32 * col;
        let w = (last - first) as f32 * col - 8.0;
        scene.path(
            rect(x, top, w, 30.0, 6.0),
            Some(SERIES[i % SERIES.len()]),
            None,
            0.0,
            false,
        );
        scene.label(x + 4.0, top + 6.0, w - 8.0, 18.0, name, 13.0, 700, "onSeries");
    }
    let cards_y = top + 40.0;
    let face_top = cards_y + 70.0;
    let face_range = 120.0;
    let mut prev: Option<(f32, f32)> = None;
    for (i, (name, score, actors)) in tasks.iter().enumerate() {
        let x = i as f32 * col;
        let (_, th) = block_size(std::slice::from_ref(name), 13.0);
        scene.path(
            rect(x, cards_y, col - 8.0, 56.0, 6.0),
            Some("node"),
            Some("nodeBorder"),
            1.2,
            false,
        );
        scene.label(
            x + 6.0,
            cards_y + (56.0 - th) / 2.0,
            col - 20.0,
            th,
            name,
            13.0,
            500,
            "text",
        );
        let cy = face_top + (5 - *score) as f32 / 4.0 * face_range + 14.0;
        let cx = x + (col - 8.0) / 2.0;
        scene.path(
            format!("M{} {}V{}", n(cx), n(cards_y + 56.0), n(cy - 14.0)),
            None,
            Some("clusterBorder"),
            1.0,
            true,
        );
        if let Some((px, py)) = prev {
            scene.path(
                format!("M{} {}L{} {}", n(px), n(py), n(cx), n(cy)),
                None,
                Some("edge"),
                1.5,
                false,
            );
        }
        prev = Some((cx, cy));
        let color = match score {
            5 | 4 => "success",
            3 => "accent",
            _ => "danger",
        };
        scene.path(circle(cx, cy, 14.0), Some(color), Some("bg"), 2.0, false);
        scene.label(
            cx - 14.0,
            cy - 9.0,
            28.0,
            18.0,
            &score.to_string(),
            13.0,
            700,
            "onSeries",
        );
        if !actors.is_empty() {
            let a = actors.join("، ");
            scene.label(
                x,
                face_top + face_range + 40.0,
                col - 8.0,
                18.0,
                &a,
                12.0,
                400,
                "muted",
            );
        }
    }
    scene.fit(10.0);
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_round_trip() {
        for d in [-1000, 0, 19000, 20500] {
            let (y, m, dd) = civil_from_days(d);
            assert_eq!(days_from_civil(y, m, dd), d);
        }
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2026, 11, 9), 20766);
    }

    #[test]
    fn pie_chart() {
        let s =
            pie("pie showData title الحيوانات\n  \"قطط\" : 42\n  \"كلاب\" : 50.5\n  \"طيور\" : 7").unwrap();
        assert_eq!(s.title.as_deref(), Some("الحيوانات"));
        assert!(s.labels.iter().any(|l| l.text == "كلاب [50.5]"));
    }

    #[test]
    fn gantt_chart() {
        let s = gantt("gantt\n  title خطة\n  dateFormat YYYY-MM-DD\n  section التصميم\n  الهوية :done, a1, 2026-01-01, 10d\n  الواجهة :active, a2, after a1, 2w\n  section البناء\n  النواة :crit, 2026-01-20, 2026-02-10\n  إصدار :milestone, 2026-02-10, 0d\n").unwrap();
        assert!(s.width > 300.0);
        assert!(s.labels.iter().any(|l| l.text == "التصميم"));
        assert!(s.labels.iter().any(|l| l.text == "الواجهة"));
    }

    #[test]
    fn journey_chart() {
        let s = journey(
            "journey\n  title يوم عمل\n  section الذهاب\n    شرب القهوة: 5: أنا\n    الطريق: 2: أنا, القط\n",
        )
        .unwrap();
        assert!(s.labels.iter().any(|l| l.text == "أنا، القط"));
    }
}
