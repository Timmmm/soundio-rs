#![allow(dead_code)]

use std::ffi::CStr;
use std::ptr;
use std::os::raw::c_char;

// Convert a Latin1 C String to a String.
// If `s` is null, an empty string is returned.
pub fn latin1_to_string(s: *const c_char) -> String {
	if s == ptr::null() {
		return String::new();
	}
	let c_str: &CStr = unsafe { CStr::from_ptr(s) };
	// This converts Latin1 to a String, instead of assuming UTF-8 (which I probably could to be fair).
	c_str.to_bytes().iter().map(|&c| c as char).collect()
}

// Convert a UTF-8 C String to a String.
// If `s` is null or contains invalid UTF-8, an empty string is returned.
pub fn utf8_to_string(s: *const c_char) -> String {
	if s == ptr::null() {
		return String::new();
	}
	let c_str: &CStr = unsafe { CStr::from_ptr(s) };

	c_str.to_str().unwrap_or("").to_string()
}