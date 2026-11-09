# Build & packaging

> **ملخص:** يتطلب البناء Rust وأدوات Visual Studio وLLVM وأداة nsh 0.4؛ ينتج `nsh package` نسخة محمولة، ويضيف Inno Setup مثبّتاً مع ربط الملفات، ويمكن توقيع الكل بـ signtool.

## Toolchain (Windows)

| Tool | Why | Install |
|---|---|---|
| Rust (stable, ≥ 1.80) | The engine and native module | [rustup.rs](https://rustup.rs) |
| Visual Studio Build Tools, "Desktop development with C++" + Windows SDK | Building the app's runner | Visual Studio Installer |
| LLVM (libclang) | QuickJS bindings in NSH's runtime | `winget install LLVM.LLVM` (or set `LIBCLANG_PATH`) |
| nsh **v0.4.0** | Run, check, package | `$env:NSH_VERSION="v0.4.0"; irm https://nsh.dev/install.ps1 \| iex` |
| Inno Setup 6 (installer only) | `.exe` installer | `winget install JRSoftware.InnoSetup` |
| Windows SDK `signtool` (signing only) | Authenticode | Comes with the SDK |

Windows 10 1809+ is required to run the app.

## Develop

```powershell
cd app
nsh run App.tsx          # a Win32 window; TSX edits hot-reload, Rust edits rebuild and restart
nsh check App.tsx        # type-check the TSX against NSH's and nashir/native's declarations
```

The engine is plain Rust:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p nashir-engine --example bench
cargo run -p nashir-engine --example mermaid_svg -- samples/flow-ar.mmd out.svg
```

Headless UI tests of `App.tsx` (any OS with clang; no window needed):

```sh
cd app/native && cargo test
```

## Package

```powershell
cd app
nsh package App.tsx --target windows -o ..\dist\nsh
```

Writes `dist\nsh\Nashir\` — `Nashir.exe` (the app's own runner, built with `app/native`, with the icon from `assets/icon.png` embedded) and `nsh\` (bundle, assets, `app.json`) — and the same as a `.zip`. Copy the folder anywhere and run `Nashir.exe`: this is the **portable** version.

### Installer

```powershell
iscc /DAppVersion=0.1.0 /DSourceDir=..\..\dist\nsh\Nashir /DOutputDir=..\..\dist packaging\windows\nashir.iss
```

`packaging/windows/nashir.iss` produces `Nashir-<version>-windows-x64-setup.exe`:

- per-user install by default (no admin), all-users on request;
- Start menu shortcut, optional desktop shortcut (with the app's AppUserModelID);
- **file association**: Nashir is always listed in *Open with* for `.md`, `.markdown`, `.mdown`, `.mkd`, and becomes their default when the user ticks the task; Explorer passes the file path, which `Desktop.launchPaths()` opens;
- uninstaller (in Settings → Apps), which removes the associations it added and keeps user settings;
- Arabic, English and French setup languages.

### Signing

```powershell
signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 /f cert.pfx /p <password> dist\nsh\Nashir\Nashir.exe
iscc ... /DSignTool=nashirsign "/Snashirsign=signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 /f cert.pfx /p <password> $f" packaging\windows\nashir.iss
```

In CI this happens automatically when the `WINDOWS_CERT_PFX_BASE64` and `WINDOWS_CERT_PASSWORD` secrets exist ([Release guide](release-guide.md)).

## Other platforms

| Target | Command | Status |
|---|---|---|
| Linux (GTK 4.14+) | `nsh package App.tsx --target linux` → folder + `.tar.gz` | Builds in CI; experimental |
| macOS 14+ | `nsh package App.tsx --target macos` → ad hoc signed `.app` | Builds in CI; experimental (needs Developer ID + notarization to distribute) |

The interface adapts automatically: the custom title bar reserves the platform's window controls (`titleBar()`), menus are the system's, and the `mod` key is ⌘ on macOS.

## Icons

`brand/app-icon.svg` is the master; `brand/png/icon-1024.png` is copied to `app/assets/icon.png` (used by `nsh package` for every platform) and `brand/nashir.ico` to `packaging/windows/nashir.ico` (installer). Regenerate with `rsvg-convert` and ImageMagick as described in [brand/BRAND.md](../brand/BRAND.md).
