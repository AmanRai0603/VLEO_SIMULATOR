//! The design these tests run: its files in `design/`, read once per test
//! binary and installed as the graph the engine runs, as every face does.

use std::sync::Once;

pub fn open() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        vleo_modules::run_on(
            vleo_modules::opened::read(&root).expect("the design's files make a graph"),
        );
    });
}
