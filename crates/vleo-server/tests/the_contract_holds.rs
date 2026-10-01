//! The contract holds: every route answers as contract/ says it does.
//!
//! The frontend and the backend meet at one written contract (contract/README.md):
//! each /v1 route, what kind of answer it gives, and for a JSON answer a schema
//! of every field. This starts the real server, makes every request
//! contract/routes.toml lists — in its order, on a scratch case and results
//! folder — and checks each answer:
//!
//! * a JSON answer validates against its schema, and carries NO FIELD THE
//!   SCHEMA DOES NOT DECLARE. So a field added to the engine without being added
//!   to the contract fails here, and the contract is what both sides read.
//! * the recorded example — what the mock engine serves the frontend — also
//!   validates, so the frontend never develops against an answer the engine no
//!   longer gives.
//! * a CSV answer that is one of the file formats has its marker line, its
//!   meta lines and its columns (contract/formats/).
//! * every route the manual documents is in the contract, and nothing else;
//!   every schema, example and format on disk is used; the contract version is
//!   the same in contract/VERSION, the engine's /v1/version and the page.
//!
//! `VLEO_CONTRACT_RECORD=1 cargo test -p vleo-server --test the_contract_holds`
//! re-records the examples from the real engine, and drafts a schema for a
//! route that has none. It never rewrites a schema: changing one is a change to
//! the contract, made by hand and reviewed by both sides.
//!
//! JSON is read with the small parser below rather than a new dependency: the
//! tests need to read it, nothing else in the tool does.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// JSON: the value, a parser and a writer

#[derive(Clone, Debug, PartialEq)]
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    /// Keys in the order written.
    Obj(Vec<(String, J)>),
}

impl J {
    fn kind(&self) -> &'static str {
        match self {
            J::Null => "null",
            J::Bool(_) => "boolean",
            J::Num(n) if n.fract() == 0.0 && n.is_finite() => "integer",
            J::Num(_) => "number",
            J::Str(_) => "string",
            J::Arr(_) => "array",
            J::Obj(_) => "object",
        }
    }
    fn get(&self, k: &str) -> Option<&J> {
        match self {
            J::Obj(m) => m.iter().find(|(key, _)| key == k).map(|(_, v)| v),
            _ => None,
        }
    }
    fn str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }
}

struct P<'a> {
    s: &'a [u8],
    i: usize,
}

fn parse(text: &str) -> Result<J, String> {
    let mut p = P {
        s: text.as_bytes(),
        i: 0,
    };
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("trailing text at byte {}", p.i));
    }
    Ok(v)
}

impl P<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\n' | b'\r' | b'\t') {
            self.i += 1;
        }
    }
    fn eat(&mut self, b: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&b) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", b as char, self.i))
        }
    }
    fn value(&mut self) -> Result<J, String> {
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut m = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(J::Obj(m));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.eat(b':')?;
                    let v = self.value()?;
                    m.push((k, v));
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(J::Obj(m));
                        }
                        _ => return Err(format!("expected ',' or '}}' at byte {}", self.i)),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut a = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(J::Arr(a));
                }
                loop {
                    a.push(self.value()?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(J::Arr(a));
                        }
                        _ => return Err(format!("expected ',' or ']' at byte {}", self.i)),
                    }
                }
            }
            Some(b'"') => Ok(J::Str(self.string()?)),
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(J::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(J::Bool(false))
            }
            Some(b'n') if self.s[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(J::Null)
            }
            Some(_) => {
                let start = self.i;
                while self.i < self.s.len()
                    && matches!(
                        self.s[self.i],
                        b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
                    )
                {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
                t.parse::<f64>()
                    .map(J::Num)
                    .map_err(|_| format!("not a value at byte {start}"))
            }
            None => Err("the text ended where a value was expected".into()),
        }
    }
    fn string(&mut self) -> Result<String, String> {
        if self.s.get(self.i) != Some(&b'"') {
            return Err(format!("expected a string at byte {}", self.i));
        }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let Some(&c) = self.s.get(self.i) else {
                return Err("a string was not closed".into());
            };
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = *self.s.get(self.i).ok_or("a string ended in an escape")?;
                    self.i += 1;
                    let ch = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut code = self.hex4()?;
                            if (0xD800..0xDC00).contains(&code)
                                && self.s[self.i..].starts_with(b"\\u")
                            {
                                self.i += 2;
                                let low = self.hex4()?;
                                code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                            }
                            char::from_u32(code).unwrap_or('\u{fffd}')
                        }
                        _ => return Err(format!("unknown escape at byte {}", self.i)),
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
                _ => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|e| e.to_string())
    }
    fn hex4(&mut self) -> Result<u32, String> {
        let h = self
            .s
            .get(self.i..self.i + 4)
            .ok_or("a \\u escape was cut short")?;
        self.i += 4;
        u32::from_str_radix(std::str::from_utf8(h).unwrap_or(""), 16)
            .map_err(|_| "a \\u escape is not hex".to_string())
    }
}

