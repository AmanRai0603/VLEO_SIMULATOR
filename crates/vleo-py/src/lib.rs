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
//! The GIL is released during evaluation, so a sweep parallelises — there is no
//! global state in the engine and nothing shared to protect.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use vleo_bus::{Case, RunMode};
use vleo_modules::{Scratch, Vleo, NODES, VARS};

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
    NODES.iter().map(|n| n.id.to_string()).collect()
}

/// Evaluate one node.
///
/// `sets` maps node identifiers to SI values. Out of range is refused, never
/// clamped, and the exception names the field, the bound and the reason.
#[pyfunction]
#[pyo3(signature = (node, case = "nominal", sets = None, mode = "branch"))]
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
    // The GIL is released for the duration: there is no global state in the
    // engine, so a sweep is a parallel map with no mutex.
    let out = py.allow_threads(|| {
        let mut scratch = Scratch::new();
        vleo_modules::evaluate(&c, &mut scratch)
    });
    let r = out.map_err(|f| PyRuntimeError::new_err(format!("{f}")))?;
    let v = r
        .values
        .iter()
        .find(|v| v.id == node)
        .ok_or_else(|| {
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
        unit: VARS[i as usize].unit.symbol().to_string(),
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
#[pyo3(signature = (node, over, start, stop, points = 64, case = "nominal"))]
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
        return Err(PyValueError::new_err("the sweep names a node that does not exist"));
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
        String::from_utf8(vleo_core::hash::short_hex(Vleo::kernel_hash()).to_vec()).unwrap_or_default(),
        String::from_utf8(vleo_core::hash::short_hex(Vleo::graph_hash()).to_vec()).unwrap_or_default(),
        NODES.len(),
    )
}

#[pymodule]
fn vleo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Result_>()?;
    m.add_function(wrap_pyfunction!(nodes, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(sweep, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
