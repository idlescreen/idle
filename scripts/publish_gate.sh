#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Fail if the *latest* NVR of each package in the pool is missing from the index.
# Older historical pool files may exist without being indexed (APT default).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKG="$ROOT/packages"
FAIL=0

echo "=== Publish gate: latest pool NVR ⊂ index ==="

python3 - "$PKG" <<'PY'
import re, sys, gzip
from pathlib import Path
from collections import defaultdict

pkg_root = Path(sys.argv[1])
fail = 0

def parse_deb(name: str):
    # name_version-rev_arch.deb
    m = re.match(r"^(.+)_([^_]+)_([^.]+)\.deb$", name)
    if not m:
        return None
    return m.group(1), m.group(2), m.group(3)

def parse_rpm(name: str):
    # name-ver-rel.arch.rpm
    m = re.match(r"^(.+)-([^-]+)-([^-]+)\.([^.]+)\.rpm$", name)
    if not m:
        return None
    return m.group(1), m.group(2), m.group(3), m.group(4)

def ver_key(v: str):
    # rough: split on non-alnum for ordering
    parts = re.split(r"[^0-9A-Za-z]+", v)
    key = []
    for p in parts:
        if not p:
            continue
        if p.isdigit():
            key.append((0, int(p)))
        else:
            key.append((1, p))
    return key

# --- APT ---
apt_pool = pkg_root / "apt" / "pool" / "main"
packages_idx = None
for cand in [
    pkg_root / "apt/dists/stable/main/binary-amd64/Packages",
    *pkg_root.glob("apt/dists/**/Packages"),
]:
    if cand.is_file():
        packages_idx = cand
        break
if apt_pool.is_dir():
    if packages_idx is None:
        print("ERROR: missing APT Packages index")
        fail = 1
    else:
        text = packages_idx.read_text()
        # map package -> set of filenames referenced
        indexed = set()
        for line in text.splitlines():
            if line.startswith("Filename:"):
                fn = line.split(":", 1)[1].strip()
                indexed.add(Path(fn).name)
        by_pkg = defaultdict(list)
        for p in apt_pool.glob("*.deb"):
            parsed = parse_deb(p.name)
            if parsed:
                by_pkg[parsed[0]].append((ver_key(parsed[1]), p.name))
        for name, versions in sorted(by_pkg.items()):
            versions.sort()
            latest = versions[-1][1]
            if latest not in indexed and not any(latest in x for x in indexed):
                # also allow Filename path contains name
                if not any(latest == i or i.endswith(latest) for i in indexed):
                    print(f"ERROR: latest DEB not in Packages: {latest}")
                    fail = 1
        print(f"  APT: checked latest of {len(by_pkg)} packages against {packages_idx}")
        # idlescreen Version vs Filename
        blocks = [b for b in text.split("\n\n") if b.strip()]
        for b in blocks:
            fields = {}
            for line in b.splitlines():
                if ": " in line and not line.startswith(" "):
                    k, v = line.split(": ", 1)
                    fields[k] = v
            if fields.get("Package") == "idlescreen":
                ver = fields.get("Version", "")
                fn = Path(fields.get("Filename", "")).name
                if ver and fn and not fn.startswith(f"idlescreen_{ver}"):
                    print(f"ERROR: idlescreen Version {ver} vs Filename {fn}")
                    fail = 1
                else:
                    print("  APT: idlescreen Version/Filename identity OK")

# --- RPM ---
rpm_pool = pkg_root / "rpm" / "pool"
primary = None
repodata = pkg_root / "rpm" / "repodata"
if repodata.is_dir():
    for p in repodata.iterdir():
        if "primary" in p.name and (p.suffix in {".xml", ".gz", ".zst"} or ".xml" in p.name):
            primary = p
            break
if rpm_pool.is_dir():
    if primary is None:
        print("ERROR: missing rpm primary metadata (run createrepo_c)")
        fail = 1
    else:
        name = primary.name
        if name.endswith(".gz"):
            data = gzip.open(primary, "rt", errors="replace").read()
        elif name.endswith(".zst"):
            import subprocess
            data = subprocess.check_output(["zstd", "-cd", str(primary)], text=True)
        else:
            data = primary.read_text(errors="replace")
        by_pkg = defaultdict(list)
        for p in rpm_pool.glob("*.rpm"):
            parsed = parse_rpm(p.name)
            if parsed:
                pname, ver, rel, arch = parsed
                by_pkg[pname].append((ver_key(ver), int(rel) if rel.isdigit() else 0, p.name))
        for pname, versions in sorted(by_pkg.items()):
            versions.sort()
            latest = versions[-1][2]
            if latest not in data:
                print(f"ERROR: latest RPM not in primary: {latest}")
                fail = 1
        print(f"  RPM: checked latest of {len(by_pkg)} packages against primary")

sys.exit(fail)
PY
rc=$?
if [ "$rc" -ne 0 ]; then
    echo "=== Publish gate FAILED ==="
    exit 1
fi
echo "=== Publish gate PASSED ==="
exit 0
