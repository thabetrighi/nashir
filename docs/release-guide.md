# Release guide

> **ملخص:** حدّث الإصدار بأمر واحد، واكتب سجل التغييرات، ثم ادفع وسماً `vX.Y.Z`؛ يبني GitHub Actions المثبّت والنسخة المحمولة ويوقّعهما وينشرهما.

## Versioning

Nashir follows [Semantic Versioning](https://semver.org/). The version appears in `app/nsh.json` (with a build number that must grow), both `Cargo.toml`s, `app/ui/SettingsDialog.tsx` and the installer script. One command keeps them together:

```sh
python3 scripts/version.py set 0.2.0     # writes every file, bumps nsh.json "build"
python3 scripts/version.py check 0.2.0   # what CI runs on a tag
```

Pre-releases (`0.2.0-beta.1`) are published as GitHub pre-releases.

## Steps

1. **Changelog** — move the `[Unreleased]` entries of `CHANGELOG.md` under `## [0.2.0] - YYYY-MM-DD` and update the links at the bottom. The release notes are taken from this section.
2. **Version** — `python3 scripts/version.py set 0.2.0`.
3. **QA** — run the [manual checklist](testing-and-qa.md#manual-qa-checklist-per-release) on Windows 10 and 11 with a build from `workflow_dispatch` (Actions → Release → *Run workflow*, which builds without publishing).
4. **Commit and push to `main`**:
   ```sh
   git commit -am "Release 0.2.0"
   git push origin main
   ```
   A push that changes `app/nsh.json` releases its version once (the workflow creates the `v0.2.0` tag itself). Pushing a tag `v0.2.0` also works.
5. **CI** (`.github/workflows/release.yml`) checks the version, then builds:
   - Windows x64: `Nashir-0.2.0-windows-x64-setup.exe` and `Nashir-0.2.0-windows-x64-portable.zip` (signed when secrets exist);
   - Linux x64 `.tar.gz` and macOS arm64 `.zip` (experimental; a failure never blocks the Windows release);
   - `SHA256SUMS-*.txt`;
   and publishes a GitHub release with the changelog section as notes.
6. **Website** — `site/` deploys to GitHub Pages on pushes to `main` (`pages.yml`); download buttons point to `releases/latest`, so nothing to change.

## Code signing setup (once)

1. Obtain an Authenticode code-signing certificate (OV or EV) as a `.pfx`.
2. In the repository: Settings → Secrets and variables → Actions:
   - `WINDOWS_CERT_PFX_BASE64` — `[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`
   - `WINDOWS_CERT_PASSWORD` — the `.pfx` password.
3. The next tag signs `Nashir.exe`, the installer and its uninstaller, with an RFC 3161 timestamp.

Without these secrets, builds are unsigned (SmartScreen will warn on first run).

## GitHub Pages setup (once)

Settings → Pages → Build and deployment → Source: **GitHub Actions**.

## Updating NSH

1. Read NSH's release notes for breaking changes.
2. Update `NSH_VERSION` in both workflows, `nsh-core`/`nsh-base`/test crates in `app/native/Cargo.toml`, and regenerate `app/native/Cargo.lock` from NSH's lock (see [Build & packaging](build-and-packaging.md)).
3. Run `nsh check App.tsx`, `cargo test` in `app/native`, and the manual checklist.
