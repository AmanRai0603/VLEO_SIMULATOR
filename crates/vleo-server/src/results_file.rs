//! Saved results in and out of one file, `results.vleor` — one implementation
//! for every face that reads or writes one: `vleo results export|import` on the
//! command line, and the results page's upload through the engine.

use std::path::Path;
use vleo_modules::results::{self, store, Saved};
use vleo_results::{Kept, Value};

/// A saved result as a results file holds it: whole, and as rows.
pub fn kept_from(name: &str, s: &Saved, pinned: bool) -> Kept {
    use vleo_modules::results::{csv, sweep_csv, Row};
    let rows = |section: &str, rows: &[Row]| -> Vec<Value> {
        rows.iter()
            .map(|r| Value {
                section: section.into(),
                id: r.id.clone(),
                name: r.name.clone(),
                value: r.value.clone(),
                unit: r.unit.clone(),
                si: r.si,
                credibility: r.credibility.clone(),
                governing: r.governing.clone(),
                note: r.note.clone(),
            })
            .collect()
    };
    let mut values = rows("input", &s.inputs);
    values.extend(rows("output", &s.outputs));
    values.extend(rows("blocked", &s.blocked));
    Kept {
        name: name.to_string(),
        question: s.question(),
        target: s.target.clone(),
        mode: s.mode.clone(),
        saved: s.saved.clone(),
        label: s.name.clone(),
        chain: s.chain.clone(),
        kernel: s.kernel.clone(),
        graph: s.graph.clone(),
        case: s.case.clone(),
        ran: s.ran,
        blocked: s.blocked_count,
        thinned: s.thinned.clone(),
        pinned,
        csv: csv(s),
        sweep_csv: s.sweep.as_ref().map(sweep_csv).unwrap_or_default(),
        values,
    }
}

/// What an import did: each result in the file, by the name it is kept under
/// here, in the file's order, and whether it was already kept.
#[derive(Clone, Debug, Default)]
pub struct Imported {
    pub results: Vec<(String, bool)>,
}

impl Imported {
    pub fn added(&self) -> usize {
        self.results.iter().filter(|(_, had)| !had).count()
    }
    pub fn already(&self) -> usize {
        self.results.len() - self.added()
    }
}

/// Put every result a results file holds into the results folder `dir`, kept
/// once each as a save is, pinned where the file says it was pinned. The file
/// is read whole before anything is kept, so a file that does not read keeps
/// nothing.
pub fn import(dir: &Path, file: &Path) -> Result<Imported, String> {
    let kept = vleo_results::read(file).map_err(String::from)?;
    let mut saved = Vec::with_capacity(kept.len());
    for k in &kept {
        let mut s = results::read(&k.csv).map_err(|e| format!("{}: {e}", k.name))?;
        if !k.sweep_csv.is_empty() {
            s.sweep =
                Some(results::read_sweep(&k.sweep_csv).map_err(|e| format!("{}: {e}", k.name))?);
        }
        saved.push((s, k.pinned));
    }
    let mut out = Imported::default();
    for (s, pinned) in &saved {
        let (name, had) = store::save(dir, s).map_err(String::from)?;
        if *pinned {
            store::pin(dir, &name, true).map_err(String::from)?;
        }
        out.results.push((name, had));
    }
    Ok(out)
}

/// The same, for a results file that arrived as bytes — an upload. It is
/// written to a file of its own in the temporary folder, read from there, and
/// removed, whatever came of it.
pub fn import_bytes(dir: &Path, bytes: &[u8]) -> Result<Imported, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp =
        std::env::temp_dir().join(format!("vleo-upload-{}-{nanos}.vleor", std::process::id()));
    std::fs::write(&tmp, bytes).map_err(|e| format!("the upload could not be held: {e}"))?;
    let done = import(dir, &tmp);
    let _ = std::fs::remove_file(&tmp);
    // The temporary name means nothing to whoever sent the file.
    done.map_err(|e| e.replace(&tmp.display().to_string(), "the file sent"))
}

/// Standard base64, as a browser's upload sends a file that is not text. Any
/// character outside the alphabet, or a length that is not whole quads, is a
/// refusal — never a file read past the damage.
pub fn unbase64(text: &str) -> Option<Vec<u8>> {
    let b = text.trim().as_bytes();
    if !b.len().is_multiple_of(4) {
        return None;
    }
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a') as u32 + 26,
            b'0'..=b'9' => (c - b'0') as u32 + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        })
    };
    let mut out = Vec::with_capacity(b.len() / 4 * 3);
    for (i, q) in b.chunks(4).enumerate() {
        let last = i == b.len() / 4 - 1;
        let pad = q.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return None;
        }
        let mut n: u32 = 0;
        for &c in &q[..4 - pad] {
            n = (n << 6) | val(c)?;
        }
        n <<= 6 * pad as u32;
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{import_bytes, unbase64};

    #[test]
    fn an_upload_that_is_not_a_results_file_keeps_nothing() {
        let dir = std::env::temp_dir().join(format!("vleo-rf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for bytes in [&b"not a database"[..], &b"SQLite format 3\0"[..], &[][..]] {
            let e = import_bytes(&dir, bytes).unwrap_err();
            assert!(
                !e.contains("vleo-upload-"),
                "the temporary name leaked: {e}"
            );
        }
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            0,
            "something was kept"
        );
        let leftover = std::fs::read_dir(std::env::temp_dir())
            .unwrap()
            .filter_map(Result::ok)
            .any(|f| {
                f.file_name()
                    .to_string_lossy()
                    .starts_with(&format!("vleo-upload-{}-", std::process::id()))
            });
        assert!(!leftover, "an upload was left in the temporary folder");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn base64_reads_as_a_browser_writes_it_and_refuses_damage() {
        assert_eq!(unbase64("").unwrap(), b"");
        assert_eq!(unbase64("TQ==").unwrap(), b"M");
        assert_eq!(unbase64("TWE=").unwrap(), b"Ma");
        assert_eq!(unbase64("TWFu").unwrap(), b"Man");
        assert_eq!(
            unbase64("U1FMaXRlIGZvcm1hdCAzAA==").unwrap(),
            b"SQLite format 3\0"
        );
        assert_eq!(unbase64("/+8=").unwrap(), [0xff, 0xef]);
        for bad in ["TWF", "TW=u", "T===", "TQ==TWFu", "TW u", "TW%u"] {
            assert!(unbase64(bad).is_none(), "{bad} was read");
        }
    }
}