fn write(v: &J, pretty: bool) -> String {
    let mut o = String::new();
    write_into(v, pretty, 0, &mut o);
    if pretty {
        o.push('\n');
    }
    o
}

fn write_into(v: &J, pretty: bool, depth: usize, o: &mut String) {
    let nl = |o: &mut String, d: usize| {
        if pretty {
            o.push('\n');
            o.push_str(&"  ".repeat(d));
        }
    };
    match v {
        J::Null => o.push_str("null"),
        J::Bool(b) => o.push_str(if *b { "true" } else { "false" }),
        J::Num(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                o.push_str(&format!("{}", *n as i64));
            } else {
                o.push_str(&format!("{n}"));
            }
        }
        J::Str(s) => {
            o.push('"');
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
        }
        J::Arr(a) => {
            o.push('[');
            for (k, x) in a.iter().enumerate() {
                if k > 0 {
                    o.push(',');
                }
                nl(o, depth + 1);
                write_into(x, pretty, depth + 1, o);
            }
            if !a.is_empty() {
                nl(o, depth);
            }
            o.push(']');
        }
        J::Obj(m) => {
            o.push('{');
            for (k, (key, x)) in m.iter().enumerate() {
                if k > 0 {
                    o.push(',');
                }
                nl(o, depth + 1);
                write_into(&J::Str(key.clone()), pretty, depth + 1, o);
                o.push(':');
                if pretty {
                    o.push(' ');
                }
                write_into(x, pretty, depth + 1, o);
            }
            if !m.is_empty() {
                nl(o, depth);
            }
            o.push('}');
        }
    }
}

// ---------------------------------------------------------------------------
// the schemas: the subset of JSON Schema the contract uses

/// The keywords a contract schema may use. Anything else is refused rather
/// than ignored: a keyword this validator does not understand would be a rule
/// that looks enforced and is not.
const KEYWORDS: &[&str] = &[
    "$schema",
    "$comment",
    "title",
    "description",
    "type",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "anyOf",
    "enum",
];

fn validate(v: &J, s: &J, at: &str, errs: &mut Vec<String>) {
    let J::Obj(keys) = s else {
        errs.push(format!("{at}: the schema here is not an object"));
        return;
    };
    for (k, _) in keys {
        if !KEYWORDS.contains(&k.as_str()) {
            errs.push(format!(
                "{at}: the schema uses `{k}`, which the contract does not"
            ));
        }
    }
    if let Some(J::Arr(any)) = s.get("anyOf") {
        let ok = any.iter().any(|alt| {
            let mut e = Vec::new();
            validate(v, alt, at, &mut e);
            e.is_empty()
        });
        if !ok {
            errs.push(format!(
                "{at}: matches none of the {} shapes allowed",
                any.len()
            ));
        }
        return;
    }
    if let Some(t) = s.get("type") {
        let allowed: Vec<&str> = match t {
            J::Str(one) => vec![one.as_str()],
            J::Arr(many) => many.iter().filter_map(J::str).collect(),
            _ => vec![],
        };
        let kind = v.kind();
        let fits = allowed
            .iter()
            .any(|a| *a == kind || (*a == "number" && kind == "integer"));
        if !fits {
            errs.push(format!(
                "{at}: is {kind}, and the contract says {}",
                allowed.join(" or ")
            ));
            return;
        }
    }
    if let Some(J::Arr(vals)) = s.get("enum") {
        if !vals.contains(v) {
            errs.push(format!(
                "{at}: {} is not one of the values the contract allows",
                write(v, false)
            ));
        }
    }
    match v {
        J::Obj(m) => {
            let props = s.get("properties");
            if let Some(J::Arr(req)) = s.get("required") {
                for r in req.iter().filter_map(J::str) {
                    if v.get(r).is_none() {
                        errs.push(format!(
                            "{at}: `{r}` is missing, and the contract requires it"
                        ));
                    }
                }
            }
            for (k, x) in m {
                let here = format!("{at}.{k}");
                match props.and_then(|p| p.get(k)) {
                    Some(ps) => validate(x, ps, &here, errs),
                    None => match s.get("additionalProperties") {
                        Some(J::Bool(true)) => {}
                        Some(extra @ J::Obj(_)) => validate(x, extra, &here, errs),
                        _ => errs.push(format!(
                            "{here}: the engine sends this field and the contract does not declare it"
                        )),
                    },
                }
            }
        }
        J::Arr(a) => {
            if let Some(items) = s.get("items") {
                for (i, x) in a.iter().enumerate() {
                    validate(x, items, &format!("{at}[{i}]"), errs);
                }
            }
        }
        _ => {}
    }
}

