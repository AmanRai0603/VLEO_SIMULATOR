//! The desktop app: the tool as a program a person double-clicks.
//!
//! It is the same server, started in-process, with three things a program
//! that has no console window needs and a terminal program does not:
//!
//! - **One at a time.** Opened while it is already running, it opens the
//!   browser on the one that is running and ends, rather than starting a
//!   second server on the next port.
//! - **It ends by itself.** There is no window to close. Every open page says
//!   it is still there (`/v1/alive`); when none has for `VLEO_APP_IDLE_MINUTES`
//!   (5), the app ends. The page's **Quit** ends it at once (`/v1/quit`).
//! - **A failure is shown, not printed.** A message on a console nobody can
//!   see is no message, so a start that fails writes a page saying why and
//!   what to do, and opens it.

use super::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Whether this process is the desktop app, which may be told to quit.
pub(crate) static APP: AtomicBool = AtomicBool::new(false);
/// When a page last said it was open, in seconds since the Unix epoch.
static LAST_SEEN: AtomicU64 = AtomicU64::new(0);
/// Set by `/v1/quit`.
static QUIT: AtomicBool = AtomicBool::new(false);

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A request arrived: somebody has the tool open.
pub(crate) fn seen() {
    LAST_SEEN.store(now(), Ordering::Relaxed);
}

/// `/v1/quit`: end the app once this answer is sent. Refused outside the app,
/// where the person who started the server stops it.
pub(crate) fn quit_json() -> String {
    if !APP.load(Ordering::Relaxed) {
        return failed(
            "only the desktop app is quit from the page; stop this server where it was started",
        );
    }
    QUIT.store(true, Ordering::Relaxed);
    "{\"ok\":true,\"quitting\":true}".to_string()
}

/// How long the app waits with no page open before it ends.
fn idle_limit() -> u64 {
    std::env::var("VLEO_APP_IDLE_MINUTES")
        .ok()
        .and_then(|m| m.trim().parse::<u64>().ok())
        .filter(|m| *m > 0)
        .unwrap_or(5)
        * 60
}

/// Whether the app has been left: nothing heard for longer than `limit`.
fn left(last: u64, now: u64, limit: u64) -> bool {
    now.saturating_sub(last) > limit
}

/// The port of a VLEO already serving on this machine, if one is.
fn running_here(port_pref: u16) -> Option<u16> {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    for p in port_pref..port_pref.saturating_add(16) {
        let addr = SocketAddr::from(([127, 0, 0, 1], p));
        let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(250)) else {
            continue;
        };
        let _ = s.set_read_timeout(Some(Duration::from_secs(1)));
        let ask = format!("GET /v1/version HTTP/1.0\r\nHost: 127.0.0.1:{p}\r\n\r\n");
        if s.write_all(ask.as_bytes()).is_err() {
            continue;
        }
        let mut body = String::new();
        let _ = s.read_to_string(&mut body);
        if body.contains("\"endpoint\":\"local-daemon\"") && body.contains("\"kernel\"") {
            return Some(p);
        }
    }
    None
}

/// Start the desktop app, finding the tool's files as a kit or an app bundle
/// would. Ends the process with a failure status if it cannot start.
pub fn app() {
    let port_pref: u16 = std::env::var("VLEO_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7777);
    if app_at(None, port_pref).is_err() {
        std::process::exit(1);
    }
}

/// The desktop app on the tool's files at `root` (`None`: find them). Returns
/// when it has been quit, or left — or at once, having opened the browser on
/// the copy already running. A start that fails is shown on a page as well as
/// returned. The Python package's `vleo-app` launcher runs this.
pub fn app_at(root: Option<PathBuf>, port_pref: u16) -> Result<(), String> {
    if let Some(p) = running_here(port_pref) {
        open_browser(&format!("http://127.0.0.1:{p}"));
        return Ok(());
    }
    APP.store(true, Ordering::Relaxed);
    seen();
    if let Err(e) = serve(root, port_pref, true, true) {
        start_failed(&e);
        return Err(e);
    }
    let limit = idle_limit();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if QUIT.load(Ordering::Relaxed) {
            // Long enough for the quit answer to reach the page.
            std::thread::sleep(std::time::Duration::from_millis(300));
            return Ok(());
        }
        if left(LAST_SEEN.load(Ordering::Relaxed), now(), limit) {
            return Ok(());
        }
    }
}

/// Say why the app could not start, on a page, and open it.
fn start_failed(why: &str) {
    let dir = vleo_data::crash::log_dir();
    let _ = std::fs::create_dir_all(&dir);
    let page = dir.join("start-failed.html");
    let e = vleo_sheet::escape::html;
    let html = format!(
        "<!doctype html><meta charset=utf-8><title>VLEO Design Tool did not start</title>\
         <body style=\"font:16px/1.5 system-ui,sans-serif;max-width:640px;margin:48px auto;padding:0 16px\">\
         <h1 style=\"font-size:22px\">VLEO Design Tool did not start</h1>\
         <p><b>{}</b></p>\
         <p>Nothing you saved is affected: your inputs and results are in <code>{}</code>.</p>\
         <ul><li>If another program uses port 7777 and the next fifteen, set <code>VLEO_PORT</code> \
         to a free one and open the app again.</li>\
         <li>If the tool's files are missing, unzip the download again and open the app from \
         the unzipped folder, not from inside the zip.</li></ul>\
         <p>Version {}. Send this page to whoever looks after the tool.</p>",
        e(why),
        e(&vleo_data::home()
            .map(|h| h.join(".vleo").display().to_string())
            .unwrap_or_else(|| "~/.vleo".into())),
        env!("CARGO_PKG_VERSION")
    );
    if std::fs::write(&page, html).is_ok() {
        open_browser(&format!("file://{}", page.display()));
    }
    eprintln!("vleo: {why}");
}

#[cfg(test)]
mod leaving {
    use super::left;

    #[test]
    fn the_app_ends_only_after_the_whole_idle_limit() {
        assert!(!left(1000, 1000, 300));
        assert!(!left(1000, 1300, 300));
        assert!(left(1000, 1301, 300));
        // A clock that steps back never ends it.
        assert!(!left(2000, 1000, 300));
    }
}
