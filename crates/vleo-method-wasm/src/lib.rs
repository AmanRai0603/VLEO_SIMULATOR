//! The method checker, for a node form opened from a file.
//!
//! A form is a single HTML file that works with no server and no network, so
//! the check it runs while the author types has to travel inside it. Writing
//! that check again in JavaScript would be a second implementation of what a
//! method means, and the two would drift the first time the language grew a
//! function. So it is this: `vleo_sheet::method::report_toml`, unchanged,
//! behind two exported functions.
//!
//! The protocol is a buffer each way. The page asks for `len` bytes with
//! `vleo_alloc`, writes the sheet's text into them as UTF-8, and calls
//! `vleo_report`, which returns a pointer to the JSON report; `vleo_report_len`
//! says how long it is. The report stays valid until the next call.

// Raw pointers across the WebAssembly boundary are the whole of this crate's
// job, and there is no way to take a buffer from JavaScript without them.
#![allow(unsafe_code)]

use std::cell::RefCell;

thread_local! {
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Room for `len` bytes of input, owned by the caller until `vleo_report`.
#[no_mangle]
pub extern "C" fn vleo_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Check the sheet text in the buffer and return the JSON report.
///
/// # Safety
/// `ptr` must come from `vleo_alloc(len)` and hold `len` written bytes; it is
/// freed here.
#[no_mangle]
pub unsafe extern "C" fn vleo_report(ptr: *mut u8, len: usize) -> *const u8 {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    let text = String::from_utf8_lossy(&bytes);
    let json = match vleo_sheet::method::report_toml(&text) {
        Ok(r) => r.json(),
        Err(e) => vleo_sheet::method::error_json(&e),
    };
    OUT.with(|o| {
        let mut o = o.borrow_mut();
        *o = json.into_bytes();
        o.as_ptr()
    })
}

/// The length of the last report.
#[no_mangle]
pub extern "C" fn vleo_report_len() -> usize {
    OUT.with(|o| o.borrow().len())
}

/// The language version this checker reads, so a form can say which it has.
#[no_mangle]
pub extern "C" fn vleo_language_version() -> u32 {
    vleo_sheet::method::LANGUAGE_VERSION
}
