#!/usr/bin/env python3
"""Build the one Python package that runs on every laptop.

    tools/build_wheel.py --version 0.2.0 --kit <files-only kit> \\
        --native windows-x86_64=<_vleo.dll> --native macos-arm64=<lib_vleo.dylib> \\
        --native linux-x86_64=<lib_vleo.so> --out dist/
    tools/build_wheel.py --selftest

The kit's programs are what Windows and its antivirus question: new, unsigned
`.exe` files. This package has none. It carries the same native engine, once
per system, and the pages and the tree; `pip install` puts it in place and
`python -m vleo` runs it inside python.exe. One file, `vleo-<version>-py3-none-
any.whl`, for Windows, macOS and Linux alike — the package loads the engine
built for the machine it is on (crates/vleo-py/python/vleo/__init__.py).

Why not maturin: maturin builds one wheel per system. The point here is one
file for everyone, and a wheel is a zip with three small text files beside the
package — writing them is simpler than bending a tool into doing it.

Each native file is renamed to what Python loads: `_vleo.pyd` on Windows,
`_vleo.abi3.so` elsewhere. `--native` names the system the way the package
does: `windows-x86_64`, `macos-arm64`, `macos-x86_64`, `linux-x86_64`.
"""

import argparse
import base64
import hashlib
import os
import sys
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACKAGE = ROOT / "crates" / "vleo-py" / "python" / "vleo"
SYSTEMS = ("windows-x86_64", "macos-arm64", "macos-x86_64", "linux-x86_64")

METADATA = """Metadata-Version: 2.1
Name: vleo
Version: {version}
Summary: The VLEO design tool: the engine and the pages, for every laptop. Run it with `python -m vleo`.
Author: Orbitt Space
Requires-Python: >=3.9
License: UNLICENSED

Start it with `python -m vleo`; it opens in the browser. `python -m vleo --check`
proves the install works. The engine is also a library: `import vleo`.
"""

WHEEL = """Wheel-Version: 1.0
Generator: tools/build_wheel.py
Root-Is-Purelib: true
Tag: py3-none-any
"""


def record_hash(data):
    digest = hashlib.sha256(data).digest()
    return "sha256=" + base64.urlsafe_b64encode(digest).rstrip(b"=").decode("ascii")


def native_name(system):
    return "_vleo.pyd" if system.startswith("windows") else "_vleo.abi3.so"


def contents(version, kit, natives):
    """Every file of the wheel as (path in the wheel, bytes), in a fixed order."""
    out = []
    for f in sorted(PACKAGE.glob("*.py")):
        out.append(("vleo/" + f.name, f.read_bytes()))
    out.append(("vleo/_build.py", ('VERSION = "%s"\n' % version).encode()))
    for system, path in sorted(natives.items()):
        out.append(("vleo/_native/%s/%s" % (system, native_name(system)), Path(path).read_bytes()))
    kit = Path(kit)
    for f in sorted(p for p in kit.rglob("*") if p.is_file()):
        rel = f.relative_to(kit).as_posix()
        out.append(("vleo/_kit/" + rel, f.read_bytes()))
    info = "vleo-%s.dist-info" % version
    out.append((info + "/METADATA", METADATA.format(version=version).encode()))
    out.append((info + "/WHEEL", WHEEL.encode()))
    return out, info


def build(version, kit, natives, out_dir):
    if not (Path(kit) / "web").is_dir() or not (Path(kit) / "layers").is_dir():
        raise SystemExit("%s is not a kit: build it with `cargo run -p xtask -- kit --files-only --out <dir>`" % kit)
    for system in natives:
        if system not in SYSTEMS:
            raise SystemExit("unknown system %r; one of %s" % (system, ", ".join(SYSTEMS)))
    if not natives:
        raise SystemExit("no --native engine given: the package would run nowhere")
    files, info = contents(version, kit, natives)
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    wheel = out_dir / ("vleo-%s-py3-none-any.whl" % version)
    record = []
    with zipfile.ZipFile(wheel, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in files:
            z.writestr(zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0)), data,
                       compress_type=zipfile.ZIP_DEFLATED)
            record.append("%s,%s,%d" % (name, record_hash(data), len(data)))
        record.append(info + "/RECORD,,")
        z.writestr(zipfile.ZipInfo(info + "/RECORD", date_time=(2026, 1, 1, 0, 0, 0)),
                   "\n".join(record) + "\n", compress_type=zipfile.ZIP_DEFLATED)
    return wheel


