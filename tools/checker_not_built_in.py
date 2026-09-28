#!/usr/bin/env python3
"""No shipped program carries the node form's checker built in.

> **Answer first.** Reads every file under the folders it is given and fails if
> any holds the method checker's bytes, other than the checker file itself.
> Exits 0 when none does, 1 when one does, naming it.
>
> **Kind:** reference · **For:** the pipeline

The checker is compressed WebAssembly, and the node form carries it with the
script that unpacks and runs it. Inside a program, that pair is the shape
browsers and antivirus block: Chrome blocked the first 0.3.0 Windows kit for
it. Forms now read the checker from `web/method.wasm.gz` beside the tree when
they are made, and this check makes sure no build puts it back inside.

    python3 tools/checker_not_built_in.py <web/method.wasm.gz> <folder>...
    python3 tools/checker_not_built_in.py --selftest
"""
import os
import sys
import tempfile


def carriers(checker, folders):
    """Every file under `folders` that holds the checker, other than a copy of it."""
    whole = open(checker, "rb").read()
    mark = whole[:64]
    found = []
    for folder in folders:
        for top, _, names in os.walk(folder):
            for n in names:
                p = os.path.join(top, n)
                b = open(p, "rb").read()
                if b != whole and mark in b:
                    found.append(p)
    return found


def selftest():
    with tempfile.TemporaryDirectory() as d:
        checker = os.path.join(d, "method.wasm.gz")
        blob = bytes(range(256)) * 2
        open(checker, "wb").write(blob)
        kit = os.path.join(d, "kit")
        os.makedirs(os.path.join(kit, "web"))
        open(os.path.join(kit, "web", "method.wasm.gz"), "wb").write(blob)
        open(os.path.join(kit, "clean.exe"), "wb").write(b"MZ" + b"\0" * 300)
        assert carriers(checker, [kit]) == [], "a kit with only its own copy was refused"
        open(os.path.join(kit, "dirty.exe"), "wb").write(b"MZ" + blob + b"\0")
        got = carriers(checker, [kit])
        assert [os.path.basename(p) for p in got] == ["dirty.exe"], got
    print("checker_not_built_in: selftest passed")
    return 0


def main(argv):
    if argv[1:] == ["--selftest"]:
        return selftest()
    if len(argv) < 3:
        print(__doc__)
        return 2
    found = carriers(argv[1], argv[2:])
    for p in found:
        print(f"::error::{p} carries the node form's checker built in. Forms read it from "
              f"{argv[1]} when they are made; a program holding it is what Chrome blocks.")
    if not found:
        print(f"no program under {', '.join(argv[2:])} carries the checker built in")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
