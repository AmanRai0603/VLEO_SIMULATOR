//! What happens when the tool itself has a bug.
//!
//! A panic used to end the whole program with nothing written anywhere: the
//! page simply stopped answering, and the only trace was whatever the terminal
//! happened to show. Now every program installs this hook first. It writes one
//! file per crash under `~/.vleo/log/` — the message, where in the code, the
//! version, the thread and a backtrace — so the person can send it, and the
//! faces that catch a panic (the server per request, the C interface per call)
//! keep running and say where the log is.

use std::path::PathBuf;
use std::sync::Mutex;

static LAST: Mutex<Option<PathBuf>> = Mutex::new(None);
static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Where crash logs are kept: `~/.vleo/log/`, or `VLEO_LOG` if set and not empty.
pub fn log_dir() -> PathBuf {
    std::env::var_os("VLEO_LOG")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| crate::home().map(|h| h.join(".vleo").join("log")))
        .unwrap_or_else(|| PathBuf::from(".vleo/log"))
}

/// The log the most recent panic in this process was written to, if any.
pub fn last_log() -> Option<PathBuf> {
    LAST.lock().ok().and_then(|g| g.clone())
}

/// Install the hook. `program` and `version` go into every log. The previous
/// hook still runs, so the message still reaches the terminal.
pub fn install(program: &'static str, version: &'static str) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let written = write_log(program, version, info);
        if let Some(p) = &written {
            if let Ok(mut g) = LAST.lock() {
                *g = Some(p.clone());
            }
        }
        previous(info);
        if let Some(p) = written {
            eprintln!(
                "vleo: this was a bug in the tool, logged in {}",
                p.display()
            );
        }
    }));
}

fn write_log(
    program: &str,
    version: &str,
    info: &std::panic::PanicHookInfo<'_>,
) -> Option<PathBuf> {
    let dir = log_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let when = crate::clock::now_utc();
    let stamp: String = when
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    // Two panics in one second must not share a file, or the second hides the first.
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = dir.join(format!("crash-{stamp}-{}-{n}.txt", std::process::id()));
    let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "(no message)".to_string()
    };
    let at = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "(unknown)".into());
    let thread = std::thread::current()
        .name()
        .unwrap_or("(unnamed)")
        .to_string();
    let text = format!(
        "{program} {version} crashed\n\
         when:    {when}\n\
         where:   {at}\n\
         thread:  {thread}\n\
         os:      {} {}\n\
         message: {message}\n\n\
         Send this file to the tool's maintainers. Your saved case and results are\n\
         written whole or not at all, so they are as they were before this.\n\n\
         backtrace:\n{}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::backtrace::Backtrace::force_capture()
    );
    std::fs::write(&path, text).ok()?;
    Some(path)
}
