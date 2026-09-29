//! What a result draws, described once, by the engine.
//!
//! A FIGURE IS PART OF THE OUTPUT, NOT A PICTURE THE PAGE INVENTS. The engine
//! says what there is to see — which kind of figure, its axes with their units,
//! every series and the row it comes from, and every point that could not be
//! computed and why — and any face draws that: the page, the report, a script
//! reading the JSON. Two faces drawing one description cannot disagree about
//! what the result showed.
//!
//! The kinds cover what this design will need to show, so that adding one to a
//! face is drawing work, never a change to what a result is: lines, scatter and
//! bars across one input; a heatmap across two; an animation, which is a line
//! figure in frames over time; and a 3D scene of bodies and paths, which grows
//! richer in the same format. `check` refuses a description that does not hold
//! together, so a face can trust the shape it is given.
//!
//! Every value is in the axis's display unit; `factor` turns it back into SI.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::results::Sweep;

/// What kind of figure it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Line,
    Scatter,
    Bar,
    Heatmap,
    Animation,
    Scene3d,
}

impl Kind {
    pub const ALL: [Kind; 6] = [
        Kind::Line,
        Kind::Scatter,
        Kind::Bar,
        Kind::Heatmap,
        Kind::Animation,
        Kind::Scene3d,
    ];

    /// Its name on the wire.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Line => "line",
            Kind::Scatter => "scatter",
            Kind::Bar => "bar",
            Kind::Heatmap => "heatmap",
            Kind::Animation => "animation",
            Kind::Scene3d => "scene3d",
        }
    }
}

/// One axis: what it measures, in which unit, and the factor from that unit
/// to SI.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Axis {
    /// The row or input it measures.
    pub id: String,
    pub label: String,
    pub unit: String,
    pub factor: f64,
}

/// One line, set of points or set of bars. `None` is a point that could not be
/// computed: drawn as a gap, never joined across.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Series {
    pub name: String,
    /// The row it shows.
    pub row: String,
    pub x: Vec<f64>,
    pub y: Vec<Option<f64>>,
}

/// A value on a grid of two inputs, row-major: `z[j * x.len() + i]` is at
/// `(x[i], y[j])`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grid {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<Option<f64>>,
}

/// One frame of an animation: its time, in the `t` axis's unit, and the
/// series as they stand then.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub t: f64,
    pub series: Vec<Series>,
}

/// One thing in a 3D scene: a body at a point, or a path through points.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    pub name: String,
    /// `point` or `path`.
    pub shape: String,
    pub points: Vec<[f64; 3]>,
}

/// A point called out, with what it is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Note {
    pub x: f64,
    pub y: f64,
    pub text: String,
}

/// A point that could not be computed, with why. Kept, never dropped.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gap {
    pub x: f64,
    pub why: String,
}

/// One figure.
#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    /// What it shows, in one sentence: answer first.
    pub says: String,
    pub x: Axis,
    pub y: Axis,
    /// The value axis of a heatmap, the time of an animation, the third
    /// dimension of a scene.
    pub z: Option<Axis>,
    pub series: Vec<Series>,
    pub grid: Option<Grid>,
    pub frames: Vec<Frame>,
    pub scene: Vec<Body>,
    pub notes: Vec<Note>,
    pub gaps: Vec<Gap>,
}

impl Figure {
    /// An empty figure of a kind, to be filled.
    pub fn new(id: &str, kind: Kind, title: &str) -> Figure {
        Figure {
            id: id.to_string(),
            kind,
            title: title.to_string(),
            says: String::new(),
            x: Axis::default(),
            y: Axis::default(),
            z: None,
            series: Vec::new(),
            grid: None,
            frames: Vec::new(),
            scene: Vec::new(),
            notes: Vec::new(),
            gaps: Vec::new(),
        }
    }
}

