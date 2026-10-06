//! A person's key: made once, locked by their passphrase, and used to sign.
//!
//! Section 14 of the operating model in code. Each person makes one key in the
//! application; the private half stays on their own computer, locked by a
//! passphrase, and the public half is registered in the file above them, so a
//! signature made with it checks through the chain to the programme manager's
//! anchor. This module is the key itself — making it, locking and unlocking
//! it, signing and checking — and nothing about where a key is registered,
//! which is the chain's business.
//!
//! The choices, each the conventional one, so a reviewer can check them
//! against their sources rather than against this code:
//!
//! * signatures are Ed25519 (RFC 8032), checked strictly, so a signature has
//!   exactly one valid encoding;
//! * a fingerprint is the SHA-256 (FIPS 180-4) of the 32-byte public key;
//! * a passphrase becomes a key by PBKDF2-HMAC-SHA256 (RFC 8018) over a
//!   random 16-byte salt, 600 000 iterations unless the key says more;
//! * the private half is sealed with AES-256-GCM (NIST SP 800-38D) under a
//!   random 96-bit nonce, and the person's name and public key are bound in
//!   as associated data, so neither can be swapped without the seal breaking.
//!
//! Randomness is never made here. The application passes bytes from the
//! operating system ([`random_bytes`]), the page passes bytes from the
//! browser's `crypto.getRandomValues`, and a test passes fixed bytes — so the
//! page's build needs nothing of its own, and the same seed gives the same key
//! everywhere.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::Aes256Gcm;
use ed25519_dalek::{Signer, Verifier};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{Error, ErrorKind};

/// Iterations of the passphrase's key derivation for a new key. A key keeps
/// the count it was locked with, so this can be raised without breaking any
/// key already made.
pub const ITERATIONS: u32 = 600_000;

/// Below this a locked key is refused: a count this low is a key a stolen
/// laptop gives up in an afternoon.
pub const MIN_ITERATIONS: u32 = 100_000;

/// What the associated data starts with, so a sealed key can never be taken
/// for anything else this library seals.
const AAD_TAG: &[u8] = b"vleo key 1\0";

/// The SHA-256 of `data`.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Bytes as lowercase hexadecimal.
pub fn hex(bytes: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(D[(b >> 4) as usize] as char);
        s.push(D[(b & 15) as usize] as char);
    }
    s
}

/// Hexadecimal as exactly `N` bytes, or why it is not.
pub fn unhex<const N: usize>(text: &str, what: &str) -> Result<[u8; N], Error> {
    let t = text.trim();
    let bad = || {
        Error::new(
            ErrorKind::Malformed,
            format!("{what} is not {} hexadecimal digits: {t:?}", N * 2),
        )
    };
    if t.len() != N * 2 {
        return Err(bad());
    }
    let mut out = [0u8; N];
    for (i, pair) in t.as_bytes().chunks(2).enumerate() {
        let d = |c: u8| (c as char).to_digit(16).ok_or_else(bad);
        out[i] = (d(pair[0])? * 16 + d(pair[1])?) as u8;
    }
    Ok(out)
}

/// `n` bytes of randomness from the operating system — the installed
/// application's source. The page has none of its own and passes the
/// browser's instead.
#[cfg(not(target_arch = "wasm32"))]
pub fn random_bytes<const N: usize>() -> Result<[u8; N], Error> {
    let mut b = [0u8; N];
    getrandom::fill(&mut b).map_err(|e| {
        Error::new(
            ErrorKind::Randomness,
            format!("the operating system gave no randomness to make a key with: {e}"),
        )
    })?;
    Ok(b)
}