/// A first draft of a schema from recorded answers — only for a route that
/// has none. Every field seen is declared; a field is required when every
/// answer has it; an object with many keys whose values all share a shape is
/// a map (node ids, say) rather than a record.
fn draft(samples: &[&J]) -> J {
    let mut kinds: Vec<&'static str> = Vec::new();
    for s in samples {
        let k = match s.kind() {
            "integer" => "number",
            k => k,
        };
        if !kinds.contains(&k) {
            kinds.push(k);
        }
    }
    let mut out: Vec<(String, J)> = Vec::new();
    out.push((
        "type".into(),
        if kinds.len() == 1 {
            J::Str(kinds[0].into())
        } else {
            J::Arr(kinds.iter().map(|k| J::Str((*k).into())).collect())
        },
    ));
    let objs: Vec<&Vec<(String, J)>> = samples
        .iter()
        .filter_map(|s| match s {
            J::Obj(m) => Some(m),
            _ => None,
        })
        .collect();
    if !objs.is_empty() {
        let is_map = objs
            .iter()
            .all(|m| m.len() >= 20 && m.iter().all(|(_, v)| matches!(v, J::Obj(_))));
        if is_map {
            let vals: Vec<&J> = objs.iter().flat_map(|m| m.iter().map(|(_, v)| v)).collect();
            out.push(("additionalProperties".into(), draft(&vals)));
        } else {
            let mut order: Vec<String> = Vec::new();
            let mut seen: BTreeMap<String, Vec<&J>> = BTreeMap::new();
            for m in &objs {
                for (k, v) in m.iter() {
                    if !seen.contains_key(k) {
                        order.push(k.clone());
                    }
                    seen.entry(k.clone()).or_default().push(v);
                }
            }
            let props: Vec<(String, J)> =
                order.iter().map(|k| (k.clone(), draft(&seen[k]))).collect();
            let req: Vec<J> = order
                .iter()
                .filter(|k| objs.iter().all(|m| m.iter().any(|(key, _)| key == *k)))
                .map(|k| J::Str(k.clone()))
                .collect();
            out.push(("properties".into(), J::Obj(props)));
            if !req.is_empty() {
                out.push(("required".into(), J::Arr(req)));
            }
        }
    }
    let items: Vec<&J> = samples
        .iter()
        .filter_map(|s| match s {
            J::Arr(a) => Some(a.iter()),
            _ => None,
        })
        .flatten()
        .collect();
    if !items.is_empty() {
        out.push(("items".into(), draft(&items)));
    }
    J::Obj(out)
}

// ---------------------------------------------------------------------------
// a client for the real server

struct Answer {
    status: u16,
    ctype: String,
    body: String,
}

fn enc(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                o.push(b as char)
            }
            _ => o.push_str(&format!("%{b:02X}")),
        }
    }
    o
}

