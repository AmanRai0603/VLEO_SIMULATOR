//! The design-file library, for a page opened from a file.
//!
//! A page has its own SQLite and no Rust, so the checks a file must pass would
//! otherwise be written again in JavaScript — a second implementation of what
//! a signature means, which would drift from the first. So it is this:
//! `vleo_files::page`, unchanged, behind exported functions.
//!
//! The protocol is a buffer each way, as the method checker's is. The page
//! asks for `len` bytes with `vleo_alloc`, writes the request into them
//! (`vleo_files::page` says what each request holds), and calls the check,
//! which returns a pointer to its JSON answer; `vleo_out_len` says how long it
//! is. The answer stays valid until the next call.

// Raw pointers across the WebAssembly boundary are the whole of this crate's
// job, and there is no way to take a buffer from JavaScript without them.
#![allow(unsafe_code)]

use std::cell::RefCell;

thread_local! {
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn answer(json: String) -> *const u8 {
    OUT.with(|o| {
        let mut o = o.borrow_mut();
        *o = json.into_bytes();
        o.as_ptr()
    })
}

/// Room for `len` bytes of request, owned by the caller until the check.
#[no_mangle]
pub extern "C" fn vleo_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Check a release through the signature chain: the request is the anchor,
/// the programme's file and the release (`vleo_files::page::check_release`).
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_check_release(ptr: *mut u8, len: usize) -> *const u8 {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    answer(vleo_files::page::check_release(&bytes))
}

/// Check what one release holds — the checks intake makes of its content
/// (`vleo_files::page::check_content`). The request is the release's rows.
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_check_content(ptr: *mut u8, len: usize) -> *const u8 {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    answer(vleo_files::page::check_content(&bytes))
}

/// The group folder's checks, the page's group checker said by the library
/// (`vleo_files::page::check_folder`). The request is a group's file's rows.
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_check_folder(ptr: *mut u8, len: usize) -> *const u8 {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    answer(vleo_files::page::check_folder(&bytes))
}

/// The seal of a group's folder: its sign-offs, current or stale, and what
/// stands between it and its seal (`vleo_files::page::seal_state`).
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_seal_state(ptr: *mut u8, len: usize) -> *const u8 {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    answer(vleo_files::page::seal_state(&bytes))
}

/// Two files compared, block by block (`vleo_files::page::compare`).
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_compare(ptr: *mut u8, len: usize) -> *const u8 {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    answer(vleo_files::page::compare(&bytes))
}

/// The length of the last answer.
#[no_mangle]
pub extern "C" fn vleo_out_len() -> usize {
    OUT.with(|o| o.borrow().len())
}

/// The file format this library reads and writes, so a page can say which.
#[no_mangle]
pub extern "C" fn vleo_format() -> u32 {
    vleo_files::meta::FORMAT as u32
}
