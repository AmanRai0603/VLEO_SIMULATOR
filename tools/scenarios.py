#!/usr/bin/env python3
"""The installed program, driven as people use it, on a copy of the test drive.

docs/PLAN_F.md, decisions 5 and 6: every workflow of 1.0.0 is proved in CI
before it ships, because nobody tries it first. Each scenario here starts from
a fresh copy of the drive `cargo run -p xtask -- test-drive` writes, starts the
program on it once for each person in the scenario — two people are two
program instances on one folder, as two computers on one shared drive would be
— opens its page in a real browser, and does what the person would do.

The scenarios grow stage by stage with phase F. Those here hold today:

  opens            the program opens on the drive, says it built today's
                   design from it, and its page loads with no error
  two-people       two people on one drive are given the same answers
  refused-release  a release whose files are not the ones sealed is refused,
                   and the group's last good release is taken in its place

    python3 tools/scenarios.py                  every scenario
    python3 tools/scenarios.py opens two-people  the ones named
    python3 tools/scenarios.py --selftest       the runner goes red on a
                                                scenario whose claim is false

`--program <path>` names the program (default: target/release/vleo-daemon,
built if it is not there). Exit 0 when every step held; 1 with the steps that
did not.
"""
import json
import os
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def chromium_path():
    """The browser panel_check already found. One rule, one place."""
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from panel_check import chromium_path as p
    return p()


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Run:
    """One scenario's steps, each held or not, with what was seen."""

    def __init__(self, name):
        self.name, self.fails = name, []

    def step(self, what, ok, seen=""):
        print(f"  {'ok  ' if ok else 'FAIL'} {self.name}: {what}{' — ' + seen if seen else ''}")
        if not ok:
            self.fails.append(f"{self.name}: {what}")
        return ok


class Person:
    """The program started for one person on a drive, with a home of its own:
    their saved case, results and crash log, as on their own computer."""

    def __init__(self, program, drive, home):
        self.port = free_port()
        home.mkdir(parents=True, exist_ok=True)
        env = dict(
            os.environ,
            VLEO_PORT=str(self.port),
            VLEO_DRIVE=str(drive),
            VLEO_CASE=str(home / "case.csv"),
            VLEO_RESULTS=str(home / "results"),
            VLEO_LOG=str(home / "log"),
            VLEO_DATA=str(home / "data"),
            VLEO_ROOT=str(ROOT),
        )
        env.pop("VLEO_DESIGN", None)
        self.log_path = home / "program.log"
        self.log_file = open(self.log_path, "w")
        self.proc = subprocess.Popen([str(program)], cwd=ROOT, env=env,
                                     stdout=self.log_file, stderr=subprocess.STDOUT)
        self.url = f"http://127.0.0.1:{self.port}"
        for _ in range(1200):
            if self.proc.poll() is not None:
                raise RuntimeError("the program stopped before it served: " + self.log())
            try:
                self.get("/v1/version")
                return
            except OSError:
                time.sleep(0.1)
        raise RuntimeError(f"the program did not serve on {self.port}: " + self.log())

    def get(self, path):
        with urllib.request.urlopen(self.url + path, timeout=120) as r:
            return json.loads(r.read().decode("utf-8"))

    def log(self):
        self.log_file.flush()
        return self.log_path.read_text(errors="replace")

    def stop(self):
        self.proc.terminate()
        try:
            self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.proc.kill()
        self.log_file.close()


class World:
    """A fresh copy of the test drive for one scenario, and the people on it."""

    def __init__(self, program, base, scratch, name):
        self.dir = scratch / name
        shutil.copytree(base, self.dir)
        self.releases = self.dir / "releases"
        self.drive = self.dir / "drive"
        self.program, self.people = program, []

    def release(self, name):
        """Put one of the sealed releases beside the drive onto it, as its
        group would; returns where it is on the drive."""
        group = name.rsplit("-", 1)[0]
        to = self.drive / "groups" / group / "releases" / name
        to.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(self.releases / name, to)
        return to

    def person(self, who):
        p = Person(self.program, self.drive, self.dir / "homes" / who)
        self.people.append(p)
        return p

    def close(self):
        for p in self.people:
            p.stop()


def tamper(release):
    """Change one method after sealing: its files are no longer the sealed ones."""
    with sqlite3.connect(release) as db:
        n = db.execute(
            "UPDATE doc SET body = body || '\n# changed after sealing' "
            "WHERE kind = 'pseudocode' AND scope = 'sw_f107_design_long'").rowcount
    if n != 1:
        raise RuntimeError("the fixture holds no method to change")


# ---------------------------------------------------------------------------
# the scenarios


