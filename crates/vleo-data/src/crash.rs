//! The crash log: when the tool hits a bug, a file a person can send.
//!
//! A bug in the tool shows up as a Rust panic. Before this, a panic in the
//! release build ended the whole program with nothing written anywhere, and the
//! person using it saw a page that stopped answering. There was nothing to send
//! and nothing to read, so the same bug could only be found again by chance.
//!
//! [`install`] adds a panic hook that writes one small text file per panic to
//! `~/.vleo/log/` (or wherever `VLEO_LOG` points): the program, its version,
//! the time, the thread, the message and the line of code. Then the panic goes
//! on as before — the server catches it for that one request, the C and Python
//! entry points turn it into an error — and [`last`] says where the file went,
//! so the page that shows the error can name it.
//!
//! The folder is kept to the newest [`KEEP`] files, so a bug hit in a loop
//! cannot fill a disk.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// How many crash files the log folder keeps. The newest are kept.
pub const KEEP: usize = 50;

static LAST: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Add the crash log to this process's panic hook, once. The hook that was
/// there before still runs after it, so a panic prints exactly as it did.
pub fn install(program: &'static str, version: &'static str) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let before = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let message = message_of(info.payload());
            let location = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_else(|| "an unknown line".into());
            let thread = std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_string();
            let text = report(program, version, &thread, &message, &location, now());
            // Never panic in here: a panic inside a panic hook ends the
            // process, which is the very thing this exists to prevent.
            if let Ok(path) = write_report(&crate::log_path(), &text) {
                eprintln!(
                    "vleo: the tool hit a bug; the details are in {}",
                    path.display()
                );
                *LAST.lock().unwrap_or_else(|p| p.into_inner()) = Some(path);
            }
            before(info);
        }));
    });
}

/// The file the most recent panic in this process was written to.
pub fn last() -> Option<PathBuf> {
    LAST.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

/// The text of one crash file. Plain text, because the person reading it may
/// be the one who hit the bug, with nothing but a text editor.
pub fn report(
    program: &str,
    version: &str,
    thread: &str,
    message: &str,
    location: &str,
    unix_secs: i64,
) -> String {
    let when = vleo_core::units::calendar::Civil::from_unix(unix_secs);
    format!(
        "The VLEO tool hit a bug.\n\
         \n\
         program   {program} {version}\n\
         when      {when}\n\
         thread    {thread}\n\
         where     {location}\n\
         message   {message}\n\
         \n\
         What was running at the time stopped with an error. In the browser tool\n\
         only that one request stopped and the tool kept running. Your case and\n\
         your saved results are always written whole, so none is left half\n\
         written. Send this file to whoever maintains the tool: the line under\n\
         `where` is the place in the code to start from.\n"
    )
}

/// Write one report into `dir`, then keep only the newest [`KEEP`] files.
pub fn write_report(dir: &Path, text: &str) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let stamp: String = vleo_core::units::calendar::Civil::from_unix(now())
        .to_string()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let path = (0..)
        .map(|n| dir.join(format!("crash-{stamp}-{}-{n:03}.txt", std::process::id())))
        .find(|p| !p.exists())
        .unwrap_or_else(|| dir.join("crash.txt"));
    crate::write_whole(&path, text)?;
    prune(dir, KEEP);
    Ok(path)
}

/// Remove all but the newest `keep` crash files. Only files this module wrote
/// are touched; anything else in the folder is left alone.
fn prune(dir: &Path, keep: usize) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut crashes: Vec<(std::time::SystemTime, PathBuf)> = rd
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("crash-") && n.ends_with(".txt")
        })
        .map(|e| {
            let t = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (t, e.path())
        })
        .collect();
    if crashes.len() <= keep {
        return;
    }
    // Newest first by time, then by name, so files written in the same
    // second still have an order.
    crashes.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    for (_, p) in crashes.into_iter().skip(keep) {
        let _ = std::fs::remove_file(p);
    }
}

fn message_of(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "a panic with no message".into())
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_names_the_program_the_line_and_the_message() {
        let r = report(
            "vleo-daemon",
            "0.4.0",
            "main",
            "index out of bounds",
            "crates/x.rs:12",
            0,
        );
        for want in [
            "vleo-daemon 0.4.0",
            "1970-01-01T00:00:00Z",
            "crates/x.rs:12",
            "index out of bounds",
        ] {
            assert!(r.contains(want), "missing {want:?} in\n{r}");
        }
    }

    #[test]
    fn a_panic_after_install_leaves_a_file_naming_it() {
        // VLEO_LOG is read only by the hook, and this is the only test in this
        // binary that panics, so setting it here disturbs nothing else.
        let d = std::env::temp_dir().join(format!("vleo-crash-hook-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::env::set_var("VLEO_LOG", &d);
        install("vleo-test", "0.0.0");
        let ended = std::thread::Builder::new()
            .name("the-bug".into())
            .spawn(|| panic!("a deliberate bug for the crash log"))
            .unwrap()
            .join();
        assert!(ended.is_err());
        let file = last().expect("the hook recorded no file");
        assert!(
            file.starts_with(&d),
            "{} is not in the log folder",
            file.display()
        );
        let text = std::fs::read_to_string(&file).unwrap();
        for want in [
            "vleo-test 0.0.0",
            "the-bug",
            "a deliberate bug for the crash log",
            "crash.rs:",
        ] {
            assert!(text.contains(want), "missing {want:?} in\n{text}");
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_folder_keeps_only_the_newest() {
        let d = std::env::temp_dir().join(format!("vleo-crash-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("notes.txt"), "not a crash file").unwrap();
        for i in 0..(KEEP + 5) {
            write_report(&d, &format!("report {i}")).unwrap();
        }
        let crashes = std::fs::read_dir(&d)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("crash-"))
            .count();
        assert_eq!(crashes, KEEP);
        assert!(
            d.join("notes.txt").exists(),
            "a file the log did not write was removed"
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
