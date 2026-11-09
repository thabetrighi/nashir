# Security & privacy

> **ملخص:** ناشر محلي بالكامل: لا حساب ولا تتبّع ولا رفع للمستندات. تُفحص الملفات قبل قراءتها، ولا تُنفّذ أي شيفرة من المستند، والصلاحيات معلنة صراحة في nsh.json.

## Privacy commitments

- **No account, no telemetry, no analytics.** Nashir contains no tracking code and makes no network request on its own.
- **No uploads.** Documents are read from disk and stay in memory in the native module.
- **Works offline.** Reading, search, diagrams and export need no connection.
- **Remote images are opt-in.** A document referencing `https://…/image.png` would reveal to that server that you opened it; Nashir shows a placeholder naming the host until you click *Load image* or enable *Load images from the internet* in Settings.
- **Local storage only.** See [Settings & storage](settings-and-storage.md) for exactly what is saved.

## Capabilities

NSH denies everything an app does not declare. `app/nsh.json`:

| Capability | Why |
|---|---|
| `network.hosts: ["*"]` | Only for remote images, after the user allows them. |
| `clipboard: true` | Copy buttons (code blocks, diagram sources, file paths). Copying selected text needs no capability. |
| `openUrl: https, http, mailto` | Opening links in the browser / mail app. `file:`, `javascript:`, `data:` and others are refused by NSH. |

No web views are used for content, and none are declared.

## Untrusted input: the document

A Markdown file is untrusted. Nashir's defenses:

| Risk | Mitigation |
|---|---|
| Huge files exhausting memory | Size checked before reading (default 64 MB, configurable up to 256 MB) and again after reading. |
| Binary files mistaken for text | NUL bytes or a high share of control bytes in the first 8 KB → refused (`binary`). |
| Invalid encodings | UTF-8 (± BOM), UTF-16 with BOM, Windows-1256 fallback, otherwise lossy decoding with a visible notice. |
| Scripts in HTML | `TextView` never runs scripts; `<script>`, `<style>`, forms and hidden elements are dropped. There is no JavaScript engine exposed to document content. |
| Malicious links | Only `https`, `http`, `mailto` open externally; relative `.md` links open in Nashir. |
| Local file disclosure via images | Only files with image extensions are read, up to 20 MB, as `data:` URIs inside the app; nothing is sent anywhere. |
| Pathological diagrams | Mermaid sources over 64 KB are refused; layout is linear/quadratic in nodes, and labels are native text (no HTML/`foreignObject`). |
| Parser crashes | The engine is safe Rust (no `unsafe`), fuzz-friendly pure functions; tests cover malformed input (unterminated fences, empty documents, invalid UTF-8). |

## Supply chain

- NSH is pinned to v0.4.0 (CLI in CI, crates in `app/native/Cargo.lock`).
- The engine depends only on `serde` and `serde_json`.
- Release artifacts are built in GitHub Actions from a tag, with SHA-256 checksums published alongside.
- **Code signing:** the release workflow signs `Nashir.exe` and the installer with `signtool` when the repository has `WINDOWS_CERT_PFX_BASE64` and `WINDOWS_CERT_PASSWORD` secrets ([Release guide](release-guide.md)). Unsigned builds trigger SmartScreen warnings.

## Reporting vulnerabilities

See [SECURITY.md](../SECURITY.md).
