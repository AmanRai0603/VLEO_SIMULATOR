//! THE WORKED EXAMPLE every node form can show, station by station.
//!
//! One node filled in completely: the speed of a circular orbit. It is
//! ILLUSTRATIVE — a teaching example of what a good answer to each question
//! looks like, not a node of this design and not evidence for one. Its numbers
//! come from the textbook relation it states, and a test runs its method on its
//! cases so the example can never show something the checker would refuse.
//!
//! Read by the node form ("Show the example" beside each question), by
//! `docs/PSEUDOCODE.md` and by the role guides, so all three show the same
//! example.

/// The node the example is about, as the form names it.
pub const TITLE: &str = "Circular orbital speed (illustrative)";

/// What every example box says about itself.
pub const TAG: &str = "illustrative — a worked example, not this node";

/// The example's answer to one scalar question of the form, by field name.
pub fn field(name: &str) -> Option<&'static str> {
    Some(match name {
        "label" => "Circular orbital speed",
        "question" => "How fast is the spacecraft moving along a circular orbit of a given radius?",
        "note" => "The speed along the orbit, not relative to the rotating atmosphere — the drag rows correct for co-rotation.",
        "explain_simply" => {
            "How fast the spacecraft travels around the Earth. The closer it orbits, the faster \
             it must go to keep falling around the Earth instead of into it: about 7.75 km every \
             second at 250 km up."
        }
        "explain_breaks" => {
            "It assumes a perfect circle and only the Earth pulling. On an orbit 5 per cent away \
             from a circle the real speed swings by 5 per cent around each lap, faster at the \
             lowest point."
        }
        "explain_wrong" => {
            "A higher orbit is not faster. Going up means going slower: at 1000 km the speed is \
             7.35 km/s, 5 per cent less than at 250 km."
        }
        "expression" => "V = sqrt(mu / r)",
        "source" => "vallado2013",
        "theory_why" => {
            "In a circular orbit gravity provides exactly the centripetal force: GMm/r² = mV²/r. \
             The mass of the spacecraft cancels, which is why every object at one radius moves \
             at one speed."
        }
        "theory_reading" => {
            "The speed a circular orbit needs at this radius. It is not the speed of any real, \
             slightly elliptical orbit at any one moment."
        }
        "symbol" => "V",
        "type" => "Velocity",
        "unit" => "MetrePerSecond",
        "lower" => "7000.0",
        "upper" => "8200.0",
        "reason_lower" => "below 7 km/s the orbit is above 1000 km, outside this tool's band",
        "reason_upper" => "8.2 km/s is faster than a circular orbit at the Earth's surface",
        "method_text" => METHOD,
        "author_name" => "Ana Rao",
        "author_language" => "MATLAB",
        "author_entry" => "orbit_speed",
        "author_code" => AUTHOR_CODE,
        "author_test_code" => AUTHOR_TEST,
        "author_how_run" => "MATLAB R2023b on Ana's laptop, 2026-09-20. The output of the test script is the table of cases.",
        _ => return None,
    })
}

/// The method, in the method language.
pub const METHOD: &str = "\
# Vallado (2013), eq. 1-18: the speed of a circular two-body orbit.
if r <= R_EARTH then
  refuse \"the orbit is inside the Earth\"
end
let v : Velocity = sqrt(MU_EARTH / r)
return v";

const AUTHOR_CODE: &str = "\
function v = orbit_speed(r)
% Circular two-body orbital speed: r in metres, v in metres per second.
mu = 3.986004418e14;          % Earth, m^3/s^2
R  = 6378137.0;               % Earth's equatorial radius, m
if r <= R
    error('orbit_speed:inside', 'the orbit is inside the Earth');
end
v = sqrt(mu / r);
end";

const AUTHOR_TEST: &str = "\
% Runs orbit_speed on each case and prints what goes in the form.
radii = [6628137.0, 6778137.0, 7378137.0];
for k = 1:numel(radii)
    fprintf('%.1f  %.8f\\n', radii(k), orbit_speed(radii(k)));
