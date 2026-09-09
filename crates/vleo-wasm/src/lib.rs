//! The browser face.
//!
//! # What this carries, and why so little
//!
//! Three things this module could have been:
//!
//! | | it holds | the URL alone can | exposure if decompiled |
//! |---|---|---|---|
//! | W1 | the full kernel | everything a laptop can | the whole engine |
//! | W2 | no physics — case building, domain check, transport | nothing without the installer | nothing |
//! | **W3** | **the demonstration node subset** | **read everything, run the demonstration** | **the subset already on the page** |
//!
//! W3 is the one taken. WebAssembly decompiles to readable pseudo-code, so the
//! honest question is not whether it can be read but what reading it buys —
//! and here the answer is: the equations that are printed in full on the page
//! beside it.
//!
//! Heavy work never comes here. Numerical loops are meaningfully slower than
//! native, a 32-bit address space caps each instance at 4 GB whatever the host,
//! and a browser cannot use a customer's own hardware. Those are the three
//! reasons the working tool is installed and this is a demonstration.
//!
//! # Two threads, one engine
//!
//! The UI thread never blocks: the engine runs in a worker and the result comes
//! back as a transferred buffer, so ownership moves and nothing is copied. A
//! hundred-millisecond evaluation on the UI thread is a hundred milliseconds of
//! frozen scroll.
//!
//! No `SharedArrayBuffer`. It requires cross-origin isolation headers, which
//! breaks opening the file from disk; a transferred buffer costs one move and
//! works everywhere. Because this module stays single-threaded and small, the
//! site never needs special response headers and any static host will serve it.

use vleo_bus::{Case, RunMode};
use vleo_modules::{Scratch, Vleo, NODES, VARS};
use wasm_bindgen::prelude::*;

/// The nodes the public build will run.
///
/// Everything else is present in the tree, readable on the page, and refuses in
/// this face with a message naming the installer. Decision D2: the subset is a
/// build flag and a list, not a fork.
const DEMONSTRATION: &[&str] = &[
    "orbit_altitude",
    "orbit_radius",
    "orbit_velocity",
    "orbit_period",
    "env_exospheric_temperature",
    "env_local_temperature",
    "env_mass_density",
    "env_number_density",
    "env_mean_molar_mass",
    "aero_speed_ratio",
    "aero_drag_coefficient",
    "aero_dynamic_pressure",
    "aero_drag_force",
    "prop_capture_efficiency",
    "prop_compression_ratio",
    "prop_collected_flow",
];

#[wasm_bindgen(start)]
pub fn start() {
    // A WebAssembly panic with no hook is a silent stop. This is the cheapest
    // thing in the whole face and it is the difference between a bug report and
    // a shrug.
    console_error_panic_hook::set_once();
}

/// The engine's identity, so a page carrying a different one refuses to run
/// rather than showing a number from an engine it was not built against.
#[wasm_bindgen]
pub fn kernel_hash() -> String {
    hex(Vleo::kernel_hash())
}

#[wasm_bindgen]
pub fn graph_hash() -> String {
    hex(Vleo::graph_hash())
}

/// Whether this face will run a node at all.
#[wasm_bindgen]
pub fn is_demonstration(node: &str) -> bool {
    DEMONSTRATION.contains(&node)
}

/// Evaluate a demonstration node.
///
/// `sets` is a `;`-separated list of `id:value` in SI. Scalars come back as a
/// small JSON object; bulk output does not come through here at all.
#[wasm_bindgen]
pub fn evaluate(node: &str, base: &str, sets: &str) -> String {
    if !is_demonstration(node) {
        return format!(
            "{{\"ok\":false,\"message\":\"{node} is not in the demonstration subset. The page shows its question, its mathematics, its interface, its evidence and its credibility; running it needs the installed engine.\"}}"
        );
    }
    let supply: Vec<(String, f64)> = sets
        .split(';')
        .filter_map(|kv| {
            let (k, v) = kv.split_once(':')?;
            Some((k.to_string(), v.parse().ok()?))
        })
        .collect();
    let case = Case {
        base: base.to_string(),
        supply,
        target: node.to_string(),
        mode: RunMode::Branch,
        // The shipped climatology travels inside the page. Anything a node
        // declares beyond that refuses by name.
        data: vec!["solar-drivers".to_string()],
        data_versions: vec!["solar-drivers@shipped".to_string()],
    };
    let mut scratch = Scratch::new();
    match vleo_modules::evaluate(&case, &mut scratch) {
        Err(f) => format!(
            "{{\"ok\":false,\"fault\":\"{}\",\"message\":\"{}\"}}",
            f.kind(),
            escape(&format!("{f}"))
        ),
        Ok(r) => match r.values.iter().find(|v| v.id == node) {
            None => "{\"ok\":false,\"message\":\"the node did not produce a value\"}".to_string(),
            Some(v) => {
                let i = Vleo::find(node).unwrap_or(0);
                let (shown, sym) = vleo_bus::present(v.value, VARS[i as usize].unit, 6);
                format!(
                    "{{\"ok\":true,\"id\":\"{}\",\"si\":{},\"shown\":\"{}\",\"unit\":\"{}\",\"cred\":{},\"governing\":\"{}\",\"ran\":{},\"chain\":\"{}\",\"endpoint\":\"demo-kernel\"}}",
                    v.id,
                    v.value,
                    shown,
                    sym,
                    v.cred.governing_score(),
                    v.governing,
                    r.manifest.ran,
                    r.manifest.chain
                )
            }
        },
    }
}

/// How many rows the tree holds, so the page can say `n of 250` honestly even
/// when it can only run sixteen of them.
#[wasm_bindgen]
pub fn node_count() -> usize {
    NODES.len()
}

fn hex(h: u64) -> String {
    String::from_utf8(vleo_core::hash::short_hex(h).to_vec()).unwrap_or_default()
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
