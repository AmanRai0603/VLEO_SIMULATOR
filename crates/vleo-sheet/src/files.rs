//! Where the tree is read from: the repository's folders, or the design's
//! files.
//!
//! The loader reads the design through this and nothing else, so the same
//! reading — every check the loader makes — applies whichever one it is. A
//! developer's checkout reads the folders it edits; the design's files
//! (`design/`, `vleo_files::convert::Served`) are read through it too, and a
//! page served from them is the page served from the folders, byte for byte.
//!
//! Paths are the ones the folders have: the tree's root joined with
//! `crates/vleo-mod-solar/nodes/…`, `layers/…`, `cases/…`, `sources/…`. The
//! design's files answer for the paths of the design they hold and for nothing
//! else; they never fall back to a folder, because a page drawn half from them
//! and half from whatever folder happens to sit beside them is two designs
//! presented as one.

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
