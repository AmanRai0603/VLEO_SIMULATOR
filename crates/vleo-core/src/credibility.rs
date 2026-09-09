//! Credibility — eight factors, and the lowest one governs.
//!
//! Adapted from the NASA-STD-7009 credibility assessment, applied per node
//! rather than per model. That change is ours and is marked as unproven: the
//! standard scores whole models, and whether three people independently score
//! one node the same way has not been measured here.
//!
//! Two properties matter more than the numbers:
//!
//! * **The lowest factor governs.** A node with excellent evidence and no
//!   stated assumptions is not a good node; it is an unassessed one. Averaging
//!   would let a strong factor hide a zero, which is the whole failure the
//!   score exists to prevent.
//! * **It is never stored.** The vector is computed on the run, from the
//!   evidence that actually executed. A stored badge is a claim about last
//!   March.

/// The eight factors, in the order they appear on a node page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Factor {
    /// Is the mathematics the right mathematics for this regime, and is it
    /// cited? Scored by the physics review, not by a machine.
    Mathematics = 0,
    /// Are the assumptions stated, each with the condition under which it
    /// stops holding?
    Assumptions = 1,
    /// Is the implementation verified — does it compute what the sheet says?
    Verification = 2,
    /// Is it validated — do the fixtures agree with an external source?
    Validation = 3,
    /// How good is the pedigree of the input data that reached it?
    InputPedigree = 4,
    /// Is the uncertainty in the result characterised at all?
    Uncertainty = 5,
    /// Has anybody who did not write it walked through it from the page?
    Understanding = 6,
    /// Is the result reproducible — pinned data, a recorded manifest, a
    /// deterministic engine?
    Reproducibility = 7,
}

impl Factor {
    pub const ALL: [Factor; 8] = [
        Factor::Mathematics,
        Factor::Assumptions,
        Factor::Verification,
        Factor::Validation,
        Factor::InputPedigree,
        Factor::Uncertainty,
        Factor::Understanding,
        Factor::Reproducibility,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Factor::Mathematics => "mathematics",
            Factor::Assumptions => "assumptions",
            Factor::Verification => "verification",
            Factor::Validation => "validation",
            Factor::InputPedigree => "input pedigree",
            Factor::Uncertainty => "uncertainty",
            Factor::Understanding => "understanding",
            Factor::Reproducibility => "reproducibility",
        }
    }
    /// What a zero in this factor means, shown beside the score so that a low
    /// number is a work item rather than a judgement.
    pub const fn zero_means(self) -> &'static str {
        match self {
            Factor::Mathematics => "no relation cited to a page",
            Factor::Assumptions => "no assumption stated",
            Factor::Verification => "the implementation has never been checked against the sheet",
            Factor::Validation => "no fixture from a source outside this code",
            Factor::InputPedigree => "an input arrived with no provenance, or from unverified data",
            Factor::Uncertainty => "no uncertainty stated for the result",
            Factor::Understanding => "nobody but the author has walked through it",
            Factor::Reproducibility => "the run cannot be reproduced from its manifest",
        }
    }
}

/// The evidence tier a node is held to. Set on the sheet; it decides how much
/// evidence the gate demands and whether differential fill runs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Tier {
    /// Two independent routes to the expected value, agreeing to a stated
    /// tolerance. Reserved for nodes a design decision is staked on.
    APlus,
    /// A published table or reference implementation, cited to the page.
    A,
    /// An independent tool run by a person — this is what MATLAB is for.
    B,
    /// A physical bound or conservation law that must hold. Detects a class of
    /// error rather than a value.
    C,
    /// Declared but not yet evidenced.
    Unset,
}

impl Tier {
    pub const fn name(self) -> &'static str {
        match self {
            Tier::APlus => "A+",
            Tier::A => "A",
            Tier::B => "B",
            Tier::C => "C",
            Tier::Unset => "unset",
        }
    }
    pub fn from_name(s: &str) -> Tier {
        match s {
            "A+" => Tier::APlus,
            "A" => Tier::A,
            "B" => Tier::B,
            "C" => Tier::C,
            _ => Tier::Unset,
        }
    }
    /// The highest validation score a node at this tier may reach. A tier-C
    /// node cannot claim full validation however many cases pass, because a
    /// conservation law does not confirm a constant.
    pub const fn validation_ceiling(self) -> u8 {
        match self {
            Tier::APlus => 4,
            Tier::A => 3,
            Tier::B => 2,
            Tier::C => 1,
            Tier::Unset => 0,
        }
    }
    /// What this tier cannot detect — stated so that a high score is read with
    /// its limits attached.
    pub const fn cannot_detect(self) -> &'static str {
        match self {
            Tier::APlus => "an error common to both independent routes",
            Tier::A => "an error in the published source itself",
            Tier::B => "an error the independent tool shares with this one",
            Tier::C => "a wrong constant that still conserves the quantity",
            Tier::Unset => "anything",
        }
    }
}

/// Eight scores in `0..=4`. Never stored; always computed on the run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CredVec(pub [u8; 8]);

impl CredVec {
    /// Nothing assessed.
    pub const ZERO: CredVec = CredVec([0; 8]);

    pub fn get(&self, f: Factor) -> u8 {
        self.0[f as usize]
    }
    pub fn set(&mut self, f: Factor, v: u8) {
        self.0[f as usize] = if v > 4 { 4 } else { v };
    }
    /// The score the node is reported at: the lowest of the eight.
    pub fn governing_score(&self) -> u8 {
        let mut m = self.0[0];
        for &v in &self.0[1..] {
            if v < m {
                m = v;
            }
        }
        m
    }
    /// Which factor is holding the node down. When several tie, the earliest in
    /// declaration order wins, so the answer is stable between runs and two
    /// people reading the same node see the same next piece of work.
    pub fn governing_factor(&self) -> Factor {
        let m = self.governing_score();
        for f in Factor::ALL {
            if self.get(f) == m {
                return f;
            }
        }
        Factor::Mathematics
    }
    /// One tier below — applied to an iterated result, unless the converged
    /// quantity has a fixture of its own.
    pub fn demoted(self) -> CredVec {
        let mut v = self;
        for i in 0..8 {
            v.0[i] = v.0[i].saturating_sub(1);
        }
        v
    }
    /// The rollup over a subtree is the minimum, factor by factor — the same
    /// rule one level up, so a subtree is never more credible than its weakest
    /// node.
    pub fn rollup(self, other: CredVec) -> CredVec {
        let mut v = [0u8; 8];
        for i in 0..8 {
            v[i] = if self.0[i] < other.0[i] { self.0[i] } else { other.0[i] };
        }
        CredVec(v)
    }
}

impl Default for CredVec {
    fn default() -> Self {
        CredVec::ZERO
    }
}