/// Whether a description holds together — refused with why, so a face is never
/// handed a shape it has to guess at.
pub fn check(f: &Figure) -> Result<(), String> {
    let series_ok = |s: &Series, at: &str| {
        if s.x.len() != s.y.len() {
            return Err(format!(
                "{}: {at}series '{}' has {} x and {} y",
                f.id,
                s.name,
                s.x.len(),
                s.y.len()
            ));
        }
        if s.x.iter().any(|v| !v.is_finite()) || s.y.iter().flatten().any(|v| !v.is_finite()) {
            return Err(format!(
                "{}: {at}series '{}' has a value that is not a number; a point that could \
                 not be computed is a gap",
                f.id, s.name
            ));
        }
        Ok(())
    };
    for s in &f.series {
        series_ok(s, "")?;
    }
    for (k, fr) in f.frames.iter().enumerate() {
        for s in &fr.series {
            series_ok(s, &format!("frame {k}: "))?;
        }
    }
    for a in [Some(&f.x), Some(&f.y), f.z.as_ref()].into_iter().flatten() {
        if !(a.factor.is_finite() && a.factor != 0.0) {
            return Err(format!(
                "{}: axis '{}' has no factor to SI; 1 when it is SI already",
                f.id, a.id
            ));
        }
    }
    let need = |what: &str, ok: bool| {
        if ok {
            Ok(())
        } else {
            Err(format!("{}: a {} figure needs {what}", f.id, f.kind.name()))
        }
    };
    match f.kind {
        Kind::Line | Kind::Scatter | Kind::Bar => need("a series", !f.series.is_empty()),
        Kind::Heatmap => {
            need("a value axis, z", f.z.is_some())?;
            let g = f
                .grid
                .as_ref()
                .ok_or_else(|| format!("{}: a heatmap figure needs a grid", f.id))?;
            need(
                "one value per grid point",
                g.z.len() == g.x.len() * g.y.len() && !g.z.is_empty(),
            )
        }
        Kind::Animation => {
            need("a time axis, z", f.z.is_some())?;
            need("frames", !f.frames.is_empty())?;
            need(
                "frames in time order",
                f.frames.windows(2).all(|w| w[0].t < w[1].t),
            )
        }
        Kind::Scene3d => {
            need("a third axis, z", f.z.is_some())?;
            need("something in the scene", !f.scene.is_empty())?;
            need(
                "a point or a path for each body",
                f.scene.iter().all(|b| match b.shape.as_str() {
                    "point" => b.points.len() == 1,
                    "path" => b.points.len() >= 2,
                    _ => false,
                }),
            )?;
            need(
                "numbers for every coordinate",
                f.scene
                    .iter()
                    .all(|b| b.points.iter().flatten().all(|v| v.is_finite())),
            )
        }
    }
}

/// A saved sweep's figure: the answer across the input it moved, every
/// refused point a gap with why, and the case it was saved at called out.
///
/// `here` is the case's own input and answer, in SI, when both are known.
pub fn from_sweep(target: &str, w: &Sweep, here: Option<(f64, f64)>) -> Figure {
    let mut f = Figure::new("sweep", Kind::Line, &format!("{target} across {}", w.over));
    let unit = |u: &str| {
        if u.is_empty() || u == "-" {
            "dimensionless".to_string()
        } else {
            u.to_string()
        }
    };
    let fx = if w.x_factor.is_finite() && w.x_factor != 0.0 {
        w.x_factor
    } else {
        1.0
    };
    let fy = if w.y_factor.is_finite() && w.y_factor != 0.0 {
        w.y_factor
    } else {
        1.0
    };
    f.x = Axis {
        id: w.over.clone(),
        label: if w.over_name.is_empty() {
            w.over.clone()
        } else {
            w.over_name.clone()
        },
        unit: unit(&w.x_unit),
        factor: fx,
    };
    f.y = Axis {
        id: target.to_string(),
        label: target.to_string(),
        unit: unit(&w.y_unit),
        factor: fy,
    };
    // Answered and refused points together, in order along x: a refused point
    // is a gap in the line exactly where it fell.
    let mut pts: Vec<(f64, Option<f64>)> =
        w.x.iter()
            .zip(&w.y)
            .map(|(x, y)| (*x / fx, Some(*y / fy)))
            .chain(w.refused.iter().map(|(x, _)| (*x / fx, None)))
            .collect();
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    f.series.push(Series {
        name: target.to_string(),
        row: target.to_string(),
        x: pts.iter().map(|p| p.0).collect(),
        y: pts.iter().map(|p| p.1).collect(),
    });
    f.gaps = w
        .refused
        .iter()
        .map(|(x, why)| Gap {
            x: *x / fx,
            why: why.clone(),
        })
        .collect();
    if let Some((x, y)) = here {
        f.notes.push(Note {
            x: x / fx,
            y: y / fy,
            text: "this case".to_string(),
        });
    }
    f.says = format!(
        "{target} across {}: {} answered, {} refused",
        w.over,
        w.x.len(),
        w.refused.len()
    );
    f
}

