//! The keys, held to values from outside this code.
//!
//! Every expected value below comes from a published test vector or from an
//! independent implementation, named beside it, never from this crate (rule 4
//! of AGENTS.md). Then the behaviour the operating model asks for: a key locks
//! and unlocks, a wrong passphrase unlocks nothing, a changed key or a changed
//! signature is refused, by kind.

use vleo_files::keys::{self, LockedKey, PublicKey, Signature, SigningKey, MIN_ITERATIONS};
use vleo_files::ErrorKind;

fn h<const N: usize>(s: &str) -> [u8; N] {
    keys::unhex::<N>(s, "a test vector").unwrap()
}

// ── Ed25519, RFC 8032 section 7.1 ────────────────────────────────────────────

/// TEST 1: an empty message.
#[test]
fn rfc8032_test_1() {
    let key = SigningKey::from_seed(h(
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
    ));
    assert_eq!(
        key.public().to_hex(),
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
    );
    let sig = key.sign(b"");
    assert_eq!(
        sig.to_hex(),
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
    );
    key.public().check(b"", &sig).unwrap();
}

/// TEST 2: one byte, 0x72. The signature is also what OpenSSL gives for this
/// key and message (Node's crypto.sign, checked when this test was written).
#[test]
fn rfc8032_test_2() {
    let key = SigningKey::from_seed(h(
        "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb",
    ));
    assert_eq!(
        key.public().to_hex(),
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c"
    );
    let sig = key.sign(&[0x72]);
    assert_eq!(
        sig.to_hex(),
        "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00"
    );
    PublicKey::from_hex("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c")
        .unwrap()
        .check(&[0x72], &sig)
        .unwrap();
}

// ── SHA-256, and the fingerprint made of it ──────────────────────────────────