def verify(wheel):
    """Every file the RECORD lists is in the wheel with the hash it states."""
    bad = []
    with zipfile.ZipFile(wheel) as z:
        names = set(z.namelist())
        rec = [n for n in names if n.endswith(".dist-info/RECORD")]
        if len(rec) != 1:
            return ["expected one RECORD, found %d" % len(rec)]
        listed = set()
        for line in z.read(rec[0]).decode().splitlines():
            name, digest, _size = line.rsplit(",", 2)
            listed.add(name)
            if name == rec[0]:
                continue
            if name not in names:
                bad.append("%s is in the RECORD and not in the wheel" % name)
            elif record_hash(z.read(name)) != digest:
                bad.append("%s does not match its RECORD hash" % name)
        for n in names - listed:
            bad.append("%s is in the wheel and not in the RECORD" % n)
    return bad


def selftest():
    failures = []
    with tempfile.TemporaryDirectory() as t:
        t = Path(t)
        kit = t / "kit"
        (kit / "web").mkdir(parents=True)
        (kit / "layers").mkdir()
        (kit / "web" / "index.html").write_text("<title>VLEO</title>")
        lib = t / "lib_vleo.so"
        lib.write_bytes(b"\x7fELF not really")
        wheel = build("9.9.9", kit, {"linux-x86_64": lib, "windows-x86_64": lib}, t / "out")
        with zipfile.ZipFile(wheel) as z:
            names = set(z.namelist())
        for need in ("vleo/__init__.py", "vleo/__main__.py", "vleo/_build.py",
                     "vleo/_native/linux-x86_64/_vleo.abi3.so",
                     "vleo/_native/windows-x86_64/_vleo.pyd",
                     "vleo/_kit/web/index.html",
                     "vleo-9.9.9.dist-info/METADATA", "vleo-9.9.9.dist-info/WHEEL",
                     "vleo-9.9.9.dist-info/RECORD"):
            if need not in names:
                failures.append("the wheel is missing " + need)
        if wheel.name != "vleo-9.9.9-py3-none-any.whl":
            failures.append("the wheel is named " + wheel.name)
        failures += verify(wheel)
        # A check nobody has watched fail is a check nobody knows works: change
        # one byte and the RECORD must catch it.
        broken = t / "broken.whl"
        with zipfile.ZipFile(wheel) as src, zipfile.ZipFile(broken, "w") as dst:
            for n in src.namelist():
                data = src.read(n)
                if n == "vleo/_kit/web/index.html":
                    data = data.replace(b"VLEO", b"VLE0")
                dst.writestr(n, data)
        if not verify(broken):
            failures.append("a changed file passed the RECORD check")
        try:
            build("9.9.9", t / "nokit", {"linux-x86_64": lib}, t / "out")
            failures.append("a directory that is not a kit was accepted")
        except SystemExit:
            pass
        try:
            build("9.9.9", kit, {}, t / "out")
            failures.append("a package with no engine was accepted")
        except SystemExit:
            pass
    for f in failures:
        print("selftest: " + f)
    print("selftest: %s" % ("FAILED" if failures else "ok"))
    return 1 if failures else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--version")
    ap.add_argument("--kit", help="a files-only kit: cargo run -p xtask -- kit --files-only --out <dir>")
    ap.add_argument("--native", action="append", default=[], metavar="SYSTEM=PATH")
    ap.add_argument("--out", default=str(ROOT / "dist"))
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    if not a.version or not a.kit:
        ap.error("--version and --kit are required")
    natives = {}
    for n in a.native:
        system, _, path = n.partition("=")
        if not path or not os.path.isfile(path):
            ap.error("--native %s: no such file" % n)
        natives[system] = path
    wheel = build(a.version, a.kit, natives, a.out)
    bad = verify(wheel)
    if bad:
        for b in bad:
            print(b)
        return 1
    print("%s — %.1f MB, engines for %s" % (wheel, wheel.stat().st_size / 1e6, ", ".join(sorted(natives))))
    return 0


if __name__ == "__main__":
    sys.exit(main())
