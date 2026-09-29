//! HTTP on loopback: connections, the request guard, its limits, and decoding what a request carries.

use super::*;

pub(super) fn accept(listener: TcpListener, ctx: std::sync::Arc<Ctx>) {
    // ONE CONNECTION NEVER HOLDS UP ANOTHER, AND ONE REQUEST IS HANDLED AT A TIME.
    //
    // This loop used to read each connection itself, in turn. A browser opens
    // connections it may never use — it keeps a spare one warm so the next
    // request does not wait for a handshake — and the loop sat reading one of
    // those, with no timeout, while every real request queued behind it. The
    // page looked hung until the browser happened to close the spare. The more
    // requests a page made at once, the more spares it opened, so a busier page
    // made it worse; the manual's browser walk caught it as page loads that
    // timed out at random.
    //
    // So each connection is READ on its own thread, with a timeout, and an idle
    // one simply ends. HANDLING stays one at a time behind the lock: the form
    // writes sheets and regenerates folders, and two saves interleaving would
    // be a race this tool has never had to think about and should not start to.
    let one_at_a_time = std::sync::Arc::new(std::sync::Mutex::new(()));
    let open = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                use std::sync::atomic::Ordering::SeqCst;
                if open.fetch_add(1, SeqCst) >= MAX_CONNECTIONS {
                    open.fetch_sub(1, SeqCst);
                    drop(s);
                    continue;
                }
                let ctx = ctx.clone();
                let gate = one_at_a_time.clone();
                // Released however the thread ends. It used to be a line after
                // serve_one, which a panic skipped — so under `python -m vleo`,
                // where a panic ends only this thread, every bug hit leaked one
                // of the MAX_CONNECTIONS places, and after that many the tool
                // closed every connection it was offered.
                let place = OpenPlace(open.clone());
                std::thread::spawn(move || {
                    let _place = place;
                    if let Err(e) = serve_one(s, &ctx, &gate) {
                        eprintln!("vleo: {e}");
                    }
                });
            }
            Err(e) => eprintln!("vleo: accept: {e}"),
        }
    }
}

pub(super) fn serve_one(
    mut stream: TcpStream,
    ctx: &Ctx,
    gate: &std::sync::Mutex<()>,
) -> std::io::Result<()> {
    // Long enough for any real request to arrive; a connection that has sent
    // nothing by then is a spare the browser kept warm, and it is let go.
    stream.set_read_timeout(Some(std::time::Duration::from_secs(20)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    match reader.read_line(&mut request) {
        Ok(0) => return Ok(()),
        Ok(_) => {}
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            return Ok(())
        }
        Err(e) => return Err(e),
    }
    if request.len() > MAX_LINE {
        return answer(
            &mut stream,
            "414 URI Too Long",
            "the request line is too long",
        );
    }
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut head = Head::default();
    let mut header_bytes = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        header_bytes += line.len();
        head.lines += 1;
        if header_bytes > MAX_HEADERS || head.lines > MAX_HEADER_LINES {
            return answer(
                &mut stream,
                "431 Request Header Fields Too Large",
                "too many or too long headers",
            );
        }
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        head.take(l);
    }
    if let Some((status, why)) = head.refusal(&method, ctx.port) {
        return answer(&mut stream, status, why);
    }
    let mut body = vec![0u8; head.length];
    if head.length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body = String::from_utf8_lossy(&body).to_string();

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let params = if method == "POST" && !body.is_empty() {
        body
    } else {
        query
    };

    let (status, ctype, payload) = {
        // A poisoned lock means a handler panicked on an earlier request; that
        // request already failed, and refusing every later one would turn one
        // bad request into a dead tool.
        let _one = gate.lock().unwrap_or_else(|p| p.into_inner());
        handled(&method, &path, || route(&method, &path, &params, ctx))
    };
    respond(&mut stream, status, ctype, &payload)
}

/// One slot of the MAX_CONNECTIONS, given back when the thread holding it ends
/// — normally or by a panic.
pub(super) struct OpenPlace(pub(super) std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Drop for OpenPlace {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Handle one request so that a bug in it ends THAT request, not the tool.
///
/// A panic inside `handle` has already been written to the crash log by the
/// hook (vleo_data::crash) by the time it arrives here. The request is answered
/// with a 500 saying so and naming the file; the tool goes on serving.
///
/// What a caught panic can leave behind was checked rather than assumed:
/// handling is one request at a time behind a lock that recovers from a
/// poisoned state; the shared context is read-only; and every write of a
/// person's data is whole (vleo_data::write_whole), so a panic between two
/// writes leaves each file either as it was or as it should be — never half.
pub(super) fn handled<F>(
    method: &str,
    path: &str,
    handle: F,
) -> (&'static str, &'static str, Vec<u8>)
where
    F: FnOnce() -> (&'static str, &'static str, Vec<u8>),
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(handle)) {
        Ok(answer) => answer,
        Err(_) => {
            let log = vleo_data::crash::last()
                .map(|p| format!(" The details were written to {}.", p.display()))
                .unwrap_or_default();
            let message = format!(
                "The tool hit a bug while handling {method} {path}. That one request \
                 stopped and the tool is still running. Your case and saved results \
                 are written whole, so none is left half written.{log} Send that file \
                 to whoever maintains the tool."
            );
            if path.starts_with("/v1/") {
                // The face reads every /v1 answer as JSON and shows `message`.
                let body = format!(
                    "{{\"ok\":false,\"fault\":\"internal\",\"message\":{}}}",
                    json::string(&message)
                );
                (
                    "500 Internal Server Error",
                    "application/json; charset=utf-8",
                    body.into_bytes(),
                )
            } else {
                (
                    "500 Internal Server Error",
                    "text/plain; charset=utf-8",
                    message.into_bytes(),
                )
            }
        }
    }
}

