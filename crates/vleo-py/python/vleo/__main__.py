"""python -m vleo — start the VLEO design tool and open it in the browser.

    python -m vleo              start it; close the window or press Ctrl-C to stop
    python -m vleo --no-open    start it without opening the browser
    python -m vleo --port 8080  start from another port
    python -m vleo --check      start it, ask it one question, stop: proves the
                                install works on this computer
"""

import argparse
import json
import os
import sys
import time
import urllib.request

import vleo


def check(port):
    """Ask the running tool what it is, and say whether the answer is whole."""
    # Straight to loopback: a company proxy must not see, or break, this.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with opener.open("http://127.0.0.1:%d/v1/version" % port, timeout=60) as r:
        v = json.loads(r.read().decode("utf-8"))
    with opener.open("http://127.0.0.1:%d/" % port, timeout=60) as r:
        page = r.read().decode("utf-8", "replace")
    problems = []
    if not v.get("nodes"):
        problems.append("the engine reported no rows")
    if not v.get("data"):
        problems.append("no reference data loaded — the solar rows would refuse")
    if "VLEO" not in page:
        problems.append("the page did not come back")
    if problems:
        print("vleo %s: NOT OK — %s" % (vleo.__version__, "; ".join(problems)))
        return 1
    print("vleo %s: ok — %d rows, data %s, page served on port %d"
          % (vleo.__version__, v["nodes"], ", ".join(d.split("#")[0] for d in v["data"]), port))
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(prog="python -m vleo",
                                 description="Start the VLEO design tool and open it in the browser.")
    ap.add_argument("--port", type=int, default=int(os.environ.get("VLEO_PORT", "7777")),
                    help="the first port to try (default 7777)")
    ap.add_argument("--no-open", action="store_true", help="do not open the browser")
    ap.add_argument("--check", action="store_true",
                    help="start, answer one request, stop — proves the install works")
    a = ap.parse_args(argv)

    port = vleo.serve(a.port, not (a.no_open or a.check))
    if a.check:
        return check(port)
    print("\nThe tool is at http://127.0.0.1:%d — close this window or press Ctrl-C to stop it." % port)
    try:
        while True:
            time.sleep(3600)
    except KeyboardInterrupt:
        return 0


if __name__ == "__main__":
    sys.exit(main())