/// One well-formed figure of every kind, with illustrative numbers — what a
/// face is built and tested against before the engine produces that kind for
/// a real result, and what the contract tests hold every kind to.
pub fn samples() -> Vec<Figure> {
    let ax = |id: &str, unit: &str| Axis {
        id: id.to_string(),
        label: id.to_string(),
        unit: unit.to_string(),
        factor: 1.0,
    };
    let line = |name: &str, k: f64| Series {
        name: name.to_string(),
        row: name.to_string(),
        x: (0..5).map(|i| i as f64).collect(),
        y: (0..5)
            .map(|i| if i == 3 { None } else { Some(k * i as f64) })
            .collect(),
    };
    Kind::ALL
        .iter()
        .map(|&kind| {
            let mut f = Figure::new(kind.name(), kind, &format!("a {} figure", kind.name()));
            f.says = "illustrative: the shape of this kind, not a result".to_string();
            f.x = ax("x", "km");
            f.y = ax("y", "m/s");
            match kind {
                Kind::Line | Kind::Scatter | Kind::Bar => {
                    f.series = alloc::vec![line("a", 1.0), line("b", 2.0)];
                    f.gaps = alloc::vec![Gap {
                        x: 3.0,
                        why: "refused here".to_string()
                    }];
                    f.notes = alloc::vec![Note {
                        x: 1.0,
                        y: 1.0,
                        text: "this case".to_string()
                    }];
                }
                Kind::Heatmap => {
                    f.z = Some(ax("z", "K"));
                    f.grid = Some(Grid {
                        x: alloc::vec![0.0, 1.0, 2.0],
                        y: alloc::vec![0.0, 1.0],
                        z: alloc::vec![Some(1.0), Some(2.0), None, Some(4.0), Some(5.0), Some(6.0)],
                    });
                }
                Kind::Animation => {
                    f.z = Some(ax("t", "s"));
                    f.frames = (0..3)
                        .map(|k| Frame {
                            t: k as f64 * 10.0,
                            series: alloc::vec![line("a", k as f64)],
                        })
                        .collect();
                }
                Kind::Scene3d => {
                    f.z = Some(ax("z", "km"));
                    f.scene = alloc::vec![
                        Body {
                            name: "Earth".to_string(),
                            shape: "point".to_string(),
                            points: alloc::vec![[0.0; 3]]
                        },
                        Body {
                            name: "orbit".to_string(),
                            shape: "path".to_string(),
                            points: alloc::vec![
                                [6700.0, 0.0, 0.0],
                                [0.0, 6700.0, 0.0],
                                [-6700.0, 0.0, 0.0]
                            ],
                        },
                    ];
                }
            }
            f
        })
        .collect()
}

/// A figure as JSON — the form every face reads.
pub fn json(f: &Figure) -> String {
    let mut o = String::from("{");
    field(&mut o, "id", &text(&f.id));
    field(&mut o, "kind", &text(f.kind.name()));
    field(&mut o, "title", &text(&f.title));
    field(&mut o, "says", &text(&f.says));
    field(&mut o, "x", &axis(&f.x));
    field(&mut o, "y", &axis(&f.y));
    field(
        &mut o,
        "z",
        &f.z.as_ref().map(axis).unwrap_or_else(|| "null".into()),
    );
    field(&mut o, "series", &list(f.series.iter().map(series)));
    let grid = f.grid.as_ref().map(|g| {
        format!(
            "{{\"x\":{},\"y\":{},\"z\":{}}}",
            list(g.x.iter().map(|v| num(*v))),
            list(g.y.iter().map(|v| num(*v))),
            list(
                g.z.iter()
                    .map(|v| v.map(num).unwrap_or_else(|| "null".into()))
            )
        )
    });
    field(&mut o, "grid", &grid.unwrap_or_else(|| "null".into()));
    field(
        &mut o,
        "frames",
        &list(f.frames.iter().map(|fr| {
            format!(
                "{{\"t\":{},\"series\":{}}}",
                num(fr.t),
                list(fr.series.iter().map(series))
            )
        })),
    );
    field(
        &mut o,
        "scene",
        &list(f.scene.iter().map(|b| {
            format!(
                "{{\"name\":{},\"shape\":{},\"points\":{}}}",
                text(&b.name),
                text(&b.shape),
                list(b.points.iter().map(|p| list(p.iter().map(|v| num(*v)))))
            )
        })),
    );
    field(
        &mut o,
        "notes",
        &list(f.notes.iter().map(|n| {
            format!(
                "{{\"x\":{},\"y\":{},\"text\":{}}}",
                num(n.x),
                num(n.y),
                text(&n.text)
            )
        })),
    );
    field(
        &mut o,
        "gaps",
        &list(
            f.gaps
                .iter()
                .map(|g| format!("{{\"x\":{},\"why\":{}}}", num(g.x), text(&g.why))),
        ),
    );
    o.push('}');
    o
}

fn field(o: &mut String, k: &str, v: &str) {
    if o.len() > 1 {
        o.push(',');
    }
    o.push_str(&text(k));
    o.push(':');
    o.push_str(v);
}

fn axis(a: &Axis) -> String {
    format!(
        "{{\"id\":{},\"label\":{},\"unit\":{},\"factor\":{}}}",
        text(&a.id),
        text(&a.label),
        text(&a.unit),
        num(a.factor)
    )
}

fn series(s: &Series) -> String {
    format!(
        "{{\"name\":{},\"row\":{},\"x\":{},\"y\":{}}}",
        text(&s.name),
        text(&s.row),
        list(s.x.iter().map(|v| num(*v))),
        list(
            s.y.iter()
                .map(|v| v.map(num).unwrap_or_else(|| "null".into()))
        )
    )
}

fn list(items: impl Iterator<Item = String>) -> String {
    let v: Vec<String> = items.collect();
    format!("[{}]", v.join(","))
}

fn num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:?}")
    } else {
        "null".to_string()
    }
}

fn text(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