fn request(port: u16, method: &str, path: &str, params: &[(String, String)]) -> Answer {
    let q: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", enc(k), enc(v)))
        .collect::<Vec<_>>()
        .join("&");
    let (target, body) = match method {
        "GET" if !q.is_empty() => (
            format!("{path}{}{q}", if path.contains('?') { "&" } else { "?" }),
            String::new(),
        ),
        "GET" => (path.to_string(), String::new()),
        _ => (path.to_string(), q),
    };
    let mut s = TcpStream::connect(("127.0.0.1", port)).expect("the server did not accept");
    write!(
        s,
        "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\
         Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let ctype = head
        .lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.trim()
                .eq_ignore_ascii_case("content-type")
                .then(|| v.trim().to_string())
        })
        .unwrap_or_default();
    Answer {
        status,
        ctype,
        body: body.to_string(),
    }
}

// ---------------------------------------------------------------------------
// the contract as written

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Example {
    name: String,
    path: String,
    params: Vec<(String, String)>,
}

struct Route {
    method: String,
    path: String,
    answer: String,
    schema: Option<String>,
    format: Option<String>,
    contains: Option<String>,
    examples: Vec<Example>,
}

fn routes() -> Vec<Route> {
    let text = std::fs::read_to_string(root().join("contract/routes.toml")).unwrap();
    let v: toml::Value = text.parse().expect("contract/routes.toml does not parse");
    let s = |t: &toml::Value, k: &str| t.get(k).and_then(|x| x.as_str()).map(str::to_string);
    v.get("route")
        .and_then(|r| r.as_array())
        .expect("contract/routes.toml has no [[route]]")
        .iter()
        .map(|r| {
            let path = s(r, "path").expect("a route with no path");
            Route {
                method: s(r, "method").expect("a route with no method"),
                answer: s(r, "answer").expect("a route with no answer kind"),
                schema: s(r, "schema"),
                format: s(r, "format"),
                contains: s(r, "contains"),
                examples: r
                    .get("example")
                    .and_then(|e| e.as_array())
                    .map(|es| {
                        es.iter()
                            .map(|e| Example {
                                name: s(e, "name").expect("an example with no name"),
                                path: s(e, "path").unwrap_or_else(|| path.clone()),
                                params: e
                                    .get("params")
                                    .and_then(|p| p.as_array())
                                    .map(|ps| {
                                        ps.iter()
                                            .map(|kv| {
                                                let kv =
                                                    kv.as_array().expect("a param is [key, value]");
                                                (
                                                    kv[0].as_str().unwrap().to_string(),
                                                    kv[1].as_str().unwrap().to_string(),
                                                )
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                path,
            }
        })
        .collect()
}

/// The routes the manual documents: `method path` for each [[route]].
fn documented() -> Vec<String> {
    let text = std::fs::read_to_string(root().join("docs/manual.toml")).unwrap();
    let v: toml::Value = text.parse().unwrap();
    v.get("route")
        .and_then(|r| r.as_array())
        .unwrap()
        .iter()
        .map(|r| {
            format!(
                "{} {}",
                r.get("method").and_then(|m| m.as_str()).unwrap(),
                r.get("path").and_then(|m| m.as_str()).unwrap()
            )
        })
        .collect()
}

fn files(dir: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root().join(dir))
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// A CSV answer against its file format: the marker line, every meta line,
/// every column.
fn check_format(name: &str, body: &str) -> Vec<String> {
    let path = root().join(format!("contract/formats/{name}.toml"));
    let f: toml::Value = match std::fs::read_to_string(&path).map(|t| t.parse()) {
        Ok(Ok(v)) => v,
        _ => return vec![format!("contract/formats/{name}.toml does not read")],
    };
    let list = |k: &str| -> Vec<String> {
        f.get(k)
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut errs = Vec::new();
    let meta: Vec<(&str, &str)> = body
        .lines()
        .filter_map(|l| l.trim().strip_prefix("#!"))
        .map(|m| m.trim().split_once(' ').unwrap_or((m.trim(), "")))
        .collect();
    let marker = f.get("marker").and_then(|m| m.as_str()).unwrap_or("");
    let fmt = f.get("format").and_then(|m| m.as_str()).unwrap_or("");
    if marker != "template" && !meta.iter().any(|(k, v)| *k == marker && *v == fmt) {
        errs.push(format!("{name}: no `#! {marker} {fmt}` line"));
    }
    for k in list("meta") {
        if !meta.iter().any(|(m, _)| *m == k) {
            errs.push(format!("{name}: no `#! {k}` line"));
        }
    }
    let header: Vec<String> = body
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("")
        .split(',')
        .map(|c| c.trim().to_string())
        .collect();
    for c in list("columns") {
        if !header.contains(&c) {
            errs.push(format!("{name}: no `{c}` column"));
        }
    }
    errs
}

fn schema(name: &str) -> Option<J> {
    let text =
        std::fs::read_to_string(root().join(format!("contract/schemas/{name}.json"))).ok()?;
    Some(parse(&text).unwrap_or_else(|e| panic!("contract/schemas/{name}.json: {e}")))
}

// ---------------------------------------------------------------------------

#[test]
fn every_route_answers_as_the_contract_says() {
    let record = std::env::var_os("VLEO_CONTRACT_RECORD").is_some_and(|v| !v.is_empty());
    let scratch = std::env::temp_dir().join(format!("vleo-contract-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    // A case, results and log of the test's own: the requests save, upload and
    // delete, and a person's own must never be what they touch.
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let port =
        vleo_server::serve(Some(root()), 18731, false, true).expect("the server did not start");

    let routes = routes();
    let mut errs: Vec<String> = Vec::new();
    let mut saved = String::new();
    let mut by_schema: BTreeMap<String, Vec<J>> = BTreeMap::new();
    let mut used_examples: Vec<String> = Vec::new();

    // Every route the manual documents is here, and nothing else.
    let here: Vec<String> = routes
        .iter()
        .map(|r| format!("{} {}", r.method, r.path))
        .collect();
    for d in documented() {
        if !here.contains(&d) {
            errs.push(format!(
                "{d}: the manual documents it and the contract does not"
            ));
        }
    }
    for h in &here {
        if !documented().contains(h) {
            errs.push(format!("{h}: in the contract and not in the manual"));
        }
    }

    for r in &routes {
        if r.answer == "json" && r.schema.is_none() {
            errs.push(format!(
                "{} {}: a JSON answer with no schema",
                r.method, r.path
            ));
        }
        let examples: Vec<Example> = if r.examples.is_empty() && !r.path.contains('<') {
            vec![Example {
                name: String::new(),
                path: r.path.clone(),
                params: Vec::new(),
            }]
        } else {
            r.examples
                .iter()
                .map(|e| Example {
                    name: e.name.clone(),
                    path: e.path.clone(),
                    params: e.params.clone(),
                })
                .collect()
        };
        if examples.is_empty() {
            errs.push(format!("{} {}: no example to request", r.method, r.path));
        }
        for ex in examples {
            let label = format!("{} {} ({})", r.method, ex.path, ex.name);
            let params: Vec<(String, String)> = ex
                .params
                .iter()
                .map(|(k, v)| {
                    let v = v.replace("@saved", &saved);
                    let v = match v.strip_prefix("@get:") {
                        Some(p) => request(port, "GET", p, &[]).body,
                        None => v,
                    };
                    (k.clone(), v)
                })
                .collect();
            let a = request(port, &r.method, &ex.path.replace("@saved", &saved), &params);
            // A FIGURE OF THE RECORD IS THE SAME FROM EVERY FACE. The command
            // line and Python ask `vleo_server::figure`, not the route; the
            // two must answer byte for byte, or a script reproducing a panel
            // is reproducing something else.
            if let Some(id) = ex.path.strip_prefix("/v1/figures/solar/") {
                let q: Vec<String> = params
                    .iter()
                    .map(|(k, v)| format!("{}={}", enc(k), enc(v)))
                    .collect();
                let direct = vleo_server::figure(Some(root()), id, &q.join("&"));
                if direct != a.body {
                    errs.push(format!(
                        "{label}: vleo_server::figure answers differently from the route"
                    ));
                }
            }
            if a.status != 200 {
                errs.push(format!(
                    "{label}: answered {} — {}",
                    a.status,
                    a.body.chars().take(200).collect::<String>()
                ));
                continue;
            }
            let want = match r.answer.as_str() {
                "json" => "application/json",
                "html" => "text/html",
                "csv" => "text/csv",
                "css" => "text/css",
                "js" => "javascript",
                "font" => "font/woff2",
                other => {
                    errs.push(format!("{label}: `{other}` is not an answer kind"));
                    continue;
                }
            };
            if !a.ctype.contains(want) {
                errs.push(format!(
                    "{label}: served as {}, and the contract says {want}",
                    a.ctype
                ));
            }
            if let Some(c) = &r.contains {
                if !a.body.contains(c.as_str()) {
                    errs.push(format!("{label}: does not carry `{c}`"));
                }
            }
            if let Some(f) = &r.format {
                errs.extend(
                    check_format(f, &a.body)
                        .into_iter()
                        .map(|e| format!("{label}: {e}")),
                );
            }
            if r.answer != "json" {
                continue;
            }
            let v = match parse(&a.body) {
                Ok(v) => v,
                Err(e) => {
                    errs.push(format!("{label}: not JSON — {e}"));
                    continue;
                }
            };
            if let Some(f) = v.get("file").and_then(J::str) {
                if v.get("ok") == Some(&J::Bool(true)) {
                    saved = f.to_string();
                }
            }
            let schema_name = r.schema.clone().unwrap_or_default();
            by_schema
                .entry(schema_name.clone())
                .or_default()
                .push(v.clone());
            let file = format!("{}.json", ex.name);
            used_examples.push(file.clone());
            let example_path = root().join("contract/examples").join(&file);
            if record {
                let text = write(&v, a.body.len() < 64 * 1024);
                std::fs::write(&example_path, text).unwrap();
            }
            match schema(&schema_name) {
                Some(s) => {
                    let mut e = Vec::new();
                    validate(&v, &s, "$", &mut e);
                    errs.extend(e.into_iter().map(|x| format!("{label}: {x}")));
                    // What the mock engine serves must be what the engine
                    // still gives.
                    match std::fs::read_to_string(&example_path) {
                        Ok(t) => match parse(&t) {
                            Ok(ev) => {
                                let mut e = Vec::new();
                                validate(&ev, &s, "$", &mut e);
                                errs.extend(e.into_iter().map(|x| {
                                    format!("contract/examples/{file}: {x} — re-record the examples")
                                }));
                            }
                            Err(e) => errs.push(format!("contract/examples/{file}: {e}")),
                        },
                        Err(_) => errs.push(format!(
                            "contract/examples/{file}: missing — record it with VLEO_CONTRACT_RECORD=1"
                        )),
                    }
                }
                None if record => {}
                None => errs.push(format!(
                    "{label}: contract/schemas/{schema_name}.json is missing"
                )),
            }
        }
    }

    if record {
        for (name, samples) in &by_schema {
            let p = root().join(format!("contract/schemas/{name}.json"));
            if !p.exists() {
                let refs: Vec<&J> = samples.iter().collect();
                let mut d = draft(&refs);
                if let J::Obj(m) = &mut d {
                    m.insert(
                        0,
                        (
                            "$schema".into(),
                            J::Str("https://json-schema.org/draft/2020-12/schema".into()),
                        ),
                    );
                    m.insert(1, ("title".into(), J::Str(name.clone())));
                }
                std::fs::write(&p, write(&d, true)).unwrap();
                eprintln!(
                    "drafted contract/schemas/{name}.json — review it: it is the contract now"
                );
            }
        }
    }

    // Nothing on disk the contract does not use.
    for f in files("contract/examples") {
        if !used_examples.contains(&f) {
            errs.push(format!(
                "contract/examples/{f}: no example in routes.toml records it"
            ));
        }
    }
    let schemas: Vec<String> = routes.iter().filter_map(|r| r.schema.clone()).collect();
    for f in files("contract/schemas") {
        let stem = f.trim_end_matches(".json").to_string();
        if !schemas.contains(&stem) {
            errs.push(format!("contract/schemas/{f}: no route uses it"));
        }
    }
    let formats: Vec<String> = routes.iter().filter_map(|r| r.format.clone()).collect();
    for f in files("contract/formats") {
        if !formats.contains(&f.trim_end_matches(".toml").to_string()) {
            errs.push(format!("contract/formats/{f}: no route answers in it"));
        }
    }

    // One contract version, said the same in three places.
    let version = std::fs::read_to_string(root().join("contract/VERSION")).unwrap();
    let version = version.trim();
    let engine = parse(&request(port, "GET", "/v1/version", &[]).body).unwrap();
    if engine.get("contract").and_then(J::str) != Some(version) {
        errs.push(format!("/v1/version does not say contract {version}"));
    }
    let page = std::fs::read_to_string(root().join("web/js/state.js")).unwrap();
    if !page.contains(&format!("export const CONTRACT = '{version}';")) {
        errs.push(format!(
            "web/js/state.js does not say `export const CONTRACT = '{version}';`"
        ));
    }

    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        errs.is_empty(),
        "the contract does not hold — {} finding(s):\n  {}",
        errs.len(),
        errs.join("\n  ")
    );
}

#[test]
fn the_validator_refuses_what_it_exists_to_refuse() {
    let s = parse(
        r#"{"type":"object","properties":{"ok":{"type":"boolean"},"n":{"type":["number","null"]},
            "tags":{"type":"array","items":{"type":"string"}},"m":{"type":"object","additionalProperties":{"type":"number"}}},
            "required":["ok"]}"#,
    )
    .unwrap();
    let check = |text: &str| {
        let mut e = Vec::new();
        validate(&parse(text).unwrap(), &s, "$", &mut e);
        e
    };
    assert!(check(r#"{"ok":true,"n":null,"tags":["a"],"m":{"x":1}}"#).is_empty());
    for (bad, says) in [
        (r#"{"n":1}"#, "`ok` is missing"),
        (r#"{"ok":1}"#, "is integer, and the contract says boolean"),
        (r#"{"ok":true,"extra":1}"#, "does not declare it"),
        (r#"{"ok":true,"tags":[1]}"#, "$.tags[0]"),
        (r#"{"ok":true,"m":{"x":"y"}}"#, "$.m.x"),
    ] {
        let e = check(bad);
        assert!(
            e.iter().any(|x| x.contains(says)),
            "{bad} was not refused as «{says}»: {e:?}"
        );
    }
    let mut e = Vec::new();
    validate(&J::Null, &parse(r#"{"minimum":1}"#).unwrap(), "$", &mut e);
    assert!(
        e.iter().any(|x| x.contains("does not")),
        "an unknown keyword was ignored"
    );
    assert_eq!(
        parse(r#"{"a":"é😀\n","b":[1,-2.5e3,true,null]}"#).unwrap(),
        J::Obj(vec![
            ("a".into(), J::Str("é😀\n".into())),
            (
                "b".into(),
                J::Arr(vec![J::Num(1.0), J::Num(-2500.0), J::Bool(true), J::Null])
            ),
        ])
    );
}

/// Every kind of figure the engine can describe keeps the contract — not only
/// the line a saved sweep draws today — so a face built against the schema can
/// draw a heatmap, an animation or a 3D scene the day the engine sends one.
#[test]
fn every_kind_of_figure_keeps_the_contract() {
    let result = schema("result").expect("no result schema");
    let item = result
        .get("properties")
        .and_then(|p| p.get("figures"))
        .and_then(|f| f.get("items"))
        .expect("the result schema does not describe figures");
    for f in vleo_modules::figure::samples() {
        let text = vleo_modules::figure::json(&f);
        let v = parse(&text).unwrap_or_else(|e| panic!("{}: not JSON: {e}\n{text}", f.id));
        let mut errs = Vec::new();
        validate(&v, item, &format!("$.figures[{}]", f.id), &mut errs);
        assert!(
            errs.is_empty(),
            "a {} figure breaks the contract: {errs:#?}",
            f.id
        );
    }
}

/// A lesson, as the route serves one, keeps the contract. No row carries a
/// lesson yet, so the recorded example is the null answer; this holds the
/// shape of a real one — the example lesson in docs/examples — to the schema.
#[test]
fn a_lesson_keeps_the_contract() {
    let s = schema("lesson").expect("no lesson schema");
    let text =
        std::fs::read_to_string(root().join("docs/examples/orbit_velocity.lesson.toml")).unwrap();
    let l = vleo_sheet::lesson::read(&text, "orbit_velocity").unwrap();
    let answer = format!(
        "{{\"ok\":true,\"checked\":true,\"lesson\":{}}}",
        vleo_sheet::lesson::json(&l)
    );
    let v = parse(&answer).unwrap_or_else(|e| panic!("not JSON: {e}\n{answer}"));
    let mut errs = Vec::new();
    validate(&v, &s, "$", &mut errs);
    assert!(errs.is_empty(), "a lesson breaks the contract: {errs:#?}");
}
