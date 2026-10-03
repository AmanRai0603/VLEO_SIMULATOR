"""The example group's own code: circular two-body orbits.

ILLUSTRATIVE. This folder shows the pattern of a group folder; it is not part
of the design. Run it from the group folder:  python3 nodes/orbit_radius/code/orbit.py
It writes every node's results/isolation.csv and the data behind the pictures.
"""
import csv
import math
import os

R_EARTH = 6378137.0          # m, WGS-84 equatorial radius (wgs84)
MU_EARTH = 3.986004418e14    # m^3/s^2, Earth's gravitational parameter (vallado2013)
HERE = os.path.dirname(os.path.abspath(__file__))
GROUP = os.path.normpath(os.path.join(HERE, '..', '..', '..'))


def radius(h_km):
    if h_km < 0:
        raise ValueError('below the surface')
    return R_EARTH + h_km * 1000.0


def speed(r):
    if r <= R_EARTH:
        raise ValueError('inside the Earth')
    return math.sqrt(MU_EARTH / r)


def period(r):
    if r <= R_EARTH:
        raise ValueError('inside the Earth')
    return 2 * math.pi * math.sqrt(r ** 3 / MU_EARTH)


def write(path, head, rows):
    path = os.path.join(GROUP, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', newline='') as f:
        w = csv.writer(f, lineterminator='\n')
        w.writerow(head)
        w.writerows(rows)


def table(fn, inputs, unit_in, unit_out, refusal):
    rows = []
    for x in inputs:
        rows.append([repr(x), repr(fn(x)), '1e-12', 'no', 'code'])
    rows.append([repr(refusal), '', '', 'yes', 'code'])
    return rows


# Each node on its own: the defaults, ordinary cases, both ends of the range, one refusal.
write('nodes/orbit_radius/results/isolation.csv', ['h [km]', 'answer [m]', 'tolerance', 'refuses', 'origin'],
      table(radius, [400.0, 150.0, 250.0, 300.0, 350.0, 450.0], 'km', 'm', -10.0))
radii = [radius(h) for h in (400.0, 150.0, 250.0, 300.0, 350.0, 450.0)]
write('nodes/orbit_velocity/results/isolation.csv', ['r [m]', 'answer [m/s]', 'tolerance', 'refuses', 'origin'],
      table(speed, radii, 'm', 'm/s', 6000000.0))
write('nodes/orbit_period/results/isolation.csv', ['r [m]', 'answer [s]', 'tolerance', 'refuses', 'origin'],
      table(period, radii, 'm', 's', 6000000.0))
write('nodes/period_achieved/results/isolation.csv', ['T [s]', 'answer [s]', 'tolerance', 'refuses', 'origin'],
      [[repr(period(radius(400.0))), repr(period(radius(400.0))), '1e-12', 'no', 'code'],
       ['5240.0', '5240.0', '1e-12', 'no', 'code'], ['5300.0', '5300.0', '1e-12', 'no', 'code'],
       ['5620.0', '5620.0', '1e-12', 'no', 'code'], ['-1.0', '', '', 'yes', 'code']])

# The group as a whole: altitude in, speed and period out.
# Inputs are the group's declared nodes, by id; answers are answer.<node id>.
write('results/group.csv', ['orbit_altitude [km]', 'answer.orbit_velocity [m/s]', 'answer.orbit_period [s]', 'tolerance', 'refuses', 'origin'],
      [[repr(h), repr(speed(radius(h))), repr(period(radius(h))), '1e-12', 'no', 'code'] for h in (400.0, 150.0, 250.0, 300.0, 450.0)])

# The data behind the pictures.
hs = [150 + 25 * i for i in range(35)]
write('nodes/orbit_velocity/figures/speed.csv', ['h [km]', 'v [km/s]'], [[h, round(speed(radius(h)) / 1000, 6)] for h in hs])
write('nodes/orbit_period/figures/period.csv', ['h [km]', 'T [min]'], [[h, round(period(radius(h)) / 60, 6)] for h in hs])
rows = []
for h in (200, 400, 600, 800, 1000):
    T = period(radius(h))
    for k in range(0, 121, 3):
        t = k * 60.0
        rows.append([h, k, round((radius(h) / 1000) * math.cos(2 * math.pi * t / T), 3)])
write('nodes/orbit_period/figures/coming-round.csv', ['h [km]', 't [min]', 'x [km]'], rows)
rows = []
for k in range(0, 73):
    a = 2 * math.pi * k / 72
    rows.append(['Earth', 'path', round(R_EARTH / 1000 * math.cos(a), 3), round(R_EARTH / 1000 * math.sin(a), 3), 0])
for k in range(0, 73):
    a = 2 * math.pi * k / 72
    r = radius(400) / 1000
    rows.append(['orbit at 400 km, inclined 51.6 deg', 'path', round(r * math.cos(a), 3),
                 round(r * math.sin(a) * math.cos(math.radians(51.6)), 3), round(r * math.sin(a) * math.sin(math.radians(51.6)), 3)])
a = math.radians(140)
rows.append(['spacecraft', 'point', round(radius(400) / 1000 * math.cos(a), 3),
             round(radius(400) / 1000 * math.sin(a) * math.cos(math.radians(51.6)), 3),
             round(radius(400) / 1000 * math.sin(a) * math.sin(math.radians(51.6)), 3)])
write('figures/orbit3d.csv', ['body', 'shape', 'x [km]', 'y [km]', 'z [km]'], rows)
