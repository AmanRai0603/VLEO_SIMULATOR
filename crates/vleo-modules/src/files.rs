//! Writing a person's files so that a crash never leaves half of one.
//!
//! A case or a result is written beside itself under a name no other writer
//! uses, flushed to the disk, and renamed over the old one — which replaces it
//! in one step on every platform this tool runs on. A crash, a full disk or a
//! sync client reading mid-write sees the whole old file or the whole new one,
//! never a truncated one. Every write of user data goes through here.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static N: AtomicU64 = AtomicU64::new(0);

/// Write `bytes` to `path` whole, or leave `path` as it was.
pub fn write_whole(path: &Path, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let tmp: PathBuf = path.with_file_name(format!(
        ".{name}.writing-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes.as_ref())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::write_whole;

    #[test]
    fn replaces_whole_and_leaves_no_temporary_behind() {
        let dir = std::env::temp_dir().join(format!("vleo-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("inputs.csv");
        write_whole(&p, "old").unwrap();
        write_whole(&p, "new").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
        let left: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().collect();
        assert_eq!(left.len(), 1, "a temporary file was left behind");
        // A write that cannot happen leaves the old file as it was.
        assert!(write_whole(&dir.join("no-such-dir").join("x.csv"), "x").is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
