//! The resolver tracks a fixed number of declared cycles, because the kernel
//! does not allocate. Past that number a cycle used to be swept again every
//! time one of its members came up; now the run is refused before it starts.

use vleo_core::graph::{NodeDef, NodeTable, VarDef};
use vleo_core::resolver::{evaluate, CycleSpec, Mode, Workspace, MAX_CYCLES};
use vleo_core::{Fault, Slot, Store};

struct Empty;
impl NodeTable for Empty {
    fn nodes(&self) -> &[NodeDef] {
        &[]
    }
    fn vars(&self) -> &[VarDef] {
        &[]
    }
    fn eval(&self, _: u16, _: &mut Store<'_>) -> Result<(), Fault> {
        Ok(())
    }
}

fn run(cycles: usize) -> Result<(), Fault> {
    let spec = CycleSpec { nodes: &[], converge_on: 0, tolerance: 1e-6, max_iter: 1, seeds: &[] };
    let specs = vec![spec; cycles];
    let mut slots: [Slot; 1] = [Slot::EMPTY];
    let mut store = Store::new(&mut slots);
    let (mut order, mut mark, mut stack, mut ran, mut blocked) = ([0u16; 1], [0u8; 1], [0u16; 1], [0u16; 1], [0u16; 1]);
    let mut blocked_fault = [Fault::NotRun { node: "x" }; 1];
    let mut ws = Workspace {
        order: &mut order,
        mark: &mut mark,
        stack: &mut stack,
        ran: &mut ran,
        blocked: &mut blocked,
        blocked_fault: &mut blocked_fault,
    };
    evaluate(&Empty, &mut store, Mode::All, &specs, &mut ws).map(|_| ())
}

#[test]
fn one_more_cycle_than_the_resolver_tracks_is_refused_by_name() {
    assert!(run(MAX_CYCLES).is_ok());
    match run(MAX_CYCLES + 1) {
        Err(Fault::TooManyCycles { declared, limit }) => {
            assert_eq!((declared, limit), (MAX_CYCLES + 1, MAX_CYCLES));
        }
        other => panic!("expected TooManyCycles, got {other:?}"),
    }
}
