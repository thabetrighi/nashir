//! Nashir's interface, run headless on NSH's real engine (`cargo test`):
//! `App.tsx` is built as `nsh run` builds it, against NSH's sets and this
//! crate's (`nashir/native`), on a Windows desktop environment, and driven
//! by texts, accessibility labels and test ids.

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use nsh_core::element::Registry;
use nsh_core::geometry::{size, Points};
use nsh_core::platform::{Idiom, Locale, Os};
use nsh_engine::{Bundle, EngineConfig};
use nsh_testing::Headless;

fn ui_sets() -> Vec<Registry> {
    vec![nsh_base::registry(), nashir_native::registry()]
}

fn app() -> Bundle {
    static APP: OnceLock<Bundle> = OnceLock::new();
    APP.get_or_init(|| {
        let entry = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../App.tsx");
        nsh_testing::compile_tsx(entry, &ui_sets()).expect("App.tsx builds")
    })
    .clone()
}

/// The app in a 1280×860 Windows window, with the user's locale `locale`.
fn desktop(locale: &str) -> Headless {
    let mut config = EngineConfig {
        ui_sets: ui_sets(),
        ..EngineConfig::new(app())
    };
    let window = size(Points(1280.0), Points(860.0));
    config.viewport = window;
    config.environment.window = window;
    config.environment.os = Os::Windows;
    config.environment.idiom = Idiom::Desktop;
    config.environment.file_requests = true;
    config.environment.rtl = locale.starts_with("ar");
    config.environment.locale = Locale::parse(locale).unwrap();
    Headless::start(config).expect("the engine starts")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nashir-ui-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn file_url(path: &std::path::Path) -> String {
    let p = path.to_string_lossy().replace('\\', "/");
    let p = p.replace(' ', "%20");
    if p.starts_with('/') {
        format!("file://{p}")
    } else {
        format!("file:///{p}")
    }
}

#[test]
fn arabic_first_run_shows_the_tour_in_arabic() {
    let mut app = desktop("ar-DZ");
    app.wait_for("مرحباً بك في ناشر").unwrap();
    // The interface is Arabic: the sidebar's tabs, the outline.
    app.wait_for("المحتويات").unwrap();
    assert!(app.shows("الإشارات"));
    assert!(app.shows("النص العربي والاتجاه"), "{:?}", app.screen().texts());
    // Pressing an outline entry scrolls the reader to its section, where
    // the Mermaid flowchart's Arabic labels are native texts.
    app.press("label:عنوان المستوى 2: مخططات Mermaid").unwrap();
    app.wait_for("تحليل في Rust").unwrap();
    assert!(app.shows("فتح الملف"));
}

#[test]
fn search_counts_and_highlights_matches() {
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    let expected = {
        let doc = nashir_engine::Document::parse(include_str!("../welcome/en.md").to_owned());
        nashir_engine::search_document(&doc, "rust", 2000).total
    };
    assert!(expected > 1);
    app.press_key("mod+f").unwrap();
    app.type_text("id:search-input", "RUST").unwrap();
    app.wait_for(&format!("1 of {expected}")).unwrap();
    app.press("label:Next result").unwrap();
    app.wait_for(&format!("2 of {expected}")).unwrap();
    app.press_key("escape").unwrap();
    app.wait_for_gone(&format!("2 of {expected}"), Duration::from_secs(5))
        .unwrap();
}

#[test]
fn dropped_files_open_and_others_are_reported() {
    let dir = temp_dir("drop");
    let doc = dir.join("دليل.md");
    std::fs::write(
        &doc,
        "# دليل الاستخدام\n\nمرحباً بالقارئ.\n\n## التثبيت\n\nخطوات التثبيت.\n",
    )
    .unwrap();
    let pdf = dir.join("report.pdf");
    std::fs::write(&pdf, b"%PDF-1.4\0\0").unwrap();

    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.drop_files("id:reader", vec![file_url(&doc), file_url(&pdf)])
        .unwrap();
    app.wait_for("دليل الاستخدام").unwrap();
    app.wait_for("Nashir skipped 1 file(s) that aren't Markdown.")
        .unwrap();
    // The new tab is labelled with its file name.
    assert!(app.screen().matches("label:Document دليل.md").unwrap());
}

#[test]
fn a_missing_file_explains_itself() {
    let dir = temp_dir("missing");
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.drop_files("id:reader", vec![file_url(&dir.join("gone.md"))])
        .unwrap();
    app.wait_for("Couldn't open the document").unwrap();
    app.wait_for("The file wasn't found. It may have been moved or deleted.")
        .unwrap();
    assert!(app.shows("Try again"));
}

#[test]
fn the_open_dialog_opens_files() {
    let dir = temp_dir("open");
    let doc = dir.join("notes.md");
    std::fs::write(&doc, "# Release notes\n\n- First item\n").unwrap();
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.on_pick_paths(vec![doc]).unwrap();
    app.press_key("mod+o").unwrap();
    app.wait_for("Release notes").unwrap();
    app.wait_for("First item").unwrap();
}

#[test]
fn unsupported_diagrams_show_a_clear_message() {
    let dir = temp_dir("mermaid");
    let doc = dir.join("diagram.md");
    std::fs::write(&doc, "# Diagrams\n\n```mermaid\nquadrantChart\n  title Reach\n```\n\n```mermaid\npie title Pets\n  \"Dogs\" : 3\n  \"Cats\" : 1\n```\n").unwrap();
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.drop_files("id:reader", vec![file_url(&doc)]).unwrap();
    app.wait_for("The “quadrantChart” diagram type isn't supported yet.")
        .unwrap();
    // The pie chart next to it is drawn: its title and legend are texts.
    app.wait_for("Pets").unwrap();
    assert!(app.shows("Dogs") && app.shows("Cats"));
    assert!(app.shows("75%"));
}

#[test]
fn settings_switch_the_interface_language() {
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.press("label:Settings (Ctrl+,)").unwrap();
    app.wait_for("Interface language").unwrap();
    app.press("label:Français").unwrap();
    app.wait_for("Langue de l'interface").unwrap();
    app.press("label:العربية").unwrap();
    app.wait_for("لغة الواجهة").unwrap();
}

#[test]
fn a_ten_megabyte_document_opens_quickly() {
    let dir = temp_dir("large");
    let doc = dir.join("large.md");
    let block = "## قسم\n\nفقرة عربية مع English text و`code` و**تنسيق**.\n\n| أ | ب |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn main() {}\n```\n\n";
    let mut text = String::from("# مستند كبير\n\n");
    while text.len() < 10 * 1024 * 1024 {
        text.push_str(block);
    }
    std::fs::write(&doc, &text).unwrap();
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    let started = Instant::now();
    app.drop_files("id:reader", vec![file_url(&doc)]).unwrap();
    app.wait_for_within("مستند كبير", Duration::from_secs(30))
        .unwrap();
    let elapsed = started.elapsed();
    eprintln!("10 MB document shown in {elapsed:?}");
    assert!(elapsed < Duration::from_secs(20), "took {elapsed:?}");
}

#[test]
fn tabs_close_reopen_and_change_detection() {
    let dir = temp_dir("tabs");
    let doc = dir.join("live.md");
    std::fs::write(&doc, "# Version one\n\nText.\n").unwrap();
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.drop_files("id:reader", vec![file_url(&doc)]).unwrap();
    app.wait_for("Version one").unwrap();

    // The file changes on disk: a banner offers to reload it.
    std::thread::sleep(Duration::from_millis(1100));
    std::fs::write(&doc, "# Version two\n\nText.\n").unwrap();
    app.wait_for_within("This file changed outside Nashir.", Duration::from_secs(8))
        .unwrap();
    app.press("label:Reload").unwrap();
    app.wait_for("Version two").unwrap();

    // Ctrl+W closes it; Ctrl+Shift+T brings it back.
    app.press_key("mod+w").unwrap();
    app.wait_for_gone("Version two", Duration::from_secs(5)).unwrap();
    app.press_key("mod+shift+t").unwrap();
    app.wait_for("Version two").unwrap();
}

#[test]
fn headings_can_be_bookmarked() {
    let dir = temp_dir("bookmarks");
    let doc = dir.join("guide.md");
    std::fs::write(
        &doc,
        "# Guide\n\nIntro.\n\n## Install\n\nSteps.\n\n## Usage\n\nMore.\n",
    )
    .unwrap();
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.drop_files("id:reader", vec![file_url(&doc)]).unwrap();
    app.wait_for("Steps.").unwrap();
    app.press("label:Add bookmark >> nth=1").unwrap();
    app.press("label:Bookmarks").unwrap();
    app.wait_for("label:Remove bookmark").unwrap();
    assert!(app.shows("Install"));
}

#[test]
fn diagrams_zoom_and_expand() {
    let dir = temp_dir("expand");
    let doc = dir.join("flow.md");
    std::fs::write(
        &doc,
        "# Flow\n\n```mermaid\nflowchart LR\n  A[Start] --> B[Finish]\n```\n",
    )
    .unwrap();
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.drop_files("id:reader", vec![file_url(&doc)]).unwrap();
    app.wait_for("Start").unwrap();
    app.press("label:Zoom in").unwrap();
    app.wait_for("125%").unwrap();
    app.press("label:Expand").unwrap();
    // The expanded view brings a second toolbar with its own zoom.
    app.wait_for("label:Zoom in >> nth=1").unwrap();
    assert_eq!(app.screen().count("label:Zoom in").unwrap(), 2);
    app.press_key("escape").unwrap();
    app.wait_for_gone("label:Zoom in >> nth=1", Duration::from_secs(5))
        .unwrap();
}

#[test]
fn themes_and_comfort_mode() {
    let mut app = desktop("en-US");
    app.wait_for("Welcome to Nashir").unwrap();
    app.press("label:Settings (Ctrl+,)").unwrap();
    app.press("label:Paper").unwrap();
    app.press("label:Dark").unwrap();
    app.press("label:Comfort reading mode").unwrap();
    app.press_key("escape").unwrap();
    // Comfort mode hides the sidebar and the status bar.
    app.wait_for_gone("Contents", Duration::from_secs(5)).unwrap();
    assert!(!app.shows("222 words"));
    app.press_key("mod+shift+m").unwrap();
    app.wait_for("Contents").unwrap();
}