def opens(world, run):
    pat = world.person("programme-manager")
    log = pat.log()
    run.step("the program builds today's design from the drive",
             "today's design, built from the drive" in log)
    run.step("it says no group has released yet",
             "no group has a release on the drive" in log)
    v = pat.get("/v1/version")
    run.step("it runs the whole design", v.get("nodes") == 1401, f"{v.get('nodes')} rows")
    from playwright.sync_api import sync_playwright
    errors = []
    with sync_playwright() as pw:
        b = pw.chromium.launch(executable_path=chromium_path())
        page = b.new_page()
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.goto(pat.url + "/")
        page.wait_for_load_state("networkidle")
        run.step("its page loads", page.locator("#tabs").count() == 1)
        b.close()
    run.step("with no page error", not errors, "; ".join(errors[:2]))


def two_people(world, run):
    world.release("l3_solar-1.1.vleo")
    a, b = world.person("solar-lead"), world.person("system-engineer")
    for who, p in (("the first", a), ("the second", b)):
        run.step(f"{who} takes Solar 1.1 into today's design", "l3_solar 1.1 — its latest" in p.log())
    q = "/v1/run?node=l3_solar_ach_01&mode=branch&inputs=defaults"
    ra, rb = a.get(q), b.get(q)
    strip = lambda r: [(v["id"], v["si"]) for v in r.get("values", [])]
    run.step("both are given the same answers", ra.get("ok") and strip(ra) == strip(rb),
             f"{len(strip(ra))} values")


def refused_release(world, run, break_it=True):
    world.release("l3_solar-1.1.vleo")
    newer = world.release("l3_solar-1.2.vleo")
    (newer.parent / "notes.txt").write_text("not a release")
    if break_it:
        tamper(newer)
    log = world.person("system-engineer").log()
    run.step("the newer release is refused, by why",
             "its latest, 1.2, is refused: its files are not the ones sealed" in log,
             next((l.strip() for l in log.splitlines() if "l3_solar" in l), ""))
    run.step("the last good release is taken in its place", "l3_solar 1.1 — its latest, 1.2" in log)
    run.step("what is not a release is named, not taken",
             "groups/l3_solar/releases/notes.txt: not a release" in log)


SCENARIOS = {
    "opens": opens,
    "two-people": two_people,
    "refused-release": refused_release,
}


# ---------------------------------------------------------------------------


def program_path():
    p = ROOT / "target/release/vleo-daemon"
    if not p.is_file():
        subprocess.run(["cargo", "build", "-q", "--release", "-p", "vleo-daemon"],
                       cwd=ROOT, check=True)
    return p


def base_drive(scratch):
    base = scratch / "base"
    subprocess.run(["cargo", "run", "-q", "-p", "xtask", "--", "test-drive", "--out", str(base)],
                   cwd=ROOT, check=True)
    return base


def run_all(names, program, scenarios=SCENARIOS):
    scratch = Path(tempfile.mkdtemp(prefix="vleo-scenarios-"))
    base = base_drive(scratch)
    fails = []
    for name in names:
        run, world = Run(name), World(program, base, scratch, name)
        try:
            scenarios[name](world, run)
        except Exception as e:  # a scenario that cannot go on is a failed step
            run.step("the scenario ran to its end", False, str(e)[:300])
        finally:
            world.close()
        fails += run.fails
    shutil.rmtree(scratch, ignore_errors=True)
    return fails


def selftest(program):
    """A scenario whose claim is false must fail: the refused-release scenario,
    with the release left as it was sealed, claims a refusal that never
    happens. A runner that passes it proves nothing."""
    broken = {"refused-release": lambda w, r: refused_release(w, r, break_it=False)}
    fails = run_all(["refused-release"], program, broken)
    if not fails:
        print("scenarios selftest: FAILED — a false claim passed")
        return 1
    print(f"scenarios selftest: ok — the false claim failed ({len(fails)} step(s))")
    return 0


def main(argv):
    args, program, named = argv[1:], None, []
    i = 0
    while i < len(args):
        if args[i] == "--program" and i + 1 < len(args):
            program = Path(args[i + 1])
            i += 2
            continue
        if args[i] != "--selftest":
            named.append(args[i])
        i += 1
    unknown = [n for n in named if n not in SCENARIOS]
    if unknown:
        print(f"scenarios: no scenario {', '.join(unknown)}; there are {', '.join(SCENARIOS)}")
        return 1
    program = program or program_path()
    if "--selftest" in args:
        return selftest(program)
    fails = run_all(named or list(SCENARIOS), program)
    if fails:
        print(f"scenarios: {len(fails)} step(s) did not hold")
        return 1
    print("scenarios: every step held")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
