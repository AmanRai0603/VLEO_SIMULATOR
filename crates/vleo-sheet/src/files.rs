//! Where the tree is read from: the repository's folders, or a design file.
//!
//! The loader reads the design through this and nothing else, so the same
//! reading — every check the loader makes — applies whichever one it is. A
//! developer's checkout reads the folders it edits. The tool a team runs reads
//! `design.vleo`, the one file the release carries in their place
//! (`vleo-design`), and a page served from it is the page served from the
//! folders, byte for byte.
//!
//! Paths are the ones the folders have: the tree's root joined with
//! `crates/vleo-mod-solar/nodes/…`, `layers/…`, `cases/…`, `sources/…`. A
//! design file answers for the paths it holds and for nothing else; it never
//! falls back to a folder, because a page drawn half from a file and half from
//! whatever folder happens to sit beside it is two designs presented as one.

use std::io;
use std::path::{Path, PathBuf};

/// A read-only view of the tree.
pub trait Files: Send + Sync {
    /// The bytes of one file.
    fn read(&self, p: &Path) -> io::Result<Vec<u8>>;
    /// The entries directly inside a directory, as full paths, in no order.
    fn entries(&self, p: &Path) -> io::Result<Vec<PathBuf>>;
    fn is_dir(&self, p: &Path) -> bool;
    fn is_file(&self, p: &Path) -> bool;
    /// The size of one file, if it is held.
    fn len(&self, p: &Path) -> Option<u64> {
        self.read(p).ok().map(|b| b.len() as u64)
    }
    fn read_to_string(&self, p: &Path) -> io::Result<String> {
        String::from_utf8(self.read(p)?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// The folders on disk.
pub struct Disk;

impl Files for Disk {
    fn read(&self, p: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(p)
    }
    fn entries(&self, p: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(std::fs::read_dir(p)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect())
    }
    fn is_dir(&self, p: &Path) -> bool {
        p.is_dir()
    }
    fn is_file(&self, p: &Path) -> bool {
        p.is_file()
    }
    fn len(&self, p: &Path) -> Option<u64> {
        std::fs::metadata(p).ok().map(|m| m.len())
    }
}

/// No folders at all: what a page has. A design served over it is read from
/// its own files and nothing else; every path it does not hold is not there.
pub struct Nowhere;

impl Files for Nowhere {
    fn read(&self, p: &Path) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}: there are no folders here", p.display()),
        ))
    }
    fn entries(&self, _: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }
    fn is_dir(&self, _: &Path) -> bool {
        false
    }
    fn is_file(&self, _: &Path) -> bool {
        false
    }
}

/// Whether a path, relative to the tree's root, is one a design file holds:
/// a file in a node folder (`crates/vleo-mod-*/nodes/…`), a layer, a case or
/// the source list.
pub fn in_design(rel: &str) -> bool {
    let mut parts = rel.split('/');
    match parts.next() {
        Some("crates") => {
            let c = parts.next().unwrap_or("");
            c.starts_with("vleo-mod-") && parts.next() == Some("nodes")
        }
        Some("layers" | "cases" | "sources") => true,
        _ => false,
    }
}

/// Every file of the tree a design file holds, relative to the root with `/`
/// between parts, sorted.
pub fn design_files(root: &Path) -> io::Result<Vec<String>> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
        for e in std::fs::read_dir(dir)? {
            let p = e?.path();
            if p.is_dir() {
                walk(root, &p, out)?;
            } else if p.is_file() {
                let rel = p
                    .strip_prefix(root)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
                let rel: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                let rel = rel.join("/");
                if in_design(&rel) {
                    out.push(rel);
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    for top in ["layers", "cases", "sources"] {
        let d = root.join(top);
        if d.is_dir() {
            walk(root, &d, &mut out)?;
        }
    }
    let crates = root.join("crates");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&crates)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("vleo-mod-"))
        })
        .collect();
    dirs.sort();
    for d in dirs {
        let nodes = d.join("nodes");
        if nodes.is_dir() {
            walk(root, &nodes, &mut out)?;
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_design_file_holds_the_node_folders_and_the_layers_only() {
        assert!(in_design("crates/vleo-mod-solar/nodes/sw_ap/node.toml"));
        assert!(in_design("layers/l3_solar.toml"));
        assert!(in_design("cases/baseline.toml"));
        assert!(in_design("sources/sources.toml"));
        assert!(!in_design("crates/vleo-core/src/lib.rs"));
        assert!(!in_design("crates/vleo-mod-solar/src/lib.rs"));
        assert!(!in_design("web/index.html"));
        assert!(!in_design("docs/manual.toml"));
    }
}
