//! The engine, for a page read without the engine.
//!
//! The readers' docs folder (`xtask readers`) is a set of pages a team opens
//! from a shared drive or an internal web server, with no tool running. A
//! lesson's try-it widgets there are answered by this: the whole kernel,
//! compiled to WebAssembly — the same relations, the same refusals, the same
//! sweep and the same figure description as the local engine. A widget still
//! computes nothing itself.
//!
//! Unlike `vleo-wasm` (the demonstration subset a public URL carries), this
//! holds every row: the folder goes to the team, who already hold the engine
//! in the kit. Reference data does not travel here, so a row that declares a
//! bundle refuses by name, as it would in an engine with none installed.
//!
//! The protocol is the form checker's: the page asks for `len` bytes with
//! `vleo_alloc`, writes a request as UTF-8 plain text, calls `vleo_run` or
//! `vleo_sweep`, and reads `vleo_out_len` bytes of JSON at the returned
//! pointer. The answer stays valid until the next call.
//!
//! A request is lines: `node <id>`; for a sweep `over <id>`, `from <si>`,
//! `to <si>`, `points <n>`; and any number of `set <id> <si>`.

// Raw pointers across the WebAssembly boundary are the whole of this crate's
// job, and there is no way to take a buffer from JavaScript without them.
#![allow(unsafe_code)]

use std::cell::RefCell;
use vleo_bus::{Case, RunMode};
use vleo_modules::{Scratch, Vleo, VARS};