end
try
    orbit_speed(6000000.0);
catch err
    fprintf('6000000.0  refused: %s\\n', err.message);
end";

const FLIGHT_CODE: &str = "\
#include <math.h>
#define MU_EARTH 3.986004418e14
#define R_EARTH  6378137.0
/* The circular speed in m/s, or -1.0 when r is inside the Earth. */
double orbit_speed(double r)
{
    if (r <= R_EARTH) { return -1.0; }
    return sqrt(MU_EARTH / r);
}";

const FLIGHT_TEST: &str = "\
#include <assert.h>
#include <math.h>
double orbit_speed(double r);
int main(void)
{
    assert(fabs(orbit_speed(6628137.0) - 7754.84549737) < 1e-6);
    assert(orbit_speed(6000000.0) < 0.0);
    return 0;
}";

/// The example's blocks for one repeated section of the form, by name: each
/// block as its keys and values.
pub fn blocks(name: &str) -> Vec<Vec<(&'static str, &'static str)>> {
    match name {
        "input" => vec![vec![("binding", "r"), ("var", "orbit_radius"), ("type", "Length")]],
        "assumption" => vec![vec![
            ("text", "Two bodies, and a circular orbit"),
            ("fails_when", "the orbit is noticeably elliptical: at e = 0.05 the speed varies by 5% around the orbit"),
        ]],
        "case" => vec![
            vec![("label", "250 km"), ("refuse", "no"), ("expect", "7754.84549737"), ("tolerance", "1e-6"), ("inputs", "{ r = 6628137.0 }")],
            vec![("label", "400 km"), ("refuse", "no"), ("expect", "7668.55817541"), ("tolerance", "1e-6"), ("inputs", "{ r = 6778137.0 }")],
            vec![("label", "1000 km — the top of the band"), ("refuse", "no"), ("expect", "7350.13862961"), ("tolerance", "1e-6"), ("inputs", "{ r = 7378137.0 }")],
            vec![("label", "inside the Earth"), ("refuse", "yes"), ("inputs", "{ r = 6000000.0 }")],
        ],
        "flight" => vec![vec![
            ("name", "orbit_speed.c"),
            ("language", "C"),
            ("purpose", "The on-board estimate of the orbital speed, used by the drag model between ground updates."),
            ("code", FLIGHT_CODE),
            ("test_code", FLIGHT_TEST),
            ("test_result", "passed, gcc 12 host build, 2026-09-21"),
        ]],
        _ => Vec::new(),
    }
}

/// The example as the part of a sheet the method checker reads, so a test can
/// hold it to the same rules as any node.
pub fn sheet_text() -> String {
    let q = |s: &str| format!("\"\"\"\n{}\n\"\"\"", s.replace('\\', "\\\\"));
    let mut o = format!(
        "[method]\ntext = {}\n\n[output]\ntype = \"Velocity\"\n\n[[input]]\nbinding = \"r\"\ntype = \"Length\"\n",
        q(METHOD)
    );
    for b in blocks("case") {
        o.push_str("\n[[case]]\n");
        for (k, v) in b {
            if k == "inputs" || k == "expect" || k == "tolerance" {
                o.push_str(&format!("{k} = {v}\n"));
            } else {
                o.push_str(&format!("{k} = \"{v}\"\n"));
            }
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_example_passes_its_own_checker() {
        let r = crate::method::report_toml(&sheet_text()).unwrap();
        assert!(r.sound(), "{}", r.json());
    }

    #[test]
    fn every_example_answer_is_one_the_form_would_take() {
        for f in crate::form::FIELDS {
            if let Some(v) = field(f.field) {
                crate::form::normalise(f.field, v)
                    .unwrap_or_else(|e| panic!("the example's {}: {e}", f.field));
            }
        }
    }
}
