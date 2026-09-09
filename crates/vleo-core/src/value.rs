//! The value store — where a run keeps what it has computed so far.
//!
//! No allocation. The caller provides one slot per variable in the graph, and
//! the resolver fills them. That is what lets the identical kernel run in a
//! browser, on a laptop, on a bench and on a flight target with no allocator
//! present.

use crate::credibility::CredVec;
use vleo_units::Unit;

/// What is known about one variable at this moment in a run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotStatus {
    /// Nothing has produced this value.
    Empty,
    /// A case supplied it, or it is a declared value published by its node.
    Supplied,
    /// A node computed it during this run.
    Computed,
    /// It was computed, and something it depends on has changed since. The
    /// value is *unknown*, not wrong, and it is shown as such — never cleared,
    /// and never presented as current.
    Stale,
    /// The node that produces it refused. The fault is on the report.
    Refused,
}

/// One variable's slot.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    /// The value, always in the variable's declared SI unit.
    pub value: f64,
    /// The unit, carried so a value crossing a boundary still says what it is.
    pub unit: Unit,
    pub status: SlotStatus,
    /// Index of the node that produced it, or `u16::MAX` when supplied by a case.
    pub producer: u16,
    /// The credibility of the value, rolled up along the chain that produced it.
    pub cred: CredVec,
}

impl Slot {
    pub const EMPTY: Slot = Slot {
        value: 0.0,
        unit: Unit::One,
        status: SlotStatus::Empty,
        producer: u16::MAX,
        cred: CredVec::ZERO,
    };
    pub fn is_known(&self) -> bool {
        matches!(self.status, SlotStatus::Supplied | SlotStatus::Computed)
    }
}

/// The whole store for one run: one slot per variable.
pub struct Store<'a> {
    pub slots: &'a mut [Slot],
}

impl<'a> Store<'a> {
    pub fn new(slots: &'a mut [Slot]) -> Store<'a> {
        for s in slots.iter_mut() {
            *s = Slot::EMPTY;
        }
        Store { slots }
    }
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
    /// Supply a value from a case. Supplying does not mark anything stale — the
    /// caller decides that, because a supply is the start of a run rather than
    /// an edit to one.
    pub fn supply(&mut self, var: u16, value: f64, unit: Unit) {
        let s = &mut self.slots[var as usize];
        s.value = value;
        s.unit = unit;
        s.status = SlotStatus::Supplied;
        s.producer = u16::MAX;
    }
    pub fn get(&self, var: u16) -> &Slot {
        &self.slots[var as usize]
    }
    pub fn known(&self, var: u16) -> Option<f64> {
        let s = self.get(var);
        if s.is_known() {
            Some(s.value)
        } else {
            None
        }
    }
    /// Mark every slot a node produced as stale, transitively. Called when an
    /// input changes: staleness has to be transitive or it is not safe to show
    /// a person.
    pub fn mark_stale(&mut self, var: u16) {
        let s = &mut self.slots[var as usize];
        if s.status == SlotStatus::Computed {
            s.status = SlotStatus::Stale;
        }
    }
}
