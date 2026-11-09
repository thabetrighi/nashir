<div align="center">

<img src="brand/logo-mark.svg" width="88" alt="">

# ناشر · Nashir

**قارئ Markdown أصلي وسريع، بالعربية أولاً**
*A native, fast, Arabic-first Markdown reader*

[![CI](https://github.com/nashir-app/nashir/actions/workflows/ci.yml/badge.svg)](https://github.com/nashir-app/nashir/actions/workflows/ci.yml)
[![Release](https://github.com/nashir-app/nashir/actions/workflows/release.yml/badge.svg)](https://github.com/nashir-app/nashir/releases)
![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011-0F766E)
![Native Shell 0.4](https://img.shields.io/badge/Native%20Shell-0.4-0F766E)
![License: MIT](https://img.shields.io/badge/license-MIT-0F766E)

[العربية](#العربية) · [English](#english)

</div>

---

<div dir="rtl">

## العربية

**ناشر** تطبيق سطح مكتب لقراءة ملفات Markdown، مبني على [Native Shell (NSH)](https://nsh.dev) بنواة Rust وواجهة TSX. ليس محرراً ولا متصفحاً داخل نافذة: كل ما تراه مكوّنات أصلية للنظام (Win32 وDirect2D وDirectWrite على Windows).

### المزايا

- **عرض Markdown احترافي:** العناوين، القوائم المتداخلة، الجداول، الاقتباسات وتنبيهات GitHub، الأكواد مع التلوين، Task Lists، الحواشي، الصيغ الرياضية، الصور المحلية، وبيانات Front Matter.
- **عربية وRTL حقيقيان:** اكتشاف الاتجاه تلقائياً لكل قسم، فقرات مختلطة عربية/إنجليزية، أكواد ومسارات وأرقام سليمة، وواجهة عربية افتراضياً مع الإنجليزية والفرنسية.
- **مخططات Mermaid محلية:** عشرة أنواع (Flowchart وSequence وClass وState وER وGantt وPie وMindmap وGit Graph وJourney) يرسمها محرك Rust دون متصفح، مع نص عربي، وتكبير وتحريك، ونسخ المصدر، وتصدير SVG.
- **تنقل وبحث:** محتويات تلقائية مع إبراز القسم الحالي، بحث يتجاهل التشكيل والهمزات مع عدّ النتائج ومقتطفات وتمييز، إشارات للعناوين، تبويبات قابلة للتثبيت.
- **أداء:** قائمة افتراضية تعرض الأقسام الظاهرة فقط؛ التحليل والبحث في خيوط Rust الخلفية؛ مستند 10 ميغابايت يُعرض في ثوانٍ.
- **تكامل مع Windows:** فتح بالسحب والإفلات، وبالنقر المزدوج (ربط الملفات)، ومثبّت ونسخة محمولة.
- **خصوصية:** محلي بالكامل؛ لا حساب ولا تتبّع ولا رفع للملفات؛ الصور البعيدة لا تُحمَّل إلا بإذنك.

### التثبيت

نزّل أحدث إصدار من [صفحة الإصدارات](https://github.com/nashir-app/nashir/releases/latest):
- `Nashir-<version>-windows-x64-setup.exe`: مثبّت مع اختصارات وربط ملفات `.md`.
- `Nashir-<version>-windows-x64-portable.zip`: نسخة محمولة؛ فك الضغط وشغّل `Nashir.exe`.

يتطلب Windows 10 (الإصدار 1809) أو أحدث.

### البناء من المصدر

```powershell
# المتطلبات: Rust، وVisual Studio Build Tools (C++)، وLLVM، وnsh 0.4
irm https://nsh.dev/install.ps1 | iex
cd app
nsh run App.tsx                            # تشغيل للتطوير
nsh package App.tsx --target windows       # حزمة قابلة للتوزيع
```

التفاصيل في [دليل البناء والتغليف](docs/build-and-packaging.md).

</div>

---

## English

**Nashir** (ناشر, *publisher*) is a desktop reader for Markdown documents, built on [Native Shell (NSH)](https://nsh.dev) with a Rust core and a TSX interface. It is neither an editor nor a web page in a window: everything you see is native UI (Win32, Direct2D and DirectWrite on Windows).

### Highlights

| | |
|---|---|
| **Markdown, done properly** | Headings, nested lists, tables, quotes and GitHub alerts, highlighted code, task lists, footnotes, math, local images, front matter — via NSH's native `TextView`. |
| **Real Arabic & RTL** | Per-section direction detection, mixed Arabic/English paragraphs, code and paths kept LTR, Arabic UI by default (English and French included). |
| **Native Mermaid** | 10 diagram types laid out by a Rust engine and drawn with native Canvas + Text: Arabic labels, zoom/pan, copy source, SVG export. |
| **Navigate & search** | Outline with the current section highlighted, Arabic-aware search (diacritics, hamza, taa marbuta) with counts, snippets and highlights, bookmarks, pinned tabs. |
| **Fast** | Virtualized rendering by section; parsing and search on Rust worker threads. 10 MB opens in ~3 s (debug build), 50 MB sections in ~1.5 s. |
| **Windows integration** | Drag and drop, double-click (file association), installer and portable builds, custom title bar with Windows 11 caption buttons. |
| **Private** | Local-first: no account, telemetry or uploads; remote images only on request. |

### Repository layout

```
app/                  The NSH app (TSX interface + Rust native module)
  App.tsx             Entry: window, layout, menu, shortcuts, start-up
  ui/                 Components, state, theme, i18n glue
  i18n/               ar.json · en.json · fr.json
  native/             Rust crate exported to TSX as `nashir/native`
    tests/app.rs      Headless UI tests of App.tsx on NSH's engine
crates/nashir-engine  Platform-independent core: loading, sections, outline,
                      bidi, Arabic search, Mermaid layout + SVG export
brand/                Logo, icons, palette, brand guide
packaging/windows/    Inno Setup installer (file associations, shortcuts)
site/                 Landing page (GitHub Pages)
docs/                 Documentation
```

### Documentation

| | |
|---|---|
| [Architecture](docs/architecture.md) | How the pieces fit, data flow, threading |
| [NSH integration guide](docs/nsh-integration.md) | Which NSH APIs Nashir uses, and why |
| [Rust native modules](docs/rust-native-modules.md) | The `nashir/native` API and the engine crate |
| [Markdown rendering](docs/markdown-rendering.md) | Sections, TextView, images, links, footnotes |
| [Mermaid rendering](docs/mermaid-rendering.md) | The native diagram engine, supported syntax |
| [Localization & RTL](docs/localization-and-rtl.md) | Languages, direction, adding a language |
| [Settings & storage](docs/settings-and-storage.md) | What is stored, where, and how |
| [Security & privacy](docs/security-and-privacy.md) | Threat model, capabilities, file validation |
| [Keyboard shortcuts](docs/keyboard-shortcuts.md) | Every shortcut |
| [Build & packaging](docs/build-and-packaging.md) | Toolchain, running, packaging, installer, signing |
| [Testing & QA](docs/testing-and-qa.md) | Unit, headless UI, performance, manual QA |
| [Release guide](docs/release-guide.md) | Versioning and publishing |
| [Roadmap & known limitations](docs/roadmap.md) | What's next, and what isn't there yet |
| [Changelog](CHANGELOG.md) | Release history |
| [Brand guide](brand/BRAND.md) | Mark, colors, typography, voice |

### Quick start (development)

```sh
cargo test --workspace                          # engine: 55 unit tests
cargo run --release -p nashir-engine --example bench
cd app && nsh run App.tsx                       # run with hot reload (Windows)
cd app/native && cargo test                     # headless UI tests of App.tsx
```

### License

[MIT](LICENSE). Built with [Native Shell](https://nsh.dev) (Apache-2.0) by Longbridge.
