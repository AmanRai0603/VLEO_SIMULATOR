# `vleo` — the Python face

```python
import vleo

vleo.version()                       # kernel, graph, node count
r = vleo.evaluate("prop_thrust_to_drag", sets={"orbit_altitude": 200_000.0})
r.value, r.credibility, r.governing, r.chain

x, y, refused = vleo.sweep(
    "prop_thrust_to_drag", "orbit_altitude", 150_000, 450_000, points=80
)
```

There is one case, and its inputs start at their declared defaults. `sets`
changes any of them, in SI. The inputs a person saved in the application — the
CSV uploaded on the Inputs page — are not applied here: read that file and pass
its values as `sets`, or run `vleo inputs` to see them.

Everything is SI. A face converts for display and never for transport.

Out of range is refused, never clamped, and the exception names the field, the
bound it broke and the reason that bound exists.

## MATLAB

    >> vleo_install    % pins the interpreter, sets out-of-process execution, verifies

MATLAB reaches this wheel out of process. MATLAB's job is fixtures: explore,
produce a golden vector, and commit that vector as what the Rust must
reproduce. Exploration becomes evidence rather than a parallel codebase.
