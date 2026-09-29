//! Writing a file whole: the one way the tool writes a person's data.
//!
//! A plain write truncates the file and then fills it. Anything that happens in
//! between — the tool crashing, the laptop sleeping, a sync client (OneDrive,
//! a network drive) copying the file mid-write — leaves a file that is cut
//! off, and the next read of it fails or, worse, reads a case with half its
//! values. For a shared results folder that is not a rare event: a sync client
//! reads files while they are being written as a matter of course.
//!
//! So every write of a case, a result, a backup or a lockfile goes through
//! [`write_whole`]: the bytes go to a file beside the target, are flushed to
//! the disk, and the new file is renamed over the old in one step. A reader —
//! this tool, another laptop, a sync client — sees the old file or the new one,
//! never a file half way through being written.

use std::io::Write;
use std::path::{Path, PathBuf};

/// Replace `path` with `bytes`, whole.
///
/// The side file is named for this process, so two programs writing the same
/// path at once cannot write into each other's side file; the last rename wins
/// with a complete file either way. On failure the side file is removed and
/// whatever was at `path` before is left exactly as it was.
pub fn write_whole(path: &Path, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let side = side_file(path);
    let written = (|| {
        let mut f = std::fs::File::create(&side)?;
        f.write_all(bytes.as_ref())?;
        // On the disk before the rename, or a power cut can leave the renamed
        // file empty: the rename is recorded and the data it names is not.
        f.sync_all()?;
        std::fs::rename(&side, path)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&side);
    }
    written
}

/// `<name>.<pid>.writing`, beside the target. A listing that reads only its own
/// extension — `.csv` for results — never sees one.
fn side_file(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{}.writing", std::process::id()));
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vleo-whole-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn it_replaces_the_file_and_leaves_nothing_beside_it() {
        let d = scratch("replace");
        let p = d.join("case.csv");
        std::fs::write(&p, "old").unwrap();
        write_whole(&p, "new").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
        let left: Vec<_> = std::fs::read_dir(&d).unwrap().flatten().collect();
        assert_eq!(left.len(), 1, "a side file was left behind");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_reader_never_sees_a_file_half_written() {
        // What a sync client or another laptop does to a shared results
        // folder: read the file while it is being replaced. A plain write
        // truncates first, and this reader catches it short within a few
        // rounds; written whole, every read is one complete version.
        let d = scratch("reader");
        let p = d.join("result.csv");
        let size = 1 << 20;
        write_whole(&p, vec![b'a'; size]).unwrap();
        let writer = {
            let p = p.clone();
            std::thread::spawn(move || {
                for i in 0..200 {
                    let fill = if i % 2 == 0 { b'b' } else { b'a' };
                    write_whole(&p, vec![fill; size]).unwrap();
                }
            })
        };
        let mut reads = 0;
        while !writer.is_finished() {
            if let Ok(bytes) = std::fs::read(&p) {
                assert_eq!(bytes.len(), size, "read a file half written");
                assert!(
                    bytes.iter().all(|&b| b == bytes[0]),
                    "read a mixture of two writes"
                );
                reads += 1;
            }
        }
        writer.join().unwrap();
        assert!(reads > 0, "the reader never ran, so this proved nothing");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_write_that_fails_leaves_the_old_file_as_it_was() {
        let d = scratch("fails");
        // A directory where the file should go: the rename cannot replace it.
        let p = d.join("results.csv");
        std::fs::create_dir_all(p.join("inside")).unwrap();
        assert!(write_whole(&p, "new").is_err());
        assert!(p.join("inside").is_dir(), "what was there was disturbed");
        let left: Vec<_> = std::fs::read_dir(&d).unwrap().flatten().collect();
        assert_eq!(left.len(), 1, "the side file was not cleaned up");
        let _ = std::fs::remove_dir_all(&d);
    }
}
