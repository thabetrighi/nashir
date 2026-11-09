# Rust native modules

> **ملخص:** الوحدة `nashir/native` تصدّر ثلاث وحدات Rust إلى TSX: `Library` و`Diagrams` و`Desktop`، فوق مكتبة `nashir-engine` المستقلة عن NSH.

## Two crates

| Crate | Depends on NSH | Purpose |
|---|---|---|
| `crates/nashir-engine` | No | The core, testable with plain `cargo test`: loader, sections, outline, direction, search, Mermaid. |
| `app/native` (`nashir-native`) | `nsh-core`, `nsh-base` 0.4 | Exposes the engine and desktop services to TSX. NSH builds it into the app's runner. |

`app/native` is excluded from the root Cargo workspace because NSH builds it with its own runners; it has its own `Cargo.lock` pinned to NSH 0.4.0's dependency versions.

## The `nashir/native` API

Generated TypeScript declarations (`app/.nsh/types/nashir/native.d.ts`) come from the Rust doc comments. Structured results are JSON strings, parsed and typed in `app/ui/native.ts`.

### `Library` — documents

| Function | Kind | Returns |
|---|---|---|
| `open(path, maxMegabytes)` | async (worker thread) | `DocumentMeta` JSON, or rejects with an error code |
| `openWelcome(lang)` | async | `DocumentMeta` JSON of the built-in tour (`app/native/welcome/*.md`) |
| `section(id, index)` | sync | The section's Markdown plus the footnote/link definitions it uses |
| `highlighted(id, index, query)` | sync | Same, matches wrapped in `<mark>` |
| `search(id, query, limit)` | async (worker) | `SearchResult` JSON |
| `anchor(id, anchor)` | sync | Section index of a heading anchor, or -1 |
| `headingAt(id, index)` | sync | Outline index current at a section, or -1 |
| `changed(id)` | sync | Whether the file's modification time moved (or it vanished) |
| `exists(path)` | sync | Whether a path is a file |
| `close(id)` | sync | Frees the document |
| `image(id, src)` | async (worker) | A local image as a `data:` URI (≤ 20 MB), or `""` |

Error codes: `not_found`, `not_a_file`, `permission_denied`, `too_large:<bytes>:<max>`, `binary`, `io:<message>`.

`DocumentMeta`: `{ id, path, name, folder, bytes, encoding, lossy, modified, direction, sectionDirections, outline[{level,text,anchor,section}], stats{words,characters,lines,headings,code_blocks,diagrams,reading_minutes}, frontMatter }`.

### `Diagrams` — Mermaid

| Function | Kind | Returns |
|---|---|---|
| `layout(source)` | sync | `{"ok":true,"scene":…}` or `{"ok":false,"error":"unsupported","detail":"<keyword>"}` |
| `supported()` | sync | The keywords the engine draws |
| `exportSvg(source, dark, name)` | async | Save dialog, then writes SVG; `true` saved, `false` cancelled |

### `Desktop` — the system

| Function | Kind | Returns |
|---|---|---|
| `pickMarkdown(title, filterLabel)` | async | Chosen paths (multi-select), `[]` when cancelled |
| `launchPaths()` | sync | Existing files passed on the command line (file association) |
| `markdownPaths(uris)` | sync | `file:` URLs from a drop → Markdown paths only |
| `reveal(path)` / `openWithSystem(path)` | async | Shows in Explorer / opens with the default app |

## Patterns used

**Background work.** `util::background(f)` runs `f` on a new thread and awaits a `futures-channel` oneshot, inside an `async fn` exported to TSX. NSH runs `async` module functions as tasks and returns Promises, so the engine thread keeps rendering while a 50 MB file is parsed.

**Shared documents.** Open documents live in a `OnceLock<Mutex<HashMap<u32, Open>>>`; `Arc<Document>` lets search clone a handle to a worker thread without copying text.

**Stable error codes.** Errors are strings with a code prefix; the interface translates them, so Rust never formats user-facing text.

## The engine crate

| Module | What it does | Tests |
|---|---|---|
| `loader` | Size limit, binary detection, UTF-8 (± BOM), UTF-16 LE/BE, Windows-1256 fallback, newline normalisation | 5 |
| `document` | Sections (headings, soft/hard limits, fence splitting), outline + GitHub anchors, footnotes/link definitions per section, stats | 9 |
| `direction` | Strong-character counting outside code/URLs/tags, 30% RTL threshold, first-strong for labels | 5 |
| `text` | Arabic normalisation with byte-offset mapping, word count | 4 |
| `search` | Matches with snippets and headings, `<mark>` highlighting that protects code, links, tags | 5 |
| `slug` | Heading text, GitHub slugs, duplicate suffixes | 3 |
| `mermaid::*` | 10 diagram types, layered layout, scene, SVG export | 23 |

Run `cargo test -p nashir-engine` and `cargo run --release -p nashir-engine --example bench`.

## Adding a native function

1. Implement the logic in `nashir-engine` with unit tests.
2. Export it from `app/native/src/<module>.rs` inside the `#[module] impl`, with a doc comment (it becomes the TypeScript doc). Use `async fn` + `background` for anything that may take more than a few milliseconds.
3. Wrap it in `app/ui/native.ts` with a typed signature.
4. Run `nsh check App.tsx` (or regenerate `.nsh/types`) and add a headless test in `app/native/tests/app.rs` if it changes the interface.
