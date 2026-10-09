//! A node whose conclusion is a set can be written as a method.
//!
//! `sw_kp_scenarios` publishes ten values from five inputs, and until the
//! method language had `publish` it could only be hand-written Rust. Here its
//! relation — the published Kp scale plus the measured slot offset, at five
//! scenarios and two slots — is written as a method, checked against the
//! node's own signature, and run on the node's own fixtures, which come from
//! outside this code (an independent derivation from the two tables). This
//! test does not give the node a method: whether it gets one is its owner's
//! form to send — the solar group's release has since given it one, transcribed
//! from its Rust, and this one, written with the kernel's relations, is held to
//! the same fixtures. It shows the language can say what the node says.

use std::path::Path;
use vleo_sheet::method::{self, Outcome};

const METHOD: &str = "\
# Kp on the published scale, plus the measured offset of each slot, at each scenario.
publish Kp_mean_nominal = kp_from_ap(ap_nominal) + kp_mean_slot_bias(ap_nominal)
publish Kp_mean_hotmean = kp_from_ap(ap_hotmean) + kp_mean_slot_bias(ap_hotmean)
publish Kp_mean_coldmean = kp_from_ap(ap_coldmean) + kp_mean_slot_bias(ap_coldmean)
publish Kp_mean_hotday = kp_from_ap(ap_hotday) + kp_mean_slot_bias(ap_hotday)
publish Kp_mean_coldday = kp_from_ap(ap_coldday) + kp_mean_slot_bias(ap_coldday)
publish Kp_peak_nominal = kp_from_ap(ap_nominal) + kp_peak_slot_bias(ap_nominal)
publish Kp_peak_hotmean = kp_from_ap(ap_hotmean) + kp_peak_slot_bias(ap_hotmean)
publish Kp_peak_coldmean = kp_from_ap(ap_coldmean) + kp_peak_slot_bias(ap_coldmean)
publish Kp_peak_coldday = kp_from_ap(ap_coldday) + kp_peak_slot_bias(ap_coldday)
# The answer: the worst slot of the worst day.
return kp_from_ap(ap_hotday) + kp_peak_slot_bias(ap_hotday)
";

#[test]
fn sw_kp_scenarios_written_as_a_method_meets_its_own_fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (tree, _) = vleo_files::convert::open(&root).expect("the design loads");
    let sh = &tree.sheets["sw_kp_scenarios"];
    assert_eq!(
        sh.publishes.len(),
        9,
        "the node's shape changed; this test needs reading again"
    );
    let sig = method::node_signature(sh).expect("every quantity is known");
    assert_eq!(sig.publishes.len(), 9);
    let p = method::compile(METHOD, &sig).unwrap_or_else(|e| panic!("{e:?}"));

    assert!(!sh.fixtures.is_empty());
    for fx in &sh.fixtures {
        let (o, published) = method::run_all(&p, &fx.inputs).expect("the method runs");
        let Outcome::Answer(answer) = o else {
            panic!("«{}»: the method refused", fx.label)
        };
        // A fixture names the member it is about by its published id; the
        // node's own answer by its symbol, or by nothing.
        let got = if fx.variable.is_empty() || fx.variable == sh.symbol {
            answer
        } else {
            let m = sh
                .publishes
                .iter()
                .find(|m| m.id == fx.variable)
                .unwrap_or_else(|| panic!("no member «{}»", fx.variable));
            published
                .iter()
                .find(|(s, _)| *s == m.symbol)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| panic!("«{}» was not published", m.symbol))
        };
        let rel = ((got - fx.expect) / fx.expect).abs();
        assert!(
            rel <= fx.tolerance,
            "«{}»: the method gives {got}, the fixture says {}, relative {rel:e}",
            fx.label,
            fx.expect
        );
    }
}
