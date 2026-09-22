//! C ABI exports for the ctypes-based Python binding.

use std::os::raw::c_char;

use crate::{jaro_winkler, levenshtein_bounded, ngram_similarity};

unsafe fn read_str(pointer: *const c_char) -> String {
    let bytes = std::ffi::CStr::from_ptr(pointer).to_bytes();
    String::from_utf8_lossy(bytes).into_owned()
}

/// # Safety
/// `a` and `b` must be valid NUL-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn fuzzyx_levenshtein(a: *const c_char, b: *const c_char) -> usize {
    levenshtein_bounded(&read_str(a), &read_str(b), usize::MAX)
}

/// # Safety
/// Same contract as `fuzzyx_levenshtein`.
#[no_mangle]
pub unsafe extern "C" fn fuzzyx_jaro_winkler(a: *const c_char, b: *const c_char) -> f64 {
    jaro_winkler(&read_str(a), &read_str(b), 0.1)
}

/// # Safety
/// Same contract as `fuzzyx_levenshtein`.
#[no_mangle]
pub unsafe extern "C" fn fuzzyx_bigram(a: *const c_char, b: *const c_char) -> f64 {
    ngram_similarity(&read_str(a), &read_str(b), 2)
}
