//! The C application binary interface.
//!
//! Everything that crosses is `#[repr(C)]`, so C, MATLAB, Simulink and the GPU
//! read the same bytes. **No allocation crosses the boundary**: the caller owns
//! every buffer and the engine fills it. A library that allocates memory the
//! caller must free is a library that will leak in somebody else's process.
//!
//! A failure is a return code the caller must inspect, never a sentinel value
//! that looks like a number. `0` is success; anything else names a fault class,
//! and `vleo_last_message` fills a caller-owned buffer with the sentence a
//! person should read.
//!
//! # MATLAB's job, stated narrowly
//!
//! Not the engine. MATLAB is where fixtures are made: somebody explores,
//! produces a golden vector, and that vector becomes what the Rust must
//! reproduce. Exploration becomes evidence rather than a parallel codebase —
//! which is the failure this whole architecture exists to prevent.

use std::cell::RefCell;
use std::ffi::{c_char, c_int, CStr};
use vleo_bus::{Case, RunMode};
use vleo_modules::{Scratch, Vleo, NODES, VARS};

/// Success.
pub const VLEO_OK: c_int = 0;
/// The node identifier does not name a row in the tree.
pub const VLEO_UNKNOWN_NODE: c_int = 1;
/// A supplied value fell outside a declared limit. Refused, never clamped.
pub const VLEO_OUT_OF_DOMAIN: c_int = 2;
/// A node the run needed has never run, or refused.
pub const VLEO_BLOCKED: c_int = 3;
/// The caller passed a null pointer or a buffer that is too small.
pub const VLEO_BAD_ARGUMENT: c_int = 4;
/// A declared reference-data bundle is absent or does not verify.
pub const VLEO_DATA: c_int = 5;

thread_local! {
    /// The last message, per thread. No global state: a sweep is a parallel map
    /// with no mutex, because there is nothing shared to protect.
    static LAST: RefCell<String> = const { RefCell::new(String::new()) };
    static SCRATCH: RefCell<Option<Scratch>> = const { RefCell::new(None) };
}

fn set_message(s: String) {
    LAST.with(|m| *m.borrow_mut() = s);
}

/// One override, in SI.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VleoSet {
    /// A NUL-terminated node identifier.
    pub id: *const c_char,
    /// The value, in the SI unit of the node's type. A face converts for
    /// display and never for transport.
    pub value: f64,
}

/// What a run asks for.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VleoCase {
    pub node: *const c_char,
    pub base_case: *const c_char,
    /// 0 alone, 1 branch, 2 everything.
    pub mode: c_int,
    pub sets: *const VleoSet,
    pub set_count: c_int,
}

/// What comes back.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VleoResult {
    /// The answer, in SI.
    pub value: f64,
    /// The lowest of the eight credibility factors, `0..=4`.
    pub credibility: c_int,
    pub ran: c_int,
    pub blocked: c_int,
    pub iterations: c_int,
    /// The chain hash: the run's identity and the cache key.
    pub chain: u64,
}

/// How many rows the tree holds.
#[no_mangle]
pub extern "C" fn vleo_node_count() -> c_int {
    NODES.len() as c_int
}

/// Copy the identifier of row `i` into a caller-owned buffer.
///
/// # Safety
/// `buf` must point to at least `len` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn vleo_node_id(i: c_int, buf: *mut c_char, len: c_int) -> c_int {
    if buf.is_null() || len <= 0 || i < 0 || i as usize >= NODES.len() {
        return VLEO_BAD_ARGUMENT;
    }
    write_cstr(NODES[i as usize].id, buf, len)
}

/// The unit symbol row `i` publishes in.
///
/// # Safety
/// `buf` must point to at least `len` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn vleo_node_unit(i: c_int, buf: *mut c_char, len: c_int) -> c_int {
    if buf.is_null() || len <= 0 || i < 0 || i as usize >= NODES.len() {
        return VLEO_BAD_ARGUMENT;
    }
    write_cstr(VARS[i as usize].unit.symbol(), buf, len)
}

/// The last message, for the thread that produced it.
///
/// # Safety
/// `buf` must point to at least `len` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn vleo_last_message(buf: *mut c_char, len: c_int) -> c_int {
    if buf.is_null() || len <= 0 {
        return VLEO_BAD_ARGUMENT;
    }
    let s = LAST.with(|m| m.borrow().clone());
    write_cstr(&s, buf, len)
}

