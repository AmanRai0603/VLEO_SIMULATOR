//! Deterministic hashing.
//!
//! Used for the chain hash that identifies a run, the case hash that keys the
//! cache, and the sheet hash that makes a stale page detectable rather than
//! merely wrong. FNV-1a, 64 bit: not a cryptographic hash and not claimed to
//! be one — its job is to be the same everywhere, forever, in three lines
//! anybody can check.
//!
//! The standard library hasher would have been wrong here for a reason worth
//! writing down: `DefaultHasher` is explicitly allowed to change between
//! releases, so a ledger row from eighteen months ago would stop matching
//! after a toolchain bump.

const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01B3;

/// An incremental FNV-1a state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hasher(u64);

impl Hasher {
    pub const fn new() -> Hasher {
        Hasher(OFFSET)
    }
    pub fn write_u8(&mut self, b: u8) {
        self.0 ^= b as u64;
        self.0 = self.0.wrapping_mul(PRIME);
    }
    pub fn write_bytes(&mut self, bs: &[u8]) {
        for &b in bs {
            self.write_u8(b);
        }
    }
    pub fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
        // A separator, so ("ab","c") and ("a","bc") do not collide.
        self.write_u8(0x1f);
    }
    pub fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }
    /// Hash a float by its bit pattern, with the two zeros normalised so that
    /// `-0.0` and `0.0` give the same key. A NaN never reaches here: a value
    /// that is not a number is a fault, and a fault does not get cached.
    pub fn write_f64(&mut self, v: f64) {
        let v = if v == 0.0 { 0.0 } else { v };
        self.write_bytes(&v.to_bits().to_le_bytes());
    }
    pub const fn finish(self) -> u64 {
        self.0
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher::new()
    }
}

/// One-shot hash of a string.
pub fn hash_str(s: &str) -> u64 {
    let mut h = Hasher::new();
    h.write_str(s);
    h.finish()
}

/// Format a hash as the six hexadecimal characters shown in a provenance bar.
/// Six is enough to distinguish the artefacts a person compares by eye, and
/// the full value is in the manifest.
pub fn short_hex(h: u64) -> [u8; 6] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 6];
    for (i, byte) in out.iter_mut().enumerate() {
        let nibble = ((h >> (4 * (5 - i))) & 0xF) as usize;
        *byte = HEX[nibble];
    }
    out
}
