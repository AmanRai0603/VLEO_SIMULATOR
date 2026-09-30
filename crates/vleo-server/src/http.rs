//! Connections: accepting them, reading one request within its limits, refusing
//! another site's, answering, and reading a request's parameters.

use super::*;

/// Open the page in the default browser.
///
/// On Windows through `explorer.exe`, which hands a web address to the default
/// browser. It used to be `cmd /C start`: a program that starts a command shell
/// is one of the first things an antivirus heuristic looks for, and the tool
/// has no reason to look like that.
pub(crate) fn open_browser(url: &str) {
    let opener = if cfg!(target_os = "windows") {
        "explorer.exe"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(opener).arg(url).spawn();
}

pub(crate) fn accept(listener: TcpListener, ctx: std::sync::Arc<Ctx>) {
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
    // AT MOST `MAX_OPEN` CONNECTIONS ARE BEING READ AT ONCE, and the count is
    // given back by a guard's Drop — so a handler that panics still returns
    // its slot. A count lowered by hand at the end of the thread is one that a
    // panic skips, and after enough of them every connection is refused.
    let open = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                if open.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= MAX_OPEN {
                    open.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    let _ = respond(
                        &mut s,
                        "503 Service Unavailable",
                        "text/plain; charset=utf-8",
                        b"the tool is busy with other connections; try again",
                    );
                    continue;
                }
                let slot = Slot(open.clone());
                let ctx = ctx.clone();
                let gate = one_at_a_time.clone();
                std::thread::spawn(move || {
                    let _slot = slot;
                    if let Err(e) = serve_one(s, &ctx, &gate) {
                        eprintln!("vleo: {e}");
                    }
                });
            }
            Err(e) => eprintln!("vleo: accept: {e}"),
        }
    }
}

/// Connections read at once. A browser keeps a handful; this is far above
/// what a page needs and far below what exhausts threads.
pub(crate) const MAX_OPEN: usize = 64;
/// The request line plus headers, and the body. The largest real body is a
/// filled node form or an uploaded result, both well under a megabyte.
pub(crate) const MAX_HEAD: usize = 64 * 1024;
pub(crate) const MAX_BODY: usize = 32 * 1024 * 1024;

/// One open connection's place in the count, given back when it is dropped.
pub(crate) struct Slot(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

pub(crate) fn respond(
    stream: &mut TcpStream,
    status: &str,
    ctype: &str,
    payload: &[u8],
) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(payload)?;
    stream.flush()
}

/// Whether a request may be served: it is addressed to this machine, and a
/// write comes from this tool's own page.
///
/// THE SERVER LISTENS ON LOOPBACK, BUT A WEB PAGE ON ANY SITE RUNS ON THIS
/// MACHINE TOO. Without these checks a page you visit could post to the tool —
/// reset your case, delete results — and, by rebinding its own name to
/// 127.0.0.1, read the answers back. So:
/// - `Host` must name loopback on this port (a rebound name does not);
/// - a POST from a browser must say it comes from this origin. Browsers send
///   `Origin` and `Sec-Fetch-Site` on every cross-site POST and cannot be made
///   to omit them; a script (the Python tools, curl) sends neither, and is a
///   program the person ran themselves.
pub(crate) fn allowed(
    method: &str,
    host: Option<&str>,
    origin: Option<&str>,
    fetch_site: Option<&str>,
    port: u16,
) -> Result<(), &'static str> {
    let here = [
        format!("127.0.0.1:{port}"),
        format!("localhost:{port}"),
        format!("[::1]:{port}"),
    ];
    match host {
        Some(h) if here.iter().any(|x| x.eq_ignore_ascii_case(h.trim())) => {}
        _ => return Err(
            "this tool answers only requests addressed to 127.0.0.1 or localhost on its own port",
        ),
    }
    if method != "GET" && method != "HEAD" {
        if let Some(o) = origin {
            let o = o.trim();
            if !here
                .iter()
                .any(|x| o.eq_ignore_ascii_case(&format!("http://{x}")))
            {
                return Err("a change may only come from this tool's own page");
            }
        }
        if let Some(f) = fetch_site {
            if !matches!(f.trim(), "same-origin" | "none") {
                return Err("a change may only come from this tool's own page");
            }
        }
    }
    Ok(())
}

