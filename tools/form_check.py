#!/usr/bin/env python3
"""The node form's own method check, in a real browser.

> **Answer first.** Opens a node form in Chromium and reads what its check
> says: the method, run on the author's cases by the checker the form carries.
> Exits 0 when the form says what it was asked to expect — "Sound" for a
> correct form, "Not sound" once one case is made wrong — and 1 otherwise.
>
> **Kind:** reference · **For:** the pipeline

The pipeline runs it on `orbit_velocity`'s form filled with the worked example
(`xtask form orbit_velocity --example`), so the WebAssembly checker inside every
form is exercised where a teammate uses it: in a browser, from a file, offline.

    python3 tools/form_check.py <form.html>
    python3 tools/form_check.py --selftest
"""
import asyncio
import os
import sys


async def check(path):
    from playwright.async_api import async_playwright

    exe = "/opt/pw-browsers/chromium" if os.path.exists("/opt/pw-browsers/chromium") else None
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path=exe) if exe else await p.chromium.launch()
        pg = await b.new_page()
        errors = []
        pg.on("pageerror", lambda e: errors.append(str(e)))
        await pg.goto("file://" + os.path.abspath(path))
        await pg.wait_for_function(
            "document.querySelector('#nf-check .nf-sound') !== null", timeout=20000
        )
        first = await pg.inner_text("#nf-check")
        # Now make one case wrong, as an author's typo would, and the form must say so.
        boxes = pg.locator('[aria-label$=" expect"]')
        if await boxes.count() == 0:
            await b.close()
            return ["the form has no case with an expected value to change"], first, ""
        await boxes.first.fill("1.0")
        await pg.wait_for_timeout(1200)
        second = await pg.inner_text("#nf-check")
        await b.close()
        problems = list(errors)
        if "Sound: the method checks" not in first:
            problems.append("the filled form does not say Sound:\n" + first)
        if "Not sound yet" not in second:
            problems.append("a wrong case did not make the form say Not sound:\n" + second)
        return problems, first, second


def selftest():
    # The two verdicts this reads are the page's own words; if they move, this
    # must move with them. Read them from the generator rather than trusting
    # a copy here.
    sheet = os.path.join(os.path.dirname(__file__), "..", "crates", "vleo-sheet")
    # The page's script and style are assets the generator compiles in.
    src = "".join(open(os.path.join(sheet, *p)).read() for p in (
        ("src", "template.rs"), ("assets", "node_form.js"), ("assets", "node_form.css")))
    for words in ("Sound: the method checks", "Not sound yet", "nf-sound", "nf-check"):
        assert words in src, "the form no longer says %r" % words
    print("selftest: ok")


def main():
    if sys.argv[1:] == ["--selftest"]:
        selftest()
        return 0
    if len(sys.argv) != 2:
        print(__doc__.split("\n\n")[-1])
        return 2
    problems, first, _ = asyncio.run(check(sys.argv[1]))
    print(first)
    for p in problems:
        print("FAIL", p)
    print("form check: %s" % ("ok" if not problems else "%d problem(s)" % len(problems)))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