thread_local! {
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Room for `len` bytes of input, owned by the caller until the next call.
#[no_mangle]
pub extern "C" fn vleo_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Run a row on the declared defaults with the request's values supplied, and
/// answer as `/v1/run` answers: the values it computed and the rows it could
/// not, each with why.
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_run(ptr: *mut u8, len: usize) -> *const u8 {
    let text = take(ptr, len);
    put(run(&Request::read(&text)))
}

/// A sweep, answered as the engine describes it: `{"ok":true,"figure":…}`.
///
/// # Safety
/// As [`vleo_run`].
#[no_mangle]
pub unsafe extern "C" fn vleo_sweep(ptr: *mut u8, len: usize) -> *const u8 {
    let text = take(ptr, len);
    put(sweep(&Request::read(&text)))
}

/// The length of the last answer.
#[no_mangle]
pub extern "C" fn vleo_out_len() -> usize {
    OUT.with(|o| o.borrow().len())
}

/// Which engine this is, as JSON — so a page can say which it carries.
#[no_mangle]
pub extern "C" fn vleo_identity() -> *const u8 {
    put(format!(
        "{{\"kernel\":\"{}\",\"graph\":\"{}\",\"rows\":{}}}",
        hex(Vleo::kernel_hash()),
        hex(Vleo::graph_hash()),
        VARS.len()
    ))
}

unsafe fn take(ptr: *mut u8, len: usize) -> String {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    String::from_utf8_lossy(&bytes).into_owned()
}

fn put(json: String) -> *const u8 {
    OUT.with(|o| {
        let mut o = o.borrow_mut();
        *o = json.into_bytes();
        o.as_ptr()
    })
}

#[derive(Default)]
struct Request {
    node: String,
    over: String,
    from: f64,
    to: f64,
    points: usize,
    sets: Vec<(String, f64)>,
}

impl Request {
    fn read(text: &str) -> Request {
        let mut r = Request {
            points: 25,
            ..Default::default()
        };
        for l in text.lines() {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.as_slice() {
                ["node", id] => r.node = id.to_string(),
                ["over", id] => r.over = id.to_string(),
                ["from", v] => r.from = v.parse().unwrap_or(f64::NAN),
                ["to", v] => r.to = v.parse().unwrap_or(f64::NAN),
                ["points", n] => r.points = n.parse().unwrap_or(0),
                ["set", id, v] => r.sets.push((id.to_string(), v.parse().unwrap_or(f64::NAN))),
                _ => {}
            }
        }
        r
    }

    /// The case: the declared defaults, with the request's values supplied —
    /// each one checked as the engine checks a supplied value.
    fn case(&self) -> Result<Case, String> {
        for (id, v) in &self.sets {
            if let Some(why) = vleo_modules::why_not_suppliable(id) {
                return Err(why);
            }
            if !v.is_finite() {
                return Err(format!("'{id}' was given a value that is not a number"));
            }
        }
        Ok(Case {
            target: self.node.clone(),
            mode: RunMode::Branch,
            supply: self.sets.clone(),
            ..Default::default()
        })
    }
}

fn run(q: &Request) -> String {
    let case = match q.case() {
        Ok(c) => c,
        Err(e) => return failed(&e),
    };
    if Vleo::find(&q.node).is_none() {
        return failed(&format!("there is no row called '{}'", q.node));
    }
    let mut scratch = Scratch::new();
    let r = match vleo_modules::evaluate(&case, &mut scratch) {
        Ok(r) => r,
        Err(f) => return failed(&format!("{f}")),
    };
    let values: Vec<String> = r
        .values
        .iter()
        .map(|v| {
            let i = Vleo::find(&v.id).unwrap_or(0) as usize;
            let (shown, unit) = vleo_bus::present(v.value, VARS[i].unit, 6);
            format!(
                "{{\"id\":{},\"label\":{},\"si\":{},\"shown\":{},\"unit\":{}}}",
                text(&v.id),
                text(VARS[i].label),
                num(v.value),
                text(&shown),
                text(unit)
            )
        })
        .collect();
    let blocked: Vec<String> = r
        .blocked
        .iter()
        .map(|b| {
            format!(
                "{{\"id\":{},\"kind\":{},\"message\":{}}}",
                text(&b.id),
                text(b.kind),
                text(&b.message)
            )
        })
        .collect();
    format!(
        "{{\"ok\":true,\"endpoint\":\"browser-kernel\",\"values\":[{}],\"blocked\":[{}]}}",
        values.join(","),
        blocked.join(",")
    )
}

fn sweep(q: &Request) -> String {
    let case = match q.case() {
        Ok(c) => c,
        Err(e) => return failed(&e),
    };
    if !(q.from.is_finite() && q.to.is_finite()) || q.points < 2 || q.points > 400 {
        return failed("a sweep needs a range and between 2 and 400 points");
    }
    match vleo_modules::results::sweep(&case, &q.node, &q.over, q.from, q.to, q.points) {
        Ok(w) => format!(
            "{{\"ok\":true,\"figure\":{}}}",
            vleo_modules::figure::json(&vleo_modules::figure::from_sweep(&q.node, &w, None))
        ),
        Err(e) => failed(e.message()),
    }
}

fn failed(why: &str) -> String {
    format!("{{\"ok\":false,\"message\":{}}}", text(why))
}

fn num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:?}")
    } else {
        "null".into()
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

fn hex(h: u64) -> String {
    format!("{h:016x}")[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_reads_as_the_page_writes_it() {
        let q =
            Request::read("node a\nover b\nfrom 1\nto 2.5\npoints 7\nset c 3\nset d 1e-3\nnoise\n");
        assert_eq!((q.node.as_str(), q.over.as_str()), ("a", "b"));
        assert_eq!((q.from, q.to, q.points), (1.0, 2.5, 7));
        assert_eq!(q.sets, vec![("c".into(), 3.0), ("d".into(), 1e-3)]);
        assert_eq!(Request::read("node a\n").points, 25);
    }

    #[test]
    fn what_the_engine_would_refuse_is_refused_here() {
        let no_row = run(&Request::read("node no_such_row\n"));
        assert!(
            no_row.starts_with("{\"ok\":false") && no_row.contains("no_such_row"),
            "{no_row}"
        );
        let supplied = run(&Request::read(
            "node sw_ap_design_long\nset sw_ap_design_long 3\n",
        ));
        assert!(supplied.starts_with("{\"ok\":false"), "{supplied}");
        let nan = run(&Request::read(
            "node sw_ap_design_long\nset sw_ap_central_expectation x\n",
        ));
        assert!(nan.contains("not a number"), "{nan}");
        let short = sweep(&Request::read(
            "node sw_ap_design_long\nover sw_ap_central_expectation\nfrom 1\nto 2\npoints 1\n",
        ));
        assert!(short.contains("between 2 and 400"), "{short}");
    }

    /// THE PAGE'S NUMBERS ARE THE TOOL'S. `tools/readers_check.py --answers`
    /// asks the engine the readers' folder carries — compiled to WebAssembly,
    /// loaded in Chromium from a file — every question it asks, and writes each
    /// answer down. The same engine built here natively is asked the same
    /// questions, and every answer must be the same text: the same values to
    /// the last bit (they print as the shortest text that reads back as the
    /// same f64), the same refusals, the same sweep. The kernel's maths is
    /// portable (`vleo_core::units::pmath`) precisely so this holds.
    #[test]
    #[ignore = "needs VLEO_BROWSER_ANSWERS, written by tools/readers_check.py --answers"]
    fn the_page_answers_every_question_as_the_tool_does() {
        let path = std::env::var("VLEO_BROWSER_ANSWERS")
            .expect("VLEO_BROWSER_ANSWERS: the file tools/readers_check.py --answers wrote");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut asked = 0;
        let mut differ = Vec::new();
        for line in text.lines() {
            let (q, page) = line
                .split_once('\t')
                .expect("a line is <question>\\t<answer>");
            let here = match q {
                "@supplied" => run(&Request::read(
                    "node sw_ap_design_long\nset sw_ap_design_long 3\n",
                )),
                "@sweep" => sweep(&Request::read(
                    "node sw_ap_design_long\nover sw_ap_central_expectation\nfrom 5\nto 35\npoints 7\n",
                )),
                id => run(&Request::read(&format!("node {id}\n"))),
            };
            asked += 1;
            if here != page {
                differ.push(format!("{q}\n  page: {page}\n  tool: {here}"));
            }
        }
        assert!(asked > 100, "only {asked} answers in {path}");
        assert!(
            differ.is_empty(),
            "{} of {asked} differ:\n{}",
            differ.len(),
            differ.join("\n")
        );
    }
}
