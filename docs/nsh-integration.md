# NSH integration guide

> **ملخص:** كل واجهة NSH يستخدمها ناشر موثقة رسمياً في nsh.dev للإصدار 0.4.0؛ هذا الدليل يربط كل ميزة بالواجهة المستخدمة ومرجعها.

Nashir targets **Native Shell v0.4.0** and uses only documented APIs ([nsh.dev/docs](https://nsh.dev/docs/)). This page maps each feature to the API that implements it.

## Project shape

| File | Meaning |
|---|---|
| `app/nsh.json` | `name`, `id` (`app.nashir.reader`, also the storage folder), `version`, `build`, `icon`, `"rust": "native"` (the crate), `appearance: "system"`, and explicit `capabilities`. |
| `app/App.tsx` | The TSX entry: `mount(App)`. |
| `app/native` | The crate; `#[native_module("nashir/native")]` registers `Library`, `Diagrams` and `Desktop` (`#[module]`). |
| `app/tsconfig.json` | Extends `.nsh/tsconfig.json`; `nsh` writes `.nsh/types` (including `nashir/native.d.ts`, generated from the Rust doc comments). |

`nsh run App.tsx` runs it with hot reload for TSX (Rust edits rebuild and restart); `nsh check App.tsx` type-checks; `nsh package App.tsx --target windows` packages it.

## Feature → API

| Feature | NSH API | Docs |
|---|---|---|
| Markdown rendering | `TextView` (`nsh/base`): `markdown`, `options.frontmatter`, `extensions`, `renderCustom`, `renderImage`, `styles` by part, `selectable`, `codeBlockActions`, `onLinkPress` | elements/text-view |
| Large documents | `List` (`nsh`): `each`, `estimatedItemSize`, `initialScrollIndex`, `onViewableItemsChange`, ref `scrollToIndex` | elements/list |
| Mermaid shapes | `Canvas` + `Path2D` (SVG path data), `setLineDash` | docs/canvas |
| Mermaid labels | `Text` absolutely positioned, `writingDirection`, `userSelect: "text"` | docs/text |
| Expanded diagrams | `Dialog` (`nsh/base`) with nested `Scroll`s | elements/dialog |
| Title bar & tabs | `Window.setTitleBar({ style: "hidden" })`, `titleBar()`, `desktop={{ windowDrag: true }}` (`nsh/desktop`) | modules/nsh-desktop |
| Menus | `Menu.setApplicationMenu`, `Menu.popup` (tab and recent-file context menus via `onContextMenu`) | modules/nsh-desktop |
| Shortcuts | `keymap.bind` (single source of truth; takes precedence over menu shortcuts of the same keys) | modules/nsh#keymap |
| Window title | `Window.setTitle` | modules/nsh-desktop |
| Open dialog | `pick_paths(OpenDialog::files().multiple(true).filter(FileFilter::new(…)))` in Rust | docs/rust#files-and-folders |
| Save dialog (SVG export) | `pick_save_path(SaveDialog::new()…)` | docs/rust#files-and-folders |
| Reveal / open with system | `reveal_path`, `open_path` | docs/rust#files-and-folders |
| Drag & drop | `onFileDrop`, `onFileDragEnter/Leave` on the root view (`file:` URLs → paths in Rust) | elements/view#desktop-file-drops |
| Persistence | `Storage` (`nsh`), 5 MB, `%APPDATA%\app.nashir.reader\storage.json` | modules/nsh#storage |
| Theme | `colorScheme()`, `setColorScheme()` | modules/nsh#setcolorscheme |
| Language & direction | `locale()`, view `direction` style, `writingDirection` | docs/layout, modules/nsh#layout-direction |
| Clipboard | `Clipboard.writeText` (capability `clipboard`) | modules/nsh#clipboard |
| External links | `openUrl` (capability `openUrl` for `https`, `http`, `mailto`) | modules/nsh#openurl |
| Screen readers | `role`, `accessibilityLabel`, `accessibilityState`, `announce()` | elements/index#accessibility |
| Images | `Image` with `data:` URIs (local files) or URLs (remote, opt-in) | elements/image |

## Capabilities

`nsh.json` declares capabilities explicitly, so everything else is denied:

```json
"capabilities": {
  "network": { "hosts": ["*"] },
  "clipboard": true,
  "openUrl": { "schemes": ["https", "http", "mailto"] }
}
```

`network` is only exercised for remote images, and only when the user allows them (globally in Settings, or per image). See [Security & privacy](security-and-privacy.md).

## Windows specifics we rely on (NSH 0.4)

- The hidden title bar keeps Windows 11-style caption buttons drawn by NSH; `titleBar().right` (138 pt) reserves their room.
- F11 (full screen) and Alt+F4 are handled by NSH in packaged apps and cannot be rebound.
- Selectable text has a native right-click menu (Copy, Select All) — the "context menu on selection" requirement.
- `nsh package --target windows` writes a portable folder and `.zip` and embeds `nsh.json`'s icon; it does not sign or build an installer — Nashir adds Inno Setup and `signtool` ([Build & packaging](build-and-packaging.md)).

Limitations of NSH 0.4 on Windows that affect Nashir are listed in the [roadmap](roadmap.md#known-limitations).

## Verifying against NSH

- `nsh check App.tsx` (CI) type-checks the TSX against declarations generated from NSH's Rust sources and this crate's exports.
- `app/native/tests/app.rs` builds `App.tsx` exactly as `nsh run` does (`nsh_testing::compile_tsx`) and drives it headless on NSH's engine with a Windows desktop environment.
