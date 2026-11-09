# Testing & QA

> **ملخص:** ثلاث طبقات آلية: 55 اختبار وحدة للنواة، و12 اختبار واجهة تشغّل App.tsx الحقيقي على محرك NSH دون نافذة، وفحص أنواع صارم؛ إضافة إلى قياس أداء لملفات 100KB حتى 50MB وقائمة فحص يدوية.

## Automated

| Layer | Command | What it covers |
|---|---|---|
| Engine unit tests (55) | `cargo test --workspace` | Decoding and limits, sections and outline, anchors, footnotes, direction, Arabic normalisation, search and highlighting, all 10 Mermaid types, layout, SVG export |
| Lint | `cargo fmt --check`, `cargo clippy -- -D warnings` | Style and common mistakes |
| Type check | `nsh check App.tsx` | Strict TypeScript against declarations generated from NSH's Rust sources and `nashir/native` |
| Headless UI tests (12) | `cd app/native && cargo test --test app` | The real `App.tsx`, built as `nsh run` builds it, on NSH's engine with a Windows desktop environment |
| Native unit tests | `cd app/native && cargo test --lib` | base64, `file:` URL → path |
| Benchmark | `cargo run --release -p nashir-engine --example bench` | 100 KB, 1 MB, 10 MB, 50 MB |

CI runs all of them on every push and pull request (`.github/workflows/ci.yml`); the benchmark table is posted to the job summary.

### Headless UI tests

`app/native/tests/app.rs` drives the app by visible text, accessibility labels and test ids:

| Test | Verifies |
|---|---|
| `arabic_first_run_shows_the_tour_in_arabic` | Arabic locale → Arabic interface and tour; outline navigation; Mermaid labels in Arabic rendered as native text |
| `search_counts_and_highlights_matches` | Ctrl+F, case-insensitive search, "1 of N" / "2 of N" navigation, Esc closes |
| `dropped_files_open_and_others_are_reported` | Drag & drop of an Arabic-named `.md` opens a tab; a PDF is reported |
| `a_missing_file_explains_itself` | Clear error screen with *Try again* |
| `the_open_dialog_opens_files` | Ctrl+O → system dialog → document |
| `unsupported_diagrams_show_a_clear_message` | Unsupported Mermaid type message; pie chart drawn with legend and percentages |
| `settings_switch_the_interface_language` | English → French → Arabic at runtime |
| `tabs_close_reopen_and_change_detection` | External change banner + reload; Ctrl+W; Ctrl+Shift+T |
| `headings_can_be_bookmarked` | Outline star → Bookmarks tab |
| `a_ten_megabyte_document_opens_quickly` | 10 MB document visible in < 20 s (≈ 3 s in a debug build) |
| `diagrams_zoom_and_expand` | Diagram zoom (125 %), expanded dialog with its own toolbar, Esc closes it |
| `themes_and_comfort_mode` | Paper and dark themes; comfort mode hides sidebar and status bar; Ctrl+Shift+M restores |

## Performance

Engine timings (release build, Linux x86-64 CI-class machine), generated Arabic/English documents with tables, code and diagrams:

| Size | Decode | Sectioning | Sections | First section | Search «احمد» |
|---|---:|---:|---:|---:|---:|
| 100 KB | 0.2 ms | 3.4 ms | 180 | < 0.01 ms | 2.1 ms |
| 1 MB | 1.9 ms | 33 ms | 1,831 | < 0.01 ms | 13 ms |
| 10 MB | 21 ms | 266 ms | 18,301 | < 0.01 ms | 12 ms* |
| 50 MB | 88 ms | 1.5 s | 91,500 | < 0.01 ms | 27 ms* |

\* stops at the 1,000-result limit used by the benchmark (the app uses 2,000).

All of this runs on a worker thread: the window keeps drawing while a document loads. Rendering cost is independent of document size because only the visible sections exist (NSH `List`). End-to-end, the 10 MB headless test shows the first section ≈ 3 s after the drop in a **debug** build.

### Measuring on Windows

`nsh run App.tsx --fps` logs frame times, main-thread phases, CPU and memory every second (`nsh::perf`); measure **release** builds (`nsh package`). For memory, compare *Private bytes* in Task Manager / Process Explorer before and after opening the 100 KB – 50 MB files (`cargo run --release -p nashir-engine --example bench` can be adapted to write them to disk).

## Manual QA checklist (per release)

**Files** — open via dialog, drag & drop (several files, mixed types), double-click in Explorer (after installing with the association task), "Open with"; recent list; missing file; >64 MB file (limit message); binary file; UTF-16 and Windows-1256 files; file edited externally (banner → reload).

**Reading** — every Markdown feature in `app/native/welcome/*.md`; local and remote images (placeholder → load); links: anchor, external, relative `.md`; select text across paragraphs and copy; code block copy button.

**Arabic & RTL** — Arabic document, English document, mixed paragraphs, tables with mixed content, paths and numbers, punctuation at line ends; switch interface language with a document open; force RTL/LTR.

**Navigation & search** — outline highlight while scrolling; filter (> 12 headings); bookmarks; Alt+↑/↓; search Arabic with/without tashkeel and hamza variants; F3/Shift+F3; Esc.

**Mermaid** — each of the 10 types in light, paper and dark; zoom; expand + pan; copy source; export SVG and open it in a browser; an unsupported type.

**Settings & session** — every setting applies immediately and persists after restart; restore session with pinned tabs and reading positions.

**Windows** — title bar drag, double-click maximize, snap layouts on the maximize button; F11; Narrator reads headings, links, buttons and tab names; high-contrast and 200 % scaling; installer/uninstaller (shortcuts, association added and removed); portable zip runs from a USB drive.

## Accessibility notes

Every icon button has an `accessibilityLabel` (and its shortcut in the label where relevant); tabs use `role="tab"` with `selected`; outline rows report the heading level; toasts are `announce()`d; `TextView` exposes headings, links, checkboxes and images with their alt text. Known gaps are listed in the [roadmap](roadmap.md#known-limitations).
