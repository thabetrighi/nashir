# Changelog

All notable changes to Nashir are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-11-09

The first release of Nashir (ناشر), a native Markdown reader for Windows built on Native Shell 0.4.

### Added

- **Reading:** CommonMark + GitHub Markdown (tables, task lists, alerts, strikethrough, autolinks, footnotes), math, highlighted code with a copy action, local images (inlined by the native module), remote images on request, YAML front matter shown as a metadata card.
- **Arabic & RTL:** per-section direction detection, `writingDirection: auto` per paragraph, code blocks kept left-to-right, Arabic interface by default with English and French; interface language independent from document direction.
- **Mermaid:** native layout and drawing of flowchart, sequence, class, state, ER, Gantt, pie, mindmap, gitGraph and journey diagrams; Arabic labels; zoom, expanded view with panning, copy source, SVG export (light/dark); clear message and source for unsupported types.
- **Navigation:** outline sidebar with the current section highlighted and filtering, bookmarks per document, previous/next heading, links to anchors and to other Markdown files.
- **Search:** Arabic-aware (diacritics, tatweel, hamza forms, alef maqsura, taa marbuta, Eastern digits) and case-insensitive, on a background thread, with result count, snippets, highlighted matches and F3 / Shift+F3 navigation.
- **Tabs & session:** multiple documents in tabs, pinning, close others, reopen closed, restore last session and reading positions, recent documents.
- **Files:** open dialog, drag and drop, file association arguments, `.md` `.markdown` `.mdown` `.mkd` (and more), file validation (size limit, binary detection), UTF-8/UTF-16/Windows-1256 decoding, change detection with a reload banner, clear error screens with retry.
- **Reading settings:** themes (system, light, paper, dark), text size, line spacing, content width, reading and code fonts, comfort reading mode, full screen.
- **Windows:** custom title bar with native caption buttons, application menu, Inno Setup installer with Start menu/desktop shortcuts and file associations, portable zip, optional code signing in CI.
- **Brand:** the ن (noon) mark, app icons 16–1024, Windows `.ico`, wordmark, social preview, brand guide.
- **Quality:** 55 engine unit tests, 12 headless UI tests of the real `App.tsx` on NSH's engine, strict TypeScript checking against NSH's generated declarations, a 100 KB–50 MB benchmark, CI on every push.
- **Website:** bilingual landing page deployed to GitHub Pages.

[Unreleased]: https://github.com/nashir-app/nashir/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nashir-app/nashir/releases/tag/v0.1.0
