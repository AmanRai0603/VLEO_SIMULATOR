//! Every file of the design, read and written again by this library, is the
//! same file, byte for byte.
//!
//! `design/` is the design, and every file in it is one this library wrote.
//! Read and written again, each must come back exactly as it was. A file
//! that does not was changed by something other than the library: a hand
//! edit, a tool of its own, a file from a newer format. Or the library
//! reads something it does not write back, and the next save of that file
//! loses it with nobody having changed it.
//!
//! The design on disk is read and never written: each file is written again
//! into a scratch folder.

use std::path::{Path, PathBuf};

use vleo_files::convert;
use vleo_files::sqlite;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_file_read_and_written_again_is_the_file_it_was() {
    let design = root().join("design");
    let (files, _) = convert::read_folder(&design).expect("design/ reads");
    assert!(files.len() > 1000, "design/ holds {} files", files.len());
    let scratch = std::env::temp_dir().join(format!("vleo-round-trip-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut differ = Vec::new();
    for (path, file) in &files {
        let was = std::fs::read(design.join(path)).unwrap();
        let again = scratch.join("again");
        let _ = std::fs::remove_file(&again);
        sqlite::write(file, &again).unwrap_or_else(|e| panic!("{path} does not write: {e}"));
        if std::fs::read(&again).unwrap() != was {
            differ.push(path.clone());
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        differ.is_empty(),
        "{} file(s) of design/ are not what this library writes from them: {:?}",
        differ.len(),
        &differ[..differ.len().min(10)]
    );
}
