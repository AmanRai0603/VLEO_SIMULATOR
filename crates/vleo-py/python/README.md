# `vleo` — the Python face

```python
import vleo

vleo.version()                       # kernel, graph, node count
r = vleo.evaluate("prop_thrust_to_drag", case="customer_2", condition="low_altitude")
r.value, r.credibility, r.governing, r.chain

x, y, refused = vleo.sweep(
    "prop_thrust_to_drag", "orbit_altitude", 150_000, 450_000, points=80
)
```

`case` names the customer and may be left out, which means the first customer
by id. A `condition` is laid over the customer, and `sets` over both. An unknown
name, or a condition that cannot be applied, raises with the reason rather than
running the plain design under that name. `vleo cases` on the command line lists
them.

Everything is SI. A face converts for display and never for transport.

Out of range is refused, never clamped, and the exception names the field, the
bound it broke and the reason that bound exists.

## MATLAB

    >> vleo_install    % pins the interpreter, sets out-of-process execution, verifies

MATLAB reaches this wheel out of process. MATLAB's job is fixtures: explore,
produce a golden vector, and commit that vector as what the Rust must
reproduce. Exploration becomes evidence rather than a parallel codebase.
