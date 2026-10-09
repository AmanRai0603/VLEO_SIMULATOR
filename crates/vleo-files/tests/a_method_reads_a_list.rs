//! A node whose relation counts along a short published list can be written
//! as a method.
//!
//! Three solar rows — the activity band, the cycle number and the design Ap —
//! each walk or index a list of three published values, and until the method
//! language had lists they could only be hand-written Rust. Here each relation
//! is written as a method, checked against the node's own signature, and run
//! on the node's own fixtures, which come from outside this code (published
//! sources). This test does not give any node a method: whether it gets one is
//! its owner's form to send — the solar group's release has since given two of
//! them theirs, transcribed as comparisons, and these, written with a list, are
//! held to the same fixtures. It shows the language can say what the nodes say.

use std::path::Path;
use vleo_sheet::method::{self, Outcome};

/// The three published F10.7 edges; a flux on an edge belongs to the band the
/// edge opens.
const ACTIVITY_BAND: &str = "\
const EDGES = [90, 130, 170] [1]
let band = 1
for edge in EDGES
  if f107 >= edge then
    set band = band + 1
  end
end
return band
";

/// The days, from the mission's reference epoch, on which cycles 23, 24 and
/// 25 begin; an epoch past the last start is still 25.
const CYCLE_NUMBER: &str = "\
const STARTS = [-1081, 3257, 7274] [d]
let cycle = 22
for start in STARTS
  if epoch >= start then
    set cycle = cycle + 1
  end
end
return cycle
";

/// The daily Ap that bounds each NOAA G level, read by entry.
const AP_DESIGN: &str = "\
const AP_AT_G = [48, 80, 132] [1]
let i = 3
if g_level < 1.5 then
  set i = 1
else if g_level < 2.5 then
  set i = 2
end
return AP_AT_G[i]
";

#[test]
fn three_list_reading_rows_written_as_methods_meet_their_own_fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (tree, _) = vleo_files::convert::open(&root).expect("the design loads");
    for (node, src) in [
        ("sw_activity_band", ACTIVITY_BAND),
        ("sw_cycle_number", CYCLE_NUMBER),
        ("sw_ap_design", AP_DESIGN),
    ] {
        let sh = &tree.sheets[node];
        let sig = method::node_signature(sh).expect("every quantity is known");
        let p = method::compile(src, &sig).unwrap_or_else(|e| panic!("{node}: {e:?}"));
        assert!(!sh.fixtures.is_empty(), "{node} has no fixtures");
        for fx in &sh.fixtures {
            let o = method::run(&p, &fx.inputs).expect("the method runs");
            let Outcome::Answer(got) = o else {
                panic!("{node} «{}»: the method refused", fx.label)
            };
            let rel = ((got - fx.expect) / fx.expect).abs();
            assert!(
                rel <= fx.tolerance,
                "{node} «{}»: the method gives {got}, the fixture says {}",
                fx.label,
                fx.expect
            );
        }
    }
}