/// Identify the engine build and the graph it was built against.
#[no_mangle]
pub extern "C" fn vleo_kernel_hash() -> u64 {
    Vleo::kernel_hash()
}

/// Identify the graph. Changes when an edge does.
#[no_mangle]
pub extern "C" fn vleo_graph_hash() -> u64 {
    Vleo::graph_hash()
}

/// Evaluate one node.
///
/// # Safety
/// Every pointer in `case` must be valid for the duration of the call, and
/// `out` must point to a writable [`VleoResult`].
#[no_mangle]
pub unsafe extern "C" fn vleo_evaluate(case: *const VleoCase, out: *mut VleoResult) -> c_int {
    if case.is_null() || out.is_null() {
        set_message("a null pointer was passed where a case and a result were expected".into());
        return VLEO_BAD_ARGUMENT;
    }
    let c = &*case;
    let node = match cstr(c.node) {
        Some(s) => s,
        None => {
            set_message("the node identifier is null or not valid UTF-8".into());
            return VLEO_BAD_ARGUMENT;
        }
    };
    if Vleo::find(&node).is_none() {
        set_message(format!("no node '{node}'"));
        return VLEO_UNKNOWN_NODE;
    }
    let base = cstr(c.base_case).unwrap_or_else(|| "nominal".into());
    let mut supply = Vec::new();
    if !c.sets.is_null() && c.set_count > 0 {
        let sets = std::slice::from_raw_parts(c.sets, c.set_count as usize);
        for s in sets {
            match cstr(s.id) {
                Some(id) => supply.push((id, s.value)),
                None => {
                    set_message("an override carries a null identifier".into());
                    return VLEO_BAD_ARGUMENT;
                }
            }
        }
    }
    let mode = match c.mode {
        0 => RunMode::Alone,
        2 => RunMode::All,
        _ => RunMode::Branch,
    };
    let case = Case {
        base,
        supply,
        target: node.clone(),
        mode,
        // A C caller resolves its own store. The engine opens nothing.
        data: Vec::new(),
        data_versions: Vec::new(),
    };
    SCRATCH.with(|sc| {
        let mut sc = sc.borrow_mut();
        if sc.is_none() {
            *sc = Some(Scratch::new());
        }
        let scratch = sc.as_mut().expect("scratch");
        match vleo_modules::evaluate(&case, scratch) {
            Err(f) => {
                set_message(format!("{f}"));
                match f.kind() {
                    "out-of-domain" => VLEO_OUT_OF_DOMAIN,
                    "data-missing" | "data-unverified" => VLEO_DATA,
                    _ => VLEO_BLOCKED,
                }
            }
            Ok(r) => {
                let v = r.values.iter().find(|v| v.id == node);
                match v {
                    None => {
                        set_message(
                            r.blocked
                                .first()
                                .map(|b| b.message.clone())
                                .unwrap_or_else(|| "the node did not produce a value".into()),
                        );
                        VLEO_BLOCKED
                    }
                    Some(v) => {
                        *out = VleoResult {
                            value: v.value,
                            credibility: v.cred.governing_score() as c_int,
                            ran: r.manifest.ran as c_int,
                            blocked: r.manifest.blocked_count as c_int,
                            iterations: r.manifest.iterations as c_int,
                            chain: 0,
                        };
                        set_message(format!(
                            "{} = {} {} — credibility {} of 4, governed by {}",
                            v.symbol,
                            v.value,
                            v.unit,
                            v.cred.governing_score(),
                            v.governing
                        ));
                        VLEO_OK
                    }
                }
            }
        }
    })
}

unsafe fn cstr(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok().map(|s| s.to_string())
}

/// Copy a Rust string into a caller-owned buffer, always NUL-terminated and
/// never overrunning. Truncation is reported rather than hidden.
unsafe fn write_cstr(s: &str, buf: *mut c_char, len: c_int) -> c_int {
    let cap = len as usize;
    let bytes = s.as_bytes();
    let n = bytes.len().min(cap.saturating_sub(1));
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, n);
    *buf.add(n) = 0;
    if n < bytes.len() {
        VLEO_BAD_ARGUMENT
    } else {
        VLEO_OK
    }
}
