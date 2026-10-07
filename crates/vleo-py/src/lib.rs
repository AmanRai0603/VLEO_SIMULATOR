//! The Python face.
//!
//! Not a second engine. This binds the same library the interface calls, which
//! is the whole point: an analyst exploring in a notebook and a reviewer
//! reading a page are looking at numbers that came from one implementation.
//!
//! # MATLAB's job, stated narrowly
//!
//! MATLAB reaches this wheel out of process, because an in-process native
//! extension can bring MATLAB down and because MATLAB ships its own runtime
//! libraries. And MATLAB's job is fixtures: somebody explores, produces a
//! golden vector, and that vector becomes what the Rust must reproduce.
//! Exploration becomes evidence rather than a parallel codebase.
//!
//! The GIL is released during evaluation, so a sweep parallelises. The engine's
//! one shared setting is which graph it runs, set when the design is opened
//! and only read after; a run holds nothing else in common.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use vleo_bus::{Case, RunMode};
use vleo_modules::{vars, Scratch, Vleo};

/// One answer, with everything needed to defend it.
#[pyclass]
#[derive(Clone)]
pub struct Result_ {
    /// The value, in SI.
    #[pyo3(get)]
    pub value: f64,
    #[pyo3(get)]
    pub unit: String,
    #[pyo3(get)]
    pub symbol: String,
    /// The lowest of the eight credibility factors.
    #[pyo3(get)]
    pub credibility: u8,
    #[pyo3(get)]
    pub governing: String,
    #[pyo3(get)]
    pub ran: usize,
    #[pyo3(get)]
    pub blocked: usize,
    /// The run's identity and the cache key.
    #[pyo3(get)]
    pub chain: String,
    #[pyo3(get)]
    pub kernel: String,
    #[pyo3(get)]
    pub graph: String,
}

#[pymethods]
impl Result_ {
    fn __repr__(&self) -> String {
        format!(
            "<vleo.Result {} = {} {} · credibility {} of 4, governed by {} · chain {}>",
            self.symbol, self.value, self.unit, self.credibility, self.governing, self.chain
        )
    }
}

/// Every row in the tree.
#[pyfunction]
fn nodes() -> Vec<String> {
    vleo_modules::nodes()
        .iter()
        .map(|n| n.id.to_string())
        .collect()
}

/// Evaluate one node.
///
/// `sets` maps node identifiers to SI values. Out of range is refused, never
/// clamped, and the exception names the field, the bound and the reason.
#[pyfunction]
// `case` names the case; empty is the one case. The inputs a person saved live
// with the application, not here: pass them as `sets`.
#[pyo3(signature = (node, case = "", sets = None, mode = "branch"))]
fn evaluate(
    py: Python<'_>,
    node: &str,
    case: &str,
    sets: Option<std::collections::HashMap<String, f64>>,
    mode: &str,
) -> PyResult<Result_> {
    if Vleo::find(node).is_none() {
        return Err(PyValueError::new_err(format!("no node '{node}'")));
    }
    let c = Case {
        base: case.to_string(),
        supply: sets.unwrap_or_default().into_iter().collect(),
        target: node.to_string(),
        mode: RunMode::from_name(mode),
        data: Vec::new(),
        data_versions: Vec::new(),
    };
    if let Some(why) = vleo_modules::case_refusal(&c) {
        return Err(PyValueError::new_err(why));
    }
    // The GIL is released for the duration: nothing a run writes is shared,
    // so a sweep is a parallel map with no mutex.
    let out = py.allow_threads(|| {
        let mut scratch = Scratch::new();
        vleo_modules::evaluate(&c, &mut scratch)
    });
    let r = out.map_err(|f| PyRuntimeError::new_err(format!("{f}")))?;
    let v = r.values.iter().find(|v| v.id == node).ok_or_else(|| {
        PyRuntimeError::new_err(
            r.blocked
                .first()
                .map(|b| b.message.clone())
                .unwrap_or_else(|| "the node did not produce a value".into()),
        )
    })?;
    let i = Vleo::find(node).unwrap_or(0);
    Ok(Result_ {
        value: v.value,
        unit: vars()[i as usize].unit.symbol().to_string(),
        symbol: v.symbol.clone(),
        credibility: v.cred.governing_score(),
        governing: v.governing.to_string(),
        ran: r.manifest.ran,
        blocked: r.manifest.blocked_count,
        chain: r.manifest.chain.clone(),
        kernel: r.manifest.kernel.clone(),
        graph: r.manifest.graph.clone(),
    })
}