/// The public half of a key: what is registered, and what a signature is
/// checked against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// From its 32 bytes, refused if they are not a point on the curve.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<PublicKey, Error> {
        ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|_| {
            Error::new(
                ErrorKind::Malformed,
                format!("{} is not a public key", hex(&bytes)),
            )
        })?;
        Ok(PublicKey(bytes))
    }

    /// From the 64 hexadecimal digits it is written as.
    pub fn from_hex(text: &str) -> Result<PublicKey, Error> {
        PublicKey::from_bytes(unhex::<32>(text, "a public key")?)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn to_hex(&self) -> String {
        hex(&self.0)
    }

    /// The fingerprint a person reads and compares: the SHA-256 of the public
    /// key, as 64 hexadecimal digits. The programme manager's is the anchor
    /// written in START HERE.
    pub fn fingerprint(&self) -> String {
        hex(&sha256(&self.0))
    }

    /// Whether `signature` was made over `message` by this key's private half.
    /// Checked strictly (RFC 8032, section 5.1.7, with the small-order and
    /// non-canonical encodings refused), so one message and one key have
    /// exactly one signature that checks.
    pub fn check(&self, message: &[u8], signature: &Signature) -> Result<(), Error> {
        let vk = ed25519_dalek::VerifyingKey::from_bytes(&self.0)
            .map_err(|_| Error::new(ErrorKind::Malformed, "not a public key"))?;
        let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
        vk.verify_strict(message, &sig)
            .and_then(|()| vk.verify(message, &sig))
            .map_err(|_| {
                Error::new(
                    ErrorKind::Signature,
                    format!(
                        "the signature does not check against the key {}",
                        self.fingerprint()
                    ),
                )
            })
    }
}

/// A signature: 64 bytes, written as 128 hexadecimal digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    pub fn from_bytes(bytes: [u8; 64]) -> Signature {
        Signature(bytes)
    }

    pub fn from_hex(text: &str) -> Result<Signature, Error> {
        Ok(Signature(unhex::<64>(text, "a signature")?))
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
    }

    pub fn to_hex(&self) -> String {
        hex(&self.0)
    }
}

/// A key, unlocked: it signs. It lives only as long as it is being used, and
/// its private half is wiped from memory when it is dropped.
pub struct SigningKey {
    inner: ed25519_dalek::SigningKey,
}

impl SigningKey {
    /// The key a 32-byte seed makes (RFC 8032, section 5.1.5). The seed is the
    /// private half; whoever holds it holds the key.
    pub fn from_seed(seed: [u8; 32]) -> SigningKey {
        let seed = Zeroizing::new(seed);
        SigningKey {
            inner: ed25519_dalek::SigningKey::from_bytes(&seed),
        }
    }

    /// A new key, from the operating system's randomness.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn generate() -> Result<SigningKey, Error> {
        Ok(SigningKey::from_seed(
            *Zeroizing::new(random_bytes::<32>()?),
        ))
    }

    pub fn public(&self) -> PublicKey {
        PublicKey(self.inner.verifying_key().to_bytes())
    }

    /// A signature over `message`. Ed25519 is deterministic: the same key and
    /// message always give the same signature.
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.inner.sign(message).to_bytes())
    }

    /// The key locked by `passphrase`, to be kept by `person` on their own
    /// computer. `salt` and `nonce` must be fresh randomness for every lock.
    pub fn lock(
        &self,
        person: &str,
        passphrase: &str,
        salt: [u8; 16],
        nonce: [u8; 12],
        iterations: u32,
    ) -> Result<LockedKey, Error> {
        if iterations < MIN_ITERATIONS {
            return Err(Error::new(
                ErrorKind::Malformed,
                format!(
                    "a key is locked with at least {MIN_ITERATIONS} iterations, not {iterations}"
                ),
            ));
        }
        let public = self.public();
        let cipher = cipher(passphrase, &salt, iterations);
        let seed = Zeroizing::new(self.inner.to_bytes());
        let sealed = cipher
            .encrypt(
                (&nonce).into(),
                Payload {
                    msg: seed.as_slice(),
                    aad: &aad(person, &public),
                },
            )
            .map_err(|_| Error::new(ErrorKind::Malformed, "the key could not be sealed"))?;
        Ok(LockedKey {
            person: person.to_string(),
            public,
            salt,
            nonce,
            iterations,
            sealed,
        })
    }
}

