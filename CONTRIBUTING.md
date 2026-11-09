# Contributing to Nashir · المساهمة في ناشر

Thank you for helping! Issues and pull requests are welcome in Arabic, English or French.

## Setup

See [docs/build-and-packaging.md](docs/build-and-packaging.md) for the toolchain. In short: Rust, nsh 0.4, and on Windows Visual Studio Build Tools and LLVM.

## Workflow

1. Open an issue first for larger changes, so we can agree on the approach.
2. Keep the engine (`crates/nashir-engine`) free of NSH dependencies and covered by unit tests.
3. Before pushing:
   ```sh
   cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
   cd app && nsh check App.tsx
   cd native && cargo test
   ```
4. User-visible strings go in `app/i18n/*.json` (all three languages; English is the fallback).
5. Add an entry under `[Unreleased]` in `CHANGELOG.md`.

## Style

- Rust: `rustfmt` (max width 110), no `unsafe` in the engine, doc comments on everything exported to TSX (they become its TypeScript docs).
- TSX: strict TypeScript; colors only from `app/ui/theme.ts`; logical sides (`start`/`end`, `paddingStart`) so layouts mirror in RTL.
- Docs: English for technical pages with a short Arabic summary at the top.

## Reporting bugs

Include your Nashir and Windows versions and, if possible, a small `.md` file that reproduces the problem. Security issues: see [SECURITY.md](SECURITY.md).