pub(super) fn respond(
    stream: &mut TcpStream,
    status: &str,
    ctype: &str,
    payload: &[u8],
) -> std::io::Result<()> {
    // Never framed: the page is the tool, and a page of someone else's that
    // puts it in a frame could click its buttons for the person looking at it.
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nContent-Security-Policy: frame-ancestors 'none'\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(payload)?;
    stream.flush()
}

/// A request refused before it is read further, with the reason as its body.
pub(super) fn answer(stream: &mut TcpStream, status: &str, why: &str) -> std::io::Result<()> {
    respond(stream, status, "text/plain; charset=utf-8", why.as_bytes())
}

// THE LIMITS OF ONE REQUEST. The largest thing a person sends is a filled node
// form or a results file, well under a megabyte; the limits sit far above that
// and far below what could exhaust the machine.
pub(super) const MAX_LINE: usize = 16 * 1024;
pub(super) const MAX_HEADERS: usize = 64 * 1024;
pub(super) const MAX_HEADER_LINES: usize = 100;
pub(super) const MAX_BODY: usize = 16 * 1024 * 1024;
/// Connections being read or answered at once. A page opens a handful; a
/// flood past this is closed rather than given a thread each.
pub(super) const MAX_CONNECTIONS: usize = 64;

/// The headers a request is judged on.
#[derive(Default)]
pub(super) struct Head {
    length: usize,
    bad_length: bool,
    chunked: bool,
    host: Option<String>,
    origin: Option<String>,
    fetch_site: Option<String>,
    lines: usize,
}

impl Head {
    pub(super) fn take(&mut self, line: &str) {
        let Some((k, v)) = line.split_once(':') else {
            return;
        };
        let v = v.trim();
        match k.trim().to_ascii_lowercase().as_str() {
            "content-length" => match v.parse() {
                Ok(n) => self.length = n,
                Err(_) => self.bad_length = true,
            },
            "transfer-encoding" => self.chunked = !v.eq_ignore_ascii_case("identity"),
            "host" => self.host = Some(v.to_ascii_lowercase()),
            "origin" => self.origin = Some(v.to_ascii_lowercase()),
            "sec-fetch-site" => self.fetch_site = Some(v.to_ascii_lowercase()),
            _ => {}
        }
    }

    /// Why this request is not answered, or `None` to answer it.
    ///
    /// THE TOOL LISTENS ON LOOPBACK, AND LOOPBACK IS NOT PRIVATE FROM THE
    /// BROWSER. Any page the person has open can send requests to
    /// 127.0.0.1, so where a request came from is checked, not assumed:
    ///
    /// * **Host** must name this server. A page on another site that has
    ///   pointed its own name at 127.0.0.1 (DNS rebinding) sends its own name
    ///   here, and is refused — without this it could read every result.
    /// * **Origin**, on anything that changes something, must be this server.
    ///   A browser sends it with every POST; a page elsewhere posting a form
    ///   here is refused instead of resetting the case or deleting results.
    ///   A request with no Origin is a program, not a page (the parity tools,
    ///   `curl`), unless the browser says it came from another site.
    pub(super) fn refusal(&self, method: &str, port: u16) -> Option<(&'static str, &'static str)> {
        let ours = |authority: &str| {
            ["127.0.0.1", "localhost", "[::1]"]
                .iter()
                .any(|h| authority == format!("{h}:{port}"))
        };
        match &self.host {
            Some(h) if ours(h) => {}
            _ => {
                return Some((
                    "421 Misdirected Request",
                    "this server answers only to its own address",
                ))
            }
        }
        if self.chunked {
            return Some(("411 Length Required", "send the body with a Content-Length"));
        }
        if self.bad_length {
            return Some(("400 Bad Request", "the Content-Length is not a number"));
        }
        if self.length > MAX_BODY {
            return Some((
                "413 Content Too Large",
                "the request body is larger than this tool accepts",
            ));
        }
        if method != "GET" && method != "HEAD" {
            match &self.origin {
                Some(o) => {
                    let authority = o.strip_prefix("http://").unwrap_or("not ours");
                    if !ours(authority) {
                        return Some((
                            "403 Forbidden",
                            "a page on another site cannot change this tool",
                        ));
                    }
                }
                None => {
                    if matches!(self.fetch_site.as_deref(), Some("cross-site" | "same-site")) {
                        return Some((
                            "403 Forbidden",
                            "a page on another site cannot change this tool",
                        ));
                    }
                }
            }
        }
        None
    }
}

pub(super) fn param<'a>(params: &'a str, key: &str) -> Option<&'a str> {
    params.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        if k == key {
            Some(v)
        } else {
            None
        }
    })
}

pub(super) fn decode(s: &str) -> String {
    // BYTES FIRST, TEXT AFTER. Each %xx is one byte of UTF-8, and a character
    // like ° or — is two or three of them; pushing each byte as a character
    // turned "°" into two unrelated letters, which a unit check then refused.
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            // The two digits are read as BYTES. Slicing the text there panicked
            // when a raw multi-byte character followed the `%`, and a release
            // build aborts on a panic: one malformed request ended the tool.
            b'%' if i + 2 < b.len() => {
                let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
                match (hex(b[i + 1]), hex(b[i + 2])) {
                    (Some(h), Some(l)) => {
                        out.push(h << 4 | l);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---------------------------------------------------------------------------
// the saved case
//
// The inputs a person saved, stored by the application and never in the
// repository: beside the reference data, in ~/.vleo/case/inputs.csv, or
// wherever VLEO_CASE points. Stored as the same CSV a person downloads, so the
// file on disk is one they can open, read and keep.
