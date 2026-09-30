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
        ".{name}.writing-{}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed),
        // Two machines writing into one shared folder can share a process
        // number; they do not share the nanosecond they started writing.
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
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

// ---------------------------------------------------------------------------
// a zip, stored

/// CRC-32 (IEEE), as zip checks each file by.
fn crc32(bytes: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in bytes {
        c ^= u32::from(b);
        for _ in 0..8 {
            c = if c & 1 != 0 {
                (c >> 1) ^ 0xEDB8_8320
            } else {
                c >> 1
            };
        }
    }
    !c
}

/// A zip of `files`, each stored as it is. Stored rather than compressed so
/// it needs nothing but this file to write and read; every unzip tool opens
/// it. The dates are fixed, so the same files make the same bytes.
pub fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let at = out.len() as u32;
        let crc = crc32(data);
        let n = data.len() as u32;
        let head = |sig: u32, central: bool| {
            let mut h = Vec::new();
            h.extend(sig.to_le_bytes());
            if central {
                h.extend(20u16.to_le_bytes()); // made by
            }
            h.extend(20u16.to_le_bytes()); // needed to extract
            h.extend(0x0800u16.to_le_bytes()); // names are UTF-8
            h.extend(0u16.to_le_bytes()); // stored
            h.extend(0u16.to_le_bytes()); // 00:00
            h.extend(0x0021u16.to_le_bytes()); // 1980-01-01
            h.extend(crc.to_le_bytes());
            h.extend(n.to_le_bytes());
            h.extend(n.to_le_bytes());
            h.extend((name.len() as u16).to_le_bytes());
            h.extend(0u16.to_le_bytes()); // extra
            if central {
                h.extend([0u8; 6]); // comment, disk, internal attributes
                h.extend(0u32.to_le_bytes()); // external attributes
                h.extend(at.to_le_bytes());
            }
            h.extend(name.as_bytes());
            h
        };
        out.extend(head(0x0403_4b50, false));
        out.extend(*data);
        central.extend(head(0x0201_4b50, true));
    }
    let cd_at = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend(central);
    out.extend(0x0605_4b50u32.to_le_bytes());
    out.extend([0u8; 4]);
    out.extend((files.len() as u16).to_le_bytes());
    out.extend((files.len() as u16).to_le_bytes());
    out.extend(cd_len.to_le_bytes());
    out.extend(cd_at.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

/// The files of a stored zip — as `zip` writes it — each checked against its
/// CRC. A zip that was compressed since is refused by name: this reads the
/// file as it was made, and says so rather than guessing at the rest.
pub fn unzip(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let u16_at = |k: usize| -> Result<usize, String> {
        bytes
            .get(k..k + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]) as usize)
            .ok_or_else(|| "it ends early".to_string())
    };
    let u32_at = |k: usize| -> Result<u32, String> {
        bytes
            .get(k..k + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or_else(|| "it ends early".to_string())
    };
    let mut files = Vec::new();
    let mut k = 0;
    while u32_at(k).ok() == Some(0x0403_4b50) {
        let flags = u16_at(k + 6)?;
        let method = u16_at(k + 8)?;
        let crc = u32_at(k + 14)?;
        let size = u32_at(k + 18)? as usize;
        let name_len = u16_at(k + 26)?;
        let extra = u16_at(k + 28)?;
        let name_at = k + 30;
        let name = bytes
            .get(name_at..name_at + name_len)
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .ok_or("it ends early")?;
        if method != 0 || flags & 0x0008 != 0 {
            return Err(format!(
                "`{name}` was compressed after the file was made — unzip it and upload \
                 result.csv instead"
            ));
        }
        let data_at = name_at + name_len + extra;
        let data = bytes
            .get(data_at..data_at + size)
            .ok_or("it ends early")?
            .to_vec();
        if crc32(&data) != crc {
            return Err(format!("`{name}` is damaged: its check does not match"));
        }
        files.push((name, data));
        k = data_at + size;
    }
    if files.is_empty() {
        return Err("it is not a zip".into());
    }
    // The directory at the end, which is what unzip tools read first: one
    // entry per file, in order, and then its end record.
    for (name, _) in &files {
        if u32_at(k)? != 0x0201_4b50
            || bytes.get(k + 46..k + 46 + name.len()) != Some(name.as_bytes())
        {
            return Err(format!("its directory does not list `{name}`"));
        }
        k += 46 + u16_at(k + 28)? + u16_at(k + 30)? + u16_at(k + 32)?;
    }
    if u32_at(k)? != 0x0605_4b50 {
        return Err("its directory has no end".into());
    }
    Ok(files)
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

    #[test]
    fn a_zip_reads_back_and_a_damaged_one_is_refused() {
        use super::{crc32, unzip, zip};
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        let z = zip(&[("a.txt", b"alpha"), ("b.csv", b"")]);
        let back = unzip(&z).unwrap();
        assert_eq!(back[0], ("a.txt".to_string(), b"alpha".to_vec()));
        assert_eq!(back[1], ("b.csv".to_string(), Vec::new()));
        let mut bad = z.clone();
        bad[36] ^= 1; // inside "alpha"
        assert!(unzip(&bad).unwrap_err().contains("damaged"));
        assert!(unzip(&z[..20]).is_err());
        assert!(unzip(b"not a zip").is_err());
    }
}