pub(crate) fn serve_one(
    mut stream: TcpStream,
    ctx: &Ctx,
    gate: &std::sync::Mutex<()>,
) -> std::io::Result<()> {
    // Long enough for any real request to arrive; a connection that has sent
    // nothing by then is a spare the browser kept warm, and it is let go.
    stream.set_read_timeout(Some(std::time::Duration::from_secs(20)))?;
    // The head is read through a limit: a request line or header that never
    // ends is cut at MAX_HEAD rather than read until memory runs out.
    let mut reader = BufReader::new(std::io::Read::take(
        stream.try_clone()?,
        (MAX_HEAD + MAX_BODY) as u64,
    ));
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
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut length = 0usize;
    let (mut host, mut origin, mut fetch_site) = (None, None, None);
    let mut head_bytes = request.len();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        head_bytes += line.len();
        if head_bytes > MAX_HEAD {
            return respond(
                &mut stream,
                "431 Request Header Fields Too Large",
                "text/plain; charset=utf-8",
                b"the request's headers are too large",
            );
        }
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        let Some((k, v)) = l.split_once(':') else {
            continue;
        };
        match k.trim().to_ascii_lowercase().as_str() {
            "content-length" => match v.trim().parse::<usize>() {
                Ok(n) => length = n,
                Err(_) => {
                    return respond(
                        &mut stream,
                        "400 Bad Request",
                        "text/plain; charset=utf-8",
                        b"Content-Length is not a number",
                    )
                }
            },
            "host" => host = Some(v.trim().to_string()),
            "origin" => origin = Some(v.trim().to_string()),
            "sec-fetch-site" => fetch_site = Some(v.trim().to_string()),
            _ => {}
        }
    }
    if let Err(why) = allowed(
        &method,
        host.as_deref(),
        origin.as_deref(),
        fetch_site.as_deref(),
        ctx.port,
    ) {
        return respond(
            &mut stream,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            why.as_bytes(),
        );
    }
    if length > MAX_BODY {
        return respond(
            &mut stream,
            "413 Payload Too Large",
            "text/plain; charset=utf-8",
            b"the request body is larger than this tool accepts",
        );
    }
    let mut body = vec![0u8; length];
    if length > 0 {
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
        // A BUG IN ONE REQUEST ENDS THAT REQUEST, NOT THE TOOL. The panic is
        // logged by vleo_data::crash, the page is told where, and the next
        // request is served. Every write of a case or result is whole-or-not
        // (vleo_modules::files), so nothing the person saved is half written.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            route(&method, &path, &params, ctx)
        })) {
            Ok(r) => r,
            Err(_) => {
                let log = vleo_data::crash::last_log()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| vleo_data::crash::log_dir().display().to_string());
                (
                    "500 Internal Server Error",
                    "text/plain; charset=utf-8",
                    format!(
                        "the tool hit a bug handling {path}. It was logged in {log}. \
                         Your saved case and results are safe, and the tool is still running."
                    )
                    .into_bytes(),
                )
            }
        }
    };
    respond(&mut stream, status, ctype, &payload)
}

pub(crate) fn param<'a>(params: &'a str, key: &str) -> Option<&'a str> {
    params.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        if k == key {
            Some(v)
        } else {
            None
        }
    })
}

pub(crate) fn decode(s: &str) -> String {
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
            // Hex read from the BYTES. Slicing the text here panicked when a
            // '%' was followed by a multi-byte character, and one request like
            // that ended the whole tool. A '%' with no two hex digits after it
            // is kept as the character it is.
            b'%' if i + 2 < b.len() => match (hex(b[i + 1]), hex(b[i + 2])) {
                (Some(h), Some(l)) => {
                    out.push(h * 16 + l);
                    i += 3;
                }
                _ => {
                    out.push(b'%');
                    i += 1;
                }
            },
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// the saved case
//
// The inputs a person saved, stored by the application and never in the
// repository: beside the reference data, in ~/.vleo/case/inputs.csv, or
// wherever VLEO_CASE points. Stored as the same CSV a person downloads, so the
// file on disk is one they can open, read and keep.

#[cfg(test)]
mod requests {
    use super::{allowed, decode};

    #[test]
    fn a_percent_before_a_multibyte_character_does_not_end_the_tool() {
        // Slicing the text at a byte position inside "é" used to panic, and a
        // release build aborts on a panic.
        assert_eq!(decode("%é"), "%é");
        assert_eq!(decode("a%"), "a%");
        assert_eq!(decode("%4"), "%4");
        assert_eq!(decode("%zz"), "%zz");
    }

    #[test]
    fn percent_escapes_still_decode_as_bytes() {
        assert_eq!(decode("%C2%B0C"), "°C");
        assert_eq!(decode("a+b%41"), "a bA");
    }

    #[test]
    fn a_request_addressed_elsewhere_is_refused() {
        // A page that rebinds its own name to 127.0.0.1 still sends its name.
        assert!(allowed("GET", Some("evil.example:7777"), None, None, 7777).is_err());
        assert!(allowed("GET", None, None, None, 7777).is_err());
        assert!(allowed("GET", Some("127.0.0.1:7778"), None, None, 7777).is_err());
        assert!(allowed("GET", Some("127.0.0.1:7777"), None, None, 7777).is_ok());
        assert!(allowed("GET", Some("localhost:7777"), None, None, 7777).is_ok());
    }

    #[test]
    fn a_write_from_another_site_is_refused_and_one_from_this_page_is_not() {
        let h = Some("127.0.0.1:7777");
        assert!(allowed(
            "POST",
            h,
            Some("https://evil.example"),
            Some("cross-site"),
            7777
        )
        .is_err());
        // Another port on localhost is the same site but not the same origin.
        assert!(allowed(
            "POST",
            h,
            Some("http://localhost:8080"),
            Some("same-site"),
            7777
        )
        .is_err());
        assert!(allowed("POST", h, None, Some("cross-site"), 7777).is_err());
        assert!(allowed(
            "POST",
            h,
            Some("http://127.0.0.1:7777"),
            Some("same-origin"),
            7777
        )
        .is_ok());
        // A script the person ran (the Python tools, curl) sends neither header.
        assert!(allowed("POST", h, None, None, 7777).is_ok());
    }
}
