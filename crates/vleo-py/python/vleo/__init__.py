"""The VLEO design tool, as a Python package.

    python -m vleo            start the tool and open it in the browser
    import vleo               the engine, from Python or MATLAB

One package for every laptop. It carries the engine built for each system it
supports and loads the one for the machine it is on, so the same file installs
on Windows, macOS and Linux. The engine is native — the same compiled code the
kit's programs run — and it runs inside python.exe, so nothing new is started.
"""

import importlib.machinery
import importlib.util
import os
import platform
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))


def _system():
    """This machine as the package names it: `windows-x86_64`, `macos-arm64`…"""
    if sys.platform.startswith("win"):
        name = "windows"
    elif sys.platform == "darwin":
        name = "macos"
    elif sys.platform.startswith("linux"):
        name = "linux"
    else:
        name = sys.platform
    machine = platform.machine().lower()
    arch = {"amd64": "x86_64", "x86_64": "x86_64", "x64": "x86_64",
            "arm64": "arm64", "aarch64": "arm64"}.get(machine, machine)
    return name + "-" + arch


def _carried():
    """The systems this package carries an engine for."""
    d = os.path.join(_HERE, "_native")
    return sorted(os.listdir(d)) if os.path.isdir(d) else []


def _load():
    """Load the engine built for this machine.

    From `_native/<system>/` when the package carries engines for several
    systems — the one file shared with the team — and otherwise the one a
    developer built beside this file (`maturin develop`).
    """
    d = os.path.join(_HERE, "_native", _system())
    if os.path.isdir(d):
        for f in sorted(os.listdir(d)):
            if f.startswith("_vleo") and f.endswith((".pyd", ".so")):
                path = os.path.join(d, f)
                loader = importlib.machinery.ExtensionFileLoader("vleo._vleo", path)
                spec = importlib.util.spec_from_file_location("vleo._vleo", path, loader=loader)
                module = importlib.util.module_from_spec(spec)
                loader.exec_module(module)
                sys.modules["vleo._vleo"] = module
                return module
    carried = _carried()
    if carried:
        raise ImportError(
            "This VLEO package has no engine for this computer (%s). It carries: %s. "
            "Use 64-bit Python, or ask for the kit for this system."
            % (_system(), ", ".join(carried))
        )
    from . import _vleo  # a developer's own build

    return _vleo


_engine = _load()

Result_ = _engine.Result_
nodes = _engine.nodes
evaluate = _engine.evaluate
sweep = _engine.sweep
version = _engine.version


def kit_root():
    """Where the pages and the tree are: the copy this package carries, or —
    for a developer's build — wherever the tool finds them itself (None)."""
    carried = os.path.join(_HERE, "_kit")
    return carried if os.path.isdir(os.path.join(carried, "web")) else None


def serve(port=7777, open=True):
    """Start the tool in the background and return the port it took."""
    return _engine.serve(kit_root(), port, open)


def figure(id, **params):
    """The numbers one figure of the solar-weather record draws, as a dict.

    The same answer the tool's panel reads, from the same function, bundle and
    saved case — so a picture can be checked, kept or drawn again here::

        vleo.figure("growth", v="ap", by="cycle")["change"]

    An id, driver or view the engine does not know is refused: the dict
    carries ``ok: False`` and a ``message`` naming what it would accept, and
    this raises ``ValueError`` with that message rather than hand back a
    figure that is not the one asked for.
    """
    import json
    from urllib.parse import urlencode

    answer = json.loads(_engine.figure(id, urlencode({k: str(v) for k, v in params.items()}),
                                       kit_root()))
    if not answer.get("ok"):
        raise ValueError(answer.get("message", "the engine refused the figure"))
    return answer


try:
    from ._build import VERSION as __version__
except ImportError:  # a developer's build
    __version__ = "dev"

__all__ = ["Result_", "nodes", "evaluate", "sweep", "version", "serve", "kit_root", "figure"]