/// A key as it is kept: its owner, its public half, and its private half
/// sealed by their passphrase. Everything here may be read by anyone; only
/// the passphrase opens it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedKey {
    pub person: String,
    pub public: PublicKey,
    pub salt: [u8; 16],
    pub nonce: [u8; 12],
    pub iterations: u32,
    /// The 32-byte seed sealed with AES-256-GCM, and its 16-byte tag.
    pub sealed: Vec<u8>,
}

impl LockedKey {
    /// The key, unlocked by `passphrase`. A wrong passphrase and a key changed
    /// after it was locked are told apart where they can be, and neither
    /// unlocks anything.
    pub fn unlock(&self, passphrase: &str) -> Result<SigningKey, Error> {
        if self.iterations < MIN_ITERATIONS {
            return Err(Error::new(
                ErrorKind::Tampered,
                format!(
                    "{}'s key says {} iterations, fewer than any key is locked with",
                    self.person, self.iterations
                ),
            ));
        }
        let cipher = cipher(passphrase, &self.salt, self.iterations);
        let seed = cipher
            .decrypt(
                (&self.nonce).into(),
                Payload {
                    msg: &self.sealed,
                    aad: &aad(&self.person, &self.public),
                },
            )
            .map(Zeroizing::new)
            .map_err(|_| {
                Error::new(
                    ErrorKind::Passphrase,
                    format!(
                        "the passphrase does not unlock {}'s key (or the key file was changed)",
                        self.person
                    ),
                )
            })?;
        let seed: [u8; 32] = seed
            .as_slice()
            .try_into()
            .map_err(|_| Error::new(ErrorKind::Tampered, "the sealed key is not 32 bytes"))?;
        let key = SigningKey::from_seed(seed);
        if key.public() != self.public {
            return Err(Error::new(
                ErrorKind::Tampered,
                format!(
                    "{}'s key does not make the public key it says it does",
                    self.person
                ),
            ));
        }
        Ok(key)
    }
}

/// The cipher a passphrase and salt make.
fn cipher(passphrase: &str, salt: &[u8; 16], iterations: u32) -> Aes256Gcm {
    let mut k = Zeroizing::new([0u8; 32]);
    pbkdf2::pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, k.as_mut());
    let key: &[u8; 32] = &k;
    Aes256Gcm::new(key.into())
}

/// What a sealed key is bound to: whose it is and which public key it makes.
fn aad(person: &str, public: &PublicKey) -> Vec<u8> {
    let mut a = AAD_TAG.to_vec();
    a.extend_from_slice(person.as_bytes());
    a.push(0);
    a.extend_from_slice(&public.0);
    a
}

impl LockedKey {
    /// The row a key file keeps it as (`locked_key`).
    pub fn to_row(&self) -> crate::model::LockedKeyRow {
        crate::model::LockedKeyRow {
            person: self.person.clone(),
            public_key: self.public.to_hex(),
            salt: hex(&self.salt),
            nonce: hex(&self.nonce),
            iterations: i64::from(self.iterations),
            sealed: hex(&self.sealed),
        }
    }

    /// The key a key file's row holds, refused if a field is not what it
    /// should be. Whether the passphrase opens it is [`LockedKey::unlock`]'s.
    pub fn from_row(row: &crate::model::LockedKeyRow) -> Result<LockedKey, Error> {
        let sealed_hex = row.sealed.trim();
        if !sealed_hex.len().is_multiple_of(2) || !sealed_hex.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err(Error::new(
                ErrorKind::Malformed,
                format!("{}'s sealed key is not hexadecimal", row.person),
            ));
        }
        let sealed = sealed_hex
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap_or("zz"), 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| Error::new(ErrorKind::Malformed, "the sealed key is not hexadecimal"))?;
        Ok(LockedKey {
            person: row.person.clone(),
            public: PublicKey::from_hex(&row.public_key)?,
            salt: unhex::<16>(&row.salt, "the salt")?,
            nonce: unhex::<12>(&row.nonce, "the nonce")?,
            iterations: u32::try_from(row.iterations).map_err(|_| {
                Error::new(
                    ErrorKind::Malformed,
                    format!("{} iterations is not a count", row.iterations),
                )
            })?,
            sealed,
        })
    }
}
