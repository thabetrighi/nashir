#!/usr/bin/env python3
"""Nashir's version, kept in one step across every file that states it.

  python3 scripts/version.py show            # the current version
  python3 scripts/version.py check 0.2.0     # fail unless every file says 0.2.0
  python3 scripts/version.py set 0.2.0       # write 0.2.0 everywhere (and bump the build number)
  python3 scripts/version.py notes 0.2.0     # print the CHANGELOG section of 0.2.0
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# (file, regex with one group around the version)
SOURCES = [
    ("Cargo.toml", r'(?m)^version = "([^"]+)"'),
    ("app/native/Cargo.toml", r'(?m)^version = "([^"]+)"'),
    ("app/ui/SettingsDialog.tsx", r'export const VERSION = "([^"]+)"'),
    ("packaging/windows/nashir.iss", r'#define AppVersion "([^"]+)"'),
]


def read_versions():
    found = {}
    nsh = json.loads((ROOT / "app/nsh.json").read_text(encoding="utf-8"))
    found["app/nsh.json"] = nsh["version"]
    for path, pattern in SOURCES:
        m = re.search(pattern, (ROOT / path).read_text(encoding="utf-8"))
        found[path] = m.group(1) if m else None
    return found


def check(version):
    bad = {p: v for p, v in read_versions().items() if v != version}
    if bad:
        for p, v in bad.items():
            print(f"{p}: {v!r}, expected {version!r}", file=sys.stderr)
        print("Run: python3 scripts/version.py set " + version, file=sys.stderr)
        sys.exit(1)
    print(f"All files are at {version}.")


def set_version(version):
    if not re.fullmatch(r"\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?", version):
        sys.exit(f"not a semantic version: {version}")
    nsh_path = ROOT / "app/nsh.json"
    nsh = json.loads(nsh_path.read_text(encoding="utf-8"))
    if nsh.get("version") != version:
        nsh["build"] = int(nsh.get("build", 0)) + 1
    nsh["version"] = version
    nsh_path.write_text(json.dumps(nsh, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    for path, pattern in SOURCES:
        file = ROOT / path
        text = file.read_text(encoding="utf-8")
        text = re.sub(pattern, lambda m: m.group(0).replace(m.group(1), version), text, count=1)
        file.write_text(text, encoding="utf-8")
    check(version)


def notes(version):
    text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    m = re.search(rf"(?ms)^## \[{re.escape(version)}\][^\n]*\n(.*?)(?=^## \[|\Z)", text)
    if not m:
        sys.exit(f"CHANGELOG.md has no section for {version}")
    print(m.group(1).strip())


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "show"
    if cmd == "show":
        print(json.dumps(read_versions(), indent=2))
    elif cmd == "check":
        check(sys.argv[2])
    elif cmd == "set":
        set_version(sys.argv[2])
    elif cmd == "notes":
        notes(sys.argv[2])
    else:
        sys.exit(__doc__)