/// A sweep, returned as two parallel lists.
///
/// One call, not five hundred: the marshalling cost per call is what makes a
/// tight loop calling evaluation per point slow, and then the engine is blamed.
#[pyfunction]
#[pyo3(signature = (node, over, start, stop, points = 64, case = ""))]
fn sweep(
    py: Python<'_>,
    node: &str,
    over: &str,
    start: f64,
    stop: f64,
    points: usize,
    case: &str,
) -> PyResult<(Vec<f64>, Vec<f64>, Vec<(f64, String)>)> {
    if Vleo::find(node).is_none() || Vleo::find(over).is_none() {
        return Err(PyValueError::new_err(
            "the sweep names a node that does not exist",
        ));
    }
    let probe = Case {
        base: case.to_string(),
        ..Default::default()
    };
    if let Some(why) = vleo_modules::case_refusal(&probe) {
        return Err(PyValueError::new_err(why));
    }
    let n = points.max(2);
    Ok(py.allow_threads(|| {
        let mut scratch = Scratch::new();
        let (mut xs, mut ys, mut refused) = (Vec::new(), Vec::new(), Vec::new());
        for i in 0..n {
            let x = start + (i as f64 / (n - 1) as f64) * (stop - start);
            let c = Case {
                base: case.to_string(),
                supply: vec![(over.to_string(), x)],
                target: node.to_string(),
                mode: RunMode::Branch,
                data: Vec::new(),
                data_versions: Vec::new(),
            };
            match vleo_modules::evaluate(&c, &mut scratch) {
                Ok(r) => match r.values.iter().find(|v| v.id == node) {
                    Some(v) => {
                        xs.push(x);
                        ys.push(v.value);
                    }
                    // Recorded, never dropped: a sweep in which some rows
                    // quietly used a substituted value is a sweep whose
                    // conclusion is unknown.
                    None => refused.push((x, "blocked".to_string())),
                },
                Err(f) => refused.push((x, format!("{f}"))),
            }
        }
        (xs, ys, refused)
    }))
}

/// Kernel, graph and node count. The same string every face reports.
#[pyfunction]
fn version() -> (String, String, usize) {
    (
        String::from_utf8(vleo_core::hash::short_hex(Vleo::kernel_hash()).to_vec())
            .unwrap_or_default(),
        String::from_utf8(vleo_core::hash::short_hex(Vleo::graph_hash()).to_vec())
            .unwrap_or_default(),
        vleo_modules::nodes().len(),
    )
}

/// One figure of the solar-weather record, as the JSON text the browser's
/// panel reads — the same function the tool's `/v1/figures/solar/<id>` route
/// answers with, from the same bundle and the same saved case. `query` is a
/// query string; `vleo.figure` builds it from keyword arguments and parses
/// the answer.
#[pyfunction]
#[pyo3(signature = (id, query = "", root = None))]
fn figure(py: Python<'_>, id: &str, query: &str, root: Option<String>) -> String {
    py.allow_threads(|| vleo_server::figure(root.map(Into::into), id, query))
}

/// Start the whole tool — the pages and the engine — on loopback, in the
/// background, and return the port it took. `root` is where the pages and the
/// tree are; the package passes the copy it carries.
#[pyfunction]
#[pyo3(signature = (root = None, port = 7777, open = true))]
fn serve(py: Python<'_>, root: Option<String>, port: u16, open: bool) -> PyResult<u16> {
    py.allow_threads(|| vleo_server::serve(root.map(Into::into), port, open, true))
        .map_err(PyRuntimeError::new_err)
}

/// Open the design — the design file, or a checkout's folders, where the tool
/// finds them under `root` — and run the engine on the graph read from its
/// files, as the server does. Says which graph runs. The package calls it
/// before the first answer; a design that does not open is refused, naming
/// why, and the compiled graph is never run in its place.
#[pyfunction]
#[pyo3(signature = (root = None))]
fn open_design(py: Python<'_>, root: Option<String>) -> PyResult<String> {
    py.allow_threads(|| vleo_server::run_the_design(root.map(Into::into)))
        .map_err(PyRuntimeError::new_err)
}

#[pymodule]
fn _vleo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Result_>()?;
    m.add_function(wrap_pyfunction!(serve, m)?)?;
    m.add_function(wrap_pyfunction!(nodes, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(sweep, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(figure, m)?)?;
    m.add_function(wrap_pyfunction!(open_design, m)?)?;
    Ok(())
}