/// FIPS 180-2, appendix B.1: "abc".
#[test]
fn sha256_of_abc() {
    assert_eq!(
        keys::hex(&keys::sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

/// The fingerprint of RFC 8032's first public key, as Python's hashlib gives
/// it: `hashlib.sha256(bytes.fromhex("d75a98…511a")).hexdigest()`.
#[test]
fn a_fingerprint_is_the_sha256_of_the_public_key() {
    let pk =
        PublicKey::from_hex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
            .unwrap();
    assert_eq!(
        pk.fingerprint(),
        "21fe31dfa154a261626bf854046fd2271b7bed4b6abe45aa58877ef47f9721b9"
    );
}

// ── The two primitives a locked key is made of ───────────────────────────────

/// PBKDF2-HMAC-SHA256, RFC 7914 section 11: P = "passwd", S = "salt", c = 1,
/// dkLen = 64. Python's hashlib.pbkdf2_hmac gives the same.
#[test]
fn pbkdf2_hmac_sha256_rfc7914() {
    let mut out = [0u8; 64];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(b"passwd", b"salt", 1, &mut out);
    assert_eq!(
        keys::hex(&out),
        "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc49ca9cccf179b645991664b39d77ef317c71b845b1e30bd509112041d3a19783"
    );
}

/// AES-256-GCM, test case 14 of McGrew and Viega's GCM specification: a zero
/// key, a zero 96-bit nonce and sixteen zero bytes. OpenSSL (Node's crypto)
/// gives the same ciphertext and tag.
#[test]
fn aes256_gcm_test_case_14() {
    use aes_gcm::aead::{Aead, KeyInit};
    let cipher = aes_gcm::Aes256Gcm::new((&[0u8; 32]).into());
    let out = cipher.encrypt((&[0u8; 12]).into(), &[0u8; 16][..]).unwrap();
    assert_eq!(
        keys::hex(&out),
        "cea7403d4d606b6e074ec5d3baf39d18d0d1c8a799996bf0265b98b5d48ab919"
    );
}

// ── What a person does with a key ────────────────────────────────────────────

fn alice() -> SigningKey {
    SigningKey::from_seed([7u8; 32])
}

fn locked() -> LockedKey {
    alice()
        .lock(
            "Alice Example",
            "correct horse",
            [1u8; 16],
            [2u8; 12],
            MIN_ITERATIONS,
        )
        .unwrap()
}

#[test]
fn a_key_locks_and_unlocks_with_its_passphrase() {
    let k = locked();
    assert_eq!(k.public, alice().public());
    assert_eq!(k.sealed.len(), 32 + 16, "the seed and its tag");
    assert!(
        !k.sealed.windows(32).any(|w| w == [7u8; 32]),
        "the seed is not in the clear"
    );
    let back = k.unlock("correct horse").unwrap();
    let sig = back.sign(b"node sw_ap_design, revision 3");
    alice()
        .public()
        .check(b"node sw_ap_design, revision 3", &sig)
        .unwrap();
}

#[test]
fn a_wrong_passphrase_unlocks_nothing() {
    let e = locked().unlock("correct horsE").err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Passphrase);
}

#[test]
fn a_key_given_to_someone_else_does_not_unlock() {
    // The name is bound into the seal: changing whose key it says it is
    // breaks the seal, so a key cannot be passed off as another person's.
    let mut k = locked();
    k.person = "Bob Example".into();
    assert_eq!(
        k.unlock("correct horse").err().unwrap().kind(),
        ErrorKind::Passphrase
    );
}

#[test]
fn a_key_whose_public_half_was_swapped_does_not_unlock() {
    let mut k = locked();
    k.public = SigningKey::from_seed([9u8; 32]).public();
    assert_eq!(
        k.unlock("correct horse").err().unwrap().kind(),
        ErrorKind::Passphrase
    );
}

#[test]
fn a_key_whose_sealed_half_was_changed_does_not_unlock() {
    let mut k = locked();
    k.sealed[5] ^= 1;
    assert_eq!(
        k.unlock("correct horse").err().unwrap().kind(),
        ErrorKind::Passphrase
    );
}

#[test]
fn too_few_iterations_are_refused_both_ways() {
    let e = alice()
        .lock(
            "Alice Example",
            "pw",
            [1u8; 16],
            [2u8; 12],
            MIN_ITERATIONS - 1,
        )
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::Malformed);
    let mut k = locked();
    k.iterations = 1;
    assert_eq!(
        k.unlock("correct horse").err().unwrap().kind(),
        ErrorKind::Tampered
    );
}

#[test]
fn a_changed_message_or_signature_does_not_check() {
    let key = alice();
    let sig = key.sign(b"release solar 1.2");
    assert_eq!(
        key.public()
            .check(b"release solar 1.3", &sig)
            .err()
            .unwrap()
            .kind(),
        ErrorKind::Signature
    );
    let mut bytes = sig.to_bytes();
    bytes[0] ^= 1;
    assert_eq!(
        key.public()
            .check(b"release solar 1.2", &Signature::from_bytes(bytes))
            .err()
            .unwrap()
            .kind(),
        ErrorKind::Signature
    );
}

#[test]
fn another_persons_key_does_not_check_the_signature() {
    let sig = alice().sign(b"seal");
    let bob = SigningKey::from_seed([9u8; 32]).public();
    assert_eq!(
        bob.check(b"seal", &sig).err().unwrap().kind(),
        ErrorKind::Signature
    );
}

/// A signature whose scalar half has had the group order added is the same
/// point on paper and a different string of bytes; strict checking refuses
/// it, so one message and one key have one signature (RFC 8032, 5.1.7).
#[test]
fn a_non_canonical_signature_is_refused() {
    let key = alice();
    let sig = key.sign(b"x").to_bytes();
    // The group order L, little-endian (RFC 8032, section 5.1).
    let l = h::<32>("edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010");
    let mut s = [0u8; 32];
    let mut carry = 0u16;
    for i in 0..32 {
        let v = sig[32 + i] as u16 + l[i] as u16 + carry;
        s[i] = v as u8;
        carry = v >> 8;
    }
    let mut bad = sig;
    bad[32..].copy_from_slice(&s);
    assert_eq!(
        key.public()
            .check(b"x", &Signature::from_bytes(bad))
            .err()
            .unwrap()
            .kind(),
        ErrorKind::Signature
    );
}

#[test]
fn text_that_is_not_a_key_is_refused_by_name() {
    for bad in ["", "zz", &"0".repeat(63), &"g".repeat(64)] {
        assert_eq!(
            PublicKey::from_hex(bad).err().unwrap().kind(),
            ErrorKind::Malformed
        );
    }
    assert_eq!(
        Signature::from_hex("abc").err().unwrap().kind(),
        ErrorKind::Malformed
    );
}

#[test]
fn a_new_key_is_new_each_time() {
    let a = SigningKey::generate().unwrap().public();
    let b = SigningKey::generate().unwrap().public();
    assert_ne!(a, b);
}
